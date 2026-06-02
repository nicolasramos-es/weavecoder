---
name: music-shorts-producer
description: "Pipeline para generar y publicar shorts de música en YouTube con visualizer de ondas animadas (FFmpeg showwaves/showfreqs) + MiniMax Music API."
tags: [youtube, music, shorts, visualizer, ffmpeg, minimax, nr-music-pop]
related_skills: [the-time-gazer-shorts]
platforms: [macos]
---

# Music Shorts Producer

Pipeline para canales de música en YouTube (ej: NR Music Pop). Genera canciones con MiniMax Music API, renderiza visualizer de ondas animadas, y publica como Short.

## Canales Configurados (v3.0 con Logo)

| Canal | ID Cron | Estilos musicales | Paleta visual | Marca de agua | Logo PNG |
|:--|:--|:--|:--|:--|:--|
| NR Music Pop | `75523a5ec203` | kpop, latin, dance, ballad, electronic | rosa/cyan | **NR Music Pop** | `nr_music_pop_200px.png` |
| NR Music Rock | `4b106c85ffc8` | metal, classic_rock, indie, hard_rock | dark_rock | **NR Music Rock** | `nr_music_rock_200px.png` |
| NR Music Hip-Hop | `c273c84e8426` | trap, boom_bap, drill, rnb | dark_hiphop | **NR Music Hip-Hop** | `nr_music_hiphop_200px.png` |
| NR Music Latino | `64a9a832e9d7` | reggaeton, salsa, bachata, dembow | magenta/cyan | **NR Music Latino** | `nr_music_latino_200px.png` |

**Cambio v3.0:** Logo del canal en esquina superior derecha (200px ancho, 50px desde borde superior) + marca de agua texto en inferior derecha. Renderizador unificado `render_v3_logo.py` usando FFmpeg `overlay` filter.

**Marca de agua dinámica:** Cada canal debe tener su propia marca de agua en esquina inferior derecha. Nunca hardcodear "NR Music Pop" para todos — usar parámetro `watermark` en `render_v2.py`.

**Estilo visualizer unificado**: todos los canales usan **circle** (`showfreqs=mode=bar:fscale=log`) — el más dinámico y atractivo para shorts.

## Diferencia clave vs The Time Gazer

| Aspecto | Time Gazer | NR Music (Pop/Rock/HipHop) |
|:--|:--|:--|
| **Contenido** | Narración histórica con imágenes | Música generada con visualizer |
| **Imágenes** | OVH SDXL (por escena) | Visualizer FFmpeg (ondas animadas) |
| **Audio** | TTS Edge (voz narradora) | MiniMax Music API (canción completa) |
| **Duración** | 15-60s con varias escenas | 30-45s (una canción + visualizer) |
| **Estilo** | Ken Burns + subtítulos | Circle showfreqs + título |
| **Frecuencia cron** | 2h (short) + 7h (resumen) | cada 2h (solo local, sin notificación por canción) |

## Pipeline Completo (Circle - Unificado)

```
1. Decidir estilo/género → rotación inteligente por canal
2. Generar canción → MiniMax Music API (modelo music-2.6)
3. Guardar MP3 en directorio de trabajo
4. Renderizar circle visualizer → FFmpeg showfreqs
   - Paleta oscura según canal (rock/hip-hop) o colorida (pop)
   - Título superpuesto con drawtext
5. Subir a YouTube usando token del canal correcto
6. Registrar en historial de publicaciones
7. Log local (sin notificación por canción en cron)
```

**Nota:** ya no se usa imagen de OHV para el visualizer — todo es FFmpeg puro con showfreqs circle.

## Estilos de Visualizer — Circle (UNIFICADO)

Todos los canales NR Music usan **circle** (`showfreqs=mode=bar:fscale=log`) — estilo de frecuencias centrales, el más dinámico para shorts. Waves y spectrum están deprecados.

### Paletas de Color por Canal

