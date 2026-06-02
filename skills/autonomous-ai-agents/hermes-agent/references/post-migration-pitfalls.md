# Post-Migration Pitfalls

Things that look like "something is broken" but are actually missing dependencies or stale state after moving Hermes to a new machine.

## `computer_use` fails with "No module named 'mcp'"

The `mcp` Python SDK is not a transitive dependency of Hermes — cua-driver's MCP backend imports it lazily. After migration:

```bash
pip install mcp
```

Then `/reset` (or start a new session). The backend singleton is per-process; it won't recover mid-session.

## `computer_use` fails with "cua-driver session not started"

Same root cause as above — the backend was created before `mcp` was available. Install `mcp` and start a new session.

## Dashboard not accessible from LAN

The dashboard binds to `127.0.0.1` by default. After migration, run:

```bash
hermes dashboard --host 0.0.0.0 --port 9119 --insecure --tui
```

And persist the host binding:

```bash
hermes config set API_SERVER_HOST 0.0.0.0
```

## Gateway starts but no messages delivered

Check `hermes gateway status`. If the gateway is running but silent:
- Verify the chat IDs / platform configs match the new machine's network
- If the hostname or IP changed, cron delivery targets may need updating
- Run `hermes gateway restart` to reload config

## Skills loaded but feel stale

Skills are just markdown files — they migrate verbatim. If a skill references a path, binary, or tool name that doesn't exist on the new machine, patch it (the skill's check_fn will skip it anyway, but the misleading content wastes tokens).
