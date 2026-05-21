# oMLX Architecture Document

## Overview

oMLX is an LLM inference server optimized for Apple Silicon Macs (M1/M2/M3/M4). It provides OpenAI-compatible and Anthropic-compatible APIs over a FastAPI server, with continuous batching, tiered KV caching, and a built-in web admin dashboard. A native macOS menu bar app provides one-click management.

## High-Level Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              User Interfaces                                 │
├─────────────────────┬─────────────────────────────────────────────────────────┤
│   macOS Menu Bar    │              External Clients & Tools                  │
│      App            │     (Claude Code, Codex, Copilot, OpenClaw, etc.)     │
│   (packaging/)      │                                                         │
└──────────┬──────────┴────────────────────────┬──────────────────────────────┘
           │                                    │
           │                          ┌─────────┴─────────┐
           │                          │   HTTP Clients    │
           │                          │ (OpenAI SDK, curl)│
           │                          └─────────┬─────────┘
           │                                    │
           └────────────────────┬─────────────┘
                                │
                    ┌───────────┴────────────┐
                    │    FastAPI Server      │
                    │      (server.py)       │
                    │                        │
                    │  OpenAI / Anthropic    │
                    │    / MCP / Admin       │
                    └───────────┬────────────┘
                                │
           ┌────────────────────┼────────────────────┐
           │                    │                    │
    ┌──────┴──────┐    ┌───────┴────────┐   ┌──────┴──────┐
    │   Admin     │    │   Engine Pool    │   │   Process   │
    │  Dashboard  │    │  (engine_pool.py)│   │   Memory    │
    │  (admin/)   │    │                  │   │  Enforcer   │
    │             │    │ - LRU eviction   │   │             │
    │ Web UI,     │    │ - Model pinning │   │ - TTL checks│
    │ benchmarks, │    │ - Pre-load mem │   │ - Mem limits│
    │ i18n        │    │   checks         │   │             │
    └─────────────┘    └───────┬──────────┘   └─────────────┘
                               │
                    ┌──────────┴──────────┐
                    │     Engine Core       │
                    │    (engine_core.py)   │
                    │                       │
                    │ - Async request mgmt  │
                    │ - Output streaming    │
                    │ - MLX thread safety   │
                    └──────────┬──────────┘
                               │
           ┌────────────────────┼────────────────────┐
           │                    │                    │
    ┌──────┴──────┐    ┌───────┴────────┐   ┌──────┴──────┐
    │  Scheduler  │    │  Model Registry  │   │   Cache     │
    │(scheduler.  │    │ (model_registry  │   │   Stack     │
    │    py)      │    │  .py)            │   │  (cache/)   │
    │             │    │                  │   │             │
    │ - FCFS     │    │ - Ownership      │   │ - Paged KV  │
    │ - Batching │    │ - Auto-discovery │   │ - Prefix    │
    │ - Admission│    │ - Aliases        │   │ - Hybrid    │
    └─────────────┘    └──────────────────┘   │ - SSD tier  │
                                              └─────────────┘
                               │
                    ┌──────────┴──────────┐
                    │     MLX Executor       │
                    │  (1-thread pool for   │
                    │   Metal safety)        │
                    └──────────┬───────────┘
                               │
           ┌────────────────────┼────────────────────┐
           │                    │                    │
    ┌──────┴──────┐    ┌───────┴────────┐   ┌──────┴──────┐
    │  Batched    │    │  Vision-Lang   │   │  Embedding  │
    │   Engine    │    │    Engine      │   │   Engine    │
    │(batched_)   │    │  (vlm_engine)  │   │ (embeddings)│
    │             │    │                │   │             │
    │ - LLMs     │    │ - VLMs         │   │ - Text emb  │
    │ - Tool call│    │ - Vision cache │   │ - Reranking │
    │ - Spec dec │    │ - Spec prefill │   │             │
    └─────────────┘    └────────────────┘   └─────────────┘
