---
name: nico-context
description: "Contexto técnico completo de Nicolás Ramos (Nico) para uso en Hermes y agentes de código."
version: 1.0.0
author: Nicolás Ramos
platforms: [linux, macos]
metadata:
  hermes:
    tags: [personal, context, odoo, opencode, codex, infrastructure]
    auto_load: true
---

# Contexto Técnico de Nico

## 1. Identidad Profesional

- **Nombre:** Nicolás Ramos, llámame Nico.
- **Perfil:** Programador Python y desarrollador de Odoo. Trabajo en Som IT cooperatiu por las mañanas (lunes a viernes, 6:15–14:00, remoto desde Canarias) y como freelance Odoo por las tardes.
- **Estilo de respuesta:** Castellano de España, informal, directo, sin rodeos. Humor ágil cuando encaje. Prioridad: producción, estabilidad, coste/rendimiento, mantenimiento a largo plazo.
- **Filosofía:** Elegir pocas piezas buenas, congelar stack, evitar dispersión tecnológica. No sobrearquitectura.

## 2. Hardware

| Equipo | RAM | Uso |
|--------|-----|-----|
| MacBook Pro 16" M1 Max | 64 GB / 1 TB SSD | Principal de trabajo |
| Mac Studio M1 Ultra (×2) | 64 GB / 1 TB SSD | Inferencia principal (MLX/OMLX) — IPs: 192.168.1.6 (principal, puerto 8000) |
| Mac mini Late 2014 i7 (×3) | 16 GB / 256 GB | Infraestructura ligera (gateways, proxies, runtimes por cliente) |
| Mac mini M1 | 8 GB | Computer Use (macOS), tareas auxiliares, STT/Whisper — IP 192.168.1.17, macOS 26.5 |
| Mac mini M4 (×2) | 16 GB | Workers ligeros, embeddings, compactor, modelos 4B–8B cuantizados |

**Regla de inferencia:** 64 GB o más para inferencia seria. M1/M2/M3/M4 Max o Ultra. Equipos de 16 GB solo para workers, embeddings, modelos pequeños.

## 3. Infraestructura Local

- **LiteLLM:** Router central obligatorio. Aliases estables ocultan modelos físicos a las apps.
- **MLX/OMLX:** Inferencia local sobre Mac Studio M1 Ultra.
- **Arquitectura:** Clientes/agentes → LiteLLM → MLX/local en Mac Studio → APIs cloud como fallback.
- **Regla:** No Postgres ni Redis innecesarios. Diseño simple y robusto.

### Aliases LiteLLM relevantes
- `chat-general`, `chat-general-max`
- `ChatGPT`, `ChatGPT-max`
- `GLM47`, `GLM47-max`
- `audio-transcription`, `audio-transcription-max`
- `audio-speech`, `audio-speech-max`
- `glm-backup`
- `agente-odoo`
- `vision-main`, `vision-fallback`
- `embed-main`

## 4. Herramientas de Desarrollo

| Herramienta | Uso |
|-------------|-----|
| **OpenCode** | IDE/herramienta principal de desarrollo |
| **Codex 5.3 GPT** | Modelo fuerte para programación/razonamiento en OpenCode |
| **Cursor Pro** | Por empresa (Som IT) y Composer2 |
| **Z.ai** | Fallback para coding intensivo |
| **Mistral Vibe Pro** | Programación diaria (NO backend de clientes) |

## 5. Proveedores de IA

- **Ollama Cloud Pro:** Para OpenClaw/Clawdia.
- **OpenCode Go:** Para OdooClaw de clientes. Contiene MiniMax.
- **MiniMax vía OpenCode Go:** Prioritario para agente-odoo/OdooClaw. Variables: `OPENCODE_ANTHROPIC_API_BASE`, `OPENCODE_API_KEY`.
- **MiniMax API directa:** Para Spacebot/coding. SEPARADA de la vía OpenCode Go.
- **Mac Studio omlx6:** Endpoint `http://192.168.1.6:8000/v1`, modelo `Qwen3.6-35B-A3B-oQ4-fp16-mtp`. Usado para procesamiento local de datos personales (Gemini, ChatGPT). API key en `OMLX6_API_KEY` en `.env`.
- **DeepSeek / Qwen:** OpenAI-compatible. Separados de MiniMax directo.
- **GLM/Z.ai:** Fallback cloud.
- **ChatGPT Plus / Codex:** Apoyo de razonamiento.
- **Gemini AI Pro 5 TB:** Parte del ecosistema.

**Regla:** No mezclar MiniMax de OpenCode Go con MiniMax directo.

## 6. Agentes y Proyectos

| Proyecto | Rol |
|----------|-----|
| **Paperclip** | Gobierno/dirección. Nico aprueba despliegues. |
| **Spacebot** | Coordinación técnica, memory, branches, workers, routing de agentes |
| **OpenCode** | Ejecución de código |
| **GitHub Actions** | Validación |
| **Dokploy / Coolify / OVH / Hetzner** | Despliegue |

