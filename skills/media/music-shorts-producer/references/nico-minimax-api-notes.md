# MiniMax Music API — Notas de Implementación

## Descubrimiento clave (2026-05-26)

**El audio de `data.audio` viene HEX-ENCODED, NO base64.**

Ejemplo de respuesta cruda:
```json
{
  "data": {
    "audio": "49443304000000001245545353450000000f0000034c61766635382e37362e313030"
  },
  "extra_info": {
    "music_duration": 30100,
    "music_sample_rate": 44100
  }
}
```

El string `'49443304000000001245...'` son **bytes hex-encoded** del MP3 (cada 2 caracteres hex = 1 byte). Decodear así:

```python
audio_hex = result["data"]["audio"]
audio_bytes = bytes.fromhex(audio_hex)  # NO base64.b64decode()
```

## Por qué falló base64

`bytes.fromhex()` funciona porque cada byte se representa con 2 caracteres hex en el string de respuesta. `base64.b64decode()` falla con `Incorrect padding` porque el input no es padding-compliant para base64.

## Flujo confirmado

```
POST /v1/music_generation
  → 200 con audio en body (no polling, no async)
  → bytes.fromhex(data["data"]["audio"])
  → archivo .mp3 válido (ID3 header = 49 44 33)
  → ffprobe confirma: 44100Hz, stereo, ~30s
```

## Códigos de error comunes

| Código | Significado |
|:--|:--|
| 2013 | `lyrics is required` — no se pasó campo `lyrics` |
| 401 | API key inválida o expirada |
| 400 | Prompt inválido o duración fuera de rango |

## Duración y tiempo de respuesta

| Duración | Tiempo aprox. |
|:--|:--|
| 15-30s | 15-40s |
| 30-45s | 40-90s |
| 45-60s | 90-180s |

El audio viene directo en la respuesta — no hay que hacer polling.

## Thumbnail (MiniMax Imagen)

Las URLs de MiniMax son firmadas con OSSAccessKeyId + Signature + Expiry. Requieren:
- Header `User-Agent: Mozilla/5.0` o similar
- Descargar antes del expiry (típicamente 1 hora)
- `response_format=url` devuelve URL corta (válida 1h)

## Archivos de referencia

```
/tmp/music_raw_resp.json      # respuesta JSON cruda guardada para debug
/Users/nramos/nr-pop/         # directorio de trabajo music shorts
```