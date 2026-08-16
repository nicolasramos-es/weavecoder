# Contributing to Weavecoder

Thank you for your interest in contributing to Weavecoder! This document covers
everything you need to know to get started.

## Table of Contents

- [Prerequisites](#prerequisites)
- [Building](#building)
- [Running Tests](#running-tests)
- [CI / Quality Gates](#ci--quality-gates)
- [Code Style](#code-style)
- [Submitting Changes](#submitting-changes)
- [Project Structure](#project-structure)

## Prerequisites

- **Rust**: edition 2024, toolchain 1.85 or later
  - Install via [rustup](https://rustup.rs/): `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
  - Verify: `rustc --version` (should be ≥ 1.85)
- **Git**: for cloning and PRs
- **A GitHub account** with write access to this repository (or a fork)

## Building

```bash
git clone https://github.com/nicolasramos-es/weavecoder.git
cd weavecoder
cargo build --release --bin wvc
```

The binary will be at `target/release/wvc`.

### Self-dev builds

For rapid iteration, use the self-dev profile which targets a separate output
directory so it does not interfere with the shared daemon:

```bash
cargo build --profile selfdev
```

The resulting binary is at `target/selfdev/wvc`. The build job count is capped
at 4 by `.cargo/config.toml` to stay memory-safe; the recommended path for
bigger machines is `scripts/dev_cargo.sh`, which sizes parallelism from
currently-available memory.

> **Note:** `cargo build` alone does not prove behaviour. The long-lived daemon
> at `~/.wvc/builds/shared-server/wvc` serves all `wvc run`
> sessions. Until that symlink is repointed and the daemon restarted
> (`wvc self-dev --build`), a freshly built binary is inert and every
> runtime check silently measures the old code.
>
> To test a change without disturbing the shared daemon:
>
> ```bash
> cargo build --profile selfdev
> ./target/selfdev/wvc run --no-update \
>   --socket /run/user/1000/weavecoder-mytest.sock '<your prompt>'
> ```

## Running Tests

### Unit tests

```bash
cargo test --workspace
```

### Integration / E2E tests

The project uses a combination of unit tests, integration tests in `tests/`,
and E2E smoke tests (`tests/e2e/`). Run the full suite with:

```bash
cargo test --workspace
```

## CI / Quality Gates

All Linux CI jobs run on the self-hosted runner. macOS and Windows workflows
use GitHub-hosted runners and are manual-only to control costs.

| Workflow | File | Trigger | Purpose |
|---|---|---|---|
| **CI** | `.github/workflows/ci.yml` | PRs to `main`/`master`, manual dispatch | Quality guardrails: clippy, tests, build |
| **CI Cross-Platform** | `.github/workflows/ci-cross-platform.yml` | Manual dispatch | macOS/Windows builds on hosted runners |
| **Matrix Test** | `.github/workflows/matrix-test.yml` | Manual dispatch | Cross-platform test matrix (Linux + macOS) |
| **Release** | `.github/workflows/release.yml` | Tags matching `v*` | Publish releases to GitHub Releases |
| **Require Issue** | `.github/workflows/require-issue.yml` | PR opened/edited/synchronize | Ensures every PR links a real GitHub issue |
| **Windows Smoke** | `.github/workflows/windows-smoke.yml` | Manual dispatch | Quick Windows smoke test |

Quality gates enforced by `.github/workflows/ci.yml`:

- Module declarations resolve (`scripts/check_module_files.py`)
- Formatting: `cargo fmt --all -- --check`
- Compilation: `cargo check --all-targets --all-features`
- Clippy with warnings denied on the Phase 1 crate:
  `cargo clippy -p wvc-code-graph --all-targets --all-features -- -D warnings`
- Workspace clippy is informational (fork-inherited crates are linted but not
  gated)

## Code Style

- **Formatting**: `cargo fmt` — run before committing
- **Linting**: `cargo clippy` — the Phase 1 crate (`wvc-code-graph`) must be
  warning-free with `-D warnings`; keep the rest of the workspace clean too
- **Commit messages**: Use [conventional commits](https://www.conventionalcommits.org/)
  (e.g., `feat:`, `fix:`, `refactor:`, `docs:`, `ci:`, `chore:`)

## Submitting Changes

1. **Create a feature branch** from `main`:
   ```bash
   git checkout main
   git pull origin main
   git checkout -b feat/<your-feature>
   ```

2. **Make your changes**. Follow the code style guidelines above.

3. **Run the quality gates** to ensure nothing is broken:
   ```bash
   cargo test --workspace
   cargo fmt --all -- --check
   cargo check --all-targets --all-features
   cargo clippy -p wvc-code-graph --all-targets --all-features -- -D warnings
   ```

4. **Commit** with a descriptive conventional commit message:
   ```bash
   git add .
   git commit -m "feat: add <description>"
   ```

5. **Push and open a PR**:
   ```bash
   git push origin feat/<your-feature>
   gh pr create --title "feat: <description>" --body-file PR_BODY.md
   ```

6. **Link a real GitHub issue**. Every PR must reference an existing GitHub
   issue in this repository — the `require-issue.yml` workflow enforces this
   and fails the PR otherwise.

## Project Structure

```
weavecoder/
├── Cargo.toml              # Workspace manifest
├── Cargo.lock
├── LICENSE
├── README.md
├── AGENTS.md               # Agent-specific guidelines
│
├── src/                    # Main binary (wvc CLI)
│   ├── main.rs             # Entry point
│   ├── lib.rs
│   └── cli/                # CLI layer: args, dispatch, provider init
│
├── crates/                 # Workspace crates (all prefixed wvc-*)
│   ├── wvc-code-graph/     # Code Knowledge Graph (tree-sitter, SQLite, FTS5)
│   ├── wvc-embedding/      # Local embeddings (all-MiniLM-L6-v2)
│   ├── wvc-swarm-core/     # Agent swarm orchestration
│   ├── wvc-app-core/
│   ├── wvc-base/
│   ├── wvc-tui/
│   └── ...                 # ~85 crates total
│
├── docs/                   # User-facing documentation
│   ├── installation.md
│   ├── commands.md
│   ├── architecture.md
│   └── troubleshooting.md
│
├── scripts/                # Build, install, and utility scripts
│   ├── install.sh          # Linux/macOS installer
│   ├── install.ps1         # Windows installer
│   └── ...
│
├── tests/                  # Integration / E2E tests
├── assets/                 # Static assets (icons, logos)
└── .github/workflows/      # CI/CD workflows
```

### Crate naming convention

All workspace crates use the `wvc-` prefix (e.g., `wvc-code-graph`,
`wvc-tui`, `wvc-swarm-core`). The main binary (`wvc`) is the root crate.

## Questions?

If you have questions, feel free to open an issue or start a discussion.
We welcome all contributions — big or small!
