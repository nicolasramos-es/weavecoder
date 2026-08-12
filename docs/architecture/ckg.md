# Arquitectura del Code Knowledge Graph (CKG)

> Documento verificado contra el código real del repo en `main` (commit `1b226b4a8`) y contra la ejecución del binario `wvc v0.67.0-dev (1b226b4a8)`. Los nombres de módulos, funciones, tablas y constantes citados aquí existen en el código; los outputs de ejemplo son ejecuciones reales.

## 1. Resumen

El CKG es un índice de código embebido en el producto Weavecoder: escanea un proyecto, parsea los archivos fuente con **tree-sitter**, extrae **símbolos** (funciones, clases, métodos, variables…) y **relaciones** (llamadas, herencia…), y los persiste en una base **SQLite** embebida con índice **FTS5**. Encima de ese almacén se construye, en memoria, un **grafo de símbolos** con `petgraph`, y una **búsqueda híbrida** que fusiona tres señales (FTS5, semántica por embeddings, grafo) con Reciprocal Rank Fusion.

Componentes:

| Componente | Crate / módulo | Responsabilidad |
|---|---|---|
| CLI | `src/cli/init.rs`, `src/cli/code_search.rs` | Comandos `wvc init` y `wvc code-search` |
| Núcleo del CKG | `crates/wvc-code-graph` | Parsing, extracción, almacenamiento, grafo, búsqueda |
| Embeddings | `crates/wvc-embedding` | Modelo all-MiniLM-L6-v2 (ONNX), tokenizer, similitud coseno |

Dependencias clave (`crates/wvc-code-graph/Cargo.toml`): `tree-sitter 0.25`, `tree-sitter-go 0.23`, `tree-sitter-python 0.23`, `tree-sitter-typescript 0.23`, `tree-sitter-rust 0.23`, `rusqlite 0.32` (feature `bundled`), `petgraph 0.8`, `sha2 0.10`, `wvc-embedding` (path).

```
┌──────────────┐   ┌──────────────┐   ┌──────────────┐   ┌──────────────┐
│  scan_project │ → │  parse_file   │ → │ extract_      │ → │  CodeGraph    │
│  (init.rs)    │   │  (parser.rs)  │   │ symbols /     │   │  (storage.rs) │
│  .gitignore-  │   │  tree-sitter  │   │ relations     │   │  SQLite+FTS5  │
│  aware        │   │  (language.rs)│   │  (init.rs)    │   │  WAL, v2      │
└──────────────┘   └──────────────┘   └──────────────┘   └──────┬───────┘
                                                                │
        ┌───────────────────────────────┬───────────────────────┤
        ▼                               ▼                       ▼
┌──────────────┐               ┌──────────────┐        ┌──────────────┐
│ SymbolGraph  │               │  Embedding    │        │ HybridSearch │
│ (graph.rs)   │               │ (embedding.rs │        │  (search.rs) │
│ petgraph     │               │  + wvc-embed- │        │  FTS5 + sem +│
│ DiGraph      │               │  ding)        │        │  grafo (RRF) │
└──────────────┘               └──────────────┘        └──────────────┘
```

## 2. Lenguajes y gramáticas tree-sitter

Módulo `crates/wvc-code-graph/src/language.rs`:

- Enum `Language { Go, Python, Typescript, Rust }`.
- `detect_language(ext)` mapea extensiones a gramáticas tree-sitter:

| Extensión | Gramática | Notas |
|---|---|---|
| `.go` | `tree_sitter_go::LANGUAGE` | |
| `.py`, `.pyi` | `tree_sitter_python::LANGUAGE` | |
| `.ts`, `.tsx` | `tree_sitter_typescript::LANGUAGE_TSX` | El crate expone `LANGUAGE_TSX`; TSX se usa también para `.ts` |
| `.rs` | `tree_sitter_rust::LANGUAGE` | |

- Extensiones soportadas en el escáner (`init.rs`, `SUPPORTED_EXTENSIONS`): `["go", "py", "pyi", "ts", "tsx", "rs"]`.

