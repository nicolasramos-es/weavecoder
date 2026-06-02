---
name: ai-conversation-centralization
description: "Extract conversations from AI chat platforms (Gemini, ChatGPT) and centralize them as local markdown archives. Covers Gemini web extraction via cua-driver, ChatGPT/OpenAI JSON takeout processing, and privacy-preserving summarization via a local MLX model."
version: 1.0.0
author: agent
metadata:
  hermes:
    tags: [data-migration, gemini, chatgpt, conversation-export, privacy]
    related_skills: [nico-context, hermes-provider-configuration]
---

# AI Conversation Centralization

Extract, process, and unify conversations from multiple AI chat platforms into a structured local archive. Designed for the user who wants all their AI context in one place without sending personal data to cloud providers.

## Trigger

Use this skill when the user asks to:
- Export/extract their conversations from **Gemini** (web interface)
- Process a **ChatGPT/OpenAI takeout ZIP** export
- Unify conversations from multiple AI platforms into one local archive
- Process conversations with a **local model** for privacy
- "Centralizar" or "unificar" their chat history in one place

## Data destinations

```
~/.hermes/gemini-conversations/       ← Gemini extractions
  ├── NNN-titulo.md                   ← raw conversation (markdown)
  ├── resumenes/
  │   └── NNN-titulo.md               ← structured summaries
  └── .conversations_index.json        ← index of all convos

~/.hermes/chatgpt-conversations/      ← ChatGPT takeout
  ├── raw/
  │   └── NNNN-titulo.md              ← raw conversations extracted from JSON
  ├── resumenes/
  │   ├── NNNN-titulo.md              ← structured summaries
  │   └── STATS.md                    ← category breakdown
  └── conversations-XXX.json          ← original takeout files (extracted ZIP)
```

## Platform-specific techniques

### Gemini (web extraction via cua-driver)

The Gemini web app (`gemini.google.com/app`) loads conversations in a sidebar. Account switching is critical:

1. **Account routing**: The user has TWO Google accounts: `agentenicolasramos@gmail.com` (agent) and `nicolasjesus@gmail.com` (personal). The personal account uses the `/u/1/` URL prefix. **Always verify the account shown in the sidebar** before proceeding.
2. **Enable JavaScript for Apple Events** first (one-time): `page(action='enable_javascript_apple_events', bundle_id='com.google.Chrome', user_has_confirmed_enabling=true)` — without this, `execute_javascript` fails.
3. **Get conversation list**: Navigate to `https://gemini.google.com/u/1/library` and run JS to extract all `<a href*="/app/">` links. Deduplicate by URL path. Filter out UI elements (Nueva, Buscar, Biblioteca, Gems, Actividad, Ajustes, Más opciones, Nicolás).
4. **Navigate to each conversation**: Use `page(action='execute_javascript', javascript="window.location.href='https://gemini.google.com/u/1/app/CONV_ID'")` — the `/u/1/` prefix is **mandatory** for the personal account.
5. **Extract content**: Use `page(action='get_text')` to get the full conversation text including user prompts and Gemini responses.
6. **Save**: Write to `~/.hermes/gemini-conversations/NNN-titulo.md` with structured header (title, user, date).

**Pitfalls:**
- Navegar sin `/u/1/` te lleva a la cuenta de agente, no a la personal
- La Biblioteca (`/u/1/library`) puede mostrar solo ~30 conversaciones recientes; Gemini no carga el histórico completo de golpe
- Algunas conversaciones tienen el mismo título (duplicados por edición) — desduplicar por URL única
- El sidebar solo muestra ~30 recientes; para ver TODAS hay que ir a Biblioteca (`/u/1/library`)

### ChatGPT takeout (OpenAI JSON export)

The OpenAI data export ZIP contains:

```
chat.html                           ← browsable HTML (large, ~40MB)
conversations-000.json ... 006.json ← conversation data (7 files, ~60MB total)
user.json                           ← user profile
library_files.json                  ← library/metadata
shared_conversations.json           ← shared convos
```

**JSON structure (conversations-N.json):**

```json
[
  {
    "conversation_id": "...",
    "title": "Suma de importes",
    "create_time": 1762416916.36243,
    "default_model_slug": "gpt-4",
    "mapping": {
      "node-id": {
        "message": {
          "author": {"role": "user" | "assistant"},
          "content": {
            "parts": ["text..."]
          }
        },
        "parent": "parent-node-id"
      }
    },
    "current_node": "last-node-id"
  }
]
```

**Extraction workflow:**

```python
# 1. Walk the conversation tree via current_node → parent chain to get ordered messages
# 2. For each message: extract role (user/assistant) and content parts
# 3. Save as markdown with: title, date, model, and alternating Usuario/ChatGPT blocks
# 4. Truncate content at ~2000 chars per message if needed
```

**Process in batches of 5-10** to avoid overloading the context window. Each conversation takes ~2-5 seconds on the local model.

### Privacy-preserving processing

When the user explicitly wants data processed locally:

1. **Do NOT use cloud providers** for summarization/analysis of personal data
2. Route through the local model provider (e.g., Mac Studio MLX at `192.168.1.6:8000` with `Qwen3.6-35B-A3B-oQ4-fp16-mtp`)
3. Use the OpenAI-compatible API directly:

```bash
curl -s -X POST http://192.168.1.6:8000/v1/chat/completions \
  -H "Authorization: Bearer $API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "model": "Qwen3.6-35B-A3B-oQ4-fp16-mtp",
    "messages": [
      {"role": "system", "content": "Resume: tema, decisión clave, datos relevantes, categoría."},
      {"role": "user", "content": "conversation text here"}
    ],
    "max_tokens": 512,
    "temperature": 0.1
  }'
```

4. Guardar cada resumen con formato estructurado: `## Tema`, `## Decisión clave`, `## Datos relevantes` (lista), `## Categoría`
5. Al final, generar `STATS.md` con conteo por categoría

**Categories used:** Odoo, IA Local, Infra Mac, Música, Desarrollo, Casa, Finanzas, Legal, Salud, Viajes, Social, Formación, Otros

## Summary format

```
## Tema
[máx 15 palabras, describe el tema principal]

## Decisión clave
[la decisión o conclusión más importante de la conversación]

## Datos relevantes
- [dato 1]
- [dato 2]

## Categoría
[una de las categorías estándar]
```