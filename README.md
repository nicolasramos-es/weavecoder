<div align="center">

<img src="assets/weavecoder-logotipo.svg" alt="Weavecoder" width="400" />

# Weavecoder

[![License: MIT](https://img.shields.io/badge/license-MIT-blue?style=flat-square)](LICENSE)
[![Platforms](https://img.shields.io/badge/platforms-Linux%20%7C%20macOS%20%7C%20Windows-blue?style=flat-square)](https://github.com/nicolasramos-es/weavecoder/releases)

**Weavecoder** (binary `wvc`) is a **Rust** coding-agent CLI with a native **Agent Swarm** for multiple fast parallel requests, **local models first**, an embedded **Code Knowledge Graph** (tree-sitter), and **atomic task decomposition** that splits large jobs into verified subtasks — so it runs *deeply, efficiently and privately* on models you already have.

</div>

## Why Weavecoder

Most coding agents are either **single-session chat** (fast but shallow — they lose context on big tasks) or **cloud-only** (powerful but send your code to a third party, need internet, and bill per token). Weavecoder is built for the middle ground that most teams actually need:

| | Weavecoder (`wvc`) | Generic chat agents | Cloud-only agents |
|---|---|---|---|
| **Local models first** | ✅ oMLX, Ollama, LM Studio, llama.cpp, vLLM, any OpenAI-compatible | ⚠️ often API-only | ❌ requires their cloud |
| **Agent Swarm (parallel)** | ✅ native, from one CLI | ❌ single session | ⚠️ orchestrated remotely |
| **Atomic task decomposition** | ✅ DAG with verify/synthesis gates | ❌ none | ⚠️ opaque |
| **Code Knowledge Graph** | ✅ embedded (tree-sitter), offline | ❌ none | ⚠️ remote index |
| **Runs offline / keeps code local** | ✅ | ✅ | ❌ sends code to cloud |
| **Cost** | ✅ free on local models | — | ❌ per-token |

### What makes it *deeply* optimal for local models

The whole architecture is tuned so a **local 1–8B model** can actually do serious multi-step work, which most agents can't:

1. **Task decomposition for local context windows.** A big task isn't thrown at the model as one giant prompt. `wvc` splits it into a **DAG of atomic subtasks**, each executed by a worker whose context stays under **~4000 tokens** (subtask + one-line summaries of completed dependencies). Small context = local models stay focused and don't lose the thread.
2. **Verification gates.** Every composite task gets a **critique/verify gate**: the parent cannot close until its synthesis survives an adversarial audit. Work is *proven*, not assumed — critical when a smaller model is prone to hallucinating "done".
3. **Local embeddings + Code Knowledge Graph.** Your whole project is indexed **on your machine** with tree-sitter (functions, classes, imports, call graphs) plus local semantic embeddings. `wvc code-search` answers "who calls X" and "what does Y depend on" instantly, offline, without sending your code anywhere.
4. **Parallel swarm on one machine.** A coordinator fans out subtasks to independent workers that run **concurrently** against your local server, so a large refactor finishes in wall-clock time that a single chat loop can't touch.
5. **Memory-first onboarding.** `/memory on` persists a session's context for later agents, so the next run doesn't start blind. `wvc init <path>` (or bare `wvc init` in a project dir) builds the project graph the agent then works from.

## Features

- **Agent Swarm** — orchestrate multiple requests in parallel, natively, from a single CLI
- **Atomic task decomposition** — split a large goal into a verified DAG of subtasks with synthesis gates
- **Local models first** — oMLX, Ollama, LM Studio, llama.cpp, vLLM, or any OpenAI-compatible endpoint (add as many as you need)
- **Code Knowledge Graph (CKG)** — embedded offline index (SQLite + FTS5 + embeddings + call graph):
  - `wvc init` — scans the project (defaults to the current directory), extracts symbols/relations, stores in SQLite + FTS5
  - `wvc code-search <query>` — hybrid search (FTS5 + embeddings + dependency graph)
  - Graph traversal: "who calls X", "what does Y depend on"
  - Incremental indexing: only re-indexes modified files
- **Local embeddings** (all-MiniLM-L6-v2) for semantic search by meaning
- **Granular permissions** — per-tool `allow`/`ask`/`deny` and a disk access mode (`full`/`limited`/`ask`) via `/permissions`
- **Dark theme by default** — black background, white text, readable everywhere
- **Cross-platform** — Linux, macOS, and Windows 11

## Installation

The installer scripts are always fetched from the project repository on GitHub — the single source of truth — never from any web domain.

### macOS & Linux

```bash
curl -fsSL https://raw.githubusercontent.com/nicolasramos-es/weavecoder/main/install.sh | bash
```

### Windows 11 (PowerShell 5.1+)

```powershell
irm https://raw.githubusercontent.com/nicolasramos-es/weavecoder/main/install.ps1 | iex
```

> The installer resolves the **latest GitHub Release**, downloads the correct binary for your OS/arch, verifies its **SHA-256** against the published `SHA256SUMS`, and (on macOS) strips the quarantine flag. No web domain, no third-party CDN — the repository is the only source. If the repository is private, export `GITHUB_TOKEN` (or `WVC_GITHUB_TOKEN`) first.

### From source

```bash
git clone https://github.com/nicolasramos-es/weavecoder.git
cd weavecoder
cargo build --release --bin wvc
# → target/release/wvc
```

## Quick Start

```bash
# 1. Connect a local model (e.g. Ollama)
brew install ollama && ollama pull llama3.2
wvc login ollama

# 2. Chat with the agent
wvc --provider ollama --model llama3.2 run 'hello'

# 3. Index a project and search it with the Code Knowledge Graph
wvc init ./my-project --db ~/.wvc/codegraph.db     # or bare `wvc init` in the project dir
wvc code-search "parseConfig" --db ~/.wvc/codegraph.db
```

## Using the agent to plan and execute a task

Inside the TUI:

- **`/plan [goal]`** — the agent produces a concrete plan (a plan card with Goal / Scope / Approach / Validation / Open questions) **without touching files**, then waits for your approval. Approve and it turns the plan into an executable todo list.
- **`/swarm <task>`** — launch a swarm that decomposes the task into subtasks and runs them in parallel across workers (local models first). Subtasks are verified with synthesis gates before the task closes.
- **`/permissions`** — review and set disk access mode and per-tool allow/ask/deny.
- **`/memory on`** — persist this session's context so a later agent has it.

## Documentation

- [Installation](docs/installation.md) — installers, platform notes, and common setup
- [Commands](docs/commands.md) — full CLI reference
- [Architecture](docs/architecture.md) — crates, data flow, and design decisions
- [Troubleshooting](docs/troubleshooting.md) — common issues and fixes

## Architecture

| Crate | Responsibility |
|---|---|
| `wvc-code-graph` | Code Knowledge Graph: tree-sitter (Go/Py/TS/Rust), SQLite+FTS5, embeddings, petgraph graph, hybrid search |
| `wvc-embedding` | Local embeddings (all-MiniLM-L6-v2, tract-onnx) |
| `wvc-swarm-core` | Agent swarm orchestration |
| `wvc-app-core` | Agent core (tools, sessions, server) |
| `wvc-plan` | Durable plan graph + atomic task decomposition (DAG) |

## License

MIT — see [LICENSE](LICENSE).

This project builds on the exceptional work of Jeremy Huang (wvc, MIT), on top of which we've added new features and improved the product. The original copyright notice is preserved in full in LICENSE.
