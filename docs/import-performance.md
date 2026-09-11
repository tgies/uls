# Weekly import performance

The selected change reuses the DAT line buffer and inspects a borrowed record
prefix, eliminating two avoidable allocation/copy passes. The existing serial
importer and public parsed-line API stay intact. The bounded parser workers
remain reproducible experiments, with no new production threading option.
This work does not release a CLI version or deploy a service image.

## Repeated full-size comparison

The fixed inputs are the August 9, 2026 Amateur and GMRS full releases:

| Archive | Bytes | SHA-256 |
| --- | ---: | --- |
| `l_amat.zip` | 198102897 | `82fcd37ecdf3f82b68382c2c0d831cc7ea83c313ecd17f0566335d94018eaf8c` |
| `l_gmrs.zip` | 53697810 | `efa2c1489c04f6a17468eef29e20215cd69995fddecfaadec44cdbb50ef0a87c` |

Each full comparison applies 12,918,713 DAT records: 10,514,698 Amateur records
from eight files and 2,404,015 GMRS records from six. Both isolated workers had
two logical CPUs, reported as two hardware threads on one Intel Xeon core,
and approximately 8 GiB RAM. Imports ran with networking disabled. The driver
pre-read private archive copies before each run, used a fresh output database,
and rotated variant order. Database verification is outside the measured import.

The first worker ran three fresh imports per variant at source
`1a478a67bc655667715993fa771d5325c96fdf4a`:

| Variant | Median wall seconds | Range | Median CPU seconds |
| --- | ---: | ---: | ---: |
| Old parser, counting pass | 299.90 | 296.78–313.64 | 297.96 |
| Old parser, immediate streaming | 279.90 | 278.85–287.67 | 277.81 |
| Buffer reuse, serial | 256.28 | 253.54–259.07 | 253.78 |
| Buffer reuse, parser worker | 262.30 | 259.24–267.45 | 311.98 |
| Parser worker, one combined index rebuild | 201.03 | 198.34–206.64 | 249.54 |

Buffer reuse reduced median complete-import time by **8.4%** against immediate
streaming. Combined with the previously merged removal of the counting pass,
the reduction is **14.5%**. Peak RSS across these runs was 619,260–622,340 KiB.
The counting baseline reconstructs the removed pass with the same locked
current dependencies; it is not a benchmark of the published 0.1.7 executable.

The worker alone made fresh imports slightly slower and used 22.9% more CPU
than serial buffer reuse. A second worker tested recycled batches at
`54cccfbd633cbb709d6d2f9ee1112537ad3e5f85`, using the same executable for its
serial and threaded modes:

| Variant | Median wall seconds | Range | Median CPU seconds |
| --- | ---: | ---: | ---: |
| Serial control | 265.72 | 259.51–296.38 | 264.04 |
| Recycled parser batches | 276.56 | 276.11–278.22 | 331.15 |

The recycled worker was 4.1% slower by median and used 25.4% more CPU. Compare
variants within each worker; their absolute timings are not interchangeable.
The complete [measurement receipt](benchmark-receipts/2026-09-11-import-cloud.json)
retains every sample, import report, executable/input hash and resource scope.

## Existing-database refreshes

Five additional full-size refreshes passed the same data checks. Each variant
has only one sample, so these are operational/correctness observations rather
than repeated performance estimates. The import timer excludes the private
seed copy; whole-process CPU, RSS and wall measurements include it.

On the first worker, the old serial parser took 359.71 seconds of import time,
buffer reuse plus the original parser worker took 320.32, and that worker with
a combined index rebuild took 274.51. The first comparison changes both parser
allocation and threading; it does not isolate the worker.

On the second worker, the same-binary serial/recycled pair took **397.06 /
341.46 seconds** of import time. Whole-process CPU was **370.76 / 399.67
seconds**, and seed copying took 5.07 / 3.52 seconds. The faster threaded refresh
is a useful signal, but it was a single serial-first pair. Repeat seeded
refreshes with rotated order before selecting threading for that workload.
These results do not establish a universal threading improvement or regression.

The production importer remains serial because the repeated fresh comparisons
regressed and the refresh signal still needs qualification. The experiments
remain available for that follow-up; they have not been discarded as evidence.

## Data and final-code verification

All **21 fresh comparisons and five seeded refreshes** produced equal complete
databases within their respective fresh/seeded groups, with zero parse or
insert errors. The two workers' independent references also match. The
[database references](benchmark-receipts/2026-09-11-import-database-references.json)
contain the exact schema, table counts and SHA3 digests of every stored field,
primary key and SQLite sequence value. Only generated `import_status.imported_at`
values are excluded after checking that each is a timestamp with a timezone.
The verifier runs SQLite quick_check and foreign_key_check and rejects nonempty WAL or
rollback journals before reading the completed private child-process output.