| Canal | bg | wave | accent |
|:--|:--|:--|:--|
| pop (kpop/latin/dance/ballad/electronic) | `0x1a0033` | `0xff1493` | `0x00ffff` |
| rock (metal/classic/indie/hard/alt/grunge) | `0x0a0000` | `0xff4400` | `0x880000` |
| hip-hop (trap/boom_bap/drill/rnb/old/phonk) | `0x0a0000` | `0x9900ff` | `0xff0066` |

### Renderización

```bash
# Un solo estilo circle
python3 render_visualizer.py cancion.mp3 -o salida.mp4 --style circle -p kpop -t "🎵 Título"

# Preview de todos los estilos (solo para desarrollo)
python3 render_visualizer.py cancion.mp3 --preview -t "🎵 Título" -s "Artista"
```

> **⚠️ En producción NO usar --preview** — renders múltiples encarecen el uso de FFmpeg y usan CPU innecesariamente. Solo circle para todos los canales.

## Archivos Clave

- `~/nr-pop/render_visualizer.py` — Renderizador LEGACY (deprecado, usar v2)
- `~/nr-pop/render_v2.py` — **Renderizador v2** (recomendado) con marca de agua dinámica y emojis Unicode
- `~/nr-pop/orchestrate_nrpop_v2.py` — Orquestador v2 con marca de agua dinámica
- `~/nr-pop/published_history_nrpop.json` — Historial de publicaciones
- `~/nr-pop/` — Directorio de trabajo para NR Music Pop

### Arquitectura v2 (Recomendada)

Separar el renderizado en módulo reutilizable que importan los orquestadores:

```
~/nr-pop/render_v2.py              # Módulo compartido de renderizado
~/nr-pop/orchestrate_nrpop_v2.py   # Orquestador específico de Pop
~/nr-rock/orchestrate_nrrock_v2.py # Orquestador específico de Rock (importa render_v2)
~/nr-hiphop/orchestrate_nrhiphop_v2.py
~/nr-latino/orchestrate_nrlatino_v2.py
```

**Ventaja:** Un solo renderizador mantenible, orquestadores simples que solo configuran marca de agua y paleta.

## Tokens OAuth (Multi-Canal)

Cada canal de YouTube necesita su propio token OAuth guardado en `~/.openclaw/workspace/.credentials/`.

| Canal | Path del token | Client ID (compartido) |
|:--|:--|:--|
| NR Music Pop | `youtube_token_nrpop.json` | `859593269662-5dcbdum3om1o...` |
| NR Music Rock | `youtube_token_nrrock.json` | mismo |
| NR Music Hip-Hop | `youtube_token_nrhiphop.json` | mismo |

**⚠️ Pitfall: OAuth codes son single-use** — si el código de autorización se intenta usar dos veces (error de código, reintento, etc.), el segundo intento falla con `invalid_grant`. Siempre generar una URL nueva para cada intento de autorización.

**⚠️ Pitfall: Multi-canal con mismo Client ID** — usar `redirect_uri` con puerto único por canal para evitar conflictos de código:
- NR Pop: puerto 8080
- NR Rock: puerto 8081
- NR Hip-Hop: puerto 9093 (dinámico, evitar conflicto)

**⚠️ Verificación post-upload OBLIGATORIA** — siempre verificar el canal del vídeo publicado ANTES de reportar éxito:

### ✅ Verificación Post-Upload (OBLIGATORIA)

Siempre verificar el canal del vídeo publicado ANTES de reportar éxito. No basta con capturar la URL devuelta — hay que confirmar el `author_name`:

```bash
curl -s "https://www.youtube.com/oembed?url=<URL>&format=json" | python3 -c "
import json, sys
d = json.load(sys.stdin)
print('Canal:', d['author_name'])
print('Título:', d['title'])
"
```

**Secuencia correcta:** subir → capturar URL → verificar oEmbed → solo entonces notificar al usuario. Si el canal no coincide: poner en privado + informar + re-subir con token correcto.

## ⚠️ FFmpeg Pitfalls (macOS con Homebrew)

### Emojis en drawtext

**Problema:** FFmpeg `drawtext` **NO soporta emojis multibyte** (🎵, 🔥, 🌴). Los emojis se muestran como cuadrados □□□.

