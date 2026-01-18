#!/usr/bin/env bash
# Wrapper for cross that uses a separate target directory per target.
# This avoids GLIBC mismatch errors when switching between targets.
# See: https://github.com/cross-rs/cross/wiki/FAQ#glibc-version-error
#
# Usage:
#   ./scripts/cross.sh build --release --target x86_64-pc-windows-msvc
#   ./scripts/cross.sh build --release --target x86_64-unknown-linux-musl

set -euo pipefail

# Extract target from arguments
TARGET=""
for arg in "$@"; do
    if [[ "$arg" == --target=* ]]; then
        TARGET="${arg#--target=}"
        break
    fi
done

# Handle --target <value> (space-separated)
if [[ -z "$TARGET" ]]; then
    prev=""
    for arg in "$@"; do
        if [[ "$prev" == "--target" ]]; then
            TARGET="$arg"
            break
        fi
        prev="$arg"
    done
fi

if [[ -z "$TARGET" ]]; then
    echo "Error: --target is required" >&2
    echo "Usage: $0 build --release --target <target>" >&2
    exit 1
fi

export CARGO_TARGET_DIR="target/${TARGET}"
echo "Using target directory: ${CARGO_TARGET_DIR}"
exec cross "$@"
