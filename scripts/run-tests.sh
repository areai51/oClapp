#!/usr/bin/env bash
set -euo pipefail

echo "=================================="
echo "oClapp Test Harness"
echo "=================================="
echo ""

# Run all workspace tests
echo "Running workspace tests..."
cargo test --workspace 2>&1

echo ""
echo "=================================="
echo "All tests passed."
echo "=================================="
