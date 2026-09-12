#!/usr/bin/env python3
"""Compare local weekly-import binaries using fixed archives and new databases.

Requires Linux, GNU time, and the SQLite CLI with sha3_query(). Hashes include
every stored field and schema object, except import_status.imported_at, whose
wall-clock value is validated independently. No network or global cache eviction.
"""

import argparse
import contextlib
import datetime
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import sqlite3
import subprocess


def digest(path):
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def quote_identifier(value):
    return '"' + value.replace('"', '""') + '"'


def quote_string(value):
    return "'" + value.replace("'", "''") + "'"


def verify_database(path):
    # Only called after our private benchmark child exits. An earlier read-only
    # inspection can leave empty WAL/shared-memory files behind. Reject actual
    # journal content; immutable readers then avoid creating more sidecars.
    for suffix in ["-wal", "-journal"]:
        sidecar = Path(str(path) + suffix)
        assert not sidecar.exists() or sidecar.stat().st_size == 0
    uri = path.as_uri() + "?mode=ro&immutable=1"
    with contextlib.closing(sqlite3.connect(uri, uri=True)) as db:
        db.execute("PRAGMA mmap_size=268435456")
        db.execute("PRAGMA cache_size=-64000")
        assert db.execute("PRAGMA quick_check").fetchall() == [("ok",)]
        assert db.execute("PRAGMA foreign_key_check").fetchone() is None
        schema = db.execute(
            "SELECT type,name,tbl_name,sql FROM sqlite_schema ORDER BY type,name"
        ).fetchall()
        hashes_sql = []
        tables = {}
        for kind, name, _, _ in schema:
            if kind != "table":
                continue
            quoted = quote_identifier(name)
            columns = db.execute(f"PRAGMA table_info({quoted})").fetchall()
            fields = [column[1] for column in columns]
            if name == "import_status":
                fields.remove("imported_at")
                for (timestamp,) in db.execute("SELECT imported_at FROM import_status"):
                    parsed = datetime.datetime.fromisoformat(timestamp)
                    assert parsed.tzinfo is not None
            primary = [c[1] for c in sorted(columns, key=lambda c: c[5]) if c[5]]
            projection = ",".join(map(quote_identifier, fields))
            ordering = ",".join(map(quote_identifier, primary or fields))
            query = f"SELECT {projection} FROM {quoted} ORDER BY {ordering}"
            hashes_sql.append(f"SELECT hex(sha3_query({quote_string(query)},256));")
            tables[name] = {
                "rows": db.execute(f"SELECT COUNT(*) FROM {quoted}").fetchone()[0],
                "columns": fields,
                "order": primary or fields,
            }
    hashes = subprocess.check_output(
        ["sqlite3", "-batch", "-bail", "-readonly", uri],
        input="\n".join(hashes_sql), text=True,
    ).splitlines()
    assert len(hashes) == len(tables)
    for table, value in zip(tables.values(), hashes):
        assert re.fullmatch(r"[0-9A-F]{64}", value)
        table["sha3_256"] = value
    return {"schema": schema, "tables": tables, "quick_check": "ok", "foreign_key_check": "ok"}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--inputs", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--variant", nargs=3, action="append", required=True,
                        metavar=("NAME", "BINARY", "MODE"))
    parser.add_argument("--repetitions", type=int, default=3)
    parser.add_argument("--keep-databases", action="store_true")
    parser.add_argument("--daily-inputs", type=Path, help="fixed HA/ZA daily ZIPs inserted after each service weekly")
    parser.add_argument("--seed", type=Path, help="completed benchmark database to copy before each refresh")
    args = parser.parse_args()
    if args.repetitions < 1:
        parser.error("repetitions must be positive")
    names = [v[0] for v in args.variant]
    if len(set(names)) != len(names) or not all(re.fullmatch(r"[A-Za-z0-9_-]+", n) for n in names):
        parser.error("variant names must be unique simple file names")
    binaries = [(name, Path(binary).resolve(strict=True), mode) for name, binary, mode in args.variant]
    for _, binary, mode in binaries:
        if not binary.is_file() or not os.access(binary, os.X_OK):
            parser.error(f"not an executable: {binary}")
        if mode not in ["count-first", "stream", "pipeline", "batch"]:
            parser.error(f"unknown mode: {mode}")
    sources = [(service, (args.inputs / name).resolve(strict=True))
               for service, name in [("HA", "l_amat.zip"), ("ZA", "l_gmrs.zip")]]
    if args.daily_inputs:
        sources = [item for service, source in sources for item in [
            (service, source),
            (service + "_DAILY", (args.daily_inputs / ("l_amat_daily.zip" if service == "HA" else "l_gmrs_daily.zip")).resolve(strict=True))
        ]]
    args.output.mkdir(mode=0o700)
    output = args.output.resolve()
    private_inputs = output / "inputs"
    private_inputs.mkdir()
    source_hashes = {str(path): digest(path) for _, path in sources}
    seed = args.seed.resolve(strict=True) if args.seed else None
    if seed:
        source_hashes[str(seed)] = digest(seed)
        for suffix in ["-wal", "-journal"]:
            sidecar = Path(str(seed) + suffix)
            assert not sidecar.exists() or sidecar.stat().st_size == 0
        copied_seed = private_inputs / "seed.db"
        shutil.copyfile(seed, copied_seed)
        assert digest(copied_seed) == source_hashes[str(seed)]
    binary_hashes = {str(path): digest(path) for _, path, _ in binaries}
    archives = []
    for service, source in sources:
        copied = private_inputs / source.name
        shutil.copyfile(source, copied)
        copied.chmod(0o400)
        assert digest(copied) == source_hashes[str(source)]
        archives.append((service, copied))
    report = {
        "format": "uls.import_benchmark.v1",
        "cpu_affinity": sorted(os.sched_getaffinity(0)),
        "sqlite_verifier": subprocess.check_output(["sqlite3", "--version"], text=True).strip(),
        "inputs_sha256": source_hashes,
        "binaries_sha256": binary_hashes,
        "cache_condition": "archive files sequentially pre-read before every run; fresh output database",
        "seeded_refresh": seed is not None,
        "resource_scope": "complete child process, including seed copy when requested; report.elapsed_seconds excludes seed copy",
        "database_comparison": "all rows including primary keys and sequence state; imported_at timestamps checked but excluded from digest",
        "runs": [],
    }
    reference = None
    reference_stats = None
    for repetition in range(args.repetitions):
        # Rotate first position so every variant runs at a different point in the cycle.
        variants = binaries[repetition % len(binaries):] + binaries[:repetition % len(binaries)]
        for name, binary, mode in variants:
            for _, archive in archives:
                with archive.open("rb") as stream:
                    while stream.read(1024 * 1024):
                        pass
            prefix = output / f"{name}-{repetition + 1}"
            resource_path = prefix.with_suffix(".resources.json")
            command = ["/usr/bin/time", "-f",
                       '{"wall_seconds":%e,"user_seconds":%U,"system_seconds":%S,"max_rss_kib":%M}',
                       "-o", str(resource_path), str(binary), str(prefix), "full", mode]
            command += [f"{service}={archive}" for service, archive in archives]
            if seed:
                command += [f"SEED={copied_seed}"]
            with prefix.with_suffix(".stdout.log").open("w") as stdout, prefix.with_suffix(".stderr.log").open("w") as stderr:
                subprocess.run(command, stdout=stdout, stderr=stderr, check=True)
            result = json.loads((prefix / "report.json").read_text())
            resources = json.loads(resource_path.read_text())
            database = prefix / "import.db"
            verified = verify_database(database)
            stats = [{key: row[key] for key in ["service", "records", "files", "parse_errors", "insert_errors"]}
                     for row in result["archives"]]
            assert all(row["parse_errors"] == 0 and row["insert_errors"] == 0 for row in stats)
            if reference is None:
                reference, reference_stats = verified, stats
                (output / "database-reference.json").write_text(json.dumps(reference, indent=2) + "\n")
            assert verified == reference, f"database mismatch: {name} repetition {repetition + 1}"
            assert stats == reference_stats, f"import report mismatch: {name} repetition {repetition + 1}"
            run = {"variant": name, "mode": mode, "repetition": repetition + 1,
                   "resources": resources, "report": result, "database_bytes": database.stat().st_size,
                   "database_equal": True, "stats_equal": True}
            report["runs"].append(run)
            (output / "metrics.json").write_text(json.dumps(report, indent=2) + "\n")
            print(json.dumps({"variant": name, "repetition": repetition + 1, **resources}), flush=True)
            if not args.keep_databases:
                database.unlink()
    assert all(digest(Path(p)) == value for p, value in source_hashes.items())
    assert all(digest(Path(p)) == value for p, value in binary_hashes.items())
    assert all(digest(copy) == source_hashes[str(source)] for (_, copy), (_, source) in zip(archives, sources))
    if seed:
        assert digest(copied_seed) == source_hashes[str(seed)]
    report["inputs_unchanged"] = True
    report["all_databases_equal"] = True
    (output / "metrics.json").write_text(json.dumps(report, indent=2) + "\n")


if __name__ == "__main__":
    main()
