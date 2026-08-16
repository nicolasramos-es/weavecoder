# Architecture

This document explains how Weavecoder (`wvc`) works under the hood: the native
Agent Swarm, the embedded Code Knowledge Graph (CKG), local embeddings, and
the provider model — plus the client/daemon split that powers the CLI.

Weavecoder is a coding-agent CLI written in Rust. It is designed around three
pillars:

1. **Native Agent Swarm** — many requests run in parallel without a
   third-party orchestrator.
2. **Local models first** — local runtimes (Ollama, LM Studio, oMLX/llama.cpp,
   any OpenAI-compatible endpoint) take priority over the cloud.
3. **Embedded Code Knowledge Graph** — a tree-sitter-powered index of your
   project's symbols and relations, stored locally in SQLite + FTS5.

The workspace is organized as a Cargo workspace: the CLI lives in `src/` and
the subsystems live in `crates/` (all prefixed `wvc-`). The main crates behind
the architecture are `wvc-swarm-core`, `wvc-code-graph`, `wvc-embedding`, and
`wvc-app-core`.

## Native Agent Swarm

The swarm contract lives in [`crates/wvc-swarm-core`](../crates/wvc-swarm-core/src/lib.rs).
It is a **native** swarm: parallel request orchestration is built into the
product, with no external orchestrator service in the loop. The crate defines
the message and reporting protocol that swarm members use to talk to each
other and to coordinating UIs.

Key constants and functions (verified in `crates/wvc-swarm-core/src/lib.rs`):

| Symbol | Meaning |
|---|---|
| `SwarmRole` (`Agent`, `Coordinator`, `Other`) | Every swarm member has a role; a coordinator can be a member like any agent. |
| `MAX_SWARM_MEMBERS` (`1000`) | Absolute hard cap on live members in a single swarm. |
| `SWARM_COMPLETION_REPORT_MARKER` | Marker a member includes in its completion report so receivers can detect it reliably. |
| `MAX_SWARM_COMPLETION_REPORT_CHARS` (`4000`) | Upper bound for a completion report body. |
| `derive_swarm_task_label` / `MAX_SWARM_TASK_LABEL_CHARS` (`48`) | Derives a short, stable one-line task label from a spawn prompt for UI chips. |
| `validate_swarm_tldr` / `SWARM_TLDR_REQUIRED_OVER_CHARS` (`240`) / `MAX_SWARM_TLDR_CHARS` (`200`) | Long member messages must carry a one-line `tldr` so UIs can render them collapsed; the validator enforces it. |

Because the swarm is native, every member is a first-class session in the same
daemon: headless/swarm work runs alongside interactive sessions and survives
until explicitly stopped. You can replay a whole swarm run in a synchronized
multi-pane view with `wvc replay --swarm`, and `wvc server stop` warns before
dropping any in-flight headless/swarm sessions.

## Code Knowledge Graph (CKG)

The CKG is an embedded, offline index of your project's source code. It lives
in [`crates/wvc-code-graph`](../crates/wvc-code-graph/src/lib.rs) and is
exposed through two CLI commands:

- `wvc init <path> --db <db>` — scan a project and build the graph.
- `wvc code-search <query> --db <db> [--top-k N]` — search it.

### Languages

Source files are parsed with **tree-sitter**. The supported languages and
extensions are defined in `crates/wvc-code-graph/src/language.rs`:

| Language | Extensions |
|---|---|
| Go | `.go` |
| Python | `.py`, `.pyi` |
| TypeScript | `.ts`, `.tsx` |
| Rust | `.rs` |

### Indexing pipeline (`wvc init`)

`wvc init` is orchestrated by `run_init` in `crates/wvc-code-graph/src/init.rs`
(types: `InitConfig`, `InitSummary`). The pipeline is:

1. **Scan** — recursively walk the project directory, `.gitignore`-aware,
   keeping only files with supported extensions (`SUPPORTED_EXTENSIONS`).
2. **Parse & extract** — parse each file with tree-sitter and extract
   **symbols** (functions, classes, imports, calls…) and **relations** (edges
   between symbols) via `crates/wvc-code-graph/src/symbols.rs` and
   `relations.rs`.
3. **Store** — persist everything in an embedded SQLite database
   (`crates/wvc-code-graph/src/storage.rs`, type `CodeGraph`), with FTS5
   indexing for fast text search. The database is opened with WAL journaling
   and foreign keys enabled; a `SCHEMA_VERSION` constant guards schema
   evolution.

