# Estado Tokens OAuth — Canales NR Music (2026-05-28)

## Verificación rápida

```bash
ls ~/.openclaw/workspace/.credentials/youtube_token_*.json | while read f; do
  echo -n "$f: "
  python3 -c "import json; d=json.load(open('$f')); print('OK' if d.get('access_token') else 'SIN TOKEN')"
done
```

## Resultado actual

| Canal | Token | Cuota API | Upload posible |
|:--|:--|:--|:--|
| NR Music Pop | ❌ SIN TOKEN | — | No |
| NR Music Rock | ✅ OK | ~6/día | Sí (hasta quota) |
| NR Music Hip-Hop | ✅ OK | ~6/día | Sí (hasta quota) |
| NR Music Latino | ❌ SIN TOKEN | — | No |

## Vídeos pendientes (2026-05-28)

- `/Users/nramos/nr-pop/output/short_20260527_*.mp4` — 5 vídeos, sin token
- `/Users/nramos/nr-rock/output/short_*.mp4` — 6 vídeos, token OK
- `/Users/nramos/nr-latino/output/short_*.mp4` — 3 vídeos, sin token
- `/Users/nramos/nr-hiphop/output/short_*.mp4` — 6 vídeos, token OK

## Para autorizar tokens que faltan

Cada canal necesita su propio código de autorización OAuth con redirect_uri de puerto único:

```
# NR Pop → puerto 8080
# NR Rock → puerto 8081  
# NR Latino → puerto 8082
# NR Hip-Hop → puerto 8083
```

Ver `references/youtube-oauth-token-format.md` para el proceso completo.

## Reset cuota API

La cuota de YouTube se reinicia a las **00:00 UTC** (~6 uploads/día por proyecto).

## Alternativa rápida para tokens que faltan

`youtube.com/upload` — upload manual sin OAuth. 2 min por vídeo, sin límite.