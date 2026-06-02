# LiteLLM Proxy & MiniMax & Mac Studio Connectivity Details

Captured from sessions 20260523 and 20260602 — Nico's Hermes provider setup.

## LiteLLM Proxy

- **URL:** `http://litellm.51.38.254.149.sslip.io/v1`
- **API Key:** `sk-TL9K6IPD__Wx2LfE-8jACQ` (stored in `.env` as `LITELLM_API_KEY`)
- **Models available:** 38 models including sb-chat-local, local-coding, opencode-deepseek-v4-*, vision-main, embed-main, audio-transcription, audio-speech, agente-odoo, ollama-deepseek-v4-pro, etc.
- **API mode:** `chat_completions`
- **Status:** ✅ Working — responds in ~1.7s for `sb-chat-local`

## MiniMax (native Hermes provider)

- **Provider name:** `minimax`
- **Base URL (via native plugin):** `https://api.minimax.io/anthropic` (Anthropic format)
- **Working fallback URL:** `https://api.minimax.io/v1/chat/completions` (OpenAI format)
- **API Key:** Stored in `.env` as `MINIMAX_API_KEY`
- **Model:** `MiniMax-M2.7`
- **Status:** ✅ Native provider works (`hermes chat -q 'test' --provider minimax --model 'MiniMax-M2.7'`)
- **Quirk:** The Anthropic endpoint returned 404 for this API key; OpenAI-compatible `/v1/chat/completions` works fine. Hermes native provider uses Anthropic path but resolves it internally.

## Mac Studio (omlx6 — local MLX server)

Configured 2026-06-02 for privacy-preserving local inference.

- **Custom provider name:** `omlx6` (custom_providers entry)
- **URL:** `http://192.168.1.6:8000/v1`
- **API Key:** Stored in `.env` as `OMLX6_API_KEY`
- **API mode:** `chat_completions`
- **Model slug in config:** `qwen3.6-35b-a3b`
- **Actual model ID on server:** `Qwen3.6-35B-A3B-oQ4-fp16-mtp`
- **Status:** ✅ Endpoint reachable at `http://192.168.1.6:8000/v1/models`
- **Purpose:** Local-only processing of personal/sensitive data that must not leave Nico's LAN
- **Reference in fallbacks:** `custom:omlx6` with model slug as model value

### Models available on Mac Studio

| Config slug | Server model ID | Description |
|---|---|---|
| `gemma-4-e4b-it-oQ4-fp16` | `gemma-4-e4b-it-oQ8-fp16` | Gemma 4 E4B (~29B), 4bit |
| `qwen3.6-35b-a3b` | `Qwen3.6-35B-A3B-oQ4-fp16-mtp` | Qwen 3.6 35B A3B MoE, 4bit with Multi-Token Prediction |

### Other cached MLX models (available for download)

These are in `~/.cache/huggingface/hub/` — not loaded on the server but could be:

- `lmstudio-community/gemma-4-E4B-it-MLX-4bit`
- `mlx-community/gemma-4-E2B-it-assistant-bf16`
- `mlx-community/gemma-4-e2b-it-4bit`
- `mlx-community/Llama-3.2-3B-Instruct-4bit`
- `mlx-community/Mistral-Nemo-Instruct-2407-4bit`
- `mlx-community/Qwen2.5-Coder-7B-Instruct-4bit`
- `mlx-community/Qwen3-Embedding-0.6B-4bit-DWQ`

## Fallback chain (current as of 2 Jun 2026)

```yaml
fallback_providers:
  - provider: minimax
    model: MiniMax-M2.7
  - provider: custom:litellm
    model: sb-chat-local
```

## Primary model

- **Provider:** `ollama-cloud`
- **Model:** `deepseek-v4-flash`
- **Auxiliary vision:** `ollama-cloud / qwen3-vl:235b-instruct`

## Privacy note

When the user wants personal/sensitive data processed without leaving the local network, use `custom:omlx6` / model slug `qwen3.6-35b-a3b` as the active provider. Do NOT route sensitive data through cloud providers (ollama-cloud, minimax, litellm) without explicit user approval.

## Gemini conversations — bulk export reference

During session 2026-06-02, ~430 Gemini conversations were discovered. A batch extraction workflow was used:
1. Navigate to Gemini `/u/1/library` to see all conversations
2. Extract all conversation URLs via `query_dom` / `execute_javascript`
3. For each URL, navigate via JS (`window.location.href = ...`), wait for load, extract via `get_text()`, save to file
4. Use URL prefix `/u/1/` to stay in Nico's personal account (`nicolasjesus@gmail.com`)

See `macos-computer-use` skill → `references/web-content-extraction.md` for details.