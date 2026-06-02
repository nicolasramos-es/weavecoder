# Persistent Chrome CDP Profile — Setup Guide

## Why

Los crons de NR Music (cada 2h) necesitan un Chrome con CDP y sesión de YouTube iniciada para subir vídeos automáticamente. Si se usa un perfil en `/tmp/`, la sesión se pierde al reiniciar.

## Solution

Perfil persistente en `~/.hermes/chrome-cdp-profile/`.

### First-time setup

1. Lanzar Chrome con el perfil:
```bash
open -na /Applications/Google\ Chrome.app --args \
  --remote-debugging-port=9222 \
  --no-first-run --no-default-browser-check \
  --user-data-dir=$HOME/.hermes/chrome-cdp-profile \
  "https://studio.youtube.com"
```

2. Iniciar sesión en YouTube Studio con la cuenta que tiene acceso de editor a los 4 canales NR Music.

3. Verificar que Playwright puede conectarse:
```bash
python3 -c "
import urllib.request, json
r = urllib.request.urlopen('http://localhost:9222/json/version')
print(json.loads(r.read())['Browser'])
"
```

### Cron wrapper (`run_nrmusic_upload.sh`)

El script en `~/.hermes/scripts/run_nrmusic_upload.sh` hace:
1. Comprueba si Chrome CDP está accesible en puerto 9222
2. Si no: lanza Chrome con `--user-data-dir=$HOME/.hermes/chrome-cdp-profile` y espera hasta 30s
3. Ejecuta `upload_cdp_music.py` para el canal solicitado

### Location

```
Perfil real:    ~/.hermes/chrome-cdp-profile/
Script upload:  ~/.hermes/video-producer/upload_cdp_music.py
Wrapper cron:   ~/.hermes/scripts/run_nrmusic_upload.sh
Logs:           /tmp/nrmusic_upload.log
```

### ⚠️ Important

- NO usar `rm -rf ~/.hermes/chrome-cdp-profile` — es el perfil con las cookies de YouTube
- Si la sesión expira, abrir Chrome con ese perfil y reloguear manualmente
- El wrapper cron lo lanza automáticamente si no está corriendo
