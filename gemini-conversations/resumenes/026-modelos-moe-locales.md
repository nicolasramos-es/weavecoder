## Tema
Selección de modelos de IA locales tipo MoE para implementación como agente de chat en Spacebot.

## Decisión clave
Se recomienda utilizar Mixtral 8x22B Instruct (Q4, ~40GB RAM) o DeepSeek-Coder-V2-Lite-Instruct (16B total, 2.4B activos) para garantizar el cumplimiento estricto de instrucciones en Spacebot.

## Datos relevantes
- Modelos MoE mencionados: DeepSeek-V3 (671B total, 37B activos), Mixtral 8x7B, Qwen2.5-1.5B-MoE, Jamba.
- Configuración recomendada para Spacebot: Temperatura baja (0.1-0.2), formato de prompt estricto, system prompt role-based.
- Recursos: Mixtral 8x22B Instruct requiere ~40GB RAM.

## Categoría
IA Local
