---
name: hermes-provider-configuration
description: "Configure LLM providers and fallback chains in Hermes Agent — custom_providers, fallback_providers, native providers, and connectivity testing."
version: 1.0.0
author: agent
created_by: agent
metadata:
  hermes:
    tags: [hermes, providers, configuration, minimax, litellm, fallback]
    related_skills: [hermes-agent]
---

# Hermes Provider Configuration

Configuring providers, custom endpoints, and fallback chains in Hermes. This skill covers the recurring task of wiring up a new LLM provider (native or custom), setting up fallback chains, and testing connectivity.

## Trigger

Use this skill when the user asks to:
- Add or configure a new provider (native or custom)
- Set up a `custom_providers` entry (e.g. LiteLLM proxy)
- Change or reorder `fallback_providers`
- Change the default model/provider
- Test whether a provider endpoint is reachable and functional
- Add API keys for a new provider

## Workflow

### 1. Check current config

```bash
hermes config show
```

Read the full `config.yaml` to understand the current layout:

```bash
python3 -c "
import yaml
with open('/Users/nramos/.hermes/config.yaml') as f:
    d = yaml.safe_load(f)
# model is a plain string for ollama-cloud setup
model_val = d.get('model', '')
providers_val = d.get('providers', {})
fallback_val = d.get('fallback_providers', [])
print('Model:', model_val)
print('Providers:', providers_val)
print('Fallbacks:', fallback_val)
"
```

**IMPORTANT:** When `model:` is a plain string (e.g. `model: deepseek-v4-flash`), the `providers:` key tells Hermes which credential pool to use (e.g. `{ollama-cloud: {}}`). Do NOT set `model:` as a dict with `default`/`provider` keys — that only works if you're using the dict form, and ollama-cloud uses the string form with a separate `providers` entry.

### 2. Verify model name against ollama-cloud catalog before configuring

