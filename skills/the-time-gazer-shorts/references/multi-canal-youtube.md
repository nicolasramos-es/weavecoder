# Multi-Canal YouTube — Workflow de Autenticación OAuth

## Estados de Canal (Mayo 2026)

| Canal | Token | Expira | Refresh |
|:--|:--|:--|:--|
| The Time Gazer | `youtube_token.json` | ⚠️ | ✅ |
| NR Music Pop | `youtube_token_nrpop.json` | ⚠️ | ✅ |
| NR Music Rock | `youtube_token_nrrock.json` | ⚠️ | ✅ |
| NR Music Hip-Hop | `youtube_token_nrhiphop.json` | ⚠️ | ✅ |
| NR Music Latino | `youtube_token_nrlatino.json` | ✅ | ✅ |

## Método Recomendado: Servidor Local con PKCE (puertos distintos por canal)

Para múltiples canales con el mismo Client ID, usar `redirect_uri` con **puerto único** por canal para evitar conflictos de códigos OAuth single-use:

| Canal | Puerto redirect_uri | Notas |
|:--|:--|:--|
| NR Music Pop | `localhost:8080` | Primer canal configurado |
| NR Music Rock | `localhost:8081` | Evita conflicto con pop |
| NR Music Hip-Hop | `localhost:9093` | Puerto dinámico, evitar conflicto |
| NR Music Latino | `localhost:9090` | Pendiente configurar |

**⚠️ Pitfall crítico: OAuth codes son single-use** — si el código se usa dos veces (error de código, reintento, etc.), el segundo intento falla con `invalid_grant`. Siempre generar URL nueva para cada intento.

**Flujo correcto:**
1. Generar URL de autorización con PKCE en script
2. Imprimir la URL en el mensaje del agente (no depender de stdout del proceso hijo)
3. Usuario abre URL en Safari y autoriza
4. Servidor local recibe callback y hace exchange automáticamente
5. Verificar que el token se grabó en disco

## Script de Autenticación

Ubicación: `/tmp/oauth_youtube_channel.py` — script genérico que acepta el nombre del canal y puerto como argumento:

```bash
python3 /tmp/oauth_youtube_channel.py <nombre_canal> <puerto>
# ej: python3 /tmp/oauth_youtube_channel.py nrhiphop 9093
```

Ruta del token: `~/.openclaw/workspace/.credentials/youtube_token_{canal}.json`

## Método OOB (`urn:ietf:wg:oauth:2.0:oob`)

Alternativa simple cuando el método localhost no funciona:

```python
flow.redirect_uri = 'urn:ietf:wg:oauth:2.0:oob'
auth_url, _ = flow.authorization_url(prompt='consent', access_type='offline')
print(auth_url)
code = input("Código: ")
flow.fetch_token(code=code)
```

## Verificación de Tokens

```bash
for f in ~/.openclaw/workspace/.credentials/youtube_token*.json; do
  python3 -c "import json; d=json.load(open('$f')); print(f'$f: refresh={\"si\" if d.get(\"refresh_token\") else \"NO\"}')" 2>/dev/null
done
```

## Verificación Post-Upload (OBLIGATORIA)

```bash
curl -s "https://www.youtube.com/oembed?url=<URL>&format=json" | python3 -c "
import json, sys
d = json.load(sys.stdin)
print('Canal:', d['author_name'])
print('Título:', d['title'])
"
```

## Pitfalls

- **Puerto ocupado:** Si el puerto está en uso, el servidor levantará con `OSError: Address already in use`. Solución: `lsof -ti :<puerto> | xargs kill -9`.
- **Código ya gastado:** El código OAuth es single-use. Si falla el primer intento, el código queda invalidado — hay que generar uno nuevo.
- **Puerto diferente por canal:** Si dos canales comparten el mismo redirect_uri:puerto, los códigos se confunden y fallan con `invalid_grant`.
