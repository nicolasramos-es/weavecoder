# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- **Worker profiles** — `wvc swarm --worker-profile <coder|tester|reviewer|researcher>` shapes a spawned worker's system prompt (NRA-719).
- **Swarm local-first default** — `swarm_prompt.md` defaults to local-first model auto-detection (oMLX → Ollama → vLLM → cloud fallback) instead of Fable 5 Anthropic (NRA-718).
- **Local swarm benchmark** — `scripts/benchmark_local_swarm.py` for measuring local-first swarm runs (NRA-720).
- **Local model auto-detection in `wvc login`** — probes llama.cpp (8080), vLLM (8000) and oMLX (8081) with a 2s timeout and lists them when they respond with a model catalog (NRA-721 S1T4).
- **Session cache** — `wvc code-search` results and chat completions cached in-memory (LRU, max 50), invalidated on `wvc init` or working-directory change (NRA-721 S1T3).

### Changed

- **Token efficiency (NRA-721)** — deep-task-graph workers receive compact one-line dependency summaries (subtask + ≤4000 tokens) instead of full artifacts (S1T2); completion-report file evidence is compressed to a unified diff capped at 80 lines with a `[truncated]` indicator (S1T1). Gates keep full artifacts to audit `what_i_did_not_check`.

## [0.67.0] — 2026-08-16

Current development version (`Cargo.toml`). This is the first entry of the
project's own changelog, so it aggregates the changes present on `main`.

### Added

- **One-line installers** — `scripts/install.sh` (Linux/macOS) and
  `scripts/install.ps1` (Windows) that download the release binary from GitHub
  Releases and verify its SHA-256 checksum against the published digest
  (`WVC_RELEASE_METADATA_BASE`, `valid_release_tag`).
- **Code Knowledge Graph (CKG)** — `wvc init <project> [--db <path>]` and
  `wvc code-search <query> [--db <path>] [--top-k N]` backed by tree-sitter
  parsing (Go, Python/`.pyi`, TypeScript/TSX, Rust), SQLite + FTS5 storage
  (`crates/wvc-code-graph`), incremental indexing with SHA-256 snapshots, and
  hybrid search fused with Reciprocal Rank Fusion.
- **Local embeddings** — `crates/wvc-embedding`: all-MiniLM-L6-v2 (384 dims)
  in ONNX with `tract-onnx` inference and a HuggingFace tokenizer, downloaded
  on demand.
- **Agent Swarm** — native orchestration of multiple parallel requests via
  `crates/wvc-swarm-core`.
- **Provider support** — Ollama, LM Studio, oMLX/llama.cpp and any
  OpenAI-compatible endpoint, plus cloud providers (full list in `wvc --help`).
- **User documentation** — `docs/installation.md`, `docs/commands.md`,
  `docs/architecture.md` and `docs/troubleshooting.md`.
- **Complete main README** in English, with the Weavecoder logo in the header
  and the final app icon (`Weavecoder.icns`, `assets/app-icons`).
- **Meta-documents** — `CHANGELOG.md`, `CONTRIBUTING.md` and `SECURITY.md` at
  the repository root.

### Changed

- Install domain updated to `weavecoder.nramos.dev` and the README unified to
  English. The README install commands point to
  `raw.githubusercontent.com`; the installer's metadata fallback
  (`RELEASE_METADATA_BASE` in `scripts/install.sh`) still defaults to a legacy
  web domain and is tracked in NRA-528.
- CI: all Linux jobs run on the self-hosted runner; macOS/Windows workflows are
  manual-only (GitHub-hosted) to control costs; CI runs on pull requests only.
- Clippy `-D warnings` scoped to `wvc-code-graph` (Phase 1 crate); the
  workspace-wide lint is informational.
- `Cargo.lock` committed to the repository.

### Fixed

- Clippy warnings across `wvc-sdk`, `wvc-tui`, `wvc-math`, `wvc-app-core`,
  `wvc-core`, `wvc-base`, `wvc-setup-hints`, `terminal-launch` and
  `wvc-code-graph` (including clippy 1.97 `-D warnings`).
- E2E system prompt identity test now accepts Weavecoder.
- Fork author references cleaned from the codebase (maintainer, URLs, LICENSE).

[Unreleased]: https://github.com/nicolasramos-es/weavecoder/compare/v0.67.0-dev...HEAD
[0.67.0]: https://github.com/nicolasramos-es/weavecoder/releases/tag/v0.67.0-dev
