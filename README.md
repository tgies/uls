# ULS - FCC Universal Licensing System CLI Tool

A fast, robust command-line tool for querying FCC ULS public data.

## Features

- 🔍 **Quick callsign lookups** - `uls W1AW`
- 📥 **Automatic data updates** - Downloads and applies FCC weekly/daily updates
- 🗄️ **Local database** - SQLite for fast offline queries
- 🌐 **REST API mode** - Serve ULS data over HTTP
- 📊 **Multiple output formats** - JSON, CSV, YAML, table

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

All 142+ FCC radio services will be supported in future releases.

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

- Rust 1.75+
- SQLite 3.x (bundled)

## Data Sources

This tool uses public data from the [FCC Universal Licensing System](https://www.fcc.gov/wireless/data/public-access-files-database-downloads):

- **Weekly databases**: Complete snapshots updated each Sunday
- **Daily transactions**: Incremental updates Monday-Saturday

## License

MIT OR Apache-2.0

## Contributing

Contributions welcome! Please read the contributing guidelines first.