Parsing (`parser.rs`):

- `parse_str(source: &str, lang: tree_sitter::Language) -> Result<Tree, String>` — crea un `Parser`, fija el lenguaje y parsea.
- `parse_file(path: &Path) -> Result<Tree, String>` — detecta la extensión, lee el archivo y parsea.
- Helper de la librería: `parse_source(source, ext)` (en `lib.rs`) y `detect_ext(path)`.

## 3. Extracción de símbolos y relaciones

Módulo `crates/wvc-code-graph/src/init.rs` — `extract_symbols(tree, source, file_path)` recorre el AST con tree-sitter y produce `SymbolInsert` (estructuras en `symbols.rs`):

- `SymbolKind` (enum con Display en minúsculas): `Function, Class, Module, Variable, Constant, Method, Property, Interface, Enum, TypeAlias, Macro, Package, File, Directory, Other(String)`.
- `SymbolInsert { name, kind, file_path, line, col, language, doc, embedding }` — el `embedding` se inserta como `None` durante `wvc init` (ver §6).

Relaciones (`relations.rs`):

- `RelationKind`: `Calls, Inherits, Implements, DependsOn, Contains, References, Defines, Uses, Other(String)`.
- `extract_relations(tree, source, symbol_names)` devuelve pares `(caller_name, callee_name)` para patrones del AST:
  - Nodos `call_expression` / `field_expression` → llamadas `foo()` / `self.foo()` (el callee debe estar en `symbol_names`).
  - Nodos `super_class` / `implements_clause` / `implies` → herencia/implementación.
- La resolución de IDs a la hora de insertar usa claves `"{file_path}::{name}"` (`name_map` en `run_init`). Limitación observada en v0.67.0-dev: los pares extraídos usan el prefijo `self::` y no resuelven contra esas claves, por lo que el flujo CLI reporta «Extracted N relations» pero almacena 0 (ver §9). La maquinaria de relaciones (enum, `SymbolGraph::add_relation`, traversal) está implementada y cubierta por tests a nivel de librería.

## 4. Almacenamiento: SQLite + FTS5

Módulo `crates/wvc-code-graph/src/storage.rs`. `CodeGraph` envuelve una conexión `rusqlite` (feature `bundled` → SQLite compilado en el binario, sin dependencias de sistema).

Apertura:

- `CodeGraph::open(path)` / `CodeGraph::open_memory()`.
- `PRAGMA journal_mode=WAL;` y `PRAGMA foreign_keys=ON;` en ambas.
- Versionado con `PRAGMA user_version` contra la constante `SCHEMA_VERSION: u32 = 2` (exportada en `lib.rs`); si la base es más nueva que el binario, aborta con error.

Esquema (`create_schema`):

```sql
CREATE TABLE IF NOT EXISTS symbols (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    name          TEXT NOT NULL,
    kind          TEXT NOT NULL,
    file_path     TEXT NOT NULL,
    line          INTEGER NOT NULL,
    col           INTEGER NOT NULL DEFAULT 0,
    language      TEXT,
    doc           TEXT,
    embedding     BLOB
);

CREATE TABLE IF NOT EXISTS relations (
    source_symbol_id INTEGER NOT NULL REFERENCES symbols(id) ON DELETE CASCADE,
    target_symbol_id INTEGER NOT NULL REFERENCES symbols(id) ON DELETE CASCADE,
    kind             TEXT NOT NULL,
    metadata         TEXT,
    PRIMARY KEY (source_symbol_id, target_symbol_id, kind)
);

CREATE TABLE IF NOT EXISTS file_snapshots (
    file_path TEXT PRIMARY KEY,
    hash      TEXT NOT NULL,
    mtime     INTEGER NOT NULL,
    size      INTEGER NOT NULL
);

CREATE VIRTUAL TABLE IF NOT EXISTS symbols_fts USING fts5(
    name, doc,
    content='symbols',
    content_rowid='id'
);
```

