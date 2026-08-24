# CLI Command Reference

Complete reference of all `wvc` commands, flags, and options. Source of truth is `src/cli/args.rs` (enum `Command` and subcommands).

## Quick Start

```bash
wvc run "Hello, weavecoder!"
```

## Global Flags

These flags work with every command:

| Flag | Description | Default |
|------|-------------|---------|
| `-p, --provider <PROVIDER>` | Initial provider to use | `auto` |
| `-C, --cwd <DIR>` | Working directory for the local client process | — |
| `--remote-working-dir <DIR>` | Working directory sent to a remote server when using `--socket` | — |
| `--no-update` | Skip the automatic update check | — |
| `--auto-update` | Auto-update when new version is available | `true` (release builds) |
| `--trace` | Log tool inputs/outputs and token usage to stderr | — |
| `--quiet` | Suppress non-error CLI/status output for scripting | — |
| `--resume [ID]` | Resume a session by ID, or list sessions if no ID provided | — |
| `--socket <PATH>` | Custom socket path for server/client communication | — |
| `--debug-socket` | Enable debug socket (broadcasts all TUI state changes) | — |
| `-m, --model <MODEL>` | Model to use (e.g., `claude-opus-4-6`, `gpt-5.5`) | — |
| `--provider-profile <NAME>` | Named provider profile from `[providers.<name>]` in config.toml | — |
| `--tool-profile <PROFILE>` | Tool profile to expose: `full`, `minimal`/`lite`, or `none` | — |
| `--tools <TOOLS>` | Comma-separated allow-list of tools (e.g., `bash,read,write,apply_patch`). Use `*` or `all` for unrestricted | — |
| `--disabled-tools <TOOLS>` | Comma-separated list of tools to hide after applying the selected profile | — |
| `--disable-base-tools` | Hide all built-in tools unless `--tools` or `[tools].enabled` opts tools back in | — |

## Commands

### `wvc run`

Run a single message and exit.

```bash
wvc run "Refactor the auth module to use JWT"
```

| Flag | Description |
|------|-------------|
| `--json` | Emit a machine-readable JSON result instead of streaming text |
| `--ndjson` | Emit newline-delimited JSON events while the response streams |

**Example:**

```bash
wvc run "What files handle authentication?" --json
```

---

### `wvc login`

Login to a provider via OAuth, API key, or local credentials.

```bash
wvc login google
wvc login openai
```

| Flag | Description |
|------|-------------|
| `<PROVIDER>` | Provider to log in to (positional, e.g., `google`, `openai`) |
| `-a, --account <LABEL>` | Account label for multi-account support |
| `--no-browser` | Do not try to open a browser locally (useful over SSH) |
| `--print-auth-url` | Print a script-friendly auth URL and persist temporary login state |
| `--callback-url <URL>` | Complete a previously printed auth flow using a full callback URL |
| `--auth-code <CODE>` | Complete a previously printed auth flow using a provider-issued authorization code |
| `--json` | Emit machine-readable JSON for script-friendly login flows |
| `--complete` | Resume a pending scriptable login flow |
| `--no-validate` | Save credentials without running post-login live provider validation |
| `--google-access-tier <TIER>` | Gmail/Google access tier: `full` or `readonly` |
| `--api-base <URL>` | OpenAI-compatible API base URL |
| `--api-key <KEY>` | OpenAI-compatible API key |
| `--api-key-env <VAR>` | Environment variable name for an OpenAI-compatible API key |

**Example:**

```bash
wvc login claude --account work
wvc login openai-compatible --api-base https://my-gateway.local/v1 --api-key-env MY_API_KEY
```

**Local model auto-detection:** in addition to Ollama (port 11434) and
LM Studio (port 1234), `wvc login` probes local OpenAI-compatible servers and
lists them as available options when they respond with a valid model catalog:

| Server | Default port | Verification endpoint |
|--------|--------------|----------------------|
| llama.cpp | `8080` | `GET /v1/models` |
| vLLM | `8000` | `GET /v1/models` |
| oMLX | `8081` | `GET /v1/models` |

Ports are probed with a 2-second timeout. If no local server responds, only
cloud providers are shown (no regression).

