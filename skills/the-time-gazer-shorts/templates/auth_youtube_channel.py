#!/usr/bin/env python3
"""
Template: Autenticación OAuth para un canal específico de YouTube.

USO:
  1. Cambia TOKEN_PATH al nombre del canal (ej: youtube_token_nrpop.json)
  2. Ejecuta: python3 auth_youtube_channel.py
  3. Se abre el navegador con el enlace de Google OAuth
  4. ELIGE LA CUENTA CORRECTA (la que gestiona el canal destino)
  5. Pega el código de autorización cuando el script lo pida
  6. El token queda guardado en TOKEN_PATH

El token incluye refresh_token, así que se renueva automáticamente
cuando expira (~1h).
"""
import os, json
from google_auth_oauthlib.flow import InstalledAppFlow
from google.auth.transport.requests import Request
from google.oauth2.credentials import Credentials

SCOPES = ['https://www.googleapis.com/auth/youtube.upload']
BASE_DIR = '/Users/nramos/.openclaw/workspace/.credentials'

# ⚠️ CAMBIAR ESTO: nombre del canal destino
CHANNEL_NAME = 'nrpop'  # ej: nrpop, timegazer, etc.
TOKEN_PATH = os.path.join(BASE_DIR, f'youtube_token_{CHANNEL_NAME}.json')
CLIENT_SECRET = os.path.join(BASE_DIR, 'google_credentials.json')

if not os.path.exists(CLIENT_SECRET):
    print(f"❌ No se encuentra {CLIENT_SECRET}")
    print("   Necesitas un proyecto en Google Cloud Console con la API de YouTube activada")
    sys.exit(1)

creds = None
if os.path.exists(TOKEN_PATH):
    creds = Credentials.from_authorized_user_file(TOKEN_PATH, SCOPES)

if creds and creds.valid:
    print(f"✅ Token para {CHANNEL_NAME} ya es válido.")
elif creds and creds.expired and creds.refresh_token:
    print(f"🔄 Token expirado, refrescando...")
    creds.refresh(Request())
    with open(TOKEN_PATH, 'w') as f:
        f.write(creds.to_json())
    print(f"✅ Token refrescado y guardado en {TOKEN_PATH}")
else:
    print(f"🔑 Autenticación para canal: {CHANNEL_NAME}")
    print(f"   Token se guardará en: {TOKEN_PATH}")
    print()
    print("═══════════════════════════════════════════════════════════════")
    print("  IMPORTANTE: Elige la cuenta de Google que gestiona")
    print(f"  el canal {CHANNEL_NAME.upper()} (no otra cuenta)")
    print("═══════════════════════════════════════════════════════════════")
    print()
    
    flow = InstalledAppFlow.from_client_secrets_file(CLIENT_SECRET, SCOPES)
    flow.redirect_uri = 'urn:ietf:wg:oauth:2.0:oob'
    auth_url, _ = flow.authorization_url(prompt='consent')
    
    print("🌐 Abre este enlace en el navegador:")
    print(auth_url)
    print()
    
    try:
        import webbrowser
        webbrowser.open(auth_url)
        print("📎 (Enlace abierto automáticamente)")
    except:
        pass
    
    code = input("🔐 Código de autorización: ").strip()
    flow.fetch_token(code=code)
    creds = flow.credentials
    
    with open(TOKEN_PATH, 'w') as f:
        f.write(creds.to_json())
    print(f"✅ Token guardado en {TOKEN_PATH}")

if creds:
    print(f"\n📋 Info:")
    print(f"   - Expira: {creds.expiry}")
    print(f"   - Refresh token: {'✅ Sí' if creds.refresh_token else '❌ No'}")
    print(f"   - Canal: {CHANNEL_NAME}")
