# oClapp

A cross-platform desktop app for local LLM inference, powered by llama.cpp.

## Features

- **Model Browser**: Search and download GGUF models from HuggingFace
- **Local Inference**: Run an OpenAI-compatible API server via llama.cpp
- **Coding Agent Integration**: Launch Claude Code, Codex CLI, and other agents with `oClapp launch`
- **Settings UI**: Curated and advanced parameter tuning with persistence
- **Cross-Platform**: Native desktop app for macOS, Linux, and Windows

## Architecture

```
Cargo Workspace
├── oclapp-core    # Shared logic: models, server lifecycle, settings
├── oclapp-cli     # CLI binary: `oClapp launch`, `oClapp serve`
└── oclapp-tauri   # Desktop app: Tauri v2 + React + TypeScript
```

## Development

### Prerequisites

- [Rust](https://rustup.rs/) (latest stable)
- [Node.js](https://nodejs.org/) 20+
- Platform-specific dependencies for Tauri:
  - **macOS**: Xcode Command Line Tools
  - **Linux**: `libgtk-3-dev`, `libwebkit2gtk-4.1-dev`, `libappindicator3-dev`, `librsvg2-dev`
  - **Windows**: Visual Studio Build Tools with C++ workload

### Setup

```bash
# Clone the repository
git clone https://github.com/your-org/oclapp.git
cd oclapp

# Install frontend dependencies
cd frontend && npm install && cd ..

# Build the entire workspace
cargo build
```

### Running the Desktop App

```bash
# Development mode (hot reload)
cargo tauri dev

# Production build
cargo tauri build
```

### Running the CLI

```bash
# List local models
cargo run --bin oclapp -- models

# Start the inference server
cargo run --bin oclapp -- serve --model llama-2-7b

# Launch Claude Code with a model
cargo run --bin oclapp -- launch claude --model llama-2-7b
```

## Distribution

Platform-specific build scripts are provided in `scripts/`:

```bash
# macOS
./scripts/build-macos.sh

# Linux
./scripts/build-linux.sh

# Windows
.\scripts\build-windows.ps1
```

CI builds are configured via GitHub Actions (`.github/workflows/build.yml`) and triggered on pushes to `main` and version tags.

## License

MIT