**Solución:** Usar **caracteres Unicode de un solo byte** que FFmpeg sí renderiza:

| Emoji problema | Reemplazo Unicode | Código |
|:--|:--|:--|
| 🎵 | ♫ Nota musical | `\u266B` o copiar directo |
| 🔥 | ⚡ Rayo | `\u26A1` |
| 🌴 | ★ Estrella | `\u2605` |
| 🎸 | ♪ Nota simple | `\u266A` |
| 💥 | ✦ Asterisco | `\u2726` |

**Ejemplo en orquestador:**
```python
SONG_TEMPLATES = {
    "kpop": [
        {"title": "♫ Fuego en la Pista", "hook": "Baila hasta el amanecer"},
        {"title": "♫ Ritmo del Corazon", "hook": "Late conmigo"}
    ]
}
```

### drawtext requiere fontfile absoluto
3. **filter_complex con overlay + showwaves** — estructura probada:
   ```
   [0:a]showwaves=mode=cline:rate=30:n=40:colors=COLOR1|COLOR2[waves];
   color=c=BG_COLOR:s=1080x1920:r=30[bg];
   [bg][waves]overlay=(W-w)/2:(H-h)/1.5[v]
   ```

## Modo Silencioso para Cronjobs

Los orquestadores deben soportar `--quiet` para cronjobs (sin output a stdout). Patrón implementado:

```python
def log(*args, **kwargs):
    if not getattr(log, 'quiet', False):
        print(*args, **kwargs)

def produce_short(style, title, *, quiet=False):
    log.quiet = quiet
    log("Generando short...")
    # ... todo el proceso ...
```

En `main()`, pasar `--quiet` al argparse y al orquestador.

## Variables de Entorno en Cronjobs

**Problema:** Los cronjobs se ejecutan con un entorno mínimo (`PATH=/usr/bin:/bin`) y sin acceso a las variables de entorno del usuario.

**Solución:** El script wrapper del cron debe cargar explícitamente el archivo `.env`:

```bash
#!/bin/bash
export $(grep -v '^#' /Users/nramos/.hermes/.env | xargs) 2>/dev/null
cd /Users/nramos/nr-latino
python3 orchestrate_nrlatino.py --quiet >> cron.log 2>&1
```

Esto asegura que `MINIMAX_API_KEY` esté disponible para el orquestador.

## YouTube Upload — Playwright + Chrome CDP

**Mejor opción para subidas batch sin límite de API.** Playwright se conecta a Chrome vía Chrome DevTools Protocol (CDP), hereda las cookies, y automatiza YouTube Studio sin que YouTube detecte headless.

Script de referencia: `templates/upload_cdp_music.py`  
Script en producción: `~/.hermes/video-producer/upload_cdp_music.py`

### Perfil persistente (NO usar /tmp/)

La sesión de YouTube debe guardarse en un perfil persistente que sobreviva reinicios:

```bash
# Primera vez: lanzar con perfil nuevo y LOGUEARSE
open -na /Applications/Google\ Chrome.app --args \
  --remote-debugging-port=9222 \
  --no-first-run --no-default-browser-check \
  --user-data-dir=$HOME/.hermes/chrome-cdp-profile \
  "https://studio.youtube.com"

# Las siguientes veces: el wrapper cron lo hace automático
```

**Ruta:** `~/.hermes/chrome-cdp-profile/` (referencia: `references/persistent-cdp-profile.md`)

### Channel IDs (obligatorio por canal)

Cada canal necesita su `channel_id` para navegar directamente al Studio correcto:

```python
# Obtener desde la página del canal:
await page.goto(f"https://www.youtube.com/@NRMusicPop")
await asyncio.sleep(5)
cid = await page.evaluate('() => {
    for (const s of document.querySelectorAll("script")) {
        const m = s.textContent.match(/"channelId":"(UC[\w-]+)"/);
        if (m) return m[1];
    }
}')
```

