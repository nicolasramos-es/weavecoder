<div align="center">

# Weavecoder

[![License: MIT](https://img.shields.io/badge/license-MIT-blue?style=flat-square)](LICENSE)
[![Platforms](https://img.shields.io/badge/platforms-Linux%20%7C%20macOS%20%7C%20Windows-blue?style=flat-square)](https://github.com/nicolasramos-es/weavecoder/releases)

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
> curl -fsSL https://raw.githubusercontent.com/nicolasramos-es/weavecoder/main/install.sh | bash
> ```

Need Homebrew, source builds, provider setup, or want an agent to set it up for you? Sigue leyendo — [Quick Start](#quick-start) y [Desde fuente](#desde-fuente).

### Desde fuente

```bash
git clone https://github.com/nicolasramos-es/weavecoder.git
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

Este proyecto parte del trabajo excepcional de [Jeremy Huang](https://github.com/nicolasramos/weavecoder) (wvc, MIT), sobre el que hemos construido, añadido funcionalidades nuevas y mejorado el producto. El aviso de copyright del original se conserva íntegro en LICENSE.
