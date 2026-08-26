# Atomic Task Decomposition

## Purpose

Split a large goal into a DAG of small, verified subtasks so local models with
small context windows can execute multi-step work reliably.

## Requirements

- A large task node can be **expanded** (`expand_node`) into a sub-DAG of child
  subtasks; the parent becomes a composite join/synthesis point.
- Each composite node auto-inserts a **critique/verify gate** — the composite
  cannot close until its synthesis survives an adversarial audit.
- Child subtasks only start once their dependencies are satisfied; cycles are
  rejected.
- The synthesis node receives the children's real artifacts (not just a "passed"
  token) so it combines actual results.
- Worker context is kept under ~4000 tokens: subtask + one-line summaries of
  completed dependencies.
- The plan is durable: `resume`/`retry` pick up from the last persisted state.
- `/plan [goal]` is plan-only: it produces a plan card and waits for user
  approval before any implementation.
