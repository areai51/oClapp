---
date: 2026-05-21
topic: oclapp-desktop-llamacpp
description: oClapp desktop app requirements — Tauri + Rust wrapper around llama.cpp server with HuggingFace model management and OpenAI-compatible API for coding agents.
---

# oClapp Requirements Document

## Summary

Build oClapp, a cross-platform desktop app (Rust + Tauri) that wraps llama.cpp to provide an OpenAI-compatible local inference API. Users browse and download models from HuggingFace, configure inference settings through a curated UI, and launch coding agents (Claude Code, Codex, Pi) via CLI. The app manages the llama.cpp server lifecycle and targets distribution to other developers.

---

## Problem Frame

oMLX is a Python + MLX-based local inference server for Apple Silicon Macs. Users who want to run GGUF models or leverage llama.cpp features like MTP currently have no path within oMLX — they must skip those capabilities entirely. Additionally, the oMLX UI and menu bar app reliability issues create friction. A Rust-based alternative using llama.cpp would unlock broader model support, better cross-platform compatibility, and a more robust native desktop experience. The goal is to validate whether a llama.cpp-backed desktop app with polished UX can replace oMLX for daily coding agent workflows.

---

## Actors

- A1. **End User**: Developer who downloads models, tweaks inference settings, and launches coding agents locally.
- A2. **Coding Agent Tool**: External CLI tool (e.g., Claude Code, Codex, Copilot) that connects to the local OpenAI-compatible API exposed by oClapp.
- A3. **llama.cpp Server**: The inference server process managed as a child process by oClapp.

---

## Key Flows

- F1. **Model Discovery and Download**
  - **Trigger:** User opens the model browser in the Tauri app.
  - **Actors:** A1
  - **Steps:**
    1. App displays a list of recommended models from HuggingFace.
    2. User selects a model or pastes a HuggingFace model name directly.
    3. App shows model details and download option.
    4. User confirms download.
    5. App downloads the model to the user-defined local models folder.
    6. Download progress is shown; completion is confirmed.
  - **Outcome:** Model is available locally and listed in the app's model manager.
  - **Covered by:** R1, R2, R3, R4

- F2. **Launch Coding Agent**
  - **Trigger:** User runs `oClapp launch claude --model <modelname>` in their terminal.
  - **Actors:** A1, A2, A3
  - **Steps:**
    1. CLI verifies the requested model is available locally.
    2. CLI starts the llama.cpp server with the selected model if not already running.
    3. CLI sets the appropriate environment variables (e.g., `OPENAI_BASE_URL`) pointing to the local server.
    4. CLI spawns the target coding agent tool (e.g., Claude Code) as a child process.
    5. Coding agent connects to the local OpenAI-compatible API and begins inference.
  - **Outcome:** Coding agent is running and using the specified local model.
  - **Covered by:** R5, R6, R7

- F3. **Configure Inference Settings**
  - **Trigger:** User opens the Settings panel in the Tauri app.
  - **Actors:** A1, A3
  - **Steps:**
    1. App displays curated llama.cpp parameters (e.g., temperature, top_p, context size, GPU layers).
    2. User adjusts visible settings.
    3. User toggles "Advanced" to expose additional parameters.
    4. User saves settings.
    5. App persists settings and restarts the server if needed.
  - **Outcome:** Server runs with updated parameters.
  - **Covered by:** R8, R9, R10

---

## Requirements

**Model Management**
- R1. The app must display a browsable list of recommended models from HuggingFace, including metadata such as size, quantization format, and download count.
- R2. The app must allow direct entry of a HuggingFace model name (copy-paste) as an alternative to browsing.
- R3. The app must support downloading models from HuggingFace to a user-defined local folder, with progress indication and resume support.
- R4. The app must allow the user to define and change the local models folder path.
- R5. The app must support whatever model formats llama.cpp natively handles (e.g., GGUF, MLX) without artificial restriction.

**Inference Server**
- R6. The app must manage the llama.cpp built-in server as a child process, starting, stopping, and restarting it as needed.
- R7. The server must expose an OpenAI-compatible API so existing coding agent tools can connect without modification.
- R8. The app must expose server status (running/stopped, loaded model, active requests) in the Tauri UI.

**CLI**
- R9. The CLI must support `oClapp launch <tool> --model <modelname>` where `<tool>` maps to known coding agents (Claude Code, Codex, Pi, etc.).
- R10. The CLI must set the necessary environment variables so the spawned coding agent connects to the local OpenAI-compatible API.
- R11. The CLI must verify the requested model exists locally before launching the agent, and prompt or fail gracefully if not.

