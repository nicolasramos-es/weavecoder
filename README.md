<div align="center">

# Weavecoder

[![License: MIT](https://img.shields.io/badge/license-MIT-blue?style=flat-square)](LICENSE)
[![Platforms](https://img.shields.io/badge/platforms-Linux%20%7C%20macOS%20%7C%20Windows-blue?style=flat-square)](https://github.com/nicolasramos/weavecoder/releases)

CLI de coding agent en **Rust** con Agent Swarm nativo para múltiples peticiones rápidas en paralelo, priorizando modelos locales, y un **Code Knowledge Graph** embebido (tree-sitter) que indexa todo el código del proyecto.

</div>

## Estado actual

```text
$ wvc --version
wvc v0.67.0-dev (1b226b4a8)
```

```text
$ wvc version
version		v0.67.0-dev (1b226b4a8)
semver		0.67.0
git_hash	1b226b4a8
release_build	false
```

## Features

- **Agent Swarm** — múltiples peticiones en paralelo, orquestadas de forma nativa (`wvc-swarm-core`)
- **Modelos locales primero** — Ollama, LM Studio, oMLX/llama.cpp, o cualquier endpoint OpenAI-compatible (ver la lista completa de proveedores en `wvc --help`)
- **Code Knowledge Graph (CKG)** — indexa símbolos y relaciones del código con tree-sitter:
  - `wvc init <proyecto>` — escanea el proyecto, extrae funciones/clases/métodos/variables con tree-sitter (Go, Python, TypeScript/TSX, Rust) y los almacena en SQLite + FTS5
  - `wvc code-search <query>` — búsqueda híbrida sobre el grafo (FTS5 + enriquecimiento por grafo de dependencias, fusionado con Reciprocal Rank Fusion)
  - Indexación incremental: solo re-indexa los archivos modificados (hash SHA-256 + mtime + tamaño)
- **Embeddings locales** (all-MiniLM-L6-v2, 384 dimensiones, inferencia ONNX vía `tract-onnx`) para búsqueda semántica por significado, disponibles a través de la librería (`wvc-code-graph` / `wvc-embedding`)

## Installation

### macOS & Linux

```bash
curl -fsSL https://weavecoder.sh/install | bash
```

### Windows 11 (PowerShell 5.1+)

```powershell
irm https://weavecoder.sh/install.ps1 | iex
```

> ⚠️ El dominio `weavecoder.sh` está pendiente de registro (NRA-508). Hasta entonces, el instalador funciona directamente desde GitHub Releases:
>
> ```bash
> curl -fsSL https://raw.githubusercontent.com/nicolasramos/weavecoder/main/install.sh | bash
> ```

### Desde fuente

```bash
git clone https://github.com/nicolasramos/weavecoder.git
cd weavecoder
cargo build --release --bin wvc
# → target/release/wvc
```

## Quick Start

```bash
# 1. Conecta un modelo local (ej. Ollama)
brew install ollama && ollama pull llama3.2
wvc login --provider ollama

# 2. Conversa con el agente
wvc --provider ollama --model llama3.2 run 'hola'

# 3. Indexa un proyecto y búscalo con el Code Knowledge Graph
wvc init /ruta/al/proyecto --db ckg.db
wvc code-search "parseConfig" --db ckg.db
```

### Ejemplo real (wvc v0.67.0-dev)

Indexado de un proyecto de prueba con 3 archivos (Python, Rust, TypeScript):

```text
$ wvc init /tmp/weavecoder-demo --db /tmp/weavecoder-demo/ckg.db
🔍 Initializing code graph for: /tmp/weavecoder-demo
🔍 Scanning project at: "/tmp/weavecoder-demo"
   Found 3 source files
   Extracted 11 symbols (3 files new/modified, 0 unchanged, 0 deleted)
   Stored 11 symbols in database
   Extracted 2 relations
✅ Code graph initialized:
   Files scanned: 3
   Symbols found: 11
   Relations found: 0
   Time: 4ms
```

Segunda ejecución: la indexación incremental detecta que nada cambió y no re-indexa:

```text
$ wvc init /tmp/weavecoder-demo --db /tmp/weavecoder-demo/ckg.db
🔍 Initializing code graph for: /tmp/weavecoder-demo
🔍 Scanning project at: "/tmp/weavecoder-demo"
   Found 3 source files
   ✅ No changes — index is up to date (3 files unchanged)
✅ Code graph initialized:
   Files scanned: 3
   Symbols found: 11
   Relations found: 0
   Time: 0ms
```

Búsqueda híbrida:

```text
$ wvc code-search "getUser" --db /tmp/weavecoder-demo/ckg.db
🔎 Searching code graph (/tmp/weavecoder-demo/ckg.db) for: getUser
  1. getUser (function) — /tmp/weavecoder-demo/src/app.ts:6 — score=0.067 [fts]
```

## Arquitectura

| Crate | Responsabilidad |
|---|---|
| `wvc-code-graph` | Code Knowledge Graph: tree-sitter (Go/Python/TS/Rust), SQLite+FTS5, embeddings, grafo `petgraph`, búsqueda híbrida (RRF) e indexación incremental |
| `wvc-embedding` | Embeddings locales (all-MiniLM-L6-v2, tract-onnx + tokenizers) |
| `wvc-swarm-core` | Orquestación de enjambres de agentes |
| `wvc-app-core` | Núcleo del agente (tools, sesiones, servidor) |

Documentación de arquitectura del CKG: [docs/architecture/ckg.md](docs/architecture/ckg.md).

## Licencia

MIT — ver [LICENSE](LICENSE).

Este proyecto parte del trabajo excepcional de [Jeremy Huang](https://github.com/1jehuang) (MIT), sobre el que hemos construido, añadido funcionalidades nuevas y mejorado el producto. El aviso de copyright del original se conserva íntegro en LICENSE.