The scan is **incremental**: each file is classified by its SHA-256 hash
(`compute_file_hash`). Unchanged files are skipped entirely, modified files
are re-indexed (old symbols removed first), and deleted files have their
symbols purged — so repeated `wvc init` runs stay fast.

### Hybrid search (`wvc code-search`)

`wvc code-search` runs a **hybrid search** (`crates/wvc-code-graph/src/search.rs`,
type `HybridSearch`) that fuses three signals:

1. **FTS5** — exact/substring text match over the SQLite FTS5 index (BM25
   rank). Single terms become prefix queries, so `parse` matches
   `parseConfig`.
2. **Semantic** — cosine similarity over symbol embeddings, used when a query
   embedding is available and symbols carry embeddings.
3. **Graph** — neighborhood enrichment via the in-memory `SymbolGraph`
   (`crates/wvc-code-graph/src/graph.rs`, built on `petgraph`): callers and
   dependencies of a direct hit surface with a degraded score.

The signals are fused with **Reciprocal Rank Fusion (RRF, k=60)**, which needs
no score normalization across incompatible signals — only rank order matters.
Each result reports which signals contributed (`SearchSignal`).

## Local embeddings

Semantic search runs on **local embeddings**, produced by the
[`crates/wvc-embedding`](../crates/wvc-embedding/src/lib.rs) crate:

- Model: **all-MiniLM-L6-v2** (`MODEL_NAME`), downloaded as ONNX from
  Hugging Face (`MODEL_URL` / `TOKENIZER_URL`).
- Dimension: **384** (`EMBEDDING_DIM`), max sequence length 256.
- Runtime: **tract** (`tract-onnx` / `tract-hir` 0.23), with the
  `tokenizers` crate for tokenization. Input tensors are bound by name and
  each input receives its model-declared dtype (exporters differ in input
  order and dtype).

Because the model runs locally, indexing and searching never leave your
machine and work offline.

## Providers: local first, cloud when you need it

Weavecoder's provider model is **local-first**: the default provider is
`auto`, and the full `ProviderChoice` enum is defined in
[`src/cli/provider_init.rs`](../src/cli/provider_init.rs).

**Local runtimes** (no API key required):

- `ollama` — Ollama.
- `lmstudio` — LM Studio.
- `openai-compatible` — any OpenAI-compatible endpoint, which covers oMLX,
  llama.cpp, and custom self-hosted servers (`--api-base` / `--api-key` on
  `wvc login`).

**Cloud providers** (subset of the real enum — see `ProviderChoice` for the
full list):

- Anthropic: `claude`, `anthropic-api`
- OpenAI: `openai`, `openai-api`
- `openrouter`, `bedrock`, `azure`, `gemini`, `gemini-api`, `copilot`,
  `cursor`, `antigravity`, `google`
- `opencode`, `opencode-go`, `zai`, `kimi`, `302ai`, `baseten`, `cortecs`,
  `comtegra`, `deepseek`, `fpt`, `firmware`, `huggingface`, `moonshotai`,
  `nebius`, `scaleway`, `stackit`, `groq`, `mistral`, `perplexity`,
  `togetherai`, `deepinfra`, `fireworks`, `minimax`, `xai`, `nvidia-nim`,
  `xiaomi-mimo`, `celeris`, `chutes`, `cerebras`, `alibaba-coding-plan`,
  and `weavecoder` (the first-party account).

Named provider profiles from `[providers.<name>]` in `config.toml` are
supported via `--provider-profile`; OpenAI-compatible profiles are implied by
that flag. Interactive sessions can switch providers on the fly with `/model`.

## Client/daemon architecture

The CLI splits into a lightweight client and a long-lived background daemon:

- `wvc serve` — start the agent server (background daemon). Optional
  `--server-name` gives it a stable display name for remote runtimes.
- `wvc server start | reload | stop | status` — manage the daemon lifecycle.
- `wvc connect` — connect a client to a running server.

Client/server communication uses a **Unix socket** (Windows equivalent
included on that platform):

- `--socket <path>` — custom socket path for server/client communication
  (global flag).
- `--resume [ID]` — resume a session by ID, or list sessions if no ID is
  given (global flag).

Other global flags that shape every invocation: `--provider` (initial
provider), `-m/--model` (model to use), `--trace` (log tool I/O and token
usage to stderr), `--quiet` (suppress non-error output for scripting),
`--tools` / `--disabled-tools` (tool allow-list / hide-list), and
`--no-update` (skip the automatic update check).

The daemon is the same binary as the CLI: a freshly built binary only takes
effect after the daemon is reloaded (`wvc server reload` or a self-dev
rebuild), and a custom `--socket` lets you test a build without disturbing
the shared daemon.