IDs verificados (ver `references/channel-ids-verified.md`):
- Pop: `UCoMn2PNx_wdhXkazzLeZJ4w`
- Rock: `UCiimUjAf4EgmNaYNwtpTcTw`
- Hip-Hop: `UCvjaeHB6AvJ4HVSjifOb76w`
- Latino: `UC2OvxQ76X6hA5nYV-xZdD0g`

Navegar directamente: `await page.goto(f"https://studio.youtube.com/channel/{channel_id}/videos")`

### Flujo del script

1. Conectar CDP una vez: `playwright.chromium.connect_over_cdp('http://localhost:9222')`
2. Por cada vídeo: `context.new_page()` → navegar al Studio del canal → upload → `page.close()`
3. **NUNCA** cerrar el navegador entre vídeos — `browser.close()` mata la conexión CDP
4. Verificar sesión al inicio: buscar "Acceder" en `body.inner_text()` — si aparece, no se puede continuar

### Pitfalls CDP (ordenados por frecuencia)

1. **"No es para niños" — selector exacto:** El radio button usa `name="VIDEO_MADE_FOR_KIDS_NOT_MFK"` (NO `NOT_MADE_FOR_KIDS`). Selector fiable: `tp-yt-paper-radio-button[name="VIDEO_MADE_FOR_KIDS_NOT_MFK"]`. El texto del label es "No, no está **creado** para niños". **NO** buscar por substring 'no' — encontrará "Sí, está creado para niños" porque contiene "niños".

2. **Scroll para revelar kids section:** La sección de audiencia queda fuera del viewport tras rellenar título+descripción. Hacer `page.evaluate("window.scrollBy(0, 300)")` y esperar 1s antes de buscar.

3. **Botón Siguiente disabled:** Si `get_attribute("disabled")` devuelve verdadero, falta marcar "No es para niños" o falta esperar procesamiento. Esperar 3-5s y reintentar.

4. **"Publicar" vs "Guardar" (timeout 120s):** En visibilidad, el botón dice "Guardar" (disabled) mientras procesa y cambia a "Publicar" (enabled). **Esperar hasta 120s** en bucle (12 intentos × 10s) comprobando ambos.

5. **Límite diario (~15-20/día):** YouTube web (no API) bloquea tras ~20 subidas. Aparece como "Límite diario de subida alcanzado" con botón "Verificar". Verificar permite eliminarlo permanentemente.

6. **Chrome con `open -na` (no `open -a`):** `open -a` recicla una ventana existente sin el flag `--remote-debugging-port`. Siempre usar `open -na`.

### Títulos y descripciones

