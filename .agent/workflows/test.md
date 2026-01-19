---
description: Run comprehensive "did we break anything" checks (fmt, clippy, tests, docs)
---

# Test Workflow

Run all quality checks to verify the codebase is in good shape.

// turbo-all

## Steps

1. Check formatting:
```bash
cargo fmt --check
```

2. Run clippy with warnings as errors:
```bash
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

3. Run all tests:
```bash
cargo nextest run --workspace --all-features
```

4. Build documentation (checks for doc errors):
```bash
cargo doc --workspace --no-deps
```
