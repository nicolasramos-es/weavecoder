#!/bin/bash
# run_nrmusic_upload.sh - Sube vídeos pendientes de canales NR Music via Chrome CDP
# Usa perfil persistente en ~/.hermes/chrome-cdp-profile/ (la sesión se guarda)

PROFILE_DIR="/Users/nramos/.hermes/chrome-cdp-profile"
CDP_PORT=9222

{
echo "[$(date)] === INICIO SUBIDA ==="

# 1. Verificar/Asegurar Chrome CDP con perfil persistente
CDP_OK=$(python3 -c "
import urllib.request
try:
    r = urllib.request.urlopen('http://localhost:$CDP_PORT/json/version', timeout=3)
    import json; d = json.loads(r.read().decode())
    print('OK:' + d.get('Browser', '?'))
except:
    print('KO')
" 2>/dev/null)

if echo "$CDP_OK" | grep -q "^KO"; then
    echo "🔄 Chrome CDP no detectado, lanzando con perfil persistente..."
    
    # Kill any stale Chrome for this profile
    pkill -f "$PROFILE_DIR" 2>/dev/null || true
    sleep 2
    
    open -na /Applications/Google\ Chrome.app --args \
        --remote-debugging-port=$CDP_PORT \
        --no-first-run \
        --no-default-browser-check \
        --user-data-dir="$PROFILE_DIR" \
        "https://studio.youtube.com"
    
    # Esperar hasta 30s a que CDP responda
    for i in $(seq 1 6); do
        sleep 5
        CDP_OK2=$(python3 -c "
import urllib.request
try:
    r = urllib.request.urlopen('http://localhost:$CDP_PORT/json/version', timeout=3)
    print('OK')
except:
    print('KO')
" 2>/dev/null)
        if [ "$CDP_OK2" = "OK" ]; then
            echo "✅ CDP listo (intento ${i})"
            break
        fi
        echo "⏳ Esperando CDP (${i}/6)..."
    done
fi

# 2. Verificar que CDP está disponible
CDP_FINAL=$(python3 -c "
import urllib.request
try:
    r = urllib.request.urlopen('http://localhost:$CDP_PORT/json/version', timeout=3)
    print('OK')
except:
    print('KO')
" 2>/dev/null)

if [ "$CDP_FINAL" != "OK" ]; then
    echo "❌ No se pudo iniciar Chrome CDP"
    exit 1
fi

# 3. Subir vídeos pendientes
CHANNEL=$1
echo "📤 Canal: ${CHANNEL:-todos}"
cd /Users/nramos/.hermes/video-producer

if [ -n "$CHANNEL" ]; then
    python3 upload_cdp_music.py "$CHANNEL"
else
    python3 upload_cdp_music.py all
fi

echo "[$(date)] === FIN SUBIDA ==="
} >> /tmp/nrmusic_upload.log 2>&1