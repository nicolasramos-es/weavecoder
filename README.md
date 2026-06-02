# Hermes Config Backup

Configuración de Hermes Agent para la cuenta de Nicolás Ramos.

## Estructura

```
config/          → Configuración principal (config.yaml, .env.example)
skills/          → Skills personalizados
cron/            → Jobs programados
scripts/         → Scripts de automatización
memory/          → Memoria persistente (MEMORY.md, USER.md)
gemini-conversations/  → Resúmenes de conversaciones de Gemini
chatgpt-conversations/ → Resúmenes de conversaciones de ChatGPT
```

## Backup automático

Este repo se actualiza automáticamente cada día a las 06:00 UTC vía cron.
Cada cambio que hace Hermes a los archivos de configuración genera un commit automático.