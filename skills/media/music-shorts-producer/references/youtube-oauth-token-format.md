# YouTube OAuth Multi-Canal — Token Format Requirements

Captured from session 2026-05-28 — Nico's NR Music channels debugging.

## El problema: tokens faltando campos obligatorios

Los archivos `youtube_token_<canal>.json` en `~/.openclaw/workspace/.credentials/` son usados por la Google OAuth client's `refresh_from()` para hacer refresh del access token.

**OAuth token mínimo requerido por google-auth:**
```
access_token          ✅
expires_in            ✅
refresh_token         ✅
token_type            ✅
scope                 ✅
token_uri             ⚠️ faltó en nrrock, nrhiphop
client_id             ⚠️ faltó en nrrock, nrhiphop
client_secret          ⚠️ faltó en nrrock, nrhiphop
```

Sin `token_uri`, `client_id` y `client_secret`, el refresh falla con:
```
"The credentials do not contain the necessary fields need to refresh the access token.
You must specify refresh_token, token_uri, client_id, and client_secret."
```

## Dónde están los tokens

```
~/.openclaw/workspace/.credentials/
  youtube_token_nrpop.json     ← completo ✅
  youtube_token_nrrock.json     ← incompleto ❌ → arreglado 2026-05-28
  youtube_token_nrhiphop.json   ← incompleto ❌ → arreglado 2026-05-28
  youtube_token_nrlatino.json  ← completo ✅
  youtube_token.json            ← genérico, no por canal
```

## Fuente de client_id / client_secret

Todos los canales comparten el mismo OAuth client del proyecto Google Cloud:
- **client_id:** `859593269662-5dcbdum3om1o443eta11jus2opvfnfvo.apps.googleusercontent.com`
- **client_secret:** `GOCSPX-...` (de `google_credentials.json` → `installed.client_secret`)
- **token_uri:** `https://oauth2.googleapis.com/token`

El archivo `~/.openclaw/workspace/.credentials/google_credentials.json` tiene el formato:
```json
{
  "installed": {
    "client_id": "859...apps.googleusercontent.com",
    "client_secret": "GOCSPX-...",
    "token_uri": "https://oauth2.googleapis.com/token",
    "redirect_uris": ["http://localhost:PORT"]
  }
}
```

## Fix rápido si un token falta campos

```python
import json

# Cargar client credentials
with open('/Users/nramos/.openclaw/workspace/.credentials/google_credentials.json') as f:
    gc = json.load(f)
client_id = gc['installed']['client_id']
client_secret = gc['installed']['client_secret']

# Cargar token incompleto
token_path = '/Users/nramos/.openclaw/workspace/.credentials/youtube_token_NRCHANNEL.json'
with open(token_path) as f:
    t = json.load(f)

# Añadir campos faltantes
for field, val in [('client_id', client_id), ('client_secret', client_secret),
                    ('token_uri', 'https://oauth2.googleapis.com/token')]:
    if field not in t:
        t[field] = val

with open(token_path, 'w') as f:
    json.dump(t, f, indent=2)

print('Fixeado:', token_path)
```

## Verificación post-fix

```bash
# Probar que el token se puede refrescar
python3 -c "
import json, os
from google.oauth2.credentials import Credentials

token = json.load(open('/Users/nramos/.openclaw/workspace/.credentials/youtube_token_nrrock.json'))
creds = Credentials.from_authorized_user_info(token)
print('Token válido, expiry:', creds.expiry)
"
```

## Error 429 Rate Limit — no es bug del código

Cuando YouTube responde `429 Quota exceeded for quota metric 'Video Uploads' and limit 'Video Uploads per day'`, el video se generó correctamente y está en disco. El problema es纯粹 cuota de la API de YouTube, no un error de código o de credenciales.

**El video queda guardado localmente** (ej: `short_20260528_122953.mp4`). Se puede:
1. Subir manualmente más tarde
2. Esperar el reset de cuota (UTC midnight)
3. Planificar menos uploads por día si es recurrente