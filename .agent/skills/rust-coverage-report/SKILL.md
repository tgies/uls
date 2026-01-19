---
name: rust-coverage-report
description: Generate comprehensive code coverage reports for Rust workspaces using cargo-llvm-cov and nextest. Produces formatted markdown tables with per-crate and per-file coverage breakdowns, including collapsible detail sections. Use when the user asks for test coverage, coverage reports, or wants to analyze which parts of their Rust codebase lack tests.
compatibility: Requires cargo-llvm-cov, cargo-nextest, and Python 3. Works with Cargo workspaces.
metadata:
  author: tgies
  version: "1.0"
---

# Rust Coverage Report Skill

This skill generates detailed code coverage reports for Rust projects using `cargo-llvm-cov` and `cargo-nextest`.

## Prerequisites

Ensure these tools are installed:

```bash
cargo install cargo-llvm-cov
cargo install cargo-nextest
```

## Workflow

### 1. Run coverage collection

Generate coverage data in JSON format:

```bash
cargo llvm-cov nextest --workspace --all-features --json --output-path coverage.json
```

**Options:**
- Use `-p <crate>` to limit to a specific crate
- Remove `--all-features` if you want default feature coverage only
- Add `--ignore-filename-regex <pattern>` to exclude files

### 2. Generate the report

Run the included Python script to parse the JSON and produce a markdown report:

```bash
python3 scripts/generate_coverage_report.py coverage.json
```

This outputs:
- A **summary table** showing coverage by crate
- **Detailed tables** for each crate showing per-file coverage

### 3. Format the output

When presenting results to the user, use this structure:

#### Summary by Crate (always visible)

```markdown
| Crate | Covered Lines | Total Lines | Percentage |
| :--- | :--- | :--- | :--- |
| crate-name | 1387 | 1397 | **99.28%** |
```

#### Per-Crate Details (use collapsible sections)

```markdown
<details>
<summary><b>crate-name (99.28%)</b></summary>

| File | Covered Lines | Total Lines | Percentage |
| :--- | :--- | :--- | :--- |
| src/lib.rs | 263 | 265 | 99.25% |
| src/utils.rs | 234 | 234 | 100.00% |
</details>
```

## Interpreting Results

| Coverage Level | Interpretation |
| :--- | :--- |
| 90%+ | Excellent - well-tested code |
| 75-90% | Good - most paths covered |
| 50-75% | Fair - significant gaps exist |
| <50% | Needs attention - many untested paths |

### Common Low-Coverage Patterns

- **0% on a file**: Often indicates placeholder/stub code or entirely untested modules
- **CLI command handlers**: Often lower coverage due to integration test limitations
- **Error paths**: Easy to miss; consider adding error injection tests

## Optional: Running alongside /test workflow

If you have a `/test` workflow, run coverage after confirming all tests pass:

```bash
# First verify tests pass
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo nextest run --workspace --all-features

# Then collect coverage
cargo llvm-cov nextest --workspace --all-features --json --output-path coverage.json
python3 scripts/generate_coverage_report.py coverage.json
```

## Cleanup

Remove generated files when done:

```bash
rm -f coverage.json
```
