# Coordinated importer qualification

The scoped service batch avoids rebuilding shared indexes between Amateur and
GMRS. These September 11 workstation comparisons use full FCC archives;
they are not measurements of deployed service latency.

## Repeated comparisons

Three repetitions per variant used fixed August 9 FCC archives, rotated variant
order, pre-read archive files, and a new private output database every time.
Refreshes copied a completed baseline import before processing the same archives.
The importer remained serial. WSL reported CPUs 18 and 19 as a sibling pair;
process affinity was limited to that pair and imports ran in a private network
namespace. Other workstation activity was not globally stopped.

| Workload | Separate rebuilds | Combined rebuild | Import-time reduction | Whole-process CPU reduction |
| --- | ---: | ---: | ---: | ---: |
| Fresh database | 117.59 s | 94.40 s | 19.7% | 14.8% |
| Reimport into a completed database | 186.26 s | 147.00 s | 21.1% | 22.1% |
| Reimport with synthetic daily suffixes | 181.67 s | 135.60 s | 25.4% | 20.7% |

Values are medians. Import time includes database initialization, all archives,
and final index/PRAGMA restoration, but excludes seed copying and verification.
Whole-process CPU includes seed copying. The raw receipts also retain complete
process wall times and peak memory; these scopes must not be interchanged.

Fresh batch repetition 2 took 161.05 seconds of whole-process wall time while
its import timer recorded 92.87 seconds. Its cause remains unproven; the Windows
event-log query found no matching sleep/resume event. The sample is retained
without correction or exclusion. Shared-host conditions and this timing gap
limit extrapolation to other hardware and production updates.

The weekly controls use the previously selected buffer-reuse serial importer.
The mixed control uses the same candidate binary with a separate batch per
archive. Mixed order is HA weekly, HA daily, ZA weekly, ZA daily. Its two daily
ZIPs contain 10,196 records selected deterministically from the fixed weeklies,
including related records and original ordering. They are synthetic replay
inputs, not an observed FCC daily workload. Daily source-metadata behavior is
covered separately by CLI/library regressions.

In a mixed batch, daily patches inherit the enclosing weekly bulk settings and
its deferred indexes. Daily-only batches retain normal indexes and durability.
The measured mixed workload supports keeping deferral across those daily
suffixes; it does not justify changing independent daily imports.

## Correctness and final source

All 18 comparison databases match their corresponding independent control in
stored values, primary keys, sequence state and schema. Every database passes
SQLite quick-check and foreign-key checks; all import counters match with zero
parse or insertion errors. Import-status wall-clock timestamps are validated
but excluded from row digests. Original inputs, private copies and binaries
remain hash-identical.

Review found that retrying failed setup could replace the saved settings. The
final guard now retains its original SQLite settings snapshot. The regression
first reproduced a leaked synchronous=OFF setting, then passed with the fix.
All 716 workspace tests, three doctests, formatting, Clippy and the comparison
oracle checks pass. The final binary separately matches the fresh, seeded and
mixed independent references. Those three final samples establish equivalence;
they are not another repeated performance comparison.

The first final fresh sample was collected during coverage compilation and was
slower than the repeated-comparison candidate. A September 12 check compared
separate and combined modes in the same final executable after compilation
finished: 112.36 versus 88.45 seconds of import time, with 110.78 versus 87.37
seconds of whole-process CPU. Both databases match the independent fresh
reference. This single pair supports retaining the final implementation; it
does not replace the repeated comparison or establish production latency.

The benchmark exercises the library's full import path. It excludes FCC HTTP
planning, private service validation, activation and publication. Source-file
and binary hashes distinguish the repeated-comparison candidate from the final
retry fix. Packaging and deployment require the reviewed final CLI source;
merging source alone does not update a service image.

Raw measurements, provenance, all input/binary hashes and final qualification
checks are in [the comparison receipt](benchmark-receipts/2026-09-11-combined-import.json).
The [database references](benchmark-receipts/2026-09-11-combined-import-database-references.json)
retain the complete compared schemas, row counts and stored-value hashes.
