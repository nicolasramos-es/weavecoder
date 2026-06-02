# Pipeline Bugs & Fixes Aplicados

## 1. Voz en Spanglish (`en-US-GuyNeural`)
- **Bug:** CONFIG tenía `"voice": "en-US-GuyNeural"` — narración en inglés americano
- **Fix:** Cambiar a `"voice": "es-ES-XimenaNeural"`
- **Archivo:** `produce_video_STABLE.py` línea 27

## 2. Hook Amarillo Feo
- **Bug:** `fontcolor=yellow:fontsize=80:y=400` — el usuario odia el amarillo
- **Fix:** Blanco con borde: `fontcolor=white:fontsize=70:y=350:borderw=3:bordercolor=black@0.8`
- **Ancho de línea:** reducido de 20 a 15 caracteres (hooks más cortos)
- **Subtítulos:** fontsize 38 en vez de 44, opacidad 0.6 en vez de 0.7

## 3. Publicación falla si no se pasa `--publish`
- **Bug:** `args.publish` es `None` → `"--category", None` → `TypeError: expected str`
- **Causa raíz:** El código de publicación estaba dentro del `else:` del bloque de validación de duración mínima, con indentación incorrecta
- **Fix:** Mover todo el bloque de título + publicación fuera de los condicionales de duración. Envolver la publicación en `if args.publish:`

## 4. Imágenes: OVH SDXL (no DALL-E 3)
- **Realidad:** No hay `OPENAI_API_KEY` definida en el entorno
- **Pipeline real:** `call_ovh()` intenta DALL-E 3 primero (falla por falta de API key), fallback automático a OVH SDXL
- **Endpoint:** `https://oai.endpoints.kepler.ai.cloud.ovh.net/v1/images/generations`
- **Modelo:** `stabilityai/stable-diffusion-xl-base-1.0`
- **Config:** 1024x1024, 30 steps, guidance 7.5
- **Estado:** Gratis — el usuario genera ~144 imágenes/día (12 shorts × 5 escenas + extras)

## 5. ComfyUI Local (alternativa futura)
- **Mac Studios disponibles:** 192.168.1.4 (M1 Ultra), 192.168.1.5, 192.168.1.6
- **Clientes ya escritos:** `comfyui_mac_client.py` (ip .5), `comfyui_client.py` (ip .4)
- **Workflow optimizado:** `comfyui_workflows/shorts_optimized_macstudio.json` (SDXL, 576x1024 9:16, 25 steps dpmpp_2m karras)
- **Estado:** ComfyUI no corriendo actualmente. Pendiente de instalar/configurar.

## 7. Validación de texto: falta `narration_es` en lista de claves válidas
- **Bug:** El orquestador selecciona un script, lo pasa al productor, pero `produce_video_STABLE.py` aborta con `⚠️ Falta texto en escena(s): [1,2,3,4,5]. Abortando generación.`
- **Causa raíz:** Muchos scripts JSON guardan la narración bajo la clave `narration_es` (no `text` o `narration`). La validación en línea 139 solo buscaba `['text', 'narration', 'script', 'content']`.
- **Fix:** Añadir `narration_es` a la lista: `['text', 'narration', 'script', 'content', 'narration_es']`
- **Archivo:** `produce_video_STABLE.py` línea 139
- **Fecha:** 2026-05-22
- **Verificación:** Tras el fix, el orquestador produjo y publicó un short de Cristóbal Colón 1492 en YouTube: https://www.youtube.com/watch?v=7f177niFoZ4 (40.49s, categoría Educación, serie history)

## 8. Ejecución: Hermes, no OpenClaw (HECHO ✅)
- OpenClaw (`/home/nramos/openclaw-main 2`) eliminado del ecosistema
- Scripts migrados a `~/.hermes/video-producer/` (independiente, portátil a futuro servidor)
- TODAS las rutas internas actualizadas (`MUSIC_DIR`, `nas_ready_dir`, `cwd`, `publish_script`)
- El usuario confirmó que OpenClaw desaparece — Hermes orquesta todo
- **Cuando se cambie de servidor, solo hay que mover `~/.hermes/video-producer/`**

## 9. `improve_title()` fallaba con títulos mixtos mayús/minús
- **Bug:** La función usaba `t.isupper()` para detectar mayúsculas, pero muchos scripts JSON legacy tienen títulos como `\"EL DÍA QUE EL SOL SE APAGÓ... El Secreto del Año 536\"` — mezcla de mayúsculas sostenidas con minúsculas. `isupper()` devuelve `False`, así que el título no se transformaba.
- **Fix (2026-05-23):** Reemplazar `isupper()` por detección por ratio de caracteres mayúsculas >50%:
  1. Convertir a sentence case con `t.lower()` + capitalizar primera letra
  2. Restaurar nombres propios FULL UPPER del original (COLÓN → Colón)
  3. Quitar subtítulo tras `:` (parte débil del título automáticamente)
  4. Recortar a 55 caracteres máx (antes del emoji)
- **Archivo:** `orchestrate_short.py` función `improve_title()`
- **Ejemplo:** `\"EL DÍA QUE EL SOL SE APAGÓ Y EL MUNDO SE VOLVIÓ OSCURO: El Secreto del Año 536\"` → `\"Día que el sol se apagó y el mundo se volvió oscuro 🏛️\"`

