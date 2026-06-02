# YouTube OAuth Credentials — Nico's Setup

## Credentials Location

All YouTube OAuth credentials are stored in:
```
/Users/nramos/.openclaw/workspace/.credentials/
```

Files:
- `youtube_token.json` — OAuth access/refresh token (auto-refreshes)
- `google_credentials.json` — OAuth client secrets (installed app flow)
- `google_client_secret.json` — Alternative client secrets (Google Cloud)
- `google_calendar_token.json` — Calendar-specific token

## Token Validation (2026-05-26)

```bash
$ cat youtube_token.json | python3 -c "import json,sys; d=json.load(sys.stdin); print('token válido:', 'access_token' in d); print('expired:', d.get('expiry', 'N/A')); print('refresh_token:', 'refresh_token' in d)"
# Output:
# token válido: False
# expired: 2026-05-26T20:57:16.779713Z
# refresh_token: True
```

**Key insight:** `access_token` key doesn't exist — the token is stored in a different format. But `refresh_token` is present, so auto-refresh works via `google.auth.transport.requests.Request()`.

## Publish Script Path

```
/Users/nramos/.hermes/video-producer/publish_to_youtube.py
```

Usage:
```bash
python3 /Users/nramos/.hermes/video-producer/publish_to_youtube.py \
  /path/to/video.mp4 \
  "Video Title" \
  "Description" \
  --category 10  # Music
  --privacy public
```

## Token Refresh Flow

The `publish_to_youtube.py` script uses `google-auth` library:

```python
from google.oauth2.credentials import Credentials
from google.auth.transport.requests import Request

creds = Credentials.from_authorized_user_file(token_path, SCOPES)
if creds and creds.expired and creds.refresh_token:
    creds.refresh(Request())
# Writes refreshed token back to file
```

This is the same mechanism used by The Time Gazer cron job — confirmed working.

## Multiple Channels

One OAuth token works for ALL YouTube channels managed by the same Google account:
- NR Music Pop: UCoMn2PNx_wdhXkazzLeZJ4w
- The Time Gazer: (different channel, same account)
- Any other channels Nico owns

The `youtube.upload` scope allows uploading to any channel accessible by the Google account.