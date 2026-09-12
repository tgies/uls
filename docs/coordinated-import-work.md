# Coordinated service import work

The September 11 weekly-only prototype saved an intermediate index rebuild.
Production adoption must keep archive transactions and exact source coverage,
including when one service succeeds and a later archive or metadata write fails.

## Contract

- Add `uls update -r all --through YYYY-MM-DD`, with one versioned batch result
  containing the existing per-service result documents in Amateur/GMRS order.
- Plan both exact routes before opening the database for writes. Reject an
  unreachable target without initializing a missing database. Recheck both
  planned bases before importing either service.
- Own one database connection and index/PRAGMA cleanup within a scoped batch;
  expose no cross-process option that can leave indexes absent after exit.
- Preserve service, file, record and archive order. Commit each archive together
  with its import status and source metadata. A failed archive rolls back while
  earlier successful archives remain a valid contiguous prefix for retry.
- Defer rebuilds between weekly archives. Daily-only and no-op batches must not
  drop indexes or weaken durability. Measure mixed routes before selecting how
  far deferral should extend across daily work.
- Report restoration errors alongside the original failure and retain panic
  cleanup. Print a successful batch result only after restoration and exact
  target checks succeed.
- Integrate the private staging wrapper with strict envelope/per-service
  validation; preserve all planning, staging, activation and rollback gates.

## Checklist

- [x] Scoped library batch and atomic archive metadata.
- [x] Exact-target CLI batch and failure/retry regressions.
- [x] Private wrapper integration and rejection tests.
- [x] Fixed-input fresh, seeded and mixed-route comparisons, with full database
      value/schema verification and explicit measurement scope.
- [x] Workspace tests, doctests, formatting, Clippy and affected shell checks.
- [x] Signed source checkpoints, CI, updated handoffs and deployment packaging.

All 23 comparisons, including repeated workloads, three final binary checks
and the final confirmation pair, match their independent database references.
The final retry guard passes all 716 workspace tests; hosted coverage is
98.39% for the patch and 97.59% overall. See
[the qualification report](coordinated-import-performance.md) for scopes,
timing limits and raw receipts.

PR #73 merged as `451466e2e1f9d4dae8ab91cfea277f7373e7bd71`, with the exact
tree from signed final commit `948497ac4cb6eeab28abd6eb9d77922840aef025`.
Final PR and main CI passed. No mutation survived; one timed-out scenario had
already failed its expected regression assertion. The downstream integration
was packaged from this pinned revision and deployed on September 12, with
its bundled batch command and public readiness verified. Production import
timings await normal scheduled updates.

Release-plz remains excluded. Source integration and measured candidate results
must remain distinct from release and deployment receipts.
