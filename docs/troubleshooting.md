# Troubleshooting

Common problems, their causes, and fixes. If your issue is not listed here,
try `wvc --help`, run with `--trace` for verbose tool/model logging, and open
an issue on the repository.

## Installation

### The installer fails with a SHA-256 checksum mismatch

The installer (`install.sh` / `install.ps1`) verifies the downloaded binary's
SHA-256 digest against the digest published in the release and **aborts with a
clear error** when they don't match. This is a safety feature, not a bug.

- Retry the install — the download may have been corrupted or truncated.
- If it keeps failing, check that nothing in your network path (proxy, MITM
  TLS inspection, antivirus) is rewriting the download.
- Download the binary manually from the [Releases](https://github.com/nicolasramos-es/weavecoder/releases)
  page and compare checksums to isolate the problem.

### Installer says the repository is private / 404

The default install URLs resolve against GitHub. If the repository is
private at install time, export a token first:

```bash
export GITHUB_TOKEN=ghp_xxx        # or WVC_GITHUB_TOKEN
curl -fsSL https://raw.githubusercontent.com/nicolasramos-es/weavecoder/main/install.sh | bash
```

### `wvc` is not found after install

The installer places a launcher symlink on your `PATH` (e.g.
`~/.local/bin/weavecoder`). If the command still isn't found:

- Make sure the install directory (e.g. `~/.local/bin`) is **before**
  `~/.cargo/bin` in `PATH`.
- On Windows, check `%LOCALAPPDATA%\weavecoder\bin`.
- Re-open your shell so `PATH` is re-read.

### Build from source fails

```bash
git clone https://github.com/nicolasramos-es/weavecoder.git
cd weavecoder
cargo build --release --bin wvc
```

The workspace uses Rust edition 2024, so use a **recent stable Rust
toolchain**. If `cargo build` fails, update the toolchain first:

```bash
rustup update stable
```

## Login and providers

### `wvc login --provider ollama` errors

`login` takes the provider as a **positional argument**, not via the global
`--provider` flag (clap drops the global flag inside `login`):

```bash
wvc login ollama      # correct
wvc login --provider ollama   # error
```

### OAuth login doesn't open a browser

Use `--no-browser` (alias `--headless`) over SSH or on headless machines:

```bash
wvc login claude --no-browser
```

For fully scripted flows: `wvc login <provider> --print-auth-url` prints a
script-friendly URL, then complete it with `--callback-url <url>` or
`--auth-code <code>`.

### A local model server isn't detected

Local runtimes are exposed through specific providers: `ollama`, `lmstudio`,
and `openai-compatible` (for oMLX, llama.cpp, or any custom endpoint). For a
custom endpoint:

```bash
wvc login openai-compatible --api-base http://localhost:8000/v1 --api-key none
```

Diagnose end-to-end with `wvc auth-test` (login, credential probe, refresh,
and provider smoke) or check stored state with `wvc auth status` and
`wvc provider list`.

## Daemon and server

### I rebuilt the binary but nothing changed

`wvc run` and interactive sessions are served by the **long-lived daemon**,
which keeps running the previously installed binary. A freshly built binary is
inert until the daemon reloads it:

```bash
wvc server reload     # or: wvc server stop && wvc serve
```

### Test a build without touching the shared daemon

Run your build against its own socket:

```bash
cargo build --profile selfdev
./target/selfdev/weavecoder run --no-update --socket /tmp/wvc-mytest.sock '<prompt>'
```

This avoids disturbing the shared daemon or any session you already have open.

### `wvc server stop` warns about dropping sessions

`server stop` terminates the daemon and drops any live headless/swarm
sessions, so it requires confirmation (`--force` to skip). If you are running
parallel swarm work, let it finish or stop it cleanly first.

## Code Knowledge Graph / indexing

### `wvc init` reports an unsupported file extension

The CKG parses **Go (`.go`), Python (`.py`, `.pyi`), TypeScript (`.ts`,
`.tsx`), and Rust (`.rs`)**. Other languages are skipped by the scanner;
parsing an unsupported file directly reports `unsupported file extension`.
Files in other languages simply don't produce symbols — the rest of the
project still indexes.

### I ran `wvc init` without `--db` and the search finds nothing

Without `--db`, the graph is stored **in-memory** and is gone when the process
exits. Always pass a database path to persist it:

```bash
wvc init /path/to/project --db ckg.db
wvc code-search "parseConfig" --db ckg.db
```

### Re-indexing is slow

`wvc init` is **incremental**: files are classified by SHA-256 hash, so
unchanged files are skipped, modified files are re-indexed, and deleted files
are purged. The first run over a large repository is the slow one; later runs
only touch what changed.

### Semantic search doesn't seem to kick in

Semantic results require symbol embeddings, which are produced by the local
all-MiniLM-L6-v2 model. The model and tokenizer are downloaded from Hugging
Face on first use — if the machine was offline during indexing, symbols carry
no embeddings and search falls back to FTS5 + graph signals. Re-run
`wvc init --db ckg.db` with network access to populate embeddings.

## FAQ

**Do I need an API key to use Weavecoder?**
Only for cloud providers. Local providers — `ollama`, `lmstudio`,
`openai-compatible` (oMLX, llama.cpp, custom endpoints) — need no key and no
cloud account.

**Why does a swarm worker hit "stream timeout" against my local model?**
A local model endpoint (oMLX, llama.cpp, Ollama) usually serves **one
generation at a time**. If you launch many swarm workers against the *same*
local model, they queue up and the later ones wait — and if the model is slow
to emit its first token (or is saturated by parallel requests), the stream can
exceed the idle timeout (default `180s`) and abort with
`OpenAI-compatible stream timeout`.

Three ways to fix it:
1. **Raise the local server's concurrency/parallelism** if it supports it (e.g.
   oMLX/vLLM `--parallel`/slots), so it can serve the number of swarm workers
   you launch.
