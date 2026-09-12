# Coordinated exact-date imports

```sh
uls update --service all --through 2026-09-11 --format json
```

This applies Amateur (`HA`) followed by GMRS (`ZA`) to the same FCC source
date. `all` currently requires `--through`; automatic target selection remains
the caller's responsibility. Use the existing per-service `--plan --format json`
commands to find dates reachable by both services. `--minimal` is supported.
`--plan`, `--check`, `--daily-only` and `--force` cannot accompany `--through`.

Both routes are planned before opening the database for writes. If either
cannot reach the exact date, an absent database remains absent. Both source
bases are checked again before importing. No service can run ahead of the
requested date; later daily archives beyond a gap remain unapplied.

## Result contract

Success emits exactly one JSON document. Progress goes to stderr. The outer
document has these fields:

| Field | Value |
| --- | --- |
| `format` | `uls.update_batch_result` |
| `format_version` | `1` |
| `target_source_date` | Requested `YYYY-MM-DD` |
| `services` | Exactly two existing `uls.update_result` documents: HA, then ZA |

Each service document retains version 1 and its existing fields:
`service_code`, `previous_source_date`, `source_date`, `target_source_date`,
`changed`, `route`, `weekly_applied` and `daily_updates_applied`, along with
`format` and `format_version`. Both source dates must equal the batch target.
A service already at that date reports `changed: false`, `route: "none"`, no
weekly import and zero daily updates. A retry may therefore combine a no-op
for one service with an import for the other.

An error exits nonzero and emits no successful batch document. Consumers must
check the exit status, one-document envelope, version, service identities/order
and exact dates before accepting the result. The single-service result format
is unchanged.

## Transactions and cleanup

Each archive has its own transaction, including its records, import-status
rows and source metadata. A failed archive rolls back; earlier committed
archives remain available for retry. This is not an all-or-nothing transaction
across both services. Software requiring atomic publication should import into
a private staging database, validate it, then promote it through its existing
publication mechanism.

`uls-db` exposes `Importer::batch`, which owns one connection and scoped
index/PRAGMA restoration. Its callback receives an `ImportBatch`; callers must
use that handle for imports and own exclusive access to their staging database.
Weekly operations defer intermediate index rebuilding. Empty and daily-only
batches keep their indexes and durability settings. Normal success/error paths
restore and verify the original settings before returning; restoration errors
are reported alongside an original operation error. Panic cleanup also attempts
restoration and logs any failure. Abrupt process termination retains the same
disposable-staging requirement as existing full imports.

Full-size and mixed-route results are in
[the qualification report](coordinated-import-performance.md), with integration
gates in [the implementation record](coordinated-import-work.md). Merging source
does not release a new CLI version or update a downstream service image.
