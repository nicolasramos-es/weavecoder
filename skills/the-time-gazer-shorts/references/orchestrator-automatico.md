# Orquestador Automático de Shorts

## Arquitectura

El sistema automático se compone de:

1. **`orchestrate_short.py`** — Orquestador inteligente en `~/.hermes/video-producer/`
2. **`run_short_cron.sh`** — Wrapper en `~/.hermes/scripts/` que ejecuta el orquestador
3. **Cron job** (ID: `976c01aff380`) — Cada 2h, modo silencioso (no_agent=true, deliver=local)
4. **`daily_summary.sh`** — Resumen diario a Telegram (cron ID: `211be7574f40`) a las 08:00 Canarias
5. **`published_history.json`** — Evita repetir scripts ya publicados

## Flujo de una ejecución

1. Orquestador elige script rotando series: expediente(peso 4), history(3), misterios(3)
2. Filtra scripts inválidos (sin `visual_prompt`/`prompt`/`image_prompt`)
3. Mejora título: quita mayúsculas, acorta a <55 chars, añade emoji según temática
4. Genera hook específico del tema (si el existente es genérico tipo "SECRET\nENIGMA")
5. Añade escena CTA al final con texto aleatorio y prompt visual
6. Genera descripción SEO con hashtags en la descripción (NUNCA en el título)
7. Ejecuta `produce_video_STABLE.py --publish <categoría>`
8. Marca script como usado en `published_history.json`

## Notas importantes

- **Hashtags SIEMPRE en descripción, NUNCA en título.** Los hashtags en título roban espacio del anzuelo y no aportan más SEO.
- Si todos los scripts se han usado, se resetea automáticamente la cola.
- Los scripts inválidos (~89/467) se filtran silenciosamente.
- El cron es silencioso (no_agent, deliver=local) — no genera ruido en Telegram.
- El error en una ejecución se captura en el log; solo se alerta al usuario si es crítico (token caducado, OVH caído, etc.).
