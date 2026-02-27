# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.4](https://github.com/tgies/uls/compare/v0.1.3...v0.1.4) - 2026-02-27

### Fixed

- *(cli)* correct update command in staleness warning

### Other

- link online demo

## [0.1.3](https://github.com/tgies/uls/compare/v0.1.2...v0.1.3) - 2026-02-23

### Fixed

- *(db)* prefer most recently granted record when no active license exists

## [0.1.2](https://github.com/tgies/uls/compare/v0.1.1...v0.1.2) - 2026-02-23

### Fixed

- *(update)* include Sunday daily files and fix chain gap detection

## [0.1.1](https://github.com/tgies/uls/compare/v0.1.0...v0.1.1) - 2026-02-23

### Fixed

- *(db)* prefer active license in callsign lookup

### Other

- switch to per-crate versioning for independent releases

## [0.1.0] - 2026-02-21

### Added

- Quick callsign lookups (`uls W1AW`)
- Advanced search by name, location, FRN, and more (`uls search`)
- FRN lookup (`uls frn`)
- Automatic FCC data downloads with weekly full and daily delta updates (`uls update`)
- Local SQLite database with optimized bulk import (~147K records/sec)
- REST API server (`uls serve`) with axum
- Multiple output formats: table, JSON, CSV, YAML
- Support for Amateur Radio (HA/HV) and GMRS (ZA) radio services
- Parser for all 89 FCC ULS pipe-delimited record types
- ETag-based HTTP caching for efficient downloads
- Database management commands (`uls db init`, `uls db info`, `uls db vacuum`)
- Database statistics (`uls stats`)
- Cross-platform binary releases for Linux (x86_64, aarch64), macOS (x86_64, aarch64), and Windows (x86_64)

[0.1.0]: https://github.com/tgies/uls/releases/tag/v0.1.0