- **FTS5 external content**: `symbols_fts` no duplica las columnas; se mantiene sincronizada con triggers sobre `symbols` (`symbols_ai` AFTER INSERT, `symbols_ad` AFTER DELETE, `symbols_au` AFTER UPDATE). Búsqueda con ranking BM25.
- Índices: `idx_symbols_file_path` (symbols.file_path), `idx_symbols_kind` (symbols.kind), `idx_relations_source` (relations.source_symbol_id), `idx_relations_target` (relations.target_symbol_id).
- API pública de `CodeGraph`: `insert_symbol`, `upsert_symbol`, `get_symbol`, `list_symbols(SymbolQuery)`, `insert_relation`, `get_relations`, `list_relations`, `search_fts(FtsQuery)`, `schema_version`, `symbol_count`, `relation_count`, `batch_insert_symbols`, `update_symbol_embedding`, `get_snapshot`/`upsert_snapshot`/`delete_snapshot`/`list_snapshots` (incremental), `delete_symbols_for_file`, `symbol_count_for_file`, `close` (+ `Drop`).

FTS5 (`fts.rs`):

- `FtsQuery { query, limit }` — la query se pasa tal cual a FTS5; sintaxis soportada: término simple (`fn_name`), prefijo (`fn_name*`), frase (`"my function"`), `OR` (`func OR method`), filtro de columna (`name:fn_name`).
- `FtsSearchResult { …, rank: f64 }` — `rank` es la puntuación BM25.

## 5. Indexación incremental

Implementada en `run_init(config: InitConfig) -> Result<InitSummary>` (`init.rs`), la pieza «T7».

- `InitConfig { root, db_path, extra_extensions }` — `db_path: None` ⇒ base en memoria.
- `InitSummary { files_scanned, symbols_found, relations_found, elapsed_ms }`.

Flujo:

1. **Scan**: `scan_project(root)` recorre el árbol recursivamente (`walk_dir`) y salta directorios generados/de terceros: `node_modules`, `target`, `.venv`, `vendor`, `.git`, `__pycache__`, `.tox`, `.mypy_cache`, `.pytest_cache`, `dist`, `build`. Solo admite las extensiones de §2.
2. **Clasificación incremental** contra `file_snapshots` (tabla con `hash`, `mtime`, `size`):
   - Nuevo (sin snapshot) → indexar completo.
   - Modificado (`compute_file_hash` SHA-256 difiere) → `delete_symbols_for_file` + re-indexar.
   - Sin cambios (hash igual) → *skip* (contador `unchanged_count`).
   - Eliminado (en snapshot pero no en disco) → purgar símbolos y snapshot.
   - Si no hay nada nuevo/modificado: `✅ No changes — index is up to date (N files unchanged)` y retorna con los contadores actuales.
3. **Parse + extracción** de símbolos de los archivos nuevos/modificados.
4. **Batch insert** (`batch_insert_symbols`).
5. **Extracción de relaciones** (re-parse) e inserción con resolución por `(file_path, name)`.

## 6. Embeddings: all-MiniLM-L6-v2

Dos capas:

### `crates/wvc-embedding` (inferencia)

- Constante pública `MODEL_NAME: &str = "all-MiniLM-L6-v2"`; dimensión `EMBEDDING_DIM = 384`; longitud de secuencia máxima `MAX_SEQ_LENGTH`.
- Modelo ONNX + tokenizer descargados de HuggingFace la primera vez (`MODEL_URL`):
  - `https://huggingface.co/sentence-transformers/all-MiniLM-L6-v2/resolve/main/onnx/model.onnx`
  - `https://huggingface.co/sentence-transformers/all-MiniLM-L6-v2/resolve/main/tokenizer.json`
