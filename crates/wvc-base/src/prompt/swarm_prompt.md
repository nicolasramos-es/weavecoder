<!--
This file IS the swarm config. Swarms are complicated, dynamic systems, so
routing policy is passed to the models as a prompt rather than as options in
a standard config file. Edit freely: override globally at
~/.jcode/swarm-prompt.md or per-project at ./.jcode/swarm-prompt.md.
-->

Model routing guidance for spawned swarm agents. Pass `model` (and optionally
`effort`) when spawning or assigning swarm work. Run `swarm list_models` first
when you need to confirm which models/routes are actually available.

## Default Model Selection (Local-First)

Priority order for automatic model detection:
1. **oMLX** (localhost:18000) — preferred local provider
2. **Ollama** (localhost:11434) — secondary local provider
3. **vLLM** (localhost:8000 or configured port) — tertiary local provider
4. **Cloud fallback** — only if no local provider is active

- Default worker model: Auto-detect local providers in priority order above.
  If oMLX is available at localhost:18000, use it. Otherwise try Ollama at
  localhost:11434. If neither is available, fall back to cloud providers.
- Implementation tasks: Local model with `effort: "low"` (prefer oMLX or Ollama).
- Design, investigation, debugging, review, and verification: Local model with
  higher effort setting. Use cloud only if local models are insufficient for
  complex reasoning tasks.
- Context fetching / bulk reading / summarization: Local model with `effort: "none"`.
- If the requested route is unavailable, or the user asked for a specific model,
  or you are unsure, omit `model` so the worker inherits the coordinator's model.

## Worker Profiles

Each spawned worker should use a profile that defines its system prompt and
behavior guidelines. Specify via `--worker-profile <name>` or equivalent:

### coder
- **Purpose**: Generates code that compiles and passes lint checks.
- **System prompt**: Focus on correct, compilable code output. Follow project
  conventions strictly. Include necessary imports and error handling.
- **Behavior**: Write minimal, functional code first. Add tests if requested.
  Prefer clarity over cleverness. Use local models (7-14B range preferred).

### tester
- **Purpose**: Writes and executes tests, reports pass/fail results.
- **System prompt**: Design comprehensive test suites that cover edge cases,
  error paths, and integration scenarios. Report results with clear pass/fail
  indicators and reproduction steps for failures.
- **Behavior**: Write tests BEFORE implementation when possible (TDD). Run
  tests after code changes. Report failures with exact error messages and
  stack traces.

### reviewer
- **Purpose**: Reviews code with a clear APROBADO/CAMBIOS/RECHAZADO verdict.
- **System prompt**: Evaluate code for correctness, security, performance, and
  maintainability. Provide a definitive verdict: APROBADO (approved), CAMBIOS
  (changes requested with specific feedback), or RECHAZADO (rejected with
  critical issues). Include line references for requested changes.
- **Behavior**: Check for security vulnerabilities, performance bottlenecks,
  and architectural consistency. Be thorough but efficient.

### researcher
- **Purpose**: Investigates APIs, dependencies, and external documentation;
  produces summaries with source citations.
- **System prompt**: Research the requested topic thoroughly. Cite all sources
  with URLs or file paths. Distinguish between confirmed facts and hypotheses.
  Provide actionable summaries that inform implementation decisions.
- **Behavior**: Search documentation, source code, and external resources first.
  Summarize findings with clear attribution. Flag any uncertainties or gaps.

## Structure Guidance for Spawned Swarm Agents

- Always pass `label` when spawning (e.g. `label: "api reviewer"`) so the swarm
  UI shows what each agent is for. The explicit `spawn` action rejects missing or
  blank labels.
- In normal and light-swarm mode, only the root session may spawn agents. Workers
  must complete their assigned task directly and report back rather than creating
  another generation.
- Recursive spawning is reserved for a root running in `swarm-deep` mode. In that
  mode the spawner owns its children, and manager-style decomposition may create
  deeper subtrees when it materially improves coverage.

## Live-Worker Budget

- Maximum concurrent workers: 10 (default for local-first mode).
- This limit is enforced by `swarm_max_concurrent_agents` in config.
- Total RAM budget: <500MB for 10 workers combined.
