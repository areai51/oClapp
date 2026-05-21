#!/bin/bash
set -euo pipefail

cd "$(dirname "$0")/.."

echo "=== Building oClapp for Linux ==="

# Install system dependencies
if command -v apt-get >/dev/null 2>&1; then
    sudo apt-get update
    sudo apt-get install -y libgtk-3-dev libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf
elif command -v dnf >/dev/null 2>&1; then
    sudo dnf install -y gtk3-devel webkit2gtk4.1-devel libappindicator-gtk3-devel librsvg2-devel
elif command -v pacman >/dev/null 2>&1; then
    sudo pacman -S --needed gtk3 webkit2gtk-4.1 libappindicator-gtk3 librsvg
fi

# Install frontend dependencies
cd frontend
npm install
cd ..

# Build
cargo tauri build

echo "=== Linux build complete ==="
echo "Artifacts:"
ls -la target/release/bundle/appimage/*.AppImage 2>/dev/null || true
ls -la target/release/bundle/deb/*.deb 2>/dev/null || true
ls -la target/release/bundle/rpm/*.rpm 2>/dev/null || true
