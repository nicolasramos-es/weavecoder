# YouTube CDP Upload — Pipeline Session Log (2026-05-31)

## Problemas encontrados y soluciones

### 1. Todos los vídeos se subían al mismo canal

**Causa:** El script navegaba a `studio.youtube.com` genérico, que mostraba el último canal usado. Como el usuario tenía acceso de editor a 4 canales desde una cuenta, YouTube mostraba el que estuviera activo.

**Solución:** Usar `channel_id` específico para cada canal:
```python
studio_url = f"https://studio.youtube.com/channel/{channel_id}/videos"
await page.goto(studio_url, wait_until="domcontentloaded")
```

Los IDs se obtienen desde la página del canal (ver `references/channel-ids-verified.md`).

### 2. "No es para niños" no se encontraba

**Causa:** El selector `NOT_MADE_FOR_KIDS` era incorrecto. El nombre real del radio button es `VIDEO_MADE_FOR_KIDS_NOT_MFK` (no `NOT_MADE_FOR_KIDS`). Además, la sección de audiencia queda fuera del viewport después de rellenar título y descripción.

**Solución:**
```python
await page.evaluate("window.scrollBy(0, 300)")
await asyncio.sleep(1)
no_radio = page.locator('tp-yt-paper-radio-button[name="VIDEO_MADE_FOR_KIDS_NOT_MFK"]').first
```

### 3. Conexión CDP se perdía entre vídeos

**Causa:** El script original abría y cerraba `browser` (y por tanto la conexión CDP) en cada `upload_video()`. Al cerrar `browser`, Chrome perdía la conexión y el siguiente intento fallaba con `ECONNREFUSED`.

**Solución:** Conectar una vez al principio y reutilizar `context.new_page()` para cada vídeo:
```python
async with async_playwright() as p:
    browser = await p.chromium.connect_over_cdp(CDP_URL)
    context = browser.contexts[0]
    for video in videos:
        page = await context.new_page()
        try:
            await upload_video(page, ...)
        finally:
            await page.close()
```

### 4. Botón "Siguiente" disabled

**Causa:** Ocurría al no seleccionar "No es para niños" (por el problema #2) y en el paso 2 ("Elementos del vídeo") donde hay que marcar algo antes de continuar.

**Solución:** Verificar `get_attribute("disabled")` y esperar antes de clickar.

### 5. Botón "Publicar" vs "Guardar"

En el paso de visibilidad, mientras el vídeo se procesa el botón dice "Guardar" (disabled). Una vez procesado, cambia a "Publicar" (enabled). El script ahora espera hasta 60s en bucle comprobando ambos.

### 6. Límite diario de subida

YouTube mostró: "Límite diario de subida alcanzado. Sube más vídeos cada día tras una verificación única o espera 24 horas." después de ~19 subidas. No es de API — es de YouTube web para cuentas de editor sin verificar el canal.
