# Weekly import performance

This work measures complete imports from fixed FCC weekly archives before
selecting implementation changes. It does not change the published CLI version.

## Acceptance plan

- [x] Record immutable archive inputs and a repeatable full-import baseline.
- [ ] Measure the already-merged removal of the archive counting pass.
- [ ] Remove duplicate parser allocations while preserving the public parsed-line
      API, continuation handling, raw records, and error behavior.
- [ ] Compare serial import with bounded parsing alongside one SQLite writer.
      Preserve file/record order, archive rollback, and worker error reporting.
- [ ] Measure index rebuilding once across a combined service refresh, and
      identify the orchestration changes required to use it safely.
- [ ] Compare complete database contents, schema/indexes, and import reports;
      record time, CPU, peak memory, and cache conditions separately.
- [x] Run relevant regressions, workspace tests, formatting, and Clippy.
- [ ] Commit and merge selected changes; leave release-plz excluded.

Run comparisons in disposable directories, using private copies of fixed
archives. Never overwrite a serving database or evict the host's global cache.
Keep exploratory workstation results separate from isolated qualification.

The first inputs are the cached August 9, 2026 Amateur and GMRS full releases:

| Archive | Bytes | SHA-256 |
| --- | ---: | --- |
| `l_amat.zip` | 198102897 | `82fcd37ecdf3f82b68382c2c0d831cc7ea83c313ecd17f0566335d94018eaf8c` |
| `l_gmrs.zip` | 53697810 | `efa2c1489c04f6a17468eef29e20215cd69995fddecfaadec44cdbb50ef0a87c` |

The source files are unchanged; measurement copies are read-only. The service
uses full import mode and invokes Amateur and GMRS updates separately.

## Reproduction

Build `cargo build --release -p uls-db --example benchmark_import --locked`.
Keep each variant's binary separately, then run:

```sh
python3 scripts/compare-imports.py --inputs /path/to/fixed/archives \
  --output /path/to/new/results --repetitions 3 \
  --variant baseline /path/to/baseline stream \
  --variant candidate /path/to/candidate pipeline
```

The driver makes its own archive copies and pre-reads them before every run.
Each run creates a new database and imports both complete services. It records
wall/CPU time and peak RSS, checks all tables and foreign keys, and compares a
SHA3 digest of every stored value, primary key and SQLite sequence value, plus
the exact schema. Only generated `import_status.imported_at` values are excluded
from equality, after checking that each remains a timestamp with a timezone.
Verification is outside the measured import. Its immutable SQLite readers are
limited to the completed private child-process outputs and reject nonempty WAL
or rollback journals; they are not a serving-database validation shortcut.

Run `python3 scripts/test_compare_imports.py` to check that the independent
comparison catches a changed FCC value, a missing index and an uncheckpointed
WAL. `count-first` measures the removed progress-counting pass using the same
baseline binary. This is a controlled reconstruction, not a timing claim about
the separately published 0.1.7 binary and its older dependencies.

The current prototype leaves pipelined parsing opt-in while qualifying it.
It uses one parser, two queued batches of 256 records, and one SQLite writer.
The public `ParsedLine` representation and continuation semantics stay intact.
The Rust 1.88 test run also required the existing `unty-next` benchmark
dependency to move from 0.1.1 (which required Rust 1.90) to compatible 0.1.2.

Initial workstation runs are exploratory: 157.13 seconds for the counting
baseline, 182.96 for buffer reuse without counting, and 136.45 for the bounded
pipeline. CPU totals were 142.39, 130.57 and 125.20 seconds. The wall/CPU mismatch
and changing host conditions prevent choosing an implementation from these
single samples. Both candidate databases match the baseline's full contents.
Repeated isolated qualification remains required.

Use `--seed /path/to/completed/benchmark/import.db` to check a full refresh of
existing data as well as a fresh import. Every child copies the fixed seed into
its new output directory. Its import timer excludes the copy; the outer process
CPU/RSS and wall-time measurements include it. Source hashes are checked again
afterward. This also checks replacement ordering and SQLite sequence values.

## Variant isolation and review checkpoint

