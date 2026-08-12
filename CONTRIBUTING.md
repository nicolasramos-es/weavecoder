# Contribuyendo a Weavecoder

Gracias por querer contribuir a Weavecoder. Este documento resume el flujo de trabajo, los estándares de calidad y la estructura del repositorio.

## Índice

- [Entorno de desarrollo](#entorno-de-desarrollo)
- [Build](#build)
- [Tests](#tests)
- [Calidad (CI)](#calidad-ci)
- [Commits convencionales](#commits-convencionales)
- [Pull requests](#pull-requests)
- [Estructura del repositorio](#estructura-del-repositorio)
- [Verificación en runtime](#verificación-en-runtime)

## Entorno de desarrollo

- **Rust**: workspace con `edition = "2024"` — usa una toolchain estable reciente (1.83+). El CI usa la toolchain estable de GitHub Actions.
- **Sistema**: Linux, macOS o Windows. El binario principal `wvc` se compila en los tres (ver `.github/workflows/ci.yml` y `windows-smoke.yml`).
- No se necesitan dependencias de sistema para el CKG: `rusqlite` se compila con la feature `bundled` y tree-sitter se compila desde el fuente.

## Build

El binario del producto es `wvc` (`src/main.rs`, declarado en `[[bin]]` del `Cargo.toml` raíz):

```bash
cargo build --release --bin wvc
# → target/release/wvc
```

Para compilar todos los bins (incluidos los de desarrollo, que requieren la feature `dev-bins`):

```bash
cargo build --release --all-targets
```

Verifica la instalación:

```bash
./target/release/wvc --version
```

## Tests

```bash
# Todo el workspace
cargo test

# Crate del Code Knowledge Graph (unidad + integración)
cargo test -p wvc-code-graph

# Tests de integración específicos del CKG
cargo test -p wvc-code-graph --test integration_tests

# Con todas las features
cargo test --all-features
```

Los tests del CKG viven en `crates/wvc-code-graph/src/tests.rs` (unitarios) y `crates/wvc-code-graph/tests/integration_tests.rs` (integración contra SQLite real en memoria).

## Calidad (CI)

El workflow `.github/workflows/ci.yml` ejecuta «Quality Guardrails» que **deben pasar** antes de mergear. Puedes reproducirlos localmente:

```bash
cargo fmt --all -- --check          # formato
cargo clippy --all-targets --all-features -- -D warnings   # clippy sin warnings
cargo test --all-features           # tests con todas las features
```

El CI además exige, entre otros: `Cargo.lock` actualizado, presupuesto de warnings, ratchets de ficheros/tests grandes, uso de pánico/errores tragados, fronteras de dependencias entre crates, paridad de superficie SDK Rust/TS y ausencia de dependencias sin usar. Si tu cambio rompe alguno de estos checks, arréglalo antes de pedir review.

## Commits convencionales

Usamos [Conventional Commits](https://www.conventionalcommits.org/):

```
<type>(<scope>): <descripción>
```

Tipos usados en este repo:

| Tipo | Uso |
|---|---|
| `feat` | Nueva funcionalidad |
| `fix` | Corrección de bugs |
| `docs` | Solo documentación (README, docs/, comentarios) |
| `refactor` | Cambio de estructura sin cambiar comportamiento |
| `test` | Añadir/modificar tests |
| `chore` | Tareas de mantenimiento (deps, limpieza, CI) |
| `build` | Cambios de build/empaquetado/instaladores |
| `perf` | Optimizaciones de rendimiento |

Reglas:

- Descripción en imperativo y sin punto final: `docs: añadir guía de contribución`, `feat(install): one-line installers`.
- Un commit = una unidad lógica atómica y revisable.
- **No** añadas `Co-Authored-By` ni atribuciones de herramientas de IA en el mensaje.
- El cuerpo (si existe) explica el *qué* y el *por qué*, no el *cómo*.

## Pull requests

- Cada PR debe enlazar un **issue real** del repositorio (el workflow `require-issue.yml` lo valida contra la API de GitHub; usa `Closes #123`, `#123` o la URL del issue).
- Trabaja en tu propia rama, partiendo de `main`. No tomes, cherry-pickees ni copies código de ramas de otras personas u otros agentes (ver `AGENTS.md`): si necesitas algo que vive en otra rama, coméntalo y que decida el mantenedor.
- Mantén el diff enfocado. Si el PR toca lógica además de documentación, sepáralo en commits por tipo.

## Estructura del repositorio

```
weavecoder/
├── Cargo.toml            # workspace + package raíz (bin wvc → src/main.rs)
├── src/
│   ├── main.rs           # binario wvc
│   ├── lib.rs            # librería raíz
│   └── cli/              # comandos CLI (init.rs, code_search.rs, login.rs, …)
│   └── bin/              # bins auxiliares (harness, benchs, test_api, …)
├── crates/               # workspace: wvc-* (85 crates)
│   ├── wvc-code-graph/   # Code Knowledge Graph (tree-sitter, SQLite+FTS5, petgraph, búsqueda híbrida)
│   ├── wvc-embedding/    # embeddings all-MiniLM-L6-v2 (tract-onnx + tokenizers)
│   ├── wvc-swarm-core/   # orquestación de enjambres
│   ├── wvc-app-core/     # núcleo del agente
│   └── …
├── tests/                # tests de integración de alto nivel (Python/Rust)
├── scripts/              # scripts de build/install/release
├── install.sh            # instalador one-line (macOS/Linux)
├── install.ps1           # instalador one-line (Windows)
├── assets/               # iconos y assets
├── docs/                 # documentación del producto (arquitectura, etc.)
└── .github/workflows/    # CI, release, windows-smoke, require-issue
```

## Verificación en runtime

`cargo build` por sí solo no demuestra nada sobre el comportamiento: las sesiones interactivas y `wvc run` las sirve el daemon de larga vida en `~/.weavecoder/builds/shared-server/weavecoder`. Hasta que ese symlink se reapunte y el daemon se reinicie (`wvc self-dev --build`), un binario recién compilado es inerte.

Para probar un cambio sin tocar el daemon compartido ni la sesión del llamante:

```bash
cargo build --profile selfdev
./target/selfdev/weavecoder run --no-update --socket /run/user/1000/weavecoder-mytest.sock '<prompt>'
```

Notas:

- `crate::logging::info` escribe a un fichero de log, no a stderr: para diagnósticos desechables usa `eprintln!` y bórralo antes de commitear.
- Confirma qué binario estás inspeccionando: `builds/shared-server/weavecoder` es un symlink; resuélvelo con `readlink -f` antes de hacer `strings`/`nm`.