- `Embedder::load_from_dir(model_dir)` — si faltan `model.onnx`/`tokenizer.json`, llama a `download_model` (descarga bloqueante en thread con `reqwest`, timeout de 300 s, User-Agent `wvc-embedding/<version>`). Luego carga el ONNX con `tract_onnx`, optimiza y hace *runnable*.
- `embed(text)`, `embed_batch(texts)` — tokeniza con `tokenizers::Tokenizer` (HuggingFace), construye los tensores según el orden/rol/dtype declarados por el modelo y ejecuta la inferencia; `CrossEncoder` comparte ese pipeline.
- `cosine_similarity(a, b)` — usada por el CKG para el signal semántico.
- `is_model_available(model_dir)` — comprueba que ambos archivos existen.
- Nota: `main.rs` del binario ajusta jemalloc para cargas/descargas del modelo ONNX (~87 MB en RAM), lo que confirma que el modelo se usa en procesos del producto.

### `crates/wvc-code-graph/src/embedding.rs` (integración)

- `EMBEDDING_DIM: usize = 384`; `SERIALIZED_EMBEDDING_SIZE = EMBEDDING_DIM * 4` (1536 bytes); `serialize_embedding` / `deserialize_embedding` (f32 little-endian → BLOB).
- `symbol_text(symbol)` — representación textual de un símbolo para embedir.
- `SymbolEmbedder { … }` — `new(model_dir)`, `embed_symbol`, `embed_batch`, `update_symbol_embedding`, `embed_all_missing` (embedir símbolos sin vector persistido).
- `EmbeddingSearch` — búsqueda por similitud coseno sobre los embeddings persistidos en SQLite (`EmbeddingSearchResult { symbol, score }`, score 0.0–1.0 con vectores normalizados).

**Estado real en v0.67.0-dev**: `wvc init` (CLI) **no** genera embeddings: `extract_symbols` los inserta como `None` y `run_init` no invoca `SymbolEmbedder`. El modelo se descarga/usa bajo demanda a través de la API de las librerías `wvc-embedding` / `wvc-code-graph` (y en los tests). El primer uso requiere red (descarga de HuggingFace); sin red, `Embedder::load_from_dir` falla al no poder descargar — esto es una limitación observada, no inventada.

## 7. Grafo de símbolos: petgraph

Módulo `crates/wvc-code-graph/src/graph.rs` — `SymbolGraph`:

- `DiGraph<Symbol, RelationKind>` de `petgraph 0.8`; `SymbolId(pub usize)`.
- Deduplicación con `symbol_map: HashMap<(file_path, name), SymbolId>` y `id_map: HashMap<i64, SymbolId>` (id de storage → nodo).
- API: `add_symbol`, `add_edge`, `node_storage_id`, `resolve(storage_id)`, `get_symbol`, `symbol_count`, `edge_count`, `callers_of` (aristas entrantes), `dependencies_of` (aristas salientes), `transitive_dependencies` (BFS por aristas salientes), `is_reachable(from, to)`, `detect_cycles` (DFS con pila de recursión), `build_from_storage(&CodeGraph)` y `add_relation(&Relation)`.
- Se construye desde la base con `build_from_storage` cada vez que se abre `HybridSearch`.

## 8. Búsqueda híbrida

Módulo `crates/wvc-code-graph/src/search.rs` — `HybridSearch { storage: CodeGraph, graph: SymbolGraph }`:

Tres señales:

1. **FTS5** (`fts_signal`) — consulta BM25 con límite 50; convierte el rank a score `1.0 / (1.0 + rank)` (menor rank ⇒ mayor score).
2. **Semántica** (`semantic_signal`) — similitud coseno del embedding de la query contra los embeddings persistidos de cada símbolo; solo se usa **si el llamante proporciona `query_embedding`**; top-k recortado.
3. **Grafo** (`graph_signal`) — sobre los hits de FTS/semántica, añade vecinos (callers y dependencias) con score degradado, máx. 5 vecinos por hit.

Fusión: **Reciprocal Rank Fusion con `k = 60`** (`rrf_fuse(ranked_lists, 60.0)`), más un pequeño blend del score crudo (`rrf + raw * 0.05`). Los resultados llevan `SearchSignal::Fts { score }`, `SearchSignal::Semantic { score }` o `SearchSignal::Graph { score, neighbor_name }`.