2. **Spread workers across different local models/endpoints** instead of
   hammering one.
3. **Raise the idle timeout** so slow local models aren't cut off mid-thought:
   ```toml
   [provider]
   stream_idle_timeout_secs = 600
   ```
   (or set `WVC_STREAM_IDLE_TIMEOUT_SECS=600` for a single launch).

The timeout is only the symptom of the model being busy/slow; the root cause is
saturation of a single-request local endpoint by parallel swarm workers.

**Why does a headless agent get "this command is blocked" or "requires your approval"?**
A tool whose permission is set to `ask` surfaces an approval prompt for you to
approve in the TUI. In a **headless** session — a swarm worker, `wvc run`, or a
server-side agent with no interactive terminal — there is nobody to approve, so
the tool would deadlock forever.

Since v0.68.8, Weavecoder downgrades `ask` to `allow` **only** in headless
sessions (detected by the absence of an interactive stdin channel), so headless
agents can run their tools. In an interactive TUI, `ask` still prompts you as
before.

The default is already permissive: with no `[tools.permissions]` overrides and
`disk_mode` unset (or `"full"`), every tool is `allow`. If you *want* a tool
hard-blocked everywhere, set it to `deny`:
```toml
[tools.permissions]
bash = "deny"   # blocked in both interactive and headless sessions
```

**Does Weavecoder send my code anywhere?**
The Code Knowledge Graph, embeddings, and search all run locally on your
machine. Code leaves your machine only when you explicitly run a request
against a cloud provider.

**Which languages does the Code Knowledge Graph support?**
Go, Python, TypeScript (including TSX), and Rust — parsed with tree-sitter.

**How do I update Weavecoder?**
Run `wvc update`. Release builds also check for updates automatically
(`--no-update` skips the check for a single run).

**Where is configuration stored?**
Under the platform config directory, e.g. `~/.config/wvc` on Linux/macOS
(`%APPDATA%\wvc` on Windows). `WVC_HOME` overrides the location. Provider
profiles live in `config.toml` under `[providers.<name>]`.