---

### `wvc init`

Initialize a Code Knowledge Graph from project source files (scan + AST extraction + SQLite storage).

```bash
wvc init /path/to/project
```

| Flag | Description |
|------|-------------|
| `<PATH>` | Project directory to scan |
| `--db <PATH>` | SQLite database path to persist the code graph (defaults to in-memory) |

**Example:**

```bash
wvc init ./my-project --db ~/.wvc/codegraph.db
```

---

### `wvc code-search`

Search the Code Knowledge Graph (hybrid: FTS5 + semantic + graph). Results are
cached in-memory for the session (LRU, max 50): the same query in the same
working directory returns instantly from cache on the second call. The cache
invalidates on `wvc init` (re-indexation) or a change of working directory.

```bash
wvc code-search "authentication handler"
```

| Flag | Description | Default |
|------|-------------|---------|
| `<QUERY>` | Search query (symbol name, meaning, or substring) | — |
| `--db <PATH>` | SQLite database path with an initialized code graph | — |
| `--top-k <N>` | Maximum number of results | `10` |

**Example:**

```bash
wvc code-search "JWT token validation" --db ~/.wvc/codegraph.db --top-k 20
```

---

### `wvc swarm`

Spawn a swarm worker with an optional profile that shapes its behavior. The
message is sent to the running server over the debug socket as a swarm task
(requires a running server with `debug_socket` enabled).

```bash
wvc swarm "investigate how payment retries are scheduled" --worker-profile researcher
```

| Flag | Description |
|------|-------------|
| `<MESSAGE>` | The task description for the spawned worker |
| `--worker-profile <PROFILE>` | Worker profile defining the system prompt: `coder` (default), `tester`, `reviewer`, `researcher` |

- `coder` — generates code that compiles and passes lint.
- `tester` — writes and runs tests, reports pass/fail.
- `reviewer` — reviews code, gives APPROVED/CHANGES/REJECTED verdict.
- `researcher` — investigates APIs/dependencies, resumes findings.

The default worker model is **local-first** auto-detection (oMLX → Ollama →
vLLM → cloud fallback).

---

### `wvc server`

Manage the background server daemon.

```bash
wvc server start
wvc server reload
wvc server stop --force
```

#### `wvc server start`

Start the background server if it is not already running.

| Flag | Description |
|------|-------------|
| `--json` | Emit JSON instead of human-readable text |

#### `wvc server reload`

Gracefully reload the running background server onto the newest binary. Preserves live sessions.

| Flag | Description |
|------|-------------|
| `--force` | Reload even if the running server is already on the newest binary |
| `--json` | Emit JSON instead of human-readable text |

#### `wvc server stop`

Stop the running background server and clear its socket. Requires `--force` to acknowledge session loss.

| Flag | Description |
|------|-------------|
| `--force` | Confirm terminating the daemon (and dropping live sessions) |
| `--json` | Emit JSON instead of human-readable text |

---

### `wvc connect`

Connect to a running server.

```bash
wvc connect
```

---

### `wvc repl`

Run in simple REPL mode (no TUI).

```bash
wvc repl
```

---

### `wvc update`

Update wvc to the latest version.

```bash
wvc update
```

---

### `wvc version`

Show build/version information in human or JSON form.

```bash
wvc version
wvc version --json
```

| Flag | Description |
|------|-------------|
| `--json` | Emit JSON instead of plain text |

---

### `wvc usage`

Show usage limits for connected providers.

```bash
wvc usage
wvc usage --json
```

| Flag | Description |
|------|-------------|
| `--json` | Emit JSON instead of plain text |

---

### `wvc self-dev` (alias: `selfdev`)

Self-development mode: run as a canary session on the shared server.

```bash
wvc self-dev
wvc self-dev --build
```

| Flag | Description |
|------|-------------|
| `--build` | Build and test a new canary version before launching |

---

### `wvc debug`

Debug socket CLI — interact with a running wvc server.

```bash
wvc debug help
wvc debug sessions
wvc debug message "Hello" --wait
```

