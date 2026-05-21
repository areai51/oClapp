#!/bin/bash
set -euo pipefail

cd "$(dirname "$0")/.."

echo "=== Building oClapp for macOS ==="

# Install dependencies
cd frontend
npm install
cd ..

# Build universal binary (ARM64 + x86_64)
echo "Building ARM64..."
rustup target add aarch64-apple-darwin
cargo tauri build --target aarch64-apple-darwin

echo "Building x86_64..."
rustup target add x86_64-apple-darwin
cargo tauri build --target x86_64-apple-darwin

echo "=== macOS build complete ==="
echo "Artifacts:"
ls -la target/aarch64-apple-darwin/release/bundle/dmg/*.dmg 2>/dev/null || true
ls -la target/x86_64-apple-darwin/release/bundle/dmg/*.dmg 2>/dev/null || true
