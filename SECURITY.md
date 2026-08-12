# Política de seguridad de Weavecoder

Weavecoder toma en serio la seguridad del software. Esta política describe cómo reportar vulnerabilidades y qué está dentro del alcance.

## Reportar una vulnerabilidad

**No abras issues públicos** para vulnerabilidades de seguridad.

- **Email**: [contacto@nicolasramos.es](mailto:contacto@nicolasramos.es) — asunto `[SECURITY] <descripción breve>`.
- **GitHub Security Advisories**: si el repositorio tiene habilitada la pestaña *Security → Advisories*, usa el flujo de *Report a vulnerability* (privado).

### Qué incluir en el reporte

Para que el triaje sea rápido, incluye:

1. **Versión afectada**: salida de `wvc version` (o al menos el commit/hash si builds de desarrollo).
2. **Plataforma**: SO, arquitectura, y si es release estable o build de fuente.
3. **Descripción del problema**: tipo (inyección SQL, XSS en TUI/render, ejecución de código, escalada de permisos, exfiltración de datos, etc.) y componente afectado.
4. **Pasos de reproducción**: mínimo reproducible, idealmente con un proyecto de ejemplo.
5. **Impacto esperado y real**: qué puede hacer un atacante.
6. **PoC** (si existe) — sin incluir datos sensibles de terceros.

### Qué esperar

- Acuse de recibo en **48–72 h** laborables.
- Evaluación de severidad y plan de fix en **7 días** desde el acuse.
- Coordinación de la divulgación: trabajaremos contigo para publicar el advisory cuando el fix esté disponible.

## Política de divulgación

- **Embargo de 90 días** por defecto desde la confirmación del reporte, prorrogable si el fix lo requiere.
- Pedimos **divulgación responsable**: no publiques detalles hasta que exista un fix o el embargo expire.
- Reconoceremos al reportero en el advisory salvo que pida anonimato.

## Alcance

### Dentro de alcance

- El binario `wvc` y todos los crates del workspace (`crates/wvc-*`).
- El Code Knowledge Graph: parsing de código de terceros con tree-sitter, almacenamiento SQLite/FTS5 e indexación (el CKG procesa **código no confiable** — proyectos arbitrarios — por diseño).
- Instaladores `install.sh` / `install.ps1` y el flujo de actualización (`wvc update`, GitHub Releases).
- Autenticación y manejo de credenciales (login OAuth/API key, configuración `config.toml`).
- Comunicación cliente–servidor local (socket) y endpoints de la API (harness/api-bridge).

### Fuera de alcance

- Vulnerabilidades en dependencias de terceros: repórtalas al proyecto correspondiente (tree-sitter, rusqlite, tract-onnx, etc.).
- Vulnerabilidades en proveedores de modelos o infraestructura externa (Ollama, OpenAI, etc.).
- Problemas que requieran acceso físico o compromiso previo de la máquina.

## Versiones soportadas

| Versión | Soporte |
|---|---|
| Última release estable | ✅ fixes de seguridad |
| Builds de desarrollo (`-dev`) | ⚠️ se revisan, pero el fix puede llegar en la siguiente release |

## Divulgaciones

Las vulnerabilidades confirmadas se documentarán en el [CHANGELOG](CHANGELOG.md) y, cuando aplique, en GitHub Security Advisories. Actualmente no hay divulgaciones publicadas.
