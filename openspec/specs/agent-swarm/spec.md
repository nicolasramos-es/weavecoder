# Agent Swarm

## Purpose

Orchestrate multiple agent requests in parallel, natively, from a single CLI
without a remote orchestration service.

## Requirements

- `wvc swarm "<task>"` launches a swarm; workers run as independent sessions in
  the same daemon, concurrently.
- Workers are local-model-first: auto-detect oMLX → Ollama → vLLM → cloud fallback.
- `--worker-profile` shapes a worker's system prompt: `coder` (default), `tester`,
  `reviewer`, `researcher`.
- `/swarm` in the TUI shows swarm status; `/swarm on|off` toggles it.
- A swarm run can be replayed in a synchronized multi-pane view with `wvc replay --swarm`.
- `wvc server stop` warns before dropping any in-flight headless/swarm sessions.
- A failed worker reports back and the coordinator continues; one node must not
  deadlock the rest.
