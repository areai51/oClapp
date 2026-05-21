---
title: oClapp Desktop App — Tauri + llama.cpp Wrapper
type: feat
status: active
date: 2026-05-21
origin: docs/brainstorms/oclapp-requirements.md
---

# oClapp Desktop App — Tauri + llama.cpp Wrapper

## Summary

Implement oClapp as a Cargo workspace containing a shared Rust core library, a CLI binary, and a Tauri v2 desktop app with a React frontend. The core library manages llama.cpp binary downloads, HuggingFace model discovery and download, server lifecycle, and settings persistence. The CLI exposes `oClapp launch <tool> --model <name>` for coding agent integration. The Tauri app provides a model browser, download manager, settings UI, and server status dashboard. The llama.cpp server binary is downloaded on first run rather than bundled.

---

## Problem Frame

oMLX is a Python + MLX-based inference server that only supports Apple Silicon Macs and MLX-native model formats. Users who need GGUF models or llama.cpp features like MTP have no supported path. Additionally, the oMLX UI and menu bar app suffer from reliability issues. A Rust-based desktop alternative would provide broader model support, cross-platform compatibility, and a more robust native experience. This plan addresses the technical implementation of that alternative.

(see origin: docs/brainstorms/oclapp-requirements.md)

---

## Requirements

- R1–R5. Model management (discovery, download, local storage, format support)
- R6–R8. Inference server lifecycle and OpenAI-compatible API exposure
- R9–R11. CLI launch command with environment injection
- R12–R14. Settings UI with curated/advanced parameter exposure and persistence
- R15. Cross-platform support (macOS, Linux, Windows)

**Origin actors:** A1 (End User), A2 (Coding Agent Tool), A3 (llama.cpp Server)
**Origin flows:** F1 (Model Discovery and Download), F2 (Launch Coding Agent), F3 (Configure Inference Settings)
**Origin acceptance examples:** AE1–AE5

---

## Scope Boundaries

### Deferred for later

- Custom Rust inference engine replacing llama.cpp server child process
- Built-in benchmarking or evaluation tools
- MCP server integration
- Admin dashboard web UI (Tauri desktop app replaces this)
- Auto-updater mechanism

### Outside this product's identity

- Mobile or web deployment
- Cloud-hosted inference or SaaS offering
- Multi-user collaboration or shared model libraries
- Model training or fine-tuning capabilities

### Deferred to Follow-Up Work

- Frontend chat interface for direct interaction with the loaded model (not required for coding agent integration)
- Model quantization or conversion utilities
- Plugin/extension system for third-party model sources

---

## Context & Research

### Relevant Code and Patterns

This is a greenfield project with no existing Rust code. The repository currently contains only requirements and architecture documents (see origin).

### External Research Findings

**llama.cpp server (llama-server):**
- Default bind: `127.0.0.1:8080` (`http://localhost:8080/v1`)
- Key CLI arguments: `-m` (model), `-c` (ctx-size), `--port`, `--host`, `-ngl` (gpu-layers), `-np` (parallel), `--api-key`, `-a` (alias), `--jinja`, `--embedding`, `-fa` (flash-attn)
- Endpoints: `GET /v1/models`, `POST /v1/completions`, `POST /v1/chat/completions`, `POST /v1/responses`, `POST /v1/embeddings`
- Responses API support is partial; PR #21174 targets full Codex CLI compatibility
- Function calling requires `--jinja` and supported model families
- No strong OpenAI spec compatibility claims; wrapper app must tolerate variance