Formato obligatorio: `"🎸 NR Rock - Nuevo tema 28 de mayo de 2026 🎵"`
- Mes en español en letras (mayo, no 05)
- Descripción con hashtags específicos del canal (#NRRock, #MúsicaRock, #IAMusic)
- Texto plano, sin emojis multibyte si FFmpeg drawtext lo usará después

### ❌ Evitar: computer_use / cua-driver para YouTube Studio

**Demasiado lento.** Cada click requiere captura de pantalla + análisis LLM → un solo upload puede llevar 30-60 minutos y consumir miles de tokens. Usar solo como último recurso si CDP falla.

### ⚠️ YouTube API (cuota limitada ~6/día)

**Límite gratuito por proyecto:** ~6 uploads/día. Usar solo cuando:
- Token OAuth tiene `access_token` válido
- Hay cuota disponible (si 429, esperar ~12h hasta reset UTC)

**Tokens multi-canal:** Cada canal NR Music tiene su token en `~/.openclaw/workspace/.credentials/youtube_token_{canal}.json`.

### ✅ youtube.com/upload (manual)

Abrir `https://www.youtube.com/upload`, arrastrar el vídeo. 2 min por vídeo.

### Flujo de decisión general

1. **Playwright + CDP** (sin límites, script con channel_id)
2. **YouTube API** (solo si hay cuota)
3. **Manual** (`youtube.com/upload`)
4. **cua-driver** (último recurso, muy lento)

---

## Hermes Workspace — Web Multi-Agent UI

Instalado en `~/hermes-workspace/` via `hermes workspace install`. Accesible desde otros equipos en LAN:

```bash
# Arrancar (Gateway + UI):
cd ~/hermes-workspace && pnpm start:all

# Desde otro equipo en la misma red:
http://192.168.1.17:3000
```

Contraseña en `~/.local/share/hermes-workspace/.env.local` (autogenerada 32 chars). Para acceso remoto (fuera de LAN), usar Tailscale.

---

## Sub-Agent Autónomo

Programar con cron job. Frecuencia: cada 2h (cron v3 reanudados 2026-05-31).

**Cron jobs v3 (generar + subir por CDP):**

| Canal | Script | Job ID |
|-------|--------|--------|
| 🎵 NR Pop | `run_nrpop_v3.sh` | `75523a5ec203` |
| 🎸 NR Rock | `run_nrrock_v3.sh` | `4b106c85ffc8` |
| 🎤 NR Hip-Hop | `run_nrhiphop_v3.sh` | `c273c84e8426` |
| 💃 NR Latino | `run_nrlatino_v3.sh` | `64a9a832e9d7` |

Helper: `run_nrmusic_upload.sh` — usa perfil persistente `~/.hermes/chrome-cdp-profile/`, lanza Chrome CDP si no está, sube pendientes. Referencia: `references/persistent-cdp-profile.md`.\n\nScripts en `~/.hermes/scripts/`. Cada v3 ejecuta: orquestador v2 gen short → upload_cdp_music.py sube.

**Referencias:** `references/channel-ids-verified.md` (IDs), `references/cdp-upload-session-2026-05-31.md` (pitfalls CDP).

El orquestador debe:
1. Rotar estilos/géneros (K-pop → Latin → Ballad → Dance → ...)
2. Verificar tokens disponibles antes de empezar
3. No repetir letras o títulos usados recientemente
4. Notificar resultado al canal de Telegram del usuario
5. **Cargar variables de entorno** al inicio del script wrapper

---

## Absorbed Skills — Quick Reference

The following sibling skills have been absorbed into this umbrella. Their content is now sub-sections of this skill or support files under `references/`, `scripts/`, and `templates/`.

### Sub-skill: MiniMax Music API Reference

**Trigger:** User asks about MiniMax music API details, parameters, response shape, or verification.

**Key points** (see `references/verified-prompts.md` for verified prompt/lyrics combos):

- **Endpoint:** `POST https://api.minimax.io/v1/music_generation` with `Authorization: Bearer <token>`
- **Model:** `music-2.6`
- **Required params:** `model`, `prompt`, `lyrics`, `duration` (15-60 int). `lyrics` IS REQUIRED — even for instrumental, pass a placeholder.
- **Response:** Audio in `data.audio` as **hex string** (starts with `4944...` = `ID3` MP3 header). Decode: `bytes.fromhex(audio_hex)`. NOT base64, NOT a URL.
- **Token verification script:** `scripts/verify-token.sh` — run `bash verify-token.sh $MINIMAX_API_KEY`
- **Codes:** 0=success, 1004=auth fail, 2013=lyrics required
- **Quota:** Depends on Token Plan (e.g. 100/day). Check at `https://platform.minimax.io/user-center/payment/token-plan`

**Workflow:**
```python
url = "https://api.minimax.io/v1/music_generation"
data = json.dumps({"model": "music-2.6", "prompt": "...", "lyrics": "[Intro]\n...", "duration": 30}).encode()
req = urllib.request.Request(url, data=data, headers={
    "Authorization": f"Bearer {api_key}", "Content-Type": "application/json"})
with urllib.request.urlopen(req, timeout=120) as resp:
    result = json.loads(resp.read())
audio_bytes = bytes.fromhex(result["data"]["audio"])
```

### Sub-skill: Nico Music Shorts (Legacy Pipeline)

**When to use:** Refer to the 4-step static-thumbnail pipeline below only when the visualizer approach is not suitable (e.g., no ffmpeg-full available).

The older `nico-music-shorts` pipeline used static thumbnail + FFmpeg assembly rather than the animated visualizer. The visualizer (circle showfreqs) is the preferred approach — it's more professional and doesn't consume MiniMax image tokens.

**Legacy static pipeline (3 steps):**
1. Generate song via MiniMax (same as above)
2. Generate thumbnail via MiniMax Image API: `POST https://api.minimax.io/v1/image_generation` (model `image-01`, `image_size`: `"9:16"`, `response_format`: `"url"`)
3. Assemble video:
```bash
ffmpeg -y -loop 1 -i thumbnail.jpg -i cancion.mp3 \
  -vf "scale=1080:1920:force_original_aspect_ratio=decrease,pad=1080:1920:(ow-iw)/2:(oh-ih)/2,setsar=1,format=yuv420p" \
  -c:v libx264 -preset fast -crf 23 -c:a aac -b:a 192k -shortest -movflags +faststart output.mp4
```

**Legacy scripts** (kept for reference): `scripts/generate_music_short.py`, `scripts/render_visualizer.py` — in the scripts/ directory.

### Sub-skill: YouTube Upload Workflow (Manual UI Fallback)

**When to use:** When YouTube API quota is exhausted (429) and you need to upload via the browser UI.

**Fallback sequence (youtube.com/upload):**
1. Open `https://www.youtube.com/upload` in Chrome/Safari
2. **Do NOT** attempt this with computer_use/cua-driver — it's extremely slow and token-expensive
3. The user can drag the video file into the browser window manually (2 min per video, no API limits)
4. For YouTube Studio UI: single-click on filename → single-click "Open" button (never double-click)

**See also:** The "YouTube Upload — Alternatives" section above for full API vs manual decision tree.

---

## Rate Limits de YouTube API

### Timeout insuficiente (causa principal de errores silenciosos)

La generación de música es **asíncrona** — la API tarda 60-90s en responder. Sin `timeout` explícito, `requests.post()` usa el default (~9s) y falla con `ReadTimeout`. **Fix obligatorio:**

```python
r = requests.post(url, headers=headers, json=data, timeout=120)  # ← 120s mínimo
```

### Audio de MiniMax viene en HEX, no en base64

### Audio de MiniMax viene en HEX, no en base64

**Bug encontrado (2026-05-28):** El código usaba `base64.b64decode(audio)` pero MiniMax devuelve el audio como **string de bytes hexadecimales** (`"49443304000000001206545353450000000f0000034c617666..."`).

**Fix correcto:**
```python
# ❌ ANTES (fallaba con "Incorrect padding")
audio = data_obj["audio"]
return base64.b64decode(audio) if isinstance(audio, str) else audio

# ✅ AHORA (funciona)
audio_hex = data_obj["audio"]
return bytes.fromhex(audio_hex) if isinstance(audio_hex, str) else audio_hex
```

**Verificado:** La respuesta de MiniMax Music API es un string hexadecimal puro (no base64). `bytes.fromhex()` la decodifica correctamente a MP3.

**Aplica a:** Los 4 canales NR Music (Pop, Rock, Latino, Hip-Hop). Todos los orquestadores `orchestrate_*_v2.py` fueron parcheados.

### Respuesta asíncrona — polling con job_id

Si MiniMax devuelve `{"data": {"job_id": "..."}}` en vez de audio directo, hacer polling:

```python
if "job_id" in data_obj:
    job_id = data_obj["job_id"]
    for _ in range(20):
        time.sleep(5)
        status_resp = requests.get(
            f"https://api.minimax.io/v1/music_generation/status/{job_id}",
            headers=headers, timeout=30
        )
        status_data = status_resp.json()
        if status_data.get("data", {}).get("status") == 2:
            audio_h = status_data["data"].get("audio", "")
            if audio_h:
                return bytes.fromhex(audio_h)
    raise Exception("Timeout esperando job de música")
```

### API key no disponible en cronjob

Los cronjobs se ejecutan con entorno mínimo. **Fix:** el script wrapper debe cargar `.env`:
```bash
export $(grep -v '^#' ~/.hermes/.env | xargs) 2>/dev/null
```

Esto asegura que `MINIMAX_API_KEY` esté disponible para todos los orquestadores de música.
