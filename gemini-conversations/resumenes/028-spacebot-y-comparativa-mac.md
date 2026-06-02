## Tema
Configuración de Spacebot y comparación de hardware Mac para inferencia de IA local.

## Decisión clave
Para cargas de trabajo de IA local, el Mac Studio M1 Max (32GB) es superior al Mac Mini M4 Pro (24GB) debido a su mayor ancho de banda de memoria (400 GB/s vs 273 GB/s) y capacidad de RAM, siendo este factor crítico para la velocidad de generación de tokens.

## Datos relevantes
- **Spacebot Config**: Provider LiteLLM en `http://litellm.51.38.254.149.sslip.io`. Routing: sb-chat-local → chat-general → Gemma4E4B. Tareas: coding, summarization, deep_reasoning, review, migration, infra.
- **Error Spacebot**: `failed to resolve API key for provider 'litellm'`. Solución: exportar `SPACEBOT_LITELLM_KEY` o añadir `/v1` al base_url.
- **Usuarios**: Nicolás Ramos (CEO), Jesús López (CEO). Company: J&J Gestiones.
- **Compaction thresholds**: background 0.75, aggressive 0.85, emergency 0.95.
- **Benchmark M4 Pro vs M1 Max**: CPU Single-core M4 Pro ~3800 vs M1 Max ~1750 (Geekbench). Neural Engine M4 Pro 38 TOPS vs M1 Max 11 TOPS.
- **Memoria**: Ancho de banda M1 Max 400 GB/s vs M4 Pro 273 GB/s.
- **Hardware**: Mac Studio M1 Max 32GB vs Mac Mini M4 Pro 24GB. El M4 Pro se queda corto con modelos de 27B+.

## Categoría
Infra Mac
