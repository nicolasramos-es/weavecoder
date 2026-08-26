# Overview

Weavecoder is a Rust coding-agent CLI. This spec defines the product's core
capabilities so the development team can implement against a shared contract.

## Capabilities

- [agent-swarm](agent-swarm/spec.md) — native parallel agent orchestration from a single CLI
- [atomic-task-decomposition](atomic-task-decomposition/spec.md) — split large goals into a verified DAG of subtasks
- [local-models-first](local-models-first/spec.md) — oMLX/Ollama/LM Studio/llama.cpp/vLLM/OpenAI-compatible with multiple named providers
- [code-knowledge-graph](code-knowledge-graph/spec.md) — embedded offline project index (tree-sitter + SQLite + embeddings + call graph)
- [permissions](permissions/spec.md) — disk access mode + per-tool allow/ask/deny
- [release-and-install](release-and-install/spec.md) — GitHub-only sourcing, 4-platform binaries, SHA256SUMS verification