| Flag | Description | Default |
|------|-------------|---------|
| `<COMMAND>` | Debug command to run | `help` |
| `<ARG>` | Optional argument for the command | `""` |
| `-S, --session <ID>` | Target a specific session by ID | — |
| `-s, --socket <PATH>` | Connect to a specific server socket path | — |
| `-w, --wait` | Wait for response to complete (for message command) | — |

---

### `wvc auth`

Authentication status and validation helpers.

#### `wvc auth status`

Show configured authentication status for model/tool providers.

```bash
wvc auth status
wvc auth status --json
```

| Flag | Description |
|------|-------------|
| `--json` | Emit JSON instead of plain text |

#### `wvc auth doctor [PROVIDER]`

Diagnose provider auth issues and suggest next steps.

```bash
wvc auth doctor
wvc auth doctor claude --validate
```

| Flag | Description |
|------|-------------|
| `<PROVIDER>` | Optional provider id or alias to focus diagnosis |
| `--validate` | Run live post-login validation for configured providers |
| `--json` | Emit JSON instead of plain text |

---

### `wvc provider`

Provider discovery and selection helpers.

#### `wvc provider list`

List provider IDs you can pass to `-p`/`--provider`.

```bash
wvc provider list
wvc provider list --json
```

| Flag | Description |
|------|-------------|
| `--json` | Emit JSON instead of plain text |

#### `wvc provider current`

Show the currently requested and resolved provider selection.

```bash
wvc provider current
wvc provider current --json
```

| Flag | Description |
|------|-------------|
| `--json` | Emit JSON instead of plain text |

#### `wvc provider add <NAME>`

Add a named OpenAI-compatible API provider profile.

```bash
wvc provider add my-gateway --base-url https://llm.example.com/v1 --model gpt-4
```

| Flag | Description |
|------|-------------|
| `<NAME>` | Profile name used with `--provider-profile` and config defaults |
| `--base-url <URL>` | OpenAI-compatible API base URL |
| `-m, --model <MODEL>` | Default model id for this provider profile |
| `--context-window <N>` | Optional model context window in tokens |
| `--api-key-env <VAR>` | Environment variable name that contains the API key |
| `--api-key <KEY>` | API key value to store (prefer `--api-key-stdin` for shell history safety) |
| `--api-key-stdin` | Read the API key from stdin |
| `--no-api-key` | Configure the provider with no API key/authentication |
| `--auth <STYLE>` | Authentication style: `bearer`, `api-key`, or `none` |
| `--auth-header <NAME>` | Header name when `--auth api-key` is used (default: `api-key`) |
| `--env-file <NAME>` | Private env file name under wvc's app config directory |
| `--set-default` | Make this profile the startup default provider/model |
| `--overwrite` | Replace an existing profile with the same name |
| `--provider-routing` | Allow provider-routing features for OpenRouter-style gateways |
| `--model-catalog` | Fetch/list models from the provider's `/models` endpoint |
| `--json` | Emit JSON instead of human-readable setup output |

---

### `wvc memory`

Memory management commands.

#### `wvc memory list`

List all stored memories.

```bash
wvc memory list
wvc memory list --scope project --tag auth
```

| Flag | Description | Default |
|------|-------------|---------|
| `-s, --scope <SCOPE>` | Filter by scope: `project`, `global`, or `all` | `all` |
| `-t, --tag <TAG>` | Filter by tag | — |

#### `wvc memory search <QUERY>`

Search memories by query.

```bash
wvc memory search "authentication"
wvc memory search "authentication" --semantic
```

| Flag | Description |
|------|-------------|
| `<QUERY>` | Search query |
| `-s, --semantic` | Use semantic search (embedding-based) instead of keyword |

#### `wvc memory export <OUTPUT>`

Export memories to a JSON file.

```bash
wvc memory export ~/memories.json
```

| Flag | Description | Default |
|------|-------------|---------|
| `<OUTPUT>` | Output file path | — |
| `-s, --scope <SCOPE>` | Export scope: `project`, `global`, or `all` | `all` |

#### `wvc memory import <INPUT>`

Import memories from a JSON file.

```bash
wvc memory import ~/memories.json
```

| Flag | Description | Default |
|------|-------------|---------|
| `<INPUT>` | Input file path | — |
| `-s, --scope <SCOPE>` | Import scope: `project` or `global` | `project` |
| `--overwrite` | Overwrite existing memories with same ID | — |