```

## Core Components

### 1. CLI Entry Point (`cli.py`)

The CLI uses Python `argparse` and provides:

- **`serve`** — Start the multi-model server
- **`launch <tool>`** — Launch external integrations (Claude Code, Copilot, etc.)
- **`diagnose menubar`** — Diagnostics for the macOS menu bar app

**Key file:** `omlx/cli.py`

### 2. FastAPI Server (`server.py`)

The HTTP server uses [FastAPI](https://fastapi.tiangolo.com/) with lifespan management.

**OpenAI-compatible endpoints:**
- `POST /v1/chat/completions`
- `POST /v1/completions`
- `POST /v1/embeddings`
- `POST /v1/rerank`
- `GET /v1/models`
- `POST /v1/responses` (OpenAI Responses API for Codex compatibility)

**Anthropic-compatible endpoint:**
- `POST /v1/messages`

**MCP routes:**
- `GET /v1/mcp/tools` — List MCP tools
- `GET /v1/mcp/servers` — MCP server status
- `POST /v1/mcp/execute` — Execute MCP tool

**Admin routes (`/admin/*`):**
- Web dashboard with chat, benchmarking, model downloading
- Authentication and profiles API
- Cache probing and metrics

**Audio routes (conditional):**
- Included if `mlx-audio` is installed
- STT, TTS, STS endpoints

**Key file:** `omlx/server.py`

### 3. Engine Pool (`engine_pool.py`)

Manages multiple models concurrently with:

- **LRU eviction** — Least-recently-used models are unloaded
- **Model pinning** — Keep specific models always in memory
- **TTL** — Time-to-live for loaded models
- **Memory pre-checks** — Reserve KV headroom before loading
- **Fallback logic** — VLM -> LLM -> DFlash -> default engine types

**Key class:** `EngineEntry` — tracks model metadata, loading state, and pin status.

### 4. Engine Core (`engine_core.py`)

The central coordinator for inference:

- **Model loading and management**
- **Request scheduling** via `Scheduler`
- **Async request processing**
- **Output streaming** with `RequestOutputCollector`

**Critical design decision:** MLX GPU operations are serialized onto a **single-threaded executor** (`ThreadPoolExecutor(max_workers=1)`) to prevent Metal command buffer races that cause segfaults. The executor thread initializes a thread-local MLX stream.

**Key classes:**
- `EngineCore` — coordinates loading, scheduling, streaming
- `RequestOutputCollector` — vLLM-pattern low-latency streaming

### 5. Scheduler (`scheduler.py`)

Implements **continuous batching** via a vLLM-style scheduler:

- **FCFS (First-Come-First-Served)** request ordering
- **Configurable concurrency** limits
- **Admission control** based on KV cache availability
- **Chunked prefill** for long contexts
- Integrates with `mlx-lm`'s `BatchGenerator`

**Key classes:**
- `Scheduler` — main scheduling loop
- `SchedulerConfig` — concurrency, batch size, etc.
- `SchedulerOutput` — scheduled batch metadata

### 6. Cache Stack (`cache/`)

Multi-tier caching system optimized for Apple Silicon memory constraints:

| Component | File | Purpose |
|-----------|------|---------|
| PagedCacheManager | `paged_cache_manager.py` | Block-based KV cache with Copy-on-Write |
| PrefixCache | `prefix_cache.py` | Block-aware prefix sharing across requests |
| HybridCache / TieredManager | `hybrid_cache.py` | Coordinates hot (RAM) and cold (SSD) tiers |
| PagedSSDCacheManager | `paged_ssd_cache_manager.py` | Offloads cache blocks to SSD in safetensors format |
| VisionFeatureCache | `vision_feature_cache.py` | Caches vision encoder outputs for VLMs |

**Key design:** Cache blocks are the unit of allocation (typically 16 tokens). Prefix blocks are shared via reference counting. When RAM is exhausted, cold blocks are written to SSD and reloaded on demand.

### 7. Engines (`engine/`)

Specialized inference engines for different model types:

| Engine | File | Purpose |
|--------|------|---------|
| BatchedEngine | `batched_engine.py` | Text LLMs with continuous batching |
| VLMBatchedEngine | `vlm_batched_engine.py` | Vision-language models |
| EmbeddingEngine | `embedding_engine.py` | Text embeddings via `mlx-embeddings` |
| RerankerEngine | `reranker_engine.py` | Document reranking |
| STTEngine | `stt_engine.py` | Speech-to-text |
| TTSEngine | `tts_engine.py` | Text-to-speech |
| STSEngine | `sts_engine.py` | Speech-to-speech |
| DFlashEngine | `dflash_engine.py` | Speculative decoding with draft models |

### 8. Model Wrappers (`models/`)

Thin wrappers around MLX ecosystem libraries:

- `models/llm.py` — `mlx-lm` wrapper
- `models/vlm.py` — `mlx-vlm` wrapper
- `models/embeddings.py` — `mlx-embeddings` wrapper

### 9. Admin Dashboard (`admin/`)

Built-in web UI for managing oMLX:

- **Chat interface** — Multi-turn conversations with loaded models
- **Benchmarking** — Built-in evals (MMLU, GSM8K, HumanEval, etc.)
- **Model download** — HuggingFace Hub integration with progress
- **Settings** — Per-model and global configuration
- **i18n** — Multi-language support (English, Chinese, Korean, Japanese, French)
- **Authentication** — API key management

**Key files:**
- `admin/routes.py` — Admin HTTP routes
- `admin/templates/` — Jinja2 HTML templates
- `admin/static/` — CSS, JS, images
- `admin/i18n/` — Translation JSON files

### 10. MCP Integration (`mcp/`)

Model Context Protocol support for tool calling:

- `mcp/client.py` — MCP client for stdio/SSE transports
- `mcp/manager.py` — MCP server lifecycle management
- `mcp/executor.py` — Tool execution engine
- `mcp/tools.py` — Tool definitions and schemas

### 11. Integrations (`integrations/`)

Launch integrations for popular tools:

- Claude Code
- Copilot CLI
- Codex
- OpenClaw
- Hermes Agent
- Cline
- Continue

**Usage:** `omlx launch <integration>`

## Data Flow

### Chat Completion Request Flow

```
1. Client POST /v1/chat/completions
2. server.py validates request, checks auth
3. engine_pool.py finds or loads target model
4. engine_core.py creates Request with SamplingParams
5. scheduler.py adds request to batch
6. MLX executor thread runs BatchGenerator
7. PagedCacheManager allocates/fetches KV blocks
8. PrefixCache checks for shared prefix hits
9. Output tokens stream via RequestOutputCollector
10. server.py formats OpenAI-compatible SSE stream
```

### Model Loading Flow

```
1. engine_pool.py receives load request
2. Memory pre-check: estimate KV cache + model weights
3. If over budget, evict LRU unpinned models
4. ProcessMemoryEnforcer verifies total process limit
5. Load model weights from disk (safetensors/GGUF)
6. Initialize PagedCacheManager with block tables
7. Register in ModelRegistry with ownership tracking
```

## Request Lifecycle

```
Request Arrives
    │
    ▼
┌──────────────┐
│   Parse &    │
│   Validate   │
└──────┬───────┘
       │
       ▼
┌──────────────┐
│  EnginePool  │
│  (load/find) │
└──────┬───────┘
       │
       ▼
┌──────────────┐
│  EngineCore  │
│  (enqueue)   │
└──────┬───────┘
       │
       ▼
┌──────────────┐
│   Scheduler  │
│  (batching)  │
└──────┬───────┘
       │
       ▼
┌──────────────┐
│ MLX Executor │
│  (1 thread)  │
└──────┬───────┘
       │
       ▼
┌──────────────┐
│  Cache Stack │
│  (KV blocks) │
└──────┬───────┘
       │
       ▼
┌──────────────┐
│   Streaming  │
│    Output    │
└──────────────┘
```

## Technology Stack

| Layer | Technology |
|-------|-----------|
| Language | Python 3.11+ |
| Web Framework | FastAPI |
| ASGI Server | Uvicorn |
| ML Framework | MLX (Apple Silicon) |
| LLM Library | mlx-lm (git commit) |
| VLM Library | mlx-vlm (git commit) |
| Embeddings | mlx-embeddings (git commit) |
| Speculative Decoding | dflash-mlx (git commit) |
| Audio | mlx-audio (optional) |
| Admin UI | Jinja2 + vanilla JS/CSS |
| Menu Bar App | PyObjC + rumps |
| Packaging | venvstacks + DMG |

## Key Metrics

| Metric | Count |
|--------|-------|
| Python files | ~200+ |
| Test files | ~130+ |
| Core package modules | ~40 |
| Admin templates | ~20 |
| Supported model families | Llama, Qwen, Mistral, Gemma, DeepSeek, Phi, etc. |
| Supported languages | EN, ZH, KO, JA, FR |

## Platform Support

| Platform | Support | Notes |
|----------|---------|-------|
| macOS 15.0+ (Sequoia) | Full | Apple Silicon only (M1/M2/M3/M4) |
| Intel Macs | None | MLX requires Apple Silicon |
| Linux | None | Not supported |
| Windows | None | Not supported |

## Memory Management

### Process Memory Enforcer (`process_memory_enforcer.py`)

- Enforces a **total process memory limit** (configurable)
- Runs TTL checks to evict stale models
- Prevents system swap thrashing on macOS

### Tiered Cache (`hybrid_cache.py`, `paged_ssd_cache_manager.py`)

- **Hot tier**: In-memory paged KV cache (GPU RAM)
- **Cold tier**: SSD-backed cache in safetensors format
- **Migration**: Automatic promotion/demotion based on access patterns
- **Format**: Safetensors for fast serialization

## Extension Points

### Custom Models

Models are auto-discovered from the model directory via `model_discovery.py`. Supported formats:
- HuggingFace safetensors
- GGUF (via mlx-lm)
- Custom MLX weights

### Custom Integrations

Integrations are defined in `integrations/` as Python modules that:
1. Detect if the target tool is installed
2. Generate configuration files
3. Set environment variables (e.g., `OPENAI_BASE_URL`)

### Custom MCP Tools

MCP servers are configured via JSON:

```json
{
  "mcpServers": {
    "filesystem": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-filesystem", "/path"]
    }
  }
}
```

## Testing Architecture

| Category | Files | Description |
|----------|-------|-------------|
| API compatibility | `test_openai_*.py`, `test_anthropic_*.py` | OpenAI/Anthropic adapter tests |
| Cache systems | `test_paged_cache.py`, `test_hybrid_cache.py` | KV cache correctness |
| Engine behavior | `test_engine_core.py`, `test_engine_pool.py` | Load/unload/streaming |
| Scheduler | `test_scheduler.py`, `test_scheduler_admission.py` | Batching logic |
| MCP | `test_mcp_client.py`, `test_mcp_manager.py` | Tool integration |
| Admin | `test_admin_auth.py`, `test_admin_profiles_api.py` | Dashboard functionality |
| VLM | `test_vlm_engine.py`, `test_vlm_mtp.py` | Vision model tests |
| Audio | `test_audio_api.py` | Speech model tests |
| Integration | `e2e_*.py` | End-to-end tests |

**Pytest markers:**
- `slow` — Requires model loading (deselected by default)
- `integration` — Requires running server

## Deployment Patterns

### macOS App

1. Download `.dmg` from releases
2. Drag to Applications
3. Auto-updater handles future updates
4. Menu bar app manages server start/stop

### Homebrew

```bash
brew tap jundot/omlx https://github.com/jundot/omlx
brew install omlx
brew services start omlx  # Background service
```

### From Source

```bash
git clone https://github.com/jundot/omlx.git
cd omlx
pip install -e ".[mcp,audio]"
omlx serve --model-dir ~/models
```

### Docker

Not officially supported (Apple Silicon-only platform).

## Configuration

### Runtime Settings (`settings.py`)

Persisted to `~/.omlx/settings.json`:

| Section | Contents |
|---------|----------|
| `server` | Host, port, max model memory, concurrency |
| `auth` | API keys, admin credentials |
| `sampling` | Default temperature, top_p, etc. |
| `cache` | Block size, SSD path, hot/cold ratios |
| `mcp` | MCP server configurations |
| `models` | Aliases, pinned models, per-model overrides |

### CLI Arguments

```bash
omlx serve \
  --model-dir ~/models \
  --max-model-memory 32GB \
  --pin llama-3b,qwen-7b \
  --mcp-config mcp.json \
  --host 0.0.0.0 \
  --port 8000
```

## Performance Characteristics

| Metric | Target |
|--------|--------|
| First token latency | ~5-20ms (hot model) |
| Throughput | ~50-200 tokens/sec (M3 Pro) |
| Max concurrent models | Depends on memory (typically 2-4) |
| Context window | Limited by unified memory (up to 128K) |
| Batch size | Dynamic (continuous batching) |

## Security Model

### Authentication

- Optional API key authentication via `HTTPBearer`
- Admin dashboard requires separate credentials
- No built-in HTTPS (use reverse proxy)

### Model Isolation

- Each model runs in its own process context
- No sandboxing between models (shared MLX thread)
- Process memory enforcer prevents runaway allocation

### Input Validation

- JSON schema validation for structured outputs
- Image size limits for VLM inputs
- Token count limits per request