| Provider type | How to configure | Credentials |
|---|---|---|
| **ollama-cloud** (Nico's primary) | `model: <name>`, `providers: {ollama-cloud: {}}` in config.yaml | `OLLAMA_API_KEY` in `.env` (credential pool `ollama-cloud`) |
| **Native** (minimax, openai, anthropic, etc.) | Just set env var, then reference by name | `MINIMAX_API_KEY` in `.env` |
| **Custom** (litellm, self-hosted, proxy) | Add `custom_providers` section in `config.yaml` | `LITELLM_API_KEY` in `.env` + `pool_key` reference |
| **OAuth** (minimax-oauth, openai-codex, etc.) | `hermes login --provider <name>` | Token in `auth.json` |

**Nico's provider chain (confirmed 2026-05-28):**
```
Primary:   ollama-cloud / deepseek-v4-flash
Fallback1: minimax / MiniMax-M2.7
Fallback2: custom:litellm / sb-chat-local
Aux vision: ollama-cloud / qwen3-vl:235b-instruct
```
**OpenRouter is NOT used** — Nico always routes through ollama-cloud. Never configure OpenRouter as primary or fallback.

### 3. Test connectivity before configuring

Always verify the endpoint responds before wiring it up:

```bash
# Step 1: Check endpoint reachable
curl -s -o /dev/null -w '%{http_code}' --connect-timeout 10 <base_url>/models

# Step 2: Test with API key
curl -s --connect-timeout 15 \
  -H "Authorization: Bearer $API_KEY" \
  <base_url>/models

# Step 3: Test actual chat completion
curl -s --connect-timeout 30 \
  -H "Authorization: Bearer $API_KEY" \
  -H "Content-Type: application/json" \
  -d '{"model":"<model_name>","max_tokens":50,"messages":[{"role":"user","content":"Respond only with: OK funciona"}]}' \
  <base_url>/v1/chat/completions
```

### 4. Add API key

```bash
echo "PROVIDER_API_KEY=<key>" >> ~/.hermes/.env
```

**Do NOT** log or display the full key after this step. Only confirm it was added.

### 5. Configure the provider

#### Native provider (e.g. MiniMax)

The env var `MINIMAX_API_KEY` is sufficient. The native provider profiles are registered in `plugins/model-providers/minimax/__init__.py` in the Hermes source.

**MiniMax quirk:** The Anthropic endpoint (`api.minimax.io/anthropic`) may return 404 for some API keys. The OpenAI-compatible endpoint (`/v1/chat/completions`) always works. The native Hermes `minimax` provider uses the Anthropic endpoint by default — test with `/v1/chat/completions` if the Anthropic path fails.

Test the native provider with:

```bash
hermes chat -q 'Responde solo con "OK funciona"' --provider minimax --model 'MiniMax-M2.7'
```

#### Custom provider (e.g. LiteLLM proxy)

Add to `config.yaml` under `custom_providers`:

```yaml
custom_providers:
- name: litellm
  display_name: LiteLLM Proxy
  slug: litellm
  pool_key: litellm
  api_key_env_vars:
  - LITELLM_API_KEY
  models:
    sb-chat-local:
      display_name: SB Chat Local
      id: sb-chat-local
    local-coding:
      display_name: Local Coding
      id: local-coding
  base_url: http://your-proxy-url/v1
  api_mode: chat_completions
```

Reference in fallbacks as `custom:litellm`.

### 6. Set up fallback chain

Use `hermes config set fallback_providers`:

```bash
hermes config set fallback_providers '[
  {"provider": "minimax", "model": "MiniMax-M2.7"},
  {"provider": "custom:litellm", "model": "sb-chat-local"}
]'
```

**⚠️ PITFALL:** `hermes config set fallback_providers` stores the value as a **JSON string** in YAML, not as a YAML list. After setting it this way, the YAML looks like:

```yaml
fallback_providers: '[{"provider": "minimax", ...}]'
```

This is technically valid YAML (a string) but Python's `yaml.safe_load` will read it as a string, not a list. To get proper YAML list format, edit `config.yaml` directly using `execute_code` / Python:

```python
import yaml
with open('/Users/nramos/.hermes/config.yaml') as f:
    d = yaml.safe_load(f)
d['fallback_providers'] = [
    {"provider": "minimax", "model": "MiniMax-M2.7"},
    {"provider": "custom:litellm", "model": "sb-chat-local"}
]
with open('/Users/nramos/.hermes/config.yaml', 'w') as f:
    yaml.dump(d, f, default_flow_style=False, sort_keys=False, allow_unicode=True)
```

**Preferred approach:** Use `execute_code` with the YAML dump method above for `fallback_providers`, as it produces clean multi-line YAML and avoids the stringification bug.

### 7. Verify the complete chain

```bash
python3 -c "
import yaml
with open('/Users/nramos/.hermes/config.yaml') as f:
    d = yaml.safe_load(f)
print('Main:', d['model']['default'], '/', d['model']['provider'])
for i, fb in enumerate(d.get('fallback_providers', [])):
    print(f'Fallback {i+1}:', fb['provider'], '/', fb['model'])
"
```

Then test each provider independently:

```bash
hermes chat -q 'test' --provider <provider> --model '<model>'
```

### 8. Update memory

Save the provider chain to memory so future sessions know the current layout without re-reading config.

### Adding a model to an existing custom provider

When the user provides a new model that runs on an already-configured server (e.g., a Mac Studio at `192.168.1.6:8000`), you only need to add the model entry:

```bash
# 1. Verify the model exists on the server first
curl -s -H "Authorization: Bearer $API_KEY" http://<ip>:<port>/v1/models | python3 -m json.tool | grep model-id
```

```yaml
# 2. Add the model to the existing provider in config.yaml
custom_providers:
- name: omlx6  # existing
  models:
    existing-model: {display_name: "...", id: "existing-id"}
    NEW-MODEL-SLUG:          # ← add this
      display_name: "Human Readable Name"
      id: "exact-model-id"
```

The API key in `.env` is usually already set if the provider was previously configured.

### Privacy-preserving local-only processing

When the user explicitly wants data processed on their local network (e.g., personal Gemini chats routed through a Mac Studio on the same LAN):

1. **Do NOT suggest cloud processing** — accept the local-model constraint proactively. The user's instruction: "los datos no tienen que salir fuera de mi red local porque son con datos personales."
2. Configure a `custom_providers` entry pointing to their local inference server
3. After extracting/collecting data with available tools, route processing/summarization through the local provider
4. Verify the local endpoint is reachable before starting: `curl -s --connect-timeout 5 http://<ip>:<port>/v1/models`

## User preference: always clarify model-switch intent

When the user says "cambia el modelo a X" or "usa Y como fallback":

1. **Always clarify** whether they want to change: (a) the primary model, (b) a fallback, or (c) BOTH
2. **Do not assume** — the user has explicitly stated they prefer to confirm intent
3. **Ask clearly**: "¿Quieres cambiar el modelo principal, el fallback, o ambos?"
4. Only proceed after the user confirms

This applies even when the request seems unambiguous. Nico's workflow preference is explicit confirmation before action.

## Common pitfalls

- **Model name must match exactly what ollama-cloud catalog reports** — ollama-cloud model IDs differ from display names. Before setting `model:` in config.yaml, verify the exact model ID with:
  ```bash
  curl -s -H "Authorization: Bearer $OLLAMA_API_KEY" https://ollama.com/v1/models | python3 -c "import json,sys; d=json.load(sys.stdin); [print(m['id']) for m in d['data']]"
  ```
  Known mismatches:
  - `kimi2.6` (does not exist) → correct ID is `kimi-k2.6`
  - `deepseek-v4-flash` (exists as shown in catalog)
  - If a model name isn't found, check the catalog — ollama-cloud uses `provider/display_name` format for some models, not the bare display name
- **Config model field is a plain string for ollama-cloud** — do NOT use dict form with `default`/`provider` keys. Set `model: <model_id>` as string + `providers: {ollama-cloud: {}}`.
- **MiniMax Anthropic endpoint may 404** — some API keys only work via `/v1/chat/completions` (OpenAI format). Test both before declaring the provider broken.
- **`hermes config set` doesn't always validate YAML output** — always verify the result with a Python/YAML read after setting structured values.
- **Custom providers must use `custom:` prefix** in fallback entries (e.g. `custom:litellm`, not just `litellm`).
- **Config changes take effect on new session** — use `/reset` or start a fresh `hermes` invocation. In gateway, use `/restart`.
- **Always clarify intent** before changing model/provider: ask if they want to change the primary, fallback, or both. Do not assume.
- **Never use OpenRouter for Nico** — he routes exclusively through ollama-cloud + minimax + custom:litellm. OpenRouter is in credential_pool but is never the primary provider.

## Verification

After setup, confirm:
- [ ] API key is in `~/.hermes/.env`
- [ ] `fallback_providers` is a YAML **list** (not a JSON string)
- [ ] Each provider responds to `hermes chat -q 'test' --provider <name> --model '<model>'`
- [ ] Memory updated with the new provider chain