#### `wvc memory stats`

Show memory statistics.

```bash
wvc memory stats
```

---

### `wvc session`

Session management commands.

#### `wvc session rename`

Rename a saved session's human-readable name/title.

```bash
wvc session rename fox "Authentication Refactor"
wvc session rename fox --clear
```

| Flag | Description |
|------|-------------|
| `<SESSION>` | Session ID or memorable short name |
| `<NAME>` | New session name/title |
| `--clear` | Clear the custom session name/title |
| `--json` | Emit JSON instead of human-readable output |

---

### `wvc account`

Log in to and manage your Weavecoder account.

#### `wvc account login`

Open browser-based device authorization and wait for plan activation.

```bash
wvc account login
wvc account login --no-browser
```

| Flag | Description |
|------|-------------|
| `--no-browser` | Do not open a browser automatically; print the public approval URL |

#### `wvc account status`

Show canonical account, plan, and usage status from `/v1/me`.

```bash
wvc account status
wvc account status --json
```

| Flag | Description |
|------|-------------|
| `--json` | Emit JSON instead of human-readable output |

#### `wvc account manage`

Open the public Weavecoder account management page.

```bash
wvc account manage
```

#### `wvc account logout`

Revoke the current key when reachable, then securely clear local state.

```bash
wvc account logout
```

---

### `wvc pair`

Generate a pairing code for iOS/web client.

```bash
wvc pair
wvc pair --list
wvc pair --revoke "My iPhone"
```

| Flag | Description |
|------|-------------|
| `--list` | List paired devices instead of generating a code |
| `--revoke <NAME>` | Revoke a paired device by name or ID |

---

### `wvc permissions`

Review and respond to pending ambient permission requests.

```bash
wvc permissions
```

---

### `wvc transcript`

Inject externally transcribed text into the active Weavecoder TUI.

```bash
wvc transcript "Transcribed text here"
wvc transcript "Transcribed text" --mode insert --session abc123
```

| Flag | Description | Default |
|------|-------------|---------|
| `<TEXT>` | Transcript text (reads from stdin if omitted) | — |
| `--mode <MODE>` | How to apply the transcript: `insert`, `append`, `replace`, or `send` | `send` |
| `-S, --session <ID>` | Target a specific live session instead of the active TUI | — |

---

### `wvc dictate`

Run configured dictation: send to last-focused wvc client or type raw text.

```bash
wvc dictate
wvc dictate --type
```

| Flag | Description |
|------|-------------|
| `--type` | Type the transcript into the focused app instead of sending to wvc |

---

### `wvc setup-hotkey`

Set up the platform global hotkey to launch wvc.

```bash
wvc setup-hotkey
wvc setup-hotkey --uninstall
```

| Flag | Description |
|------|-------------|
| `--uninstall` | Remove the installed platform global hotkey listener |

---

### `wvc setup-launcher`

Install a launcher so wvc appears in your app launcher.

```bash
wvc setup-launcher
```

---

### `wvc browser`

Browser automation setup and status.

```bash
wvc browser setup
wvc browser status
```

| Flag | Description | Default |
|------|-------------|---------|
| `<ACTION>` | Action: `setup` or `status` | `setup` |

---

### `wvc replay`

Replay a saved session in the TUI.

```bash
wvc replay my-session
wvc replay session.json --swarm --speed 2.0 --auto-edit
```

| Flag | Description | Default |
|------|-------------|---------|
| `<SESSION>` | Session ID, name, or path to session JSON file | — |
| `--swarm` | Replay related swarm sessions together in a synchronized multi-pane view | — |
| `--export` | Export timeline as JSON instead of playing | — |
| `--speed <N>` | Playback speed multiplier | `1.0` |
| `--timeline <PATH>` | Path to an edited timeline JSON file (overrides session timing) | — |
| `--auto-edit` | Auto-edit timeline: compress tool call wait times and gaps | — |
| `--video [PATH]` | Export as video file (auto-generates name if no path given) | — |
| `--cols <N>` | Video width in columns | `120` |
| `--rows <N>` | Video height in rows | `40` |
| `--fps <N>` | Video frames per second | `60` |
| `--centered` | Force centered layout (overrides config) | — |
| `--no-centered` | Force left-aligned (non-centered) layout (overrides config) | — |