### OdooClaw / agente-odoo
- Backend OpenAI-compatible: `https://odooclaw.tudominio.com/v1/chat/completions`
- Modelo: `odooclaw`
- Skills desacopladas, async, idempotentes. Sin Redis/Postgres por meter.
- MiniMax de OpenCode Go es el modelo principal para agente-odoo.

### OpenClaw / Clawdia
- Móvil dedicado como interfaz (NO inferencia local). Xiaomi Redmi Note 10 5G como MVP.
- Lock-task / kiosk mode. HTTPS/WSS obligatorio.
- Backend externo siempre.

### Xiaozhi (ESP32-S3)
- Backend self-hosted para apuntar a OdooClaw.
- `base_url: https://odooclaw.tudominio.com/v1`, `model: odooclaw`

## 7. Preferencias de Implementación

- Python cuando encaje. Compatible con Odoo.
- OpenAI-compatible para endpoints de agentes.
- Servicios pequeños y claros, no monolitos.
- Producción real, no prototipos frágiles.
- Coste controlado. Inferencia local cuando tenga sentido, cloud cuando sea más práctico.
- Skills desacopladas. Sin acoplar cada proyecto a una API concreta.

## 8. Contacto y Datos

- **Email profesional:** hola@nicolasramos.es
- **LinkedIn:** nramosdev
- **Google account de mi identidad agente:** agentenicolasramos@gmail.com (Multilock00@). Calendario de Nico compartido con esta cuenta. El correo de Nico redirige correos de clientes aquí para que cree tareas.

## 8a. Archivos de Conversaciones Archivadas

Todas las conversaciones exportadas se guardan localmente para mantener el contexto unificado:

| Fuente | Ubicación | Cantidad |
|--------|-----------|----------|
| Gemini (nicolasjesus@gmail.com) | `~/.hermes/gemini-conversations/` | 32 conversaciones |
| ChatGPT (exportación ZIP) | `~/.hermes/chatgpt-conversations/` | 611 conversaciones |

Cada carpeta contiene `raw/` (conversaciones originales en .md), `resumenes/` (resúmenes estructurados generados por Qwen local), y un index. Los resúmenes siguen formato: Tema, Decisión clave, Datos relevantes, Categoría.

## 9. Infraestructura Real (corregida May 2026) (corregida May 2026)

| Capa | Ubicación | Qué va |
|------|-----------|--------|
| Servicios críticos, agentes, routing | **OVH / Proxmox / Coolify** | LiteLLM, n8n, Evolution API, Redis, Hermes/OpenClaw/Goose por cliente |
| Inferencia ligera | **Mac mini M4 (×2)** vía Tailscale | Gemma E4B, embeddings, compactor |
| Voz/STT | **Mac mini M1 8GB** vía Tailscale | Whisper large-v3-turbo |
| Inferencia pesada | **Mac Studio Ultra (×2)** vía Tailscale | Modelo grande principal + fallback |
| Staging / laboratorio | **Mac mini 2014 i7 (×3)** | Pruebas de agentes, watchdog, runner QA |

**Regla clave de arquitectura:** OVH aloja todo lo crítico. Lo local es solo capacidad bruta de IA. Si se va la luz en casa, LiteLLM en OVH sigue vivo y fallbackea a cloud.

**Tailnet:** Todas las máquinas bajo `nicolasjesus@` en Tailscale. IPs conocidas: coolify=`100.95.37.116`, nas-ramos=`100.111.213.128`, macstudio-ultra=`100.110.159.116`, mac-studio-ultra-64-2=`100.95.113.23`, openclaw-1 (yo)=`100.72.129.128`.

**API Server (Open WebUI):** Hermes expone `/v1` en puerto `8642` con key `266e5532bc2343a1a3715ebacde65b4aa6be9601e512f237af07e269b2539539`. Gateway como systemd user service hermes-gateway.

## 10. Reglas de Estilo para Asistentes

- Tratamiento informal. "Tú", no "usted".
- Directo al grano. Sin explicaciones de principiante salvo que se pidan.
- Priorizar: producción, estabilidad, simplicidad, coste/rendimiento, mantenimiento.
- No vender humo. No cambiar de stack sin razón clara.
- Justificar decisiones. Evitar respuestas genéricas.
- **Boceto/plan detallado primero, ejecución después**: No empezar cambios sin aprobación explícita. Cuando Nico pide algo complejo, presentar plan → esperar visto bueno → SOLO entonces ejecutar.
- **Datos personales/procesamiento local**: Cuando se trata de datos personales de Nico (conversaciones de Gemini, datos de clientes, etc.), procesar con modelo local (Mac Studio MLX) en lugar de cloud. No asumir cloud processing. Preguntar primero o usar local por defecto si hay duda.
- **Multi-cuenta en web apps**: Al interactuar con servicios Google (Gemini, etc.), Nico tiene DOS cuentas: `agentenicolasramos@gmail.com` (agente) y `nicolasjesus@gmail.com` (personal). Usar prefijo `/u/1/` en URLs para la cuenta personal. Verificar siempre en qué cuenta se está antes de actuar. Regla de oro: sin `/u/1/` en la URL de Gemini, estás en la cuenta de agente por defecto.
