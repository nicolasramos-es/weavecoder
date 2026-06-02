---
name: the-time-gazer-shorts
description: "Estrategia integral de contenido para The Time Gazer — canal de historia, misterio y crímenes basado en Shorts de YouTube producidos por Claudia VideoProducer."
tags: [youtube, shorts, content-strategy, video-producer, openclaw, historia, misterio]
related_skills: [youtube-content, video-frames]
platforms: [linux, macos, windows]
---

# The Time Gazer — Estrategia de Shorts

## ⚠️ Separación de Pipelines (IMPORTANTE)

**The Time Gazer y NR Music son sistemas completamente separados.** NO tratarlos como un mismo pipeline bajo ningún concepto. El usuario se queja explícitamente si se mezclan.

| Aspecto | The Time Gazer | NR Music (4 canales) |
|:--|:--|:--|
| Pipeline | `~/.hermes/video-producer/` | `~/nr-pop/`, `~/nr-rock/`, `~/nr-latino/`, `~/nr-hiphop/` |
| Cron | short cada 2h + daily summary (✅ activos) | 4 crons (❌ pausados por cuota YouTube API) |
| Script | `orchestrate_short.py` | `orchestrate_nr{canal}_v2.py` |
| Fallo típico | Timeout 120s del scheduler | 429 Rate Limit de YouTube API |
| Formato | Shorts narrados (historia/misterio) | Vídeos musicales con visualizador |

**NUNCA mezclar estos pipelines en una misma respuesta o plan.** Si el usuario pregunta por Time Gazer, no mencionar NR Music a menos que lo pregunte explícitamente y viceversa.

---

## Estado Actual del Canal