**Tauri v2 patterns:**
- Sidecars are build-time only via `bundle.externalBin`; runtime downloads must use custom `reqwest` + `std::process::Command`
- `tauri-plugin-store` is the official settings persistence mechanism
- IPC: Commands (`invoke`) for RPC, Events (`emit`/`listen`) for notifications
- macOS notarization has a known bug with `externalBin` sidecars (Issue #11992); runtime download avoids this entirely
- WebView CORS blocks direct `fetch()` to `localhost`; frontend should communicate with server only through Rust backend commands

### Institutional Learnings

None — this repository has no `docs/solutions/` entries.

---

## Key Technical Decisions

- **Cargo workspace with three crates.** Rationale: Separates shared logic (`oclapp-core`) from the CLI (`oclapp-cli`) and Tauri app (`oclapp-tauri`), enabling independent builds and clear dependency boundaries.
- **llama.cpp downloaded at runtime via custom Rust routine.** Rationale: User chose download-on-first-run over bundled binaries. This avoids macOS notarization issues with sidecars and keeps initial app bundle small. Uses `reqwest` to download platform-appropriate binaries into the app's local data directory.
- **Tauri v2 with React + TypeScript frontend.** Rationale: Tauri provides native desktop APIs from Rust. React + TypeScript is well-supported and familiar. The frontend communicates with the Rust backend exclusively through Tauri Commands and Events.
- **Settings persisted via `tauri-plugin-store`.** Rationale: Official Tauri v2 plugin for JSON settings persistence. Works from both Rust and JS with async get/set.
- **HuggingFace integration via `hf-hub` crate.** Rationale: Official Rust client for HuggingFace Hub. Supports model search, metadata, and resumable downloads.
- **Server managed via `std::process::Command`, not Tauri sidecar.** Rationale: The llama.cpp binary is downloaded at runtime, so it cannot be a build-time sidecar. The core library wraps process spawning, stdout/stderr streaming, and graceful shutdown.
- **Curated settings default.** Rationale: Temperature, Top P, Context Size, Max Tokens, and GPU Layers cover the most common tuning needs. Advanced panel exposes Seed, Repeat Penalty, Frequency/Presence Penalty, Batch Size, Threads, Flash Attention, MMAP, and MLOCK.

---

## Open Questions

### Resolved During Planning

- **llama.cpp distribution mechanism:** Download on first run via custom Rust routine (user decision).
- **Curated vs advanced settings split:** Default curated set defined in Key Technical Decisions above (assumption; user can adjust during implementation).
- **Frontend framework:** React + TypeScript (assumption; user can adjust during implementation).

### Deferred to Implementation

- **Exact HuggingFace API endpoint for recommended model list.** The `hf-hub` crate's search capabilities may need supplementation with direct HuggingFace Hub API calls. Research during U2 implementation.
- **llama.cpp version pinning and update strategy.** Whether to pin to a specific release, track latest, or support version selection. Decision deferred to U3 implementation.
- **Exact OpenAI endpoint coverage needed by Claude Code / Codex.** Claude Code primarily uses chat completions; Codex uses Responses API. Verify endpoint compatibility during U3/U4 implementation against actual tool behavior.
- **Platform-specific binary packaging.** macOS `.app`/`.dmg`, Linux `.AppImage`/`.deb`, Windows `.msi`/`.exe`. Exact formats and signing strategy deferred to U6 implementation.

---

## Output Structure

```
oclapp/
├── Cargo.toml                          # Workspace root
├── crates/
│   ├── oclapp-core/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs                  # Public API
│   │       ├── settings.rs             # Settings persistence
│   │       ├── models/
│   │       │   ├── mod.rs
│   │       │   ├── discovery.rs        # HuggingFace model search
│   │       │   └── download.rs         # Model download with resume
│   │       ├── server/
│   │       │   ├── mod.rs
│   │       │   ├── binary.rs           # llama.cpp binary download/management
│   │       │   ├── lifecycle.rs        # Start/stop/monitor server
│   │       │   └── config.rs           # Server parameter configuration
│   │       └── error.rs                # Error types
│   ├── oclapp-cli/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       └── main.rs                 # CLI entry point
│   └── oclapp-tauri/
│       ├── Cargo.toml
│       ├── src/
│       │   └── main.rs                 # Tauri entry point + command handlers
│       ├── capabilities/
│       └── tauri.conf.json
├── frontend/                           # React + TypeScript frontend
│   ├── package.json
│   ├── tsconfig.json
│   ├── vite.config.ts                  # or similar bundler config
│   └── src/
│       ├── main.tsx
│       ├── App.tsx
│       ├── components/
│       │   ├── ModelBrowser.tsx
│       │   ├── DownloadManager.tsx
│       │   ├── SettingsPanel.tsx
│       │   └── ServerStatus.tsx
│       └── api/
│           └── tauri.ts                # Tauri command wrappers
└── .github/
    └── workflows/
        └── build.yml                   # Cross-platform CI build
```

---

## High-Level Technical Design

> *This illustrates the intended approach and is directional guidance for review, not implementation specification. The implementing agent should treat it as context, not code to reproduce.*

### Component Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                         User Interfaces                        │
├─────────────────────────┬───────────────────────────────────┤
│   Tauri Desktop App     │           External Clients           │
│   (React + TypeScript)  │   (Claude Code, Codex, Copilot, ...)│
└───────────┬─────────────┴─────────────────┬───────────────────┘
            │                               │
            │                     ┌───────┴────────┐
            │                     │  HTTP Clients  │
            │                     │ (OpenAI SDK)   │
            │                     └───────┬────────┘
            │                               │
            ▼                               │
┌──────────────────────┐                    │
│  Tauri Command Layer │                    │
│  (oclapp-tauri)    │                    │
└──────────┬───────────┘                    │
           │                                │
           ▼                                ▼
┌─────────────────────────────────────────────────────────────┐
│                      oclapp-core Library                       │
│  ┌──────────────┐  ┌──────────────┐  ┌─────────────────────┐  │
│  │   Settings   │  │    Models    │  │   Server Lifecycle  │  │
│  │  (tauri-     │  │  (hf-hub     │  │  (llama.cpp proc    │  │
│  │   plugin-    │  │   + resume)  │  │   + health check)   │  │
│  │   store)     │  │              │  │                     │  │
│  └──────────────┘  └──────────────┘  └─────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
            │
            ▼
┌─────────────────────────────────────────────────────────────┐
│                    External Dependencies                       │
│  ┌──────────────┐  ┌──────────────┐  ┌─────────────────────┐  │
│  │ HuggingFace  │  │ llama.cpp    │  │   File System       │  │
│  │ Hub API      │  │ Server       │  │   (models dir)      │  │
│  └──────────────┘  └──────────────┘  └─────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

### Server Lifecycle State Machine

```
[Idle] --start()--> [Starting] --health ok--> [Running]
   ^                    |                           |
   |                    | health timeout            |
   |                    v                           |
   +--------------- [Failed] <------ error -------+
   |                                               |
   +---------------- stop() -------------------------+
```

The core library manages transitions. The Tauri frontend observes state changes via Events.

---

## Implementation Units

- U1. **[Project Scaffolding and Core Library Foundation]**

**Goal:** Initialize the Cargo workspace, create crate skeletons, scaffold the Tauri v2 app with React + TypeScript, and establish the core library's public API surface and error types.

**Requirements:** R15 (cross-platform foundation)

**Dependencies:** None

**Files:**
- Create: `Cargo.toml`
- Create: `crates/oclapp-core/Cargo.toml`, `crates/oclapp-core/src/lib.rs`, `crates/oclapp-core/src/error.rs`
- Create: `crates/oclapp-cli/Cargo.toml`, `crates/oclapp-cli/src/main.rs`
- Create: `crates/oclapp-tauri/Cargo.toml`, `crates/oclapp-tauri/src/main.rs`, `crates/oclapp-tauri/tauri.conf.json`
- Create: `frontend/package.json`, `frontend/tsconfig.json`, `frontend/vite.config.ts`, `frontend/src/main.tsx`, `frontend/src/App.tsx`
- Modify: `.gitignore` (add Rust and Node artifacts)

**Approach:**
- Set up Cargo workspace with `resolver = "2"` and three member crates.
- `oclapp-core` exports a `OclappCore` struct or module-level API with async functions. Define a unified `OclappError` enum covering IO, network, HuggingFace, server, and validation errors.
- `oclapp-tauri` uses Tauri v2 template with `create-tauri-app` or manual scaffold. Configure `tauri.conf.json` with appropriate permissions (shell execute for server spawn, filesystem for model directory).
- Frontend uses React 18+ with TypeScript. Set up basic routing or component structure for Model Browser, Settings, and Server Status views.

**Patterns to follow:**
- Standard Cargo workspace layout
- Tauri v2 configuration patterns from official docs

**Test scenarios:**
- Happy path: `cargo check` and `cargo test` pass in all three crates.
- Edge case: Workspace builds cleanly on a fresh clone with only Rust toolchain installed.
- Integration: Tauri dev server starts and displays the frontend shell.

**Verification:**
- All three crates compile successfully.
- `cargo test` in `oclapp-core` runs with zero failures.
- Tauri dev command launches the desktop app window.

---

- U2. **[Model Discovery and Download]**

**Goal:** Implement HuggingFace model search, metadata display, and resumable download in the core library. Expose these capabilities through Tauri Commands.

**Requirements:** R1, R2, R3, R4

**Dependencies:** U1

**Files:**
- Create: `crates/oclapp-core/src/models/mod.rs`
- Create: `crates/oclapp-core/src/models/discovery.rs`
- Create: `crates/oclapp-core/src/models/download.rs`
- Modify: `crates/oclapp-core/src/lib.rs`
- Modify: `crates/oclapp-tauri/src/main.rs` (add command handlers)
- Create: `frontend/src/components/ModelBrowser.tsx`, `frontend/src/components/DownloadManager.tsx`

**Approach:**
- Use `hf-hub` crate for HuggingFace Hub interactions. If search functionality is limited, supplement with direct HTTP calls to HuggingFace Hub API.
- Model discovery: Search by keyword, filter by GGUF/MLX tags, return metadata (name, size, quantization, downloads, last updated).
- Direct entry: Parse `org/model-name` format and validate existence via HEAD request or `hf-hub` API before download.
- Download: Use `hf-hub`'s download capabilities with progress callbacks. If `hf-hub` lacks resume support, implement direct HTTP with Range headers.
- Persist model list in the settings store or a simple JSON file in the models directory.
- Tauri Commands: `search_models(query)`, `download_model(repo_id, filename)`, `get_download_progress()`, `list_local_models()`, `set_models_dir(path)`.

**Patterns to follow:**
- `hf-hub` crate patterns for authentication and caching
- Tauri async command patterns with progress streaming via Events or Channels

**Test scenarios:**
- Happy path: Search returns models matching a known GGUF model name.
- Happy path: Download completes successfully and file exists in configured directory.
- Edge case: Resume an interrupted download from partial file.
- Edge case: Direct entry of non-existent model fails gracefully with clear error.
- Error path: Network failure during download propagates error and cleans up partial file.
- Integration: Frontend search and download flows end-to-end via Tauri dev app.

**Verification:**
- Unit tests for discovery and download logic pass.
- Integration test: download a small test model (e.g., a tiny GGUF) and verify file integrity.
- Frontend model browser displays search results and download progress.

---

- U3. **[llama.cpp Binary and Server Lifecycle Management]**

**Goal:** Implement llama.cpp binary download/management and server lifecycle (start, stop, health check, log streaming) in the core library. Wire server status into Tauri Events.

**Requirements:** R6, R7, R8

**Dependencies:** U1

**Files:**
- Create: `crates/oclapp-core/src/server/mod.rs`
- Create: `crates/oclapp-core/src/server/binary.rs`
- Create: `crates/oclapp-core/src/server/lifecycle.rs`
- Create: `crates/oclapp-core/src/server/config.rs`
- Modify: `crates/oclapp-core/src/lib.rs`
- Modify: `crates/oclapp-tauri/src/main.rs` (add server command handlers and event emitters)
- Create: `frontend/src/components/ServerStatus.tsx`

**Approach:**
- Binary management: Detect platform (macOS ARM/x86, Linux, Windows). Download appropriate llama.cpp release binary from GitHub releases or a configured mirror into `app_local_data_dir()/bin/`. Verify checksum if available. Set executable permissions on Unix.
- Server lifecycle: Spawn `llama-server` via `std::process::Command` with configured arguments. Capture stdout/stderr in async tasks to prevent buffer deadlocks. Implement health check via periodic `GET /health` (or `GET /v1/models` if no health endpoint) to the server HTTP port.
- State machine: Idle → Starting → Running / Failed → Idle. Emit state changes.
- Configuration: Convert settings (context size, GPU layers, temperature, etc.) into `llama-server` CLI arguments. Support model alias (`-a`) for OpenAI client compatibility.
- Tauri integration: Commands for `start_server(model_id)`, `stop_server()`, `get_server_status()`. Emit Events for state changes, logs, and errors.

**Patterns to follow:**
- Process management best practices: drain stdout/stderr immediately, handle graceful shutdown (SIGTERM on Unix, Ctrl+Break on Windows), detect zombie/orphan processes.
- Tauri Event patterns for streaming logs and state changes to frontend.

**Test scenarios:**
- Happy path: Start server with a valid model, health check succeeds, server reaches Running state.
- Happy path: Stop running server gracefully; process exits and state returns to Idle.
- Edge case: Start server with invalid model path fails fast with clear error; state reaches Failed.
- Edge case: Server process crashes unexpectedly; core library detects exit and transitions to Failed.
- Error path: Binary download fails (network error, checksum mismatch); error is propagated.
- Integration: Frontend server status component reflects state changes and displays recent logs.

**Verification:**
- Unit tests for server config mapping and state machine transitions.
- Integration test: download a tiny llama.cpp binary (or use a mock), spawn it with `--help`, verify process management works.
- Frontend server status panel shows Running/Stopped state and allows start/stop.

---

- U4. **[CLI Launch Command]**

**Goal:** Implement the CLI binary with `clap`, supporting `oClapp launch <tool> --model <name>` and environment variable injection.

**Requirements:** R9, R10, R11

**Dependencies:** U1, U2, U3

**Files:**
- Create: `crates/oclapp-cli/src/main.rs`
- Modify: `crates/oclapp-cli/Cargo.toml` (add clap dependency)

**Approach:**
- Use `clap` for CLI parsing with subcommands: `launch` and potentially `serve`/`status` for direct server control.
- `launch` subcommand: `--model` flag, `<tool>` positional arg mapping to known agents (claude, codex, pi, copilot, etc.).
- Tool registry: A mapping from tool name to executable name and required env vars. Support `claude` → `claude`, `codex` → `codex`, `pi` → `pi` (or generic command pattern).
- Execution flow:
  1. Parse args.
  2. Verify model exists locally via `oclapp-core` model list.
  3. Start server with requested model via `oclapp-core` server lifecycle.
  4. Set `OPENAI_BASE_URL=http://localhost:<port>/v1` and any tool-specific env vars.
  5. Spawn the target tool as a child process, inherit stdout/stderr.
  6. On CLI exit (Ctrl+C), optionally stop the server or leave it running.
- Settings loading: Load persisted settings (models dir, server port, etc.) from the same store the Tauri app uses.

**Patterns to follow:**
- `clap` derive macro patterns for typed CLI args
- `std::process::Command` for spawning the target tool
- Signal handling for graceful shutdown (`ctrlc` crate or Tokio signal handling)

**Test scenarios:**
- Happy path: `launch claude --model llama-2-7b` starts server, sets env var, spawns `claude`.
- Happy path: `launch` with server already running reuses existing server.
- Edge case: Launch with non-existent model exits with error code and clear message (Covers AE4).
- Error path: Target tool executable not found in PATH exits with helpful error.
- Error path: Server fails to start propagates error and exits without spawning tool.

**Verification:**
- `cargo run --bin oclapp-cli -- launch --help` shows correct usage.
- Manual test: launch a mock tool (e.g., `echo`) with model verification mocked.

---

- U5. **[Tauri Desktop App UI and IPC]**

**Goal:** Build the frontend components (Model Browser, Settings Panel, Server Status) and wire them to Rust backend Commands and Events.

**Requirements:** R1, R2, R3, R4, R8, R12, R13, R14

**Dependencies:** U1, U2, U3

**Files:**
- Create: `frontend/src/components/ModelBrowser.tsx`
- Create: `frontend/src/components/DownloadManager.tsx`
- Create: `frontend/src/components/SettingsPanel.tsx`
- Create: `frontend/src/components/ServerStatus.tsx`
- Create: `frontend/src/api/tauri.ts`
- Modify: `frontend/src/App.tsx`
- Modify: `crates/oclapp-tauri/src/main.rs` (command handlers for all frontend operations)

**Approach:**
- Frontend architecture: Single-page app with tabbed or sidebar navigation for Models, Settings, and Server sections.
- Model Browser: Search input, results list with metadata, download button, local models list with delete option.
- Download Manager: Progress bars for active downloads, cancel buttons.
- Settings Panel:
  - Models directory path (with directory picker via Tauri dialog plugin).
  - Curated parameters: Temperature, Top P, Context Size, Max Tokens, GPU Layers (sliders/number inputs with sensible defaults).
  - Advanced toggle: Seed, Repeat Penalty, Frequency/Presence Penalty, Batch Size, Threads, Flash Attention, MMAP, MLOCK (checkboxes/number inputs).
  - Save button persists via `tauri-plugin-store`.
- Server Status: Indicator (Idle/Starting/Running/Failed), loaded model name, active request count (if available), start/stop button, recent log tail.
- IPC layer: Centralized `frontend/src/api/tauri.ts` wraps all `invoke` calls and event listeners with TypeScript types.
- Rust Commands: One command per frontend operation, delegating to `oclapp-core`.

**Patterns to follow:**
- Tauri v2 Command/Event IPC patterns
- React hooks for state management (useState/useEffect or a lightweight store)
- Tauri dialog plugin for native directory picker

**Test scenarios:**
- Happy path: Model search returns results and displays in browser.
- Happy path: Settings changes persist after app restart (Covers AE5).
- Happy path: Server start/stop from UI updates status indicator.
- Edge case: Frontend handles slow command responses with loading states.
- Error path: Backend error from `invoke` displays user-friendly message in UI.
- Integration: End-to-end flow — search model, download, configure settings, start server — all within Tauri dev app.

**Verification:**
- Frontend compiles without TypeScript errors.
- All UI components render correctly in Tauri dev app.
- Settings persist across app restarts.
- Server status updates in real-time via Events.

---

- U6. **[Cross-Platform Distribution Setup]**

**Goal:** Configure Tauri bundling and CI for macOS, Linux, and Windows builds. Set up platform-specific build scripts and document distribution steps.

**Requirements:** R15

**Dependencies:** U1–U5

**Files:**
- Create: `.github/workflows/build.yml`
- Create: `.github/workflows/release.yml`
- Modify: `crates/oclapp-tauri/tauri.conf.json` (bundle configuration)
- Create: `scripts/build-macos.sh`, `scripts/build-linux.sh`, `scripts/build-windows.sh`
- Create: `README.md` (build and distribution instructions)

**Approach:**
- Tauri bundle config: Define app metadata (name, identifier, icon, category). Configure macOS `.app`/`.dmg`, Linux `.AppImage`/`.deb`/`.rpm`, Windows `.msi`/`.exe` targets.
- macOS: Code signing configuration (developer cert or ad-hoc with `"signingIdentity": "-"` for initial distribution). Document notarization requirements for production.
- Linux: Build `AppImage` and `.deb` via GitHub Actions with `ubuntu-latest`. No special signing needed initially.
- Windows: Build `.msi` via GitHub Actions with `windows-latest`. Consider WebView2 runtime bundling.
- CI: GitHub Actions workflow triggered on pushes to `main` and tags. Matrix build across three platforms. Artifact upload for PR builds; release asset upload for tags.
- Release process: Tag-based releases with automated changelog (optional). Manual step: codesign/notarize macOS builds before public distribution.

**Patterns to follow:**
- Tauri v2 bundler configuration from official docs
- GitHub Actions matrix build patterns
- macOS code signing and notarization Tauri docs

**Test scenarios:**
- Happy path: CI builds produce installable artifacts for all three platforms.
- Happy path: macOS `.dmg` installs and runs on Apple Silicon.
- Happy path: Linux `.AppImage` runs on Ubuntu/Debian.
- Happy path: Windows `.msi` installs and runs on Windows 10/11.
- Edge case: App launches and reaches initial screen on each platform.

**Verification:**
- CI workflow passes on all platforms.
- At least one manual test install and launch on each target platform.
- Build artifacts are generated and downloadable from CI runs.

---

## System-Wide Impact

- **Interaction graph:**
  - `oclapp-core` is the central dependency for both `oclapp-cli` and `oclapp-tauri`. Changes to core APIs affect both consumers.
  - Settings storage is shared between CLI and Tauri app via `tauri-plugin-store` (which writes to the app data directory). CLI reads the same store.
  - Server process is a singleton: only one llama-server instance runs at a time, managed by `oclapp-core`. Both CLI and Tauri app can start/stop it.
  - HuggingFace API calls originate from `oclapp-core` and propagate through to the frontend via Tauri Commands.

- **Error propagation:**
  - `oclapp-core` errors bubble up as `OclappError` to CLI (printed to stderr with exit code) and to Tauri frontend (serialized to JS-friendly error messages).
  - Server process crashes are detected via exit status monitoring and emitted as Tauri Events.

- **State lifecycle risks:**
  - Partial download: interrupted downloads may leave incomplete files. Implement cleanup or resume logic.
  - Orphan processes: server or spawned coding agent processes may outlive the parent. Implement process group cleanup on app exit.
  - Settings corruption: malformed JSON in store. Implement validation and fallback to defaults.

- **API surface parity:**
  - CLI and Tauri app both use `oclapp-core` public API, ensuring consistent behavior.

- **Integration coverage:**
  - CLI launch flow: verify server starts, env vars are set, tool spawns, and server is accessible.
  - Frontend download flow: verify search → download → local model listing → server start all work end-to-end.

- **Unchanged invariants:**
  - This plan creates new crates and files without modifying any existing code (greenfield).

---

## Risks & Dependencies

| Risk | Mitigation |
|------|------------|
| llama.cpp server API incompatibility with Claude Code / Codex | Test against actual tools early (U3/U4). Be prepared to add a lightweight proxy layer if needed. Monitor PR #21174 for Responses API compliance. |
| HuggingFace API rate limiting or breaking changes | Use `hf-hub` crate (official client). Implement caching for model metadata. Document rate limit behavior. |
| macOS notarization blocking distribution | Runtime download avoids sidecar notarization bug. Plan for eventual code signing with Apple Developer account. |
| Cross-platform process management differences | Test server lifecycle on all three platforms. Use platform-specific signal handling where needed. |
| Large model downloads failing or resuming incorrectly | Implement HTTP Range request resume. Provide clear error messages and retry options. |
| Tauri WebView compatibility on older OS versions | Target Tauri v2 minimum supported OS versions. Document requirements. |

---

## Documentation / Operational Notes

- Build instructions for developers: `cargo build`, `npm install` in frontend, `cargo tauri dev`.
- End-user installation: download platform-specific artifact from releases page.
- Troubleshooting: logs location, how to reset settings, how to manually download llama.cpp binary.

---

## Sources & References

- **Origin document:** [docs/brainstorms/oclapp-requirements.md](docs/brainstorms/oclapp-requirements.md)
- **oMLX Architecture:** [omlx-ARCHITECTURE.md](omlx-ARCHITECTURE.md)
- **External docs:**
  - [Tauri v2 Sidecars](https://v2.tauri.app/develop/sidecar/)
  - [Tauri v2 Store Plugin](https://v2.tauri.app/plugin/store/)
  - [llama.cpp Server README](https://github.com/ggml-org/llama.cpp/blob/master/tools/server/README.md)
  - [llama.cpp Server Parameters](https://manpages.debian.org/unstable/llama.cpp-tools/llama-server.1)
  - [PR #21174: Responses API Compliance](https://github.com/ggml-org/llama.cpp/pull/21174)
  - [Tauri macOS Notarization Issue #11992](https://github.com/tauri-apps/tauri/issues/11992)
