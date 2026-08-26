# Local Models First

## Purpose

Run the agent on local models (oMLX, Ollama, LM Studio, llama.cpp, vLLM) or any
OpenAI-compatible endpoint, with multiple named providers, without sending code
to a cloud.

## Requirements

- Login supports oMLX (port 8000), Ollama, LM Studio, llama.cpp (8080), vLLM
  (8000), and a generic OpenAI-compatible provider.
- Choosing "OpenAI-compatible" walks the user through name → base URL → model →
  API key; each provider is saved as a named `[providers.<name>]` profile in
  `config.toml`. Multiple providers can be added.
- Local endpoint providers let the user edit the base URL before the API-key
  step; the override persists as `WVC_<PROFILE>_API_BASE`.
- Auto-detection probes local ports; if none respond, only cloud providers are
  shown.
- No subscription system exists; there are no paid tiers.
