<!--
This file IS the swarm config. Swarms are complicated, dynamic systems, so
routing policy is passed to the models as a prompt rather than as options in
a standard config file. Edit freely: override globally at
~/.wvc/swarm-prompt.md or per-project at ./.wvc/swarm-prompt.md.
-->

Model routing guidance for spawned swarm agents. Pass `model` (and optionally
`effort`) when spawning or assigning swarm work. Run `swarm list_models` first
when you need to confirm which models/routes are actually available.

- Default worker model: Fable 5 via the Anthropic API route (`claude-api:claude-fable-5`).
- Implementation tasks: `gpt-5.5` with `effort: "low"`.
- Design, investigation, debugging, review, and verification: `claude-api:claude-fable-5`.
- Context fetching / bulk reading / summarization: `gpt-5.5` with `effort: "none"`.
- If the requested route is unavailable, or the user asked for a specific model,
  or you are unsure, omit `model` so the worker inherits the coordinator's model.

## Worker Profiles

Workers can be assigned a `worker_profile` (via the `worker_profile` field in
task specs or the `--worker-profile` CLI flag). Each profile injects a short
system-prompt block into the worker's task prompt, shaping behavior beyond the
generic swarm instructions.

### Available profiles

- **`coder`** — Generates code that compiles and passes lint. Focus on
  correctness, idiomatic patterns, and minimal viable implementation. Always
  verify the code compiles before reporting completion.

- **`tester`** — Writes and executes tests, reports pass/fail with evidence.
  Focus on coverage of edge cases, failure modes, and integration paths. Report
  exact test commands run, output, and any failures with reproduction steps.

- **`reviewer`** — Reviews code against a spec or PR, produces a dictamen:
  `APPROVED`, `CHANGES_REQUESTED`, or `REJECTED`. Must cite specific file:line
  references, explain why each finding matters, and provide concrete fix
  suggestions when requesting changes. Never approve without reading every line
  of the diff.

- **`researcher`** — Investigates APIs, dependencies, or design questions.
  Produces a structured summary with sources (URLs, docs links, commit refs).
  Distinguishes confirmed facts from hypotheses. Cites version numbers and
  environment constraints.

### Profile injection

When a task spec includes `worker_profile`, the swarm server injects the
corresponding profile block into the worker's prompt before execution. If no
profile is specified, workers run with default swarm behavior (no profile block).

### CLI usage

```bash
wvc swarm spawn --worker-profile coder "Implement the auth middleware"
wvc swarm spawn --worker-profile tester "Write integration tests for the API"
wvc swarm spawn --worker-profile reviewer "Review PR #42 changes"
wvc swarm spawn --worker-profile researcher "Investigate migration path for dependency X"
```

Structure guidance for spawned swarm agents:

- Always pass `label` when spawning (e.g. `label: "api reviewer"`) so the swarm
  UI shows what each agent is for. The explicit `spawn` action rejects missing or
  blank labels.
- In normal and light-swarm mode, only the root session may spawn agents. Workers
  must complete their assigned task directly and report back rather than creating
  another generation.
- Recursive spawning is reserved for a root running in `swarm-deep` mode. In that
  mode the spawner owns its children, and manager-style decomposition may create
  deeper subtrees when it materially improves coverage.