The selected code checkpoint is `3b2ad54e2505bcdd5430dae9001cfee44bb405fe`.
Its parser file is identical to the measured buffer-reuse variants, and
`crates/uls-db/src/importer.rs` is identical to the pre-existing serial importer.
All **697 workspace tests**, three doctests, current-stable workspace Clippy
and formatting pass. Three independent comparison-oracle tests cover a changed
FCC value, a missing index, and uncheckpointed WAL. Regressions preserve Unicode,
CRLF, continuation and raw-record behavior, duplicate-record order, late
stream-error rollback, and callback-panic restoration/retry. The existing
`unty-next` benchmark dependency moves from Rust-1.90-requiring 0.1.1 to compatible
0.1.2 so the locked all-feature build works on Rust 1.88.

The [exact-final-binary fresh and seeded checks](benchmark-receipts/2026-09-11-import-final.json)
both passed against the isolated references, with every value, index and
sequence equal and all source hashes unchanged. A WSL restart removed the
original temporary output; the rebuilt executable had the same SHA-256 as
before the restart:
`0e120928a96a1ea26a499d0d5b26a2d511d13b909e24f0860522a4981402939e`.
The isolated comparisons completed independently of WSL and their reports were
recovered and hash-verified from retained artifact generations.

## Reproduction

Build `cargo +1.88.0 build --release -p uls-db --example benchmark_import --locked`.
Use a separate source checkout **and Cargo target directory for each variant**.
An initial shared-target build reused an executable across source trees; those
artifacts were rejected before qualification. All qualified variants have
separately verified source and executable hashes.

```sh
python3 scripts/compare-imports.py --inputs /path/to/fixed/archives \
  --output /path/to/new/results --repetitions 3 \
  --variant baseline /path/to/baseline stream \
  --variant buffers /path/to/buffers stream
```

Use `--seed /path/to/completed/benchmark/import.db` for existing-data refreshes.
Every child gets a private writable copy. The driver verifies source, copy and
executable hashes before/afterward and removes only its generated databases,
unless `--keep-databases` is requested. Never use a serving database, overwrite
an existing output directory, or evict the host's global cache for this driver.
Run `python3 scripts/test_compare_imports.py` to check the comparison oracle.

The current example accepts `count-first` and `stream`. The Python driver also
accepts `pipeline` for historical executables. The
[prototype reconstruction patches](benchmark-receipts/2026-09-11-import-prototype-patches.json)
recreate either worker from the selected serial source. Join one variant's
`patch_lines` and apply it, then verify every recorded source-file hash before
building. Both reconstructions have been checked against the original hashes.
For the old-parser baseline, restore `crates/uls-parser/src/dat.rs` from
`a47e55127b837a5bc480bc1fdd4d0ae4ab2c3be1` in the original-worker checkout and
use serial mode. This keeps dependencies and instrumentation matched.

## Next: index ownership across services

Rebuilding indexes once saved **61.27 seconds**, or **23.4%**, against the
otherwise identical original-worker variant. The exact benchmark-only patch is
in the [original source receipt](benchmark-receipts/2026-09-11-import-provenance.json).
It omits successful intermediate index restoration and rebuilds in the example
after both full archives. It is not a production index-deferral option.

Adoption needs one owner spanning the complete service batch, restoring indexes
and PRAGMAs on later archive/metadata/progress failure while retaining archive
rollback, record order and committed source/status prefixes. Separate CLI
processes cannot share the current restoration guard. Qualify mixed weekly/daily
batches as well: applying dailies while indexes are absent changes the tradeoff.

SQLite sorting threads are a separate experiment. Bundled SQLite 3.51.1 sets
sorter workers to zero when temporary storage is in memory. The importer uses
`PRAGMA temp_store=MEMORY`, so `PRAGMA threads=2` alone would not parallelize
sorting. File-backed sorting needs its own I/O/memory comparison. See the
[thread limit](https://www.sqlite.org/pragma.html#pragma_threads) and
[sorter source](https://github.com/sqlite/sqlite/blob/version-3.51.1/src/vdbesort.c).

The [parser-only handoff diagnostic](benchmark-receipts/2026-09-11-parser-handoff.json)
and [earlier workstation refresh check](benchmark-receipts/2026-09-11-import-wsl-refresh.json)
remain supplementary evidence. Recycling reduced handoff CPU in the diagnostic;
that improvement did not carry through to repeated fresh imports. Shared-host
workstation timings are not used for the selected performance claims.
