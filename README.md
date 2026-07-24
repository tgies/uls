# ULS - FCC Universal Licensing System CLI Tool

A fast, robust command-line tool for querying FCC ULS public data.

**[Try it online](https://wrck692.net/lookup/)** | `cargo install uls-cli`

## Features

- **Quick callsign lookups** - `uls W1AW`
- **Automatic data updates** - Downloads and applies FCC weekly/daily updates
- **Local database** - SQLite for fast offline queries
- **REST API mode** - Serve ULS data over HTTP
- **Multiple output formats** - JSON, CSV, YAML, table

## Quick Start

```bash
# Install
cargo install uls-cli

# Initialize database with amateur radio data
uls update --service amateur

# Look up a callsign
uls W1AW

# Search by name
uls search --name "ARRL"

# JSON output
uls W1AW --format json
```

## Supported Radio Services

Currently prioritized:
- **Amateur Radio (HA/HV)** - Full support
- **GMRS (ZA)** - Full support

Additional FCC radio services are planned for future releases.

## Commands

| Command | Description |
|---------|-------------|
| `uls <CALLSIGN>` | Quick callsign lookup |
| `uls lookup <CALLSIGN>` | Detailed license information |
| `uls search` | Search by name, location, etc. |
| `uls update` | Download/update database |
| `uls stats` | Database statistics |
| `uls serve` | Start REST API server |
| `uls download` | Download FCC files without building database |
| `uls completions <SHELL>` | Generate shell completions (bash, zsh, fish, elvish, powershell) |

### Safe update planning

Automation can inspect the exact FCC source dates that are currently reachable
without changing the database:

```bash
uls update --service amateur --plan --format json
```

The versioned `uls.update_plan` document includes the current weekly and source
dates, every exactly reachable source date, the recommended date, observed FCC
archive dates, and any gap in the current daily path. The reachable dates may be
disjoint when a newer weekly snapshot safely bridges missing daily archives.

After coordinating a common date across services, apply no farther than that
date and require the database to reach it:

```bash
uls update --service amateur --through 2026-07-23 --format json
```

An unreachable target is rejected before database initialization or migration.
Each imported archive is transactional. If a later archive fails, the command
returns an error while retaining only the earlier, valid contiguous prefix.

## Configuration

Environment variables:
- `ULS_DB_PATH` - Database location (default: `~/.uls/uls.db`)
- `ULS_CACHE_DIR` - Download cache (default: `~/.uls/cache`)

## Building from Source

```bash
git clone https://github.com/tgies/uls
cd uls
cargo build --release
```

### Requirements

- Rust 1.88+

## Data Sources

This tool uses public data from the [FCC Universal Licensing System](https://www.fcc.gov/wireless/data/public-access-files-database-downloads):

- **Weekly databases**: Complete snapshots updated each Sunday
- **Daily transactions**: Incremental updates Monday-Saturday

## License

MIT OR Apache-2.0

## Contributing

Contributions welcome! Please read the contributing guidelines first.
