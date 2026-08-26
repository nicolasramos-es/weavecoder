# Code Knowledge Graph (CKG)

## Purpose

Index the whole project source code on the user's machine so the agent can
answer structural questions instantly and offline.

## Requirements

- `wvc init <path>` scans a project directory (defaults to the **current
  directory** when `<path>` is omitted) and builds the graph.
- Parses source with tree-sitter: Go, Python, TypeScript, Rust.
- Extracts symbols (functions/classes/imports/calls) and call relations, stores
  them in SQLite + FTS5.
- `wvc code-search <query>` runs hybrid search (FTS5 + embeddings + dependency
  graph); results cached in-memory (LRU max 50) and invalidated on `wvc init` or
  working-directory change.
- Graph traversal: "who calls X", "what does Y depend on".
- Incremental indexing: only re-indexes modified files.
- `--db <path>` persists the graph; without it, graph is in-memory.