---

### `wvc model`

Model management commands.

#### `wvc model list`

List model names you can pass to `-m`/`--model`.

```bash
wvc model list
wvc model list --json --verbose
```

| Flag | Description |
|------|-------------|
| `--json` | Emit JSON instead of plain text |
| `--verbose` | Show provider/selection summary before the list |

---

### `wvc provider-test-coverage`

Show live verification coverage. With no provider/model, prints the full coverage summary.

```bash
wvc provider-test-coverage
wvc provider-test-coverage claude claude-opus-4-6
```

| Flag | Description | Default |
|------|-------------|---------|
| `<PROVIDER>` | Provider to look up | — |
| `<MODEL>` | Model to look up (defaults to global `--model` only when PROVIDER is supplied) | — |
| `--coverage-file <PATH>` | Read coverage from this JSON file instead of the default live-test coverage ledger | — |
| `--coverage-limit <N>` | Maximum provider/model pairs to list in the full summary (`0` = show all) | `0` |

---

### `wvc provider-doctor <PROVIDER>`

Diagnose why a provider/model or the model picker is broken by walking the strict end-to-end checkpoints (catalog, picker, model-switch, chat, streaming, tools).

```bash
wvc provider-doctor claude
wvc provider-doctor claude --tier full --json
```

| Flag | Description | Default |
|------|-------------|---------|
| `<PROVIDER>` | OpenAI-compatible provider id to diagnose (e.g., `cerebras`, `fpt`, `nvidia-nim`) | — |
| `--tier <TIER>` | How much to exercise: `offline` (no key/no spend), `catalog` (key, ~no spend), or `full` (key, spends balance: chat + streaming + tools) | `catalog` |
| `--json` | Emit the report as JSON for scripting | — |

---

### `wvc auth-test`

Test authentication end-to-end: login (optional), credential probe, refresh, and provider smoke.

```bash
wvc auth-test
wvc auth-test --login --all-configured
```

| Flag | Description |
|------|-------------|
| `--login` | Run the provider login flow before validation (interactive/browser-based) |
| `--all-configured` | Test all currently configured supported auth providers instead of just `--provider` |
| `--no-smoke` | Skip the provider runtime smoke prompt |
| `--no-tool-smoke` | Skip the tool-enabled runtime smoke prompt |
| `--prompt <TEXT>` | Custom smoke prompt (default asks for `AUTH_TEST_OK`) |
| `--json` | Emit JSON report instead of human-readable output |
| `--output <PATH>` | Write the full auth-test report JSON to a file |
| `--coverage` | Show strict live provider/model E2E coverage instead of running auth tests |
| `--context-audit` | Fetch live model catalogs and verify context-window resolution for each model |
| `--coverage-file <PATH>` | Read coverage from this JSON file (requires `--coverage`) |
| `--coverage-limit <N>` | Maximum uncovered provider/model gaps to show in the text coverage report | `50` |

---

### `wvc restart`

Save or restore the current set of open wvc windows across a system reboot.

#### `wvc restart save`

Save a reboot snapshot of currently active wvc windows.

```bash
wvc restart save --auto-restore
```

| Flag | Description |
|------|-------------|
| `--auto-restore` | Restore this reboot snapshot automatically the next time plain `wvc` starts |

#### `wvc restart restore`

Restore the most recently saved reboot snapshot.

```bash
wvc restart restore
```

#### `wvc restart status`

Show the currently saved reboot snapshot.

```bash
wvc restart status
```

#### `wvc restart clear`

Remove the currently saved reboot snapshot.

```bash
wvc restart clear
```

---

### `wvc menubar` (aliases: `menu-bar`, `statusbar`)

Show a live macOS menu bar indicator with running/streaming session counts.

```bash
wvc menubar
wvc menubar --once
wvc menubar --json
```

| Flag | Description |
|------|-------------|
| `--once` | Print the current counts once as text and exit (no menu bar item) |
| `--json` | Emit the current counts as JSON and exit |

