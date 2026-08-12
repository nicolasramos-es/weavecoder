# Changelog

Todos los cambios notables de **Weavecoder** se documentan en este archivo.

El formato sigue [Keep a Changelog](https://keepachangelog.com/es-ES/1.1.0/) y el versionado sigue [Semantic Versioning](https://semver.org/lang/es/). Este changelog es propio del producto Weavecoder y arranca en la versión actual de desarrollo.

## [v0.67.0-dev] - 2026-08-12

Versión de desarrollo actual del producto. Punto de partida del changelog propio: commit `1b226b4a8` («feat(install): one-line installers»).

### Añadido

- **Code Knowledge Graph (CKG)**:
  - `wvc init <proyecto> [--db <path>]` — escaneo de proyectos, parseo con tree-sitter (Go, Python/`.pyi`, TypeScript/TSX, Rust), extracción de símbolos y persistencia en SQLite + FTS5 (`crates/wvc-code-graph`).
  - `wvc code-search <query> [--db <path>] [--top-k N]` — búsqueda híbrida (FTS5 + enriquecimiento por grafo) fusionada con Reciprocal Rank Fusion (`HybridSearch`, `search.rs`).
  - Indexación incremental: re-indexa solo archivos nuevos/modificados usando snapshots (hash SHA-256, mtime, tamaño) y purga símbolos de archivos eliminados.
  - Esquema SQLite versionado (`SCHEMA_VERSION = 2`) con tablas `symbols`, `relations`, `file_snapshots` y el índice virtual FTS5 `symbols_fts` (external content + triggers).
  - Librería de embeddings `wvc-embedding`: modelo all-MiniLM-L6-v2 (384 dims) en ONNX con inferencia `tract-onnx` y tokenizer HuggingFace; descarga automática bajo demanda desde HuggingFace.
  - Grafo de símbolos en memoria con `petgraph` (`SymbolGraph`): callers, dependencias, dependencias transitivas, alcanzabilidad y detección de ciclos.
- **Agent Swarm** — orquestación nativa de múltiples peticiones en paralelo (`wvc-swarm-core`).
- **Instaladores one-line** — `install.sh` (macOS/Linux) e `install.ps1` (Windows) que descargan el binario desde GitHub Releases.
- **Soporte de proveedores** — Ollama, LM Studio, oMLX/llama.cpp y cualquier endpoint OpenAI-compatible, además de los proveedores en nube (lista completa en `wvc --help`).

### Corregido

- Renombrado del icono de la app a `Weavecoder.icns` en los builds de macOS.
- Purgados artefactos `._*` (AppleDouble) y normalizados los nombres de binario/símbolos a `wvc`/Weavecoder en todo el árbol.

### Notas técnicas

- Binario principal: `wvc` (`src/main.rs`), declarado en `[[bin]]` del workspace.
- Workspace Rust `edition = "2024"` con 85 crates bajo `crates/wvc-*`.
- El CKG se verifica con tests de unidad e integración en `crates/wvc-code-graph` (`src/tests.rs`, `tests/integration_tests.rs`).

[Unreleased]: https://github.com/nicolasramos/weavecoder
[v0.67.0-dev]: https://github.com/nicolasramos/weavecoder/tree/1b226b4a8
