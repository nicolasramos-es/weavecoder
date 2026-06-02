# Visualizer FFmpeg: Filtros y Técnicas

Resumen de filtros FFmpeg para visualizer de ondas de música, probados con `ffmpeg-full 8.1.1` en macOS.

## Filtros Disponibles

| Filtro | Uso | Estado |
|:--|:--|:--|
| `showwaves` | Barras espectrales horizontales | ⚠️ Deprecated — no se usa |
| `showfreqs` | Barras de frecuencias centrales (circle) | ✅ **UNIFICADO** — único estilo en producción |
| `showspectrum` | Espectro de colores arcoíris | ⚠️ No probado |
| `showcqt` | Espectrograma tipo CQT | No probado |

## Estructura filter_complex — CIRCLE (producción)

```python
# showfreqs circle — estilo unificado para todos los canales
filter_complex = (
    "[0:a]showfreqs=mode=bar:rate=30:fscale=log:colors=WAVE_COLOR|ACCENT_COLOR[freqs];"
    "color=c=BG_COLOR:s=1080x1920:r=30[bg];"
    "[bg][freqs]overlay=(W-w)/2:(H-h)/2[v]"
)
```

### Parámetros clave

- `rate=N`: FPS del visualizer. 10 para pruebas, 30 para producción.
- `fscale=log` (showfreqs): Escala logarítmica — más natural para música.
- `colors=WAVE|ACCENT`: Dos colores para gradiente de barras.
- `overlay=(W-w)/2:(H-h)/2`: Centrado vertical (más arriba que waves).

## Paletas por Canal

| Canal | bg | wave | accent |
|:--|:--|:--|:--|
| pop (kpop/latin/dance/ballad/electronic) | `0x1a0033` | `0xff1493` | `0x00ffff` |
| rock (metal/classic/indie/hard/alt/grunge) | `0x0a0000` | `0xff4400` | `0x880000` |
| hip-hop (trap/boom_bap/drill/rnb/old/phonk) | `0x0a0000` | `0x9900ff` | `0xff0066` |

## Overlays de texto (drawtext)

Todos requieren `fontfile=` con ruta absoluta. NO usar `font=`.

```python
# Título
drawtext=text='Mi Título':fontfile=/System/Library/Fonts/Supplemental/Arial.ttf:fontcolor=white:fontsize=48:x=(w-text_w)/2:y=Y_POS:shadowcolor=black@0.6:shadowx=2:shadowy=2

# Barra semitransparente para legibilidad
drawbox=x=0:y=Y:w=1080:h=HEIGHT:color=black@0.5:t=fill
```

## Referencia de colores

Colores en formato `0xRRGGBB` para FFmpeg. Nombres como `white`, `red`, `blue` también funcionan.
Para transparencia en drawbox usar `color=black@0.5` (50% opaco).
