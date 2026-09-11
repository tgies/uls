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
- [ ] Run relevant regressions, workspace tests, formatting, and Clippy.
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