## 10. Path de ffmpeg hardcodeado — cron falla por PATH mínimo
- **Bug:** `produce_video_STABLE.py` línea 23 tenía `FFMPEG = \"/usr/bin/ffmpeg\"`. En macOS con Homebrew, ffmpeg está en `/opt/homebrew/bin/ffmpeg`. Cuando el cronjob ejecuta el script, el PATH heredado es `/usr/bin:/bin`, no encuentra ffmpeg, y toda la producción falla con `FileNotFoundError: [Errno 2] No such file or directory: '/usr/bin/ffmpeg'`
- **Síntoma:** El orquestador (`orchestrate_short.py`) funciona perfectamente (genera `_processed_*.json` en `scripts/`), pero `produce_video_STABLE.py` petaba en la línea 265 (zoompan) al intentar llamar a ffmpeg para la primera escena
- **Causa adicional:** Aunque se ponga el PATH correcto en `run_short_cron.sh`, el cron de Hermes ejecuta el script directamente, no a través de bash login, así que no carga ~/.zshrc ni el PATH del usuario
- **Fix:** `import shutil; FFMPEG = shutil.which(\"ffmpeg\") or \"/opt/homebrew/bin/ffmpeg\"` — resuelve el binario en tiempo de ejecución con fallback a Homebrew
- **Archivo:** `produce_video_STABLE.py` línea 22-24
- **Principio:** NUNCA hardcodear paths absolutos de binarios del sistema. Usar `shutil.which()` con fallback al path típico de Homebrew. Aplica también a ffprobe, edge-tts, python, y cualquier otra herramienta externa.
- **Fecha:** 2026-05-24

## 11. Homebrew ffmpeg no tiene filtro `drawtext`
- **Bug:** El ffmpeg estándar de Homebrew se compila SIN `--enable-libfreetype` ni `--enable-libfontconfig`, por lo que el filtro `drawtext` no existe. El script usa drawtext para overlays de texto (subtítulos + hooks). Error: `No such filter: 'drawtext'`
- **Fix:** Instalar `brew install ffmpeg-full` que SÍ incluye drawtext. Forzar uso de `/opt/homebrew/opt/ffmpeg-full/bin/ffmpeg` en el script.
- **Fix adicional de sintaxis drawtext:** `font='Arial'` NO funciona con este ffmpeg. Usar `fontfile=/System/Library/Fonts/Supplemental/Arial.ttf` (ruta absoluta al .ttf) y **NO** incluir `:fontname=...` porque esa opción no existe en esta versión.
- **Archivo:** `produce_video_STABLE.py` líneas 22-28 (FFMPEG/FFPROBE) y líneas 273-280 (draw_narration/draw_hook)
- **Fecha:** 2026-05-24
- **Verificación:** `ffmpeg -filters | grep drawtext` debe devolver algo. Si no devuelve nada, el ffmpeg instalado no sirve para este pipeline.

## 12. Directorio `shorts_history.md` no existía — producción abortaba
- **Bug:** `produce_video_STABLE.py` intenta escribir en `~/.openclaw/workspace/memory/shorts_history.md` (append mode) cuando falla la validación de escenas. Pero el directorio `memory/` no existe → `FileNotFoundError`.
- **Síntoma:** El orquestador seleccionaba script y generaba `_processed_*.json` bien, pero `produce_video_STABLE.py` abortaba antes de producir nada.
- **Fix:** Añadir `os.makedirs(os.path.dirname(hist_path), exist_ok=True)` justo antes del `open()`.
- **Archivo:** `produce_video_STABLE.py` línea 164
- **Fecha:** 2026-05-28

## 13. DALL-E 3 — NUNCA se usó; OVH funciona perfectamente
- **Confirmado por el usuario:** "nunca hemos usado Dalí. Usamos OVH y el token funciona perfectamente, no está caducado."
- **Realidad del pipeline:**
  1. El código intenta DALL-E 3 primero (modelo `dall-e-3`) → falla con `"model 'dall-e-3' does not exist"` (la clave no tiene acceso a DALL-E 3)
  2. Hace fallback a OVH SDXL → funciona, respuesta en ~10s, formato `data[0].b64_json`
- **OVH funciona porque el formato de llamada es correcto** (verificado 2026-05-28)
- **No cambiar nada de OVH** — funciona bien
- **DALL-E 3 no es relevante** — no intentar integrarlo de nuevo

## 14. Scripts con estructura diferente — escenas sin `text`/`narration`
- **Bug:** `expediente_hoffa_v14.json` y otros scripts tienen escenas con solo `['scene_number', 'duration_sec', 'style', 'prompt', 'caption']` — sin `text` ni `narration`.
- **El orquestador** (`orchestrate_short.py`) los marca como inválidos si falta `visual_prompt`/`prompt`/`image_prompt`/`visual`, pero NO valida que haya texto de narración.
- **El productor** (`produce_video_STABLE.py`) aborta en validación de texto si alguna escena no tiene `text`/`narration`/`narration_es`/`script`/`content`.
- **Impacto:** Scripts inválidos por falta de texto se rechazan en producción. El cron avanza al siguiente script correctamente — no hay loop, solo retraso.
- **Comportamiento actual:** robusto. El orquestador sigue funcionando, solo se saltan los scripts sin texto.

## Estado de Providers de Imagen (2026-05-28 — VERIFICADO HOY)

| Provider | Estado | Formato respuesta | Notas |
|:--|:--|:--|:--|
| **OVH SDXL** (Kepler) | ✅ FUNCIONA | `data[0].b64_json` | Token confirmado válido. 1024x1024, ~10s. |
| **MiniMax Image** (`image-01`) | ✅ FUNCIONA | `image_urls[]` | ~1s respuesta. 100 usos/día compartidos. |
| **DALL-E 3** | ❌ No acceso | — | La clave no tiene este modelo. No relevante. |
| **ComfyUI local** (Mac Studio/Mac Mini) | ❌ No corriendo | — | Servicios apagados. |

**Recomendación actual:** OVH como primario (funciona, no usa cuota de otros servicios). MiniMax Image como fallback rápido si OVH falla.