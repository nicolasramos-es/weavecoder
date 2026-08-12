<div align="center">

# Weavecoder

[![License: MIT](https://img.shields.io/badge/license-MIT-blue?style=flat-square)](LICENSE)
[![Platforms](https://img.shields.io/badge/platforms-Linux%20%7C%20macOS%20%7C%20Windows-blue?style=flat-square)](https://github.com/nicolasramos/weavecoder/releases)

CLI de coding agent en **Rust** con Agent Swarm nativo para múltiples peticiones rápidas en paralelo, priorizando modelos locales, y un **Code Knowledge Graph** embebido (tree-sitter) que indexa todo el código del proyecto.

</div>

## Features

- **Agent Swarm** — múltiples peticiones en paralelo, orquestadas de forma nativa
- **Modelos locales primero** — Ollama, LM Studio, oMLX/llama.cpp, o cualquier endpoint OpenAI-compatible
- **Code Knowledge Graph (CKG)** — indexa símbolos y relaciones del código con tree-sitter:
  - `wvc init` — escanea el proyecto, extrae funciones/clases/imports/calls y los almacena en SQLite + FTS5
  - `wvc code-search <query>` — búsqueda híbrida (FTS5 + embeddings + grafo de dependencias)
  - Graph traversal: "quién llama a X", "de qué depende Y"
  - Indexación incremental: solo re-indexa los archivos modificados
- **Embeddings locales** (all-MiniLM-L6-v2) para búsqueda semántica por significado

## Instalación

### macOS (arm64)

```bash
# Descarga el binario desde GitHub Releases
curl -fsSL https://github.com/nicolasramos/weavecoder/releases/latest/download/wvc-macos-arm64 -o ~/bin/wvc
chmod +x ~/bin/wvc
xattr -d com.apple.quarantine ~/bin/wvc   # primer uso

wvc --version
```

> Instalador de una línea (`curl | bash`) en desarrollo — [NRA-510].

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

## Arquitectura

| Crate | Responsabilidad |
|---|---|
| `wvc-code-graph` | Code Knowledge Graph: tree-sitter (Go/Py/TS/Rust), SQLite+FTS5, embeddings, grafo petgraph, búsqueda híbrida |
| `wvc-embedding` | Embeddings locales (all-MiniLM-L6-v2, tract-onnx) |
| `wvc-swarm-core` | Orquestación de enjambres de agentes |
| `wvc-app-core` | Núcleo del agente (tools, sesiones, servidor) |

## Licencia

MIT — ver [LICENSE](LICENSE).

Este proyecto parte del trabajo excepcional de [Jeremy Huang](https://github.com/1jehuang) (wvc, MIT), sobre el que hemos construido, añadido funcionalidades nuevas y mejorado el producto. El aviso de copyright del original se conserva íntegro en LICENSE.
