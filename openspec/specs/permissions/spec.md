# Permissions

## Purpose

Give the user fine-grained control over which tools the agent may run and what
the agent may touch on disk.

## Requirements

- `[tools] disk_mode`: `full` / `limited` / `ask`.
- `[tools] permissions`: per-tool override `allow` / `ask` / `deny`.
- `/permissions` TUI command lists the current settings, sets disk mode, and
  sets a per-tool permission.
- The gate applies at tool execution: `deny` blocks the tool, `ask` queues an
  approval request in the shared SafetySystem (TUI can approve/deny), `allow`
  (and unlisted tools) execute normally.
- Applies to normal chat sessions and ambient flows alike.