**Settings UI**
- R12. The Tauri settings UI must expose a curated subset of llama.cpp parameters, chosen to be meaningful without overwhelming the user.
- R13. The settings UI must provide an "Advanced" toggle that exposes additional llama.cpp parameters.
- R14. Settings changes must be persisted across app restarts.

**Cross-Platform**
- R15. The app must run on macOS, Linux, and Windows.

---

## Acceptance Examples

- AE1. **Covers R1, R3.** Given the user opens the model browser, when they select a recommended GGUF model and click download, the model downloads to the configured folder and appears in the local models list upon completion.
- AE2. **Covers R2, R3.** Given the user pastes `TheBloke/Llama-2-7B-GGUF` into the direct input field, when they confirm download, the app resolves the model and downloads it.
- AE3. **Covers R6, R7, R9, R10.** Given the server is stopped and model `llama-2-7b` is available locally, when the user runs `oClapp launch claude --model llama-2-7b`, the server starts with that model, `OPENAI_BASE_URL` is set to the local endpoint, and Claude Code launches successfully.
- AE4. **Covers R11.** Given the user runs `oClapp launch codex --model nonexistent-model`, when the model is not found locally, the CLI exits with a clear error message and does not start the server.
- AE5. **Covers R12, R13, R14.** Given the user sets temperature to 0.5 and context size to 4096 in the curated settings, when they toggle Advanced and set `n_gpu_layers` to 35, the settings persist after app restart and the server uses those values on next launch.

---

## Success Criteria

- **Human outcome:** A developer can install oClapp, download a model from HuggingFace within the app, and run `oClapp launch claude --model <name>` to start coding with a local LLM without manual server configuration.
- **Downstream handoff:** The requirements document provides clear scope boundaries, defined flows, and acceptance examples that a planner can use to produce a technical implementation plan without re-inventing product behavior.

---

## Scope Boundaries

### Deferred for later

- Custom Rust inference engine (replacing the llama.cpp server child process).
- Built-in benchmarking or evaluation tools.
- MCP server integration.
- Admin dashboard web UI (the Tauri desktop app replaces this).
- Auto-updater mechanism.

### Outside this product's identity

- Mobile or web deployment.
- Cloud-hosted inference or SaaS offering.
- Multi-user collaboration or shared model libraries.
- Model training or fine-tuning capabilities.

---

## Key Decisions

- **Approach 2 (Tauri shell over llama.cpp server) chosen.** Rationale: Faster validation of UX and core workflows before investing in a custom Rust inference engine. The server can be swapped later once the app shape is proven.
- **OpenAI API compatibility chosen.** Rationale: Maximizes interoperability with existing coding agent tools without requiring custom adapters.
- **Curated settings with advanced toggle chosen.** Rationale: Balances ease of use for most users with flexibility for power users, matching the goal of not overwhelming the user.
- **No artificial model format restriction.** Rationale: llama.cpp supports multiple formats natively; restricting to GGUF only would unnecessarily limit utility.

---

## Dependencies / Assumptions

- llama.cpp must be available as a binary that can be bundled with or downloaded by the app. The exact distribution mechanism (bundled, user-provided, or download-on-first-run) is deferred to planning.
- HuggingFace Hub API must remain accessible for model metadata and downloads.
- Coding agent tools (Claude Code, Codex, etc.) must continue to support the `OPENAI_BASE_URL` environment variable or equivalent configuration mechanism.
- Cross-platform Tauri + Rust toolchain is viable for all target platforms.

---

## Outstanding Questions

### Resolve Before Planning

- [Affects R3][User decision] How should llama.cpp be distributed? Options: bundled binary in the app, download-on-first-run, or require user to install separately.
- [Affects R12][User decision] Which specific llama.cpp parameters should appear in the curated UI vs the advanced panel?

### Deferred to Planning

- [Affects R1][Needs research] Which HuggingFace API or library should be used for model discovery and metadata?
- [Affects R3][Needs research] Should model downloads use `hf-hub` Rust crate, or direct HTTP with resume support?
- [Affects R7][Technical] What is the exact OpenAI-compatible endpoint shape that llama.cpp server exposes, and does it cover all endpoints needed by Claude Code / Codex?
- [Affects R15][Technical] How should platform-specific packaging and code signing be handled for distribution?