- **Canal:** [The Time Gazer](https://www.youtube.com/@TheTimeGazer)
- **Contenido:** 1.241 Shorts + 4 vídeos largos
- **Temática:** Historia universal, misterios históricos, crímenes famosos (expedientes)
- **Problema principal:** Views muy bajas (1-180 por short) para 1.241 publicaciones. El algoritmo dejó de recomendar.
- **Causas raíz identificadas:**
  1. Títulos demasiado largos y en mayúsculas (formato anticuado para Shorts 2026)
  2. Sin nicho definido — salta de historia antigua a crímenes modernos sin hilo
  3. Sin series/sagas que enganchen
  4. Hook visual débil en primeros 2 segundos
  5. Bajo engagement rate → algoritmo deja de recomendar

---

## Pipeline de Producción (Claudia VideoProducer)

### Arquitectura

El productor de vídeo es el pipeline Python en `~/.hermes/video-producer/` — ejecutado directamente por Hermes. OpenClaw se eliminó; no hay dependencia de él.

### Flujo Completo

```
JSON Script → Validación → Música única → Por cada escena:
  1. Generar imagen (OVH SDXL — NO DALL-E 3 porque no hay API key de OpenAI)
  2. Generar narración TTS (Edge-TTS, voz español)
  3. Crear clip de vídeo con Ken Burns + subtítulos
→ Concatenar clips → Mezclar audio + música → Validar 15s mín → Publicar YouTube (solo si --publish) → Log Mission Control
```

### Tecnologías Usadas

| Componente | Tecnología | Detalle |
|:--|:--|:--|
| **Script principal** | `produce_video_STABLE.py` (Python) | V14.5 estable, ejecutado por Hermes |
| **Imágenes** | MiniMax Image API (`image-01`) como PRIMARIO (funciona, ~1s respuesta, 100 usos/día compartidos). OVH SDXL como SEGUNDO fallback (token caducado 2026-05 — 403 Forbidden, necesita renewal). DALL-E 3 como TERCER fallback (model name: `dall-e-3`, timeout 60s). | 1024x1024, 30 steps, guidance 7.5. Alternativa ComfyUI en Mac Studios preparándose. |
| **Voz narradora (vídeo)** | Edge-TTS `es-ES-XimenaNeural` | Español castellano para la narración de shorts. **NO** `en-US-GuyNeural`. La voz del agente Hermes (Claudia) es `es-ES-ElviraNeural` — son independientes. |
| **Estilo visual hooks** | Texto blanco, fontsize 70, borde negro sutil, ancho de línea 15 chars | **NO** amarillo — el usuario odia el estilo amarillo feo |
| **Estilo subtítulos** | Texto blanco, fontsize 38, fondo negro semitransparente, Arial | En tercio inferior (y=1400) |
| **Música** | `generate_unique_music.py` (numpy + pydub) | Música procedural única por video (evita copyright) |
| **Montaje** | FFmpeg + libx264 | Ken Burns zoom, drawtext subtítulos |
| **Publicación** | YouTube Data API v3 (OAuth) | Solo si se pasa `--publish` (antes fallaba con `NoneType` por indentación incorrecta) |
| **Formato** | 1080x1920 vertical (9:16) | Shorts |
| **Duración mínima** | 15 segundos | Relleno negro automático si es menor |

### Archivos Clave

- **`produce_video_STABLE.py`** — Orquestador principal (~17KB). Parcheado: voz Ximena, hook blanco, publicación opcional, duración mínima, soporte `narration_es`.
- **`orchestrate_short.py`** — Orquestador automático. Rotación inteligente de series, mejora de títulos, hooks específicos, CTA al final. Se ejecuta vía cron cada 2h. **Timeout del scheduler: 900s** (ver sección bugs).
- **`run_short_cron.sh`** — Script wrapper en `~/.hermes/scripts/` que llama al orquestador.
- **`published_history.json`** — Historial de scripts publicados (evita repetir).
- **`daily_summary.py`** / **`daily_summary.sh`** — Resumen diario de publicaciones a Telegram.
- **`hook_generator_v14_6.py`** — Obsoleto. La mejora de hooks la hace `orchestrate_short.py`.
- **`generate_unique_music.py`** — Música procedural única (numpy + pydub)
- **`generate_images_comfy.py`** — Alternativa ComfyUI para Mac Studios
- **`publish_to_youtube.py`** — Subida a YouTube (OAuth, YouTube Data API v3)
- **`VIDEO_PRODUCER_GUIDE.md`** — Documentación completa del pipeline
- **📄 `references/pipeline-bugs-and-fixes.md`** — Bugs y fixes aplicados
- **📄 `references/orchestrator-automatico.md`** — Documentación del sistema automático de publicación (rotación, hooks, CTA, filtrado)
- **📄 `references/multi-canal-youtube.md`** — Publicar en canales diferentes (Time Gazer vs NR Music Pop): tokens separados, OAuth por canal, pitfalls
- **📄 `templates/auth_youtube_channel.py`** — Template reusable para autenticar OAuth a cualquier canal de YouTube (copiar, cambiar `CHANNEL_NAME`, ejecutar)

### Configuración del Script (Actualizada)

```python
CONFIG = {
    "ovh_url": "https://oai.endpoints.kepler.ai.cloud.ovh.net/v1/images/generations",
    "ovh_token": "[hardcodeado]",
    "voice": "es-ES-XimenaNeural",  # CAMBIADO: antes era en-US-GuyNeural
    "nas_ready_dir": "/home/nramos/.openclaw/workspace/.mnt/youtube"
}
```

**⚠️ Lecciones aprendidas (¡no repetir!):**
- **Voz**: NUNCA usar voz inglesa. Forzar `es-ES-XimenaNeural`.
- **Hooks**: NUNCA amarillo. Blanco con borde (`fontcolor=white:borderw=3:bordercolor=black@0.8`).
- **Hooks específicos**: No plantillas genéricas `SECRET\\nENIGMA`. Deben referirse al tema concreto.
- **Publicación**: `--publish` debe ser opcional — si no se pasa, solo generar MP4. El código de publicación debe estar fuera del `else:` de validación de duración mínima (ese bug causaba `NoneType` error).
- **Pipeline**: Ejecutado por Hermes, NO por OpenClaw. OpenClaw se elimina, los scripts Python se llaman directamente desde terminal/shell de Hermes.
- **Scripts inválidos**: Muchos scripts JSON en `scripts/` tienen escenas sin `visual_prompt`/`prompt`/`image_prompt` (solo `search_terms`, `visual_keywords`, `visual_description`). El orquestador los filtra automáticamente — solo usa ~378 de ~467 totales. No intentar ejecutarlos directamente.
- **Claves de texto**: El productor soporta múltiples claves de narración: `script`, `narration`, `narration_es`, `text`, `content`. Si un script usa `narration_es` con `prompt` como clave visual, funciona. Si usa `search_terms` sin prompt, FALLA.

**⚠️ Estado de proveedores de imagen (2026-05-28) — ACTUALIZAR REGULARMENTE:**
- **MiniMax Image** (`image-01`): ✅ FUNCIONA — respuesta en ~1s, usar como PRIMARY
- **OVH SDXL** (Kepler): ❌ 403 Forbidden — token caducado, necesita renewal en https://kepler.ai.cloud.ovh.net/v1/oauth/ovh/authorize
- **DALL-E 3**: ❌ "model 'dall-e-3' does not exist" — clave OpenAI actual no tiene acceso a DALL-E 3
- **ComfyUI local** (Mac Studio 192.168.1.5, Mac Mini 192.168.1.17): ❌ Servicios no corriendo actualmente
- **Verificar servicios antes de producción**: `nc -z -w3 192.168.1.5 8890` (ComfyUI Mac Studio), `nc -z -w3 192.168.1.17 8181` (ComfyUI Mac Mini)
- **Si todos fallan**: hacer fallback a MiniMax Image API para generación de imágenes

**⚠️ Bug conocido (FIXED 2026-05-22):** La validación de escenas en `produce_video_STABLE.py` línea 139 originalmente solo buscaba las keys `text`, `narration`, `script`, `content`. Muchos scripts del orquestador usan `narration_es` como clave de texto, lo que provocaba el error "⚠️ Falta texto en escena(s)". Se añadió `narration_es` a la lista de keys validadas.

**⚠️ Bug conocido (FIXED 2026-05-24):** Path de ffmpeg/ffprobe hardcodeado como `/usr/bin/ffmpeg` — en macOS con Homebrew está en `/opt/homebrew/bin/ffmpeg`. El cron se ejecuta con PATH mínimo (`/usr/bin:/bin`) y no lo encuentra, provocando `FileNotFoundError: [Errno 2] No such file or directory: '/usr/bin/ffmpeg'`. El orquestador (orchestrate_short.py) genera el script procesado ok, pero produce_video_STABLE.py petaba en la línea 265 (zoompan). **Fix:** usar `import shutil; FFMPEG = shutil.which("ffmpeg") or "/opt/homebrew/bin/ffmpeg"`. **NUNCA hardcodear paths de binarios del sistema** — usar `shutil.which()` con fallback. Aplica también a cualquier script que invoque herramientas externas (ffmpeg, edge-tts, python).

**⚠️ Bug conocido (FIXED 2026-05-28):** El scheduler de Hermes cron tiene un timeout por defecto de 120s para scripts `no-agent`. El script `orchestrate_short.py` necesita ~5-8 minutos para generar un short completo (generación de imágenes + TTS + vídeo). Si el cron timeout es < 300s, el script morirá con `"Script timed out after 120s"`.

**Fix obligatorio:** Añadir `script_timeout_seconds: 900` (15 min) en la sección `cron:` del `config.yaml`:
```yaml
cron:
  wrap_response: true
  max_parallel_jobs: null
  script_timeout_seconds: 900
```
Sin esto, el cron de short cada 2h falla sistemáticamente.

**⚠️ Bug conocido (FIXED 2026-05-24):** El Homebrew `ffmpeg` estándar se compila **sin el filtro `drawtext`** (falta `--enable-libfreetype` y `--enable-libfontconfig`). El script usa `drawtext` para overlays de texto (subtítulos y hooks) — sin él, `ffmpeg` devuelve error `No such filter: 'drawtext'` y todo el pipeline casca. **Fix:** Instalar `ffmpeg-full` via Homebrew (`brew install ffmpeg-full`) que SÍ incluye drawtext. Forzar uso de `/opt/homebrew/opt/ffmpeg-full/bin/ffmpeg` en el script. **NO** usar `font='Arial'` como parámetro del filtro drawtext — hay que pasar `fontfile=/System/Library/Fonts/Supplemental/Arial.ttf` con la ruta absoluta al .ttf (sin `:fontname=` porque esa opción no existe en esta versión de ffmpeg). El filtro drawtext está en `--enable-libfreetype`, que viene incluido solo en `ffmpeg-full`.

### Estructura del JSON de Entrada

```json
{
  "title": "Título del Short (YouTube)",
  "hook_text": "Texto gancho primera escena (blanco grande, NUNCA amarillo)",
  "theme": "history|mystery|news",
  "music_style": "epic|dark|news",
  "scenes": [
    {
      "text": "Texto narración para esta escena",
      "visual_prompt": "Prompt para DALL-E / SDXL",
      "i": 1
    }
  ]
}
```

### Ejecución

```bash
python3 produce_video_STABLE.py /ruta/al/guion.json --publish 27
# 27 = categoría Educación en YouTube
```

---

## Estrategia de Contenido para The Time Gazer

### 1. Nicho y Marca

**Propuesta:** Canal de **"Historia Oscura"** — especializado en:
- Historias de la historia que parecen ficción
- Misterios históricos resueltos y sin resolver
- Crímenes históricos con contexto
- Personajes históricos olvidados

**Diferenciación:** NO es historia genérica. Es **historia contada como true crime** — ritmo, emoción, investigación.

### 2. Estructura de Shorts que Funcionan

#### A. Formato "3 Datos Impactantes"
- Escena 1: Hook visual + "3 datos que NO sabías sobre X"
- Escena 2-4: Un dato cada una
- Escena 5: Cierre con pregunta

#### B. Formato "El Misterio en 30s"
- Escena 1: Hook + imagen del misterio
- Escena 2-3: Contexto rápido
- Escena 4-5: El giro/clímax
- **Clave: Tiempos muy ajustados (máx 3s por escena)**

### 3. Títulos que Funcionan en Shorts 2026

**NO usar:** `"EL DÍA QUE UN MURO DIVIDIÓ AL MUNDO: La Caída del Muro de Berlín"`
**USAR:**
- `"El muro que partió Alemania 🧱"`
- `"Así cayó Berlín"`
- `"La noche que cambiaron las reglas"`

**Reglas:**
- ❌ **No mayúsculas sostenidas** — tampoco mayúscula parcial (ej: "EL DÍA QUE... El Secreto")
- ❌ No más de 60 caracteres
- ❌ **No dos puntos ni subtítulos** — se elimina todo lo que va tras `:` automáticamente
- ❌ **NO incluir hashtags en el título** — roban espacio del anzuelo, quedan a spam, y los hashtags funcionan igual en la descripción
- ✅ Corto, intrigante, con emoji
- ✅ Primera palabra capitalizada (sentence case, no title case)
- ✅ Máximo 40 caracteres ideal

**Hashtags deben ir SOLO en la descripción** (3-5 máximo). El título es para enganchar, no para etiquetar.

#### ⚠️ Bugs conocidos en `improve_title()` (FIXED 2026-05-22)

La función `improve_title()` en `orchestrate_short.py` tenía 3 bugs que provocaban títulos en mayúsculas rotas como "EL DÍA QUE EL SOL SE APAGÓ Y EL MUNDO SE VOLVIÓ OSCURO: El Secreto del Año 536":

1. **`isupper()` no detecta títulos mixtos** — detecta si TODOS los caracteres son mayúsculas (100%). Un título como "EL DÍA QUE... El Secreto" tiene mezcla → `isupper()` devuelve `False` → no se transforma nada. **Fix:** detectar por ratio de caracteres mayúsculas > 50% en vez de `isupper()`.

2. **`.title()` capitaliza cada palabra** — daba resultados horribles tipo "El Día Que El Sol Se Apagó". **Fix:** convertir a sentence case (solo primera letra de cada oración capitalizada), y restaurar nombres propios que estaban en FULL UPPER en el original (ej: COLÓN → Colón).

3. **No limpiaba subtítulos tras dos puntos** — títulos como "TEMA PRINCIPAL: Subtítulo débil" se quedaban con la parte redundante. **Fix:** `t = re.sub(r':\s*.*$', '', t)` elimina todo tras `:`.

**Importante:** Si aparece otro título mal formateado en producción, revisar `improve_title()` primero — el fix está en el orquestador, no en los scripts JSON fuente.

### 4. Hooks Visuales (Primeros 2 Segundos)

**Actual (problema):** El hook generado por `hook_generator_v14_6.py` usa plantillas como `"SECRET\\nENIGMA"` que:
- Son genéricas y no enganchan
- No comunican el tema específico del short
- Se ven iguales en todos los vídeos

**Mejora:** Hooks deben ser:
1. **Específicos del contenido** — ej: `"LA VERDAD\\nSOBRE X"` en vez de `"SECRET\\nENIGMA"`
2. **Visualmente impactantes** — la primera imagen debe ser la más potente de todo el short
3. **Con texto grande y legible** — usar yellow hook pero con contenido real

### 5. Series/Sagas Semanales

Crear **series identificables** que el usuario pueda seguir:

| Serie | Temática | Frecuencia |
|:--|:--|:--|
| **🔍 Expediente Abierto** | Crímenes sin resolver | Lunes |
| **🏛️ CIVILIZACIONES** | Imperios antiguos | Miércoles |
| **👻 HISTORIA FANTASMA** | Misterios inexplicados | Viernes |
| **⚔️ BATALLAS ÉPICAS** | Conflictos históricos | Sábado |

### 6. Mejoras Técnicas en VideoProducer

#### a) Hooks Dinámicos

Modificar `hook_generator_v14_6.py` para que genere hooks **específicos del tema** en lugar de plantillas fijas. Ej: si el tema es "Stonehenge", hook = `"STONEHENGE\\nENIGMA"`.

#### b) Transiciones Más Rápidas

En `produce_video_STABLE.py`, línea 261 (zoompan):
```python
# Actual: zoom lento 0.0005 por frame
# Mejora para 2026: zoom más dinámico 
zoom_speed = 0.001 if dur < 3 else 0.0005
vf_zoom = f"scale=1920:1920,crop=1080:1920,zoompan=z='min(zoom+{zoom_speed},1.3)':d={frames}:s=1080x1920:fps=30"
```

#### c) Duración de Escenas Más Ágil
En Shorts 2026, **las escenas deben durar 2-3 segundos máximo**. Ajustar el TTS a textos más cortos por escena (15-20 palabras máximo).

#### d) Engagement (CTA al final)
Añadir una escena final con:
- Texto tipo `"¿QUÉ OPINAS? 👇"`
- O `"SÍGUENOS PARA MÁS HISTORIAS"`
- Esto incrementa comentarios → mejora algoritmo

### 7. Frecuencia de Publicación (ACTUAL)

**Sistema automático:** 1 short cada 2 horas vía cron job → **~12 shorts/día** (~84/semana).

Esto es intencionado para la fase de **recuperación del algoritmo**: inundar el feed con contenido de calidad consistente durante 7-14 días, y luego ajustar según rendimiento.

| Objetivo | Shorts/día | Total/semana |
|:--|:--:|:--:|
| **Recuperación (actual)** | ~12 | ~84 |
| **Mantenimiento** | 1 | 7 |
| **Crecimiento** | 2-3 | 14-21 |

**⚠️ Importante:** El cron cada 2h está en modo silencioso (no genera notificaciones a Telegram). Solo llega un resumen diario a las 08:00 Canarias.

### 8. SEO para Shorts

| Elemento | Práctica |
|:--|:--|
| **Título** | < 60 chars, sin mayúsculas, emoji |
| **Descripción** | 2-3 líneas con palabras clave + "#shorts #historia #misterio" |
| **Tags** | `shorts, historia, misterio, [tópico específico]` |
| **Hashtag en video** | `#shorts` (opcional, ya no es obligatorio) |

### 9. Reactivación del Canal

**Pasos concretos para los primeros 30 días:**

1. **Día 1-7:** Publicar 1 short/día de la serie "Expediente Abierto" (la que mejor rendimiento tenía)
2. **Revisar analytics** a los 7 días: qué shorts tienen más retención
3. **Día 8-14:** Doblar down en lo que funciona + añadir segunda serie
4. **Observar** si el algoritmo empieza a recomendar otra vez (indicador: >500 views en menos de 24h)
5. **Mantener ritmo** y empezar a fijar shorts a la sección "Series"

### 10. Publicación Multi-Canal

Nico tiene **5 canales de YouTube** en cuentas diferentes:

| Canal | Token | Cron | Estado |
|:--|:--|:--|:--|
| The Time Gazer | `youtube_token.json` | ✅ 2h (short) + 7h (resumen) | Short con error (Step 4 HTTP 403); daily OK |
| NR Music Pop | `youtube_token_nrpop.json` | ❌ pausado | Sin token OAuth |
| NR Music Rock | `youtube_token_nrrock.json` | ❌ pausado | Token OK; cuota API agotada |
| NR Music Hip-Hop | `youtube_token_nrhiphop.json` | ❌ pausado | Token OK; cuota API agotada |
| NR Music Latino | `youtube_token_nrlatino.json` | ❌ pausado | Sin token OAuth |

**Arquitectura v2.1 (2025-05-27):** Todos los canales de música actualizados a orquestadores v2 con:
- Marca de agua dinámica por canal (ej: "NR Music Rock", "NR Music Latino")
- Emojis Unicode (♫ ♪ ★ ⚡) en lugar de emojis multibyte que FFmpeg no renderiza
- Módulo `render_v2.py` compartido importado por todos los orquestadores
- Scripts de cron que exportan explícitamente `MINIMAX_API_KEY` antes de ejecutar

### Reglas de Oro

1. **Cada canal necesita su propio token OAuth.** El scope `youtube.upload` sube al canal por defecto de la cuenta autenticada — no se puede elegir canal vía API sin re-autenticar.
2. **Si subes al canal equivocado**, el usuario lo pone en privado manualmente. No hay API para mover videos entre canales — hay que re-subir.
3. **Para autenticar un canal nuevo:** ejecutar `/tmp/oauth_youtube_channel.py <nombre_canal>` (imprime URL → usuario abre en Safari → callback automático). Ver `references/multi-canal-youtube.md` para técnicas avanzadas (OOB vs localhost, pitfalls de PKCE, verificación de tokens).

### ⚠️ Pitfall Crítico
No asumir que el token actual (`youtube_token.json`) corresponde al canal que el usuario quiere. Siempre verificar. Si hay ambigüedad (Time Gazer vs NR Music Pop), preguntar.

### ✅ Verificación Post-Upload (OBLIGATORIA)

Siempre verificar el canal del vídeo publicado ANTES de reportar éxito al usuario. Método infalible:

```bash
# Verificar canal de cualquier vídeo YouTube
curl -s "https://www.youtube.com/oembed?url=<URL_DEL_VIDEO>&format=json" | python3 -c "
import json, sys
d = json.load(sys.stdin)
print('Canal:', d['author_name'])
print('Título:', d['title'])
"
```

Esto devuelve `author_name` con el nombre exacto del canal (ej: "NR Music Pop", "The Time Gazer"). Si no coincide con el canal esperado:
1. Poner el vídeo en privado inmediatamente
2. Informar al usuario con transparencia
3. No hay API para mover vídeos entre canales — hay que re-subir con el token correcto

**Secuencia correcta:**
1. Subir con `--publish`
2. Capturar URL del vídeo devuelta
3. Verificar con oEmbed que `author_name` = canal esperado
4. Solo entonces reportar éxito al usuario

---

## 11. Cambios en el Pipeline

**Prioridad alta:**
1. ✅ Actualizar `produce_video_STABLE.py` — voz a `es-ES-XimenaNeural` (HECHO)
2. ✅ Estilo visual hooks: blanco con borde, NO amarillo (HECHO)
3. ✅ Publicación opcional (solo si `--publish`, no falla con None) (HECHO)
4. ✅ Migrar pipeline OpenClaw → `~/.hermes/video-producer/` (HECHO)
1. ✅ Mejorar `hook_generator_v14_6.py` — hooks específicos por tema (INCORPORADO EN ORQUESTADOR)
2. ✅ Añadir CTA final en todas las escenas de cierre (INCORPORADO EN ORQUESTADOR)
3. ✅ Orquestador automático cada 2h (orchestrate_short.py + cronjob)

**Prioridad media:**
1. Títulos dinámicos (no hardcodeados en JSON)
2. Zoom dinámico según duración de escena
3. Música que cambie de intensidad según la escena

**Prioridad baja (futuro):**
1. Instalar ComfyUI en Mac Studios (192.168.1.4/5) para imágenes locales gratis
2. Analítica: capturar views después de publicar
3. Colas de publicación inteligentes (mejor hora del día)

---

## Referencias

**Pipeline fuente:** `~/.hermes/video-producer/`
- **Script principal:** `produce_video_STABLE.py`
- **Generador hooks:** `hook_generator_v14_6.py`
- **Canal:** https://www.youtube.com/@TheTimeGazer