API pública:

```rust
pub fn search(&self, query: &str, query_embedding: Option<&[f32]>, top_k: usize)
              -> anyhow::Result<Vec<SearchResult>>
```

### CLI

- `src/cli/init.rs` — `run_init(project_path, db_path)` construye `InitConfig` y llama a `wvc_code_graph::run_init`; imprime el resumen.
- `src/cli/code_search.rs` — `run_code_search(query, db_path, top_k)`:
  - DB por defecto `code-graph.db`; si no existe: `Error: code graph database not found at {path}. Run wvc init <project> --db {path} first.`
  - `HybridSearch::open` + `engine.search(query, None, top_k)` — **en v0.67.0-dev el CLI pasa `None` como embedding**, por lo que la búsqueda efectiva desde CLI es **FTS5 + grafo**; el signal semántico se activa cuando el llamante genera el embedding de la query.
  - Imprime `N. nombre (kind) — ruta:línea — score=X.XXX [señales]`.

## 9. Verificación observada (wvc v0.67.0-dev, commit 1b226b4a8)

Proyecto de prueba con 3 archivos (Python `main.py`, Rust `lib.rs`, TypeScript `app.ts`):

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

Segunda ejecución (incremental — no hay cambios):

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

Búsqueda (señal FTS5; `--top-k` por defecto 10):

```text
$ wvc code-search "getUser" --db /tmp/weavecoder-demo/ckg.db
🔎 Searching code graph (/tmp/weavecoder-demo/ckg.db) for: getUser
  1. getUser (function) — /tmp/weavecoder-demo/src/app.ts:6 — score=0.067 [fts]
```

```text
$ wvc code-search "parse" --db /tmp/weavecoder-demo/ckg.db --top-k 5
🔎 Searching code graph (/tmp/weavecoder-demo/ckg.db) for: parse
  1. Parser (struct) — /tmp/weavecoder-demo/src/lib.rs:1 — score=0.067 [fts]
  2. parse (function) — /tmp/weavecoder-demo/src/lib.rs:10 — score=0.066 [fts]
  3. parse_tokens (function) — /tmp/weavecoder-demo/src/lib.rs:15 — score=0.066 [fts]
```

## 10. Limitaciones observadas (v0.67.0-dev)

- **Relaciones**: los pares extraídos se reportan («Extracted 2 relations») pero la resolución de IDs con el prefijo `self::` no encuentra coincidencias en el `name_map`, así que el CLI almacena 0 relaciones en este build. La extracción (`extract_relations`), los tipos (`RelationKind`) y el grafo (`SymbolGraph::add_relation`, traversals) están implementados y probados a nivel de librería.
- **Semántica**: el signal semántico requiere `query_embedding`; el CLI `code-search` lo pasa `None`, de modo que la búsqueda efectiva desde CLI es FTS5 + enriquecimiento de grafo. La ruta semántica completa (embedir símbolos → persistir BLOB → coseno) está implementada en `SymbolEmbedder`/`EmbeddingSearch` y requiere el modelo ONNX.
- **Modelo**: la primera carga de `wvc-embedding::Embedder` descarga ~90 MB (modelo + tokenizer) desde HuggingFace; sin conectividad falla. El CLI `wvc init` no dispara esta descarga.
- **Cobertura de lenguajes**: solo Go, Python, TypeScript/TSX y Rust (extensiones `go, py, pyi, ts, tsx, rs`); el resto se ignora silenciosamente en el escaneo.

## 11. Tests

- Unitarios: `crates/wvc-code-graph/src/tests.rs` (incluye tests de coseno en `embedding.rs` y de serialización de embeddings).
- Integración: `crates/wvc-code-graph/tests/integration_tests.rs` (schema version, existencia de tablas, inserciones FTS, snapshots).
- Ejecución: `cargo test -p wvc-code-graph`.
