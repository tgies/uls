---
description: Run benchmarks (criterion, gungraun) and generate flamegraphs
---

# Benchmark Suite

Run benchmarks using Criterion (wall-clock), Gungraun (instruction counts), or generate flamegraphs.

## 1. Quick Criterion benchmarks (all crates)

```bash
cargo bench --workspace
```

This runs all criterion benchmarks and generates HTML reports in `target/criterion/`.

## 2. Run a specific benchmark

```bash
cargo bench --bench parser_bench
cargo bench --bench db_bench
cargo bench --bench query_bench
```

## 3. Gungraun instruction counts (requires valgrind)

```bash
cargo bench --bench parser_gungraun
```

Outputs precise CPU instruction counts. Useful for CI to detect performance regressions.

## 4. Generate a flamegraph

```bash
cargo flamegraph --bench parser_bench -- --bench "parse_line"
```

Produces `flamegraph.svg` in the project root. Open in browser to explore.

## 5. Profile a CLI command

```bash
cargo flamegraph -- lookup W1AW
```

## Notes

- **Reports**: Open `target/criterion/report/index.html` for benchmark history and charts
- **Flamegraph**: Requires `linux-tools` (`perf`) to be installed
- **Gungraun**: Requires `valgrind` to be installed; Linux/macOS only
