# AGENTS.md

This file provides guidance to AI coding assistants when working with code in this repository.

## Project Overview

ULS is a Rust CLI tool for querying FCC Universal Licensing System public data. It downloads FCC license data, stores it in a local SQLite database, and provides fast offline queries for amateur radio and GMRS licenses.

## Crate Architecture

```
uls-cli      CLI entry point (lookup, search, frn, update, db, stats, serve, auto_update)
    ↓
uls-query    Query engine with filtering and output formatting (json, csv, yaml, table)
    ↓
uls-db       SQLite layer (schema, bulk import, queries, r2d2 connection pooling)
    ↓
uls-parser   Parser for FCC's pipe-delimited DAT files from ZIP archives
    ↓
uls-core     Core types: 89 ULS record types, radio service codes, error types

uls-download   Async HTTP client for FCC data (weekly/daily downloads, ETag caching)
uls-api        REST API server (axum-based)
```

**Dependency flow:** cli → query → db → parser → core. Download crate is used by cli for updates.

## Build & Test Commands

```bash
# Build
cargo build                              # Development build
cargo build --release                    # Release build (LTO enabled)

# Test (uses cargo-nextest)
cargo nextest run --workspace --all-features    # All tests
cargo nextest run -p uls-core                   # Single crate
cargo test --doc --workspace                    # Doctests only

# Code quality (CI runs these with -D warnings)
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings

# Run from source
cargo run -p uls-cli -- lookup W1AW
```

## CLI Commands

| Command | Description |
|---------|-------------|
| `uls <CALLSIGN>` | Quick callsign lookup |
| `uls search` | Advanced search (name, location, FRN, etc.) |
| `uls frn <FRN>` | Look up licenses by FCC Registration Number |
| `uls update` | Download and update local database |
| `uls stats` | Database statistics |
| `uls serve` | Start REST API server |
| `uls db` | Database management (init, info, vacuum) |

## Data Flow

1. **Download** → `uls-download` fetches ZIP files from FCC servers (weekly full + daily deltas)
2. **Parse** → `uls-parser` reads pipe-delimited DAT files
3. **Ingest** → `uls-db` inserts records into SQLite
4. **Query** → `uls-query` constructs SQL from user input
5. **Output** → Results formatted as table, JSON, CSV, or YAML

## Key Directories

- `crates/*/src/` - Source code for each crate
- `crates/*/tests/` - Integration tests
- `tests/fixtures/fcc-sample/` - Sample FCC data (l_amat, l_gmrs)
- `scripts/` - Utility scripts (including `extract_test_fixture.py` for regenerating fixtures)
- `fcc-docs/` - FCC reference documentation

## Environment Variables

- `ULS_DB_PATH` - Database location (default: `~/.uls/uls.db`)
- `ULS_CACHE_DIR` - Download cache (default: `~/.uls/cache`)

## FCC Record Types

Records are pipe-delimited in DAT files. Common types:
- **HD** - Header
- **EN** - Entity
- **AM** - Amateur
- **LA** - Location/Antenna
- **CO** - Control Point
- **HS** - History
- **SC** - Special Conditions

All 89 record types are defined in `uls-core/src/records/`.

## Coding Conventions

- Rust 2021 edition, MSRV 1.78
- `cargo fmt` for formatting (4-space indentation)
- `snake_case` for modules/functions/files, `CamelCase` for types/traits, `SCREAMING_SNAKE_CASE` for constants
- Shared record models in `uls-core`, output formatting in `uls-query`
- Error handling: `anyhow` for applications, `thiserror` for libraries
- Logging: `tracing` with `-v` flags for verbosity
- Async: `tokio` for I/O operations

## Commit Guidelines

Follow Conventional Commits format seen in history:
- `feat(cli): add new search flag`
- `fix(db): handle null values in import`
- `test(cli): add integration coverage`
- `chore: cargo fmt`

Keep summaries short and scoped.