---

### `wvc api-bridge` (alias: `api`)

Serve the stable harness API on a Unix socket, for SDK clients. This is the endpoint the TypeScript SDK (`@nicolasramos-es/weavecoder-sdk`) connects to.

```bash
wvc api-bridge
wvc api-bridge --api-socket /tmp/wvc-api.sock
```

| Flag | Description | Default |
|------|-------------|---------|
| `--api-socket <PATH>` | Path of the API socket to listen on | `$XDG_RUNTIME_DIR/wvc-api.sock` |

---

### `wvc ambient`

Ambient mode management.

```bash
wvc ambient status
wvc ambient trigger
wvc ambient stop
```

| Subcommand | Description |
|------------|-------------|
| `status` | Show ambient mode status |
| `log` | Show recent ambient activity log |
| `trigger` | Manually trigger an ambient cycle |
| `stop` | Stop ambient mode |

---

### `wvc cloud`

Optional Weavecoder Cloud/Jade integration commands.

#### `wvc cloud sessions`

Upload, list, verify, and view cloud-synced sessions.

| Subcommand | Description |
|------------|-------------|
| `configure` | Configure Jade API defaults for cloud sessions |
| `status` | Show saved Jade API defaults without printing secrets |
| `upload <FILE>` | Upload a specific local session JSON file to Jade cloud storage |
| `upload-latest` | Upload the newest local Weavecoder session to Jade cloud storage |
| `sync` | Sync new or changed local sessions to Jade cloud storage (idempotent; safe to schedule) |
| `list` | List cloud-uploaded sessions from the Jade index |
| `verify <SESSION_ID>` | Verify that cloud metadata and the S3 session blob both exist |
| `dashboard` | Render a local HTML dashboard of cloud-uploaded sessions |
| `view <SESSION_ID>` | Download and view a cloud-uploaded session |

---

## Concepts

### `--provider`

Selects the AI provider to use. Values from `ProviderChoice` enum:

```
wvc, claude, anthropic-api, claude-subprocess (deprecated), openai, openai-api,
openrouter, bedrock, azure, opencode, opencode-go, zai, kimi, 302ai, baseten,
cortecs, comtegra, deepseek, fpt, firmware, huggingface, moonshotai, nebius,
scaleway, stackit, groq, mistral, perplexity, togetherai, deepinfra, fireworks,
minimax, xai, nvidia-nim, xiaomi-mimo, celeris, lmstudio, ollama, chutes,
cerebras, alibaba-coding-plan, openai-compatible, cursor, copilot, gemini,
gemini-api, antigravity, google, auto
```

Default: `auto` (auto-detect). Interactive sessions can switch providers with `/model`.

### `--model`

Specifies the model to use (e.g., `claude-opus-4-6`, `gpt-5.5`). See `wvc model list` for available models.

### `--socket`

Custom socket path for server/client communication. Use this to connect to a specific server instance.

### `--resume [ID]`

Resume a session by ID, or list all sessions if no ID is provided.

### `--trace`

Log tool inputs/outputs and token usage to stderr. Useful for debugging.

### `--quiet`

Suppress non-error CLI/status output. Useful for scripting and wrappers.

### `--tools` / `--disabled-tools`

Fine-grained control over which tools are exposed to the model. `--tools` is an allow-list (comma-separated), `--disabled-tools` is a hide-list applied after the tool profile.

### `--tool-profile`

Predefined tool sets: `full` (all tools), `minimal`/`lite` (reduced set), or `none` (no tools).

### `--provider-profile`

Named provider profile from `[providers.<name>]` in `config.toml`. Implies `--provider openai-compatible` for OpenAI-compatible profiles.

### `--disable-base-tools`

Hide all built-in tools unless `--tools` or `[tools].enabled` opts tools back in.

### Config

Configuration is stored in `~/.wvc/` (or the platform-appropriate config directory). Key paths:

- Config file: `~/.wvc/config.toml`
- Sessions: `~/.wvc/sessions/`
- Builds: `~/.weavecoder/builds/` (stable, canary, versions, shared-server)
- Launcher symlink: `~/.local/bin/weavecoder`
