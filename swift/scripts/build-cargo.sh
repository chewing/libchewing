#!/usr/bin/env bash
set -euo pipefail

# Build the chewing_capi Rust crate for use with Swift Package
# Outputs static library and artifacts into `target/cargo-target/release`.

# NOTE: script moved under `swift/scripts`, so go up two levels to reach repo root
REPO_ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
CARGO_MANIFEST_PATH="$REPO_ROOT/capi/Cargo.toml"
TARGET_DIR="$REPO_ROOT/target/cargo-target"

echo "Building chewing_capi (release) -> $TARGET_DIR"
mkdir -p "$TARGET_DIR"

cargo build --release --manifest-path "$CARGO_MANIFEST_PATH" --target-dir "$TARGET_DIR"

echo "Done. Built artifacts are in $TARGET_DIR/release"