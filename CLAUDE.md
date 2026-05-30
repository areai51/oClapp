# oClapp — Claude Code Notes

## Test Harness

Always run the workspace test suite before declaring a build complete or a fix verified:

```bash
cargo test --workspace
```

Or use the provided script:

```bash
./scripts/run-tests.sh
```

The harness covers 36 tests across model discovery, binary resolution, server config, download progress, settings roundtrip, and Tauri IPC validation. If you change any of these areas, the tests will catch regressions without needing manual screenshots.

### Test files to know
- `crates/oclapp-core/tests/model_discovery.rs` — GGUF / MLX / Safetensors / HF dir scanning
- `crates/oclapp-core/tests/binary_discovery.rs` — `llama` and `llama-server` PATH / well-known-dir lookup
- `crates/oclapp-core/tests/server_config.rs` — CLI arg generation for `llama serve` / `llama-server`
- `crates/oclapp-core/tests/download.rs` — progress callback behavior
- `crates/oclapp-tauri/src/main.rs` (`#[cfg(test)]`) — settings serde roundtrip, HF model rejection logic
- `crates/oclapp-tauri/tests/tauri_commands.rs` — `ServerManager` state, `resolve_model_path` integration

## Common Gotchas

- `cargo tauri build` requires `frontend/dist` to exist. The `frontendDist` path in `tauri.conf.json` is relative to `crates/oclapp-tauri/` (needs `../../frontend/dist`).
- Tauri v2 `.setup()` has **no Tokio runtime** — use `tauri::async_runtime::spawn`, never `tokio::spawn`.
- The bundled macOS app does **not** inherit the shell's `PATH`. Binary discovery checks well-known dirs (`~/.local/bin`, `~/.llama-app`, `/opt/homebrew/bin`) in addition to `PATH`.
- llama.cpp only loads **GGUF** single-file models. HuggingFace safetensors directories must be converted first or downloaded as GGUF from HuggingFace.
- The `download_from_hf` backend downloads directly from the HF CDN (`https://huggingface.co/{repo}/resolve/main/{file}`) with byte-level progress events — it does not use `hf_hub::Api` caching.