The tested source checkpoint is `1a478a67bc655667715993fa771d5325c96fdf4a`.
Build each source variant in its own checkout **and its own Cargo target
directory**. Record each source and executable hash before running. An initial
shared-target build reused an executable across different source trees; those
artifacts were rejected before qualification. The rebuilt baseline, candidate
and combined-index binaries have different verified hashes.

The instrumented baseline restores only `crates/uls-parser/src/dat.rs` from
`a47e55127b837a5bc480bc1fdd4d0ae4ab2c3be1`, using the same locked dependencies,
benchmark example and serial importer as the candidate. Compare its
`count-first` and `stream` modes to isolate the removed counting pass; compare
baseline `stream` with candidate `stream` to isolate buffer reuse; compare
candidate `stream` with `pipeline` to isolate the worker.

The [source and binary receipt](benchmark-receipts/2026-09-11-import-provenance.json)
includes the exact combined-index experiment as `combined_index_patch_lines`.
Join those lines to recover a patch against the tested source checkpoint.
Apply it only in a separate benchmark checkout. It defers successful index
restoration to the example after both archives; the shipping importer does
not defer indexes. A production implementation needs one owner for the
complete service batch, including restoration on a later archive failure and
preservation of per-archive status/source metadata. Independent CLI invocations
cannot share the current restoration guard. The benchmark does not establish
that production orchestration contract.

All 703 workspace tests and three doctests passed on Rust 1.88, as did
current-stable workspace Clippy and formatting. Three independent comparison
oracle tests and fresh/seeded smoke imports passed. The draft
[PR #72](https://github.com/tgies/uls/pull/72) passed all 11 hosted checks at this
checkpoint. Repeated full-size performance qualification and final default
selection remain pending.

The [full-size workstation refresh check](benchmark-receipts/2026-09-11-import-wsl-refresh.json)
also passed: both variants replaced existing data and produced identical full
databases and import reports, preserving input and binary hashes. One serial
sample took 343.46 seconds of import time; one pipeline sample took 275.28.
Whole-process CPU totals were 276.17 and 235.90 seconds. Seed copies alone took
46.95 and 5.44 seconds, showing substantial shared-host variability. These
single observations establish the full-size refresh correctness check, not a
controlled performance improvement. Repeated isolated measurement is running.

SQLite's auxiliary sorting threads are a separate possible experiment. In the
bundled SQLite 3.51.1, `sqlite3VdbeSorterInit` sets the worker count to zero when
`sqlite3TempInMemory(db)` is true. The importer currently uses
`PRAGMA temp_store=MEMORY`, so adding `PRAGMA threads=2` alone would not parallelize
index sorting. Testing file-backed sorting would change the temporary-I/O and
memory tradeoff and needs its own comparison; it is outside this implementation.
See SQLite's [thread limit](https://www.sqlite.org/pragma.html#pragma_threads)
and [sorter source](https://github.com/sqlite/sqlite/blob/version-3.51.1/src/vdbesort.c).


## Recycling the parser batches

The first isolated round showed the original pipeline spending more CPU than
serial buffer reuse. A [local diagnostic](benchmark-receipts/2026-09-11-parser-handoff.json)
then parsed all 12,918,713 records without SQLite work. Two serial samples took
10.39–10.61 seconds and 10.38–10.61 CPU seconds. Transferring owned records to
another thread for destruction took 16.83–17.14 seconds and 20.04–20.35 CPU
seconds. Returning consumed batches to the parser for destruction and reuse
reduced that to 13.24–13.32 seconds and 13.48–13.56 CPU seconds. This isolates
handoff/ownership overhead; it does not establish a full-import speedup.

The revised opt-in pipeline returns batches to their producer, consumes records
by reference and reuses at most four batch buffers. Both channels are bounded.
Parser allocation/destruction stays on its thread except the last in-flight
batches after it finishes or a consumer unwinds. Existing ordering and callback
panic regressions pass, and the stream-error test now fails after multiple
reuse cycles with a partial final batch. All 703 workspace tests, three doctests,
Rust 1.88 formatting and current-stable Clippy passed again. The revised binary
still needs repeated complete-import qualification before default selection.
