#!/usr/bin/env python3
"""
Music Shorts Producer - render_v2.py
Renderizador unificado de visualizer para todos los canales NR Music.

Uso:
    from render_v2 import render
    render(mp3_path, mp4_path, title, palette, watermark)

Parámetros:
    - mp3_path: ruta al audio de entrada
    - mp4_path: ruta de salida del video
    - title: título del tema (usar caracteres Unicode ♫, no emojis 🎵)
    - palette: string del canal ("pop", "rock", "hiphop", "latino")
    - watermark: marca de agua (ej: "NR Music Pop", "NR Music Rock")

Paletas disponibles:
    - pop:     bg=0x1a0033, wave=0xff1493, accent=0x00ffff
    - rock:    bg=0x0a0500, wave=0xff5500, accent=0xaa3300
    - hiphop:  bg=0x0a001a, wave=0x9900ff, accent=0xff0066
    - latino:  bg=0x1a001a, wave=0xff00aa, accent=0x00ffcc
    - salsa:   bg=0x1a0a00, wave=0xff6600, accent=0xffdd00
    - bachata: bg=0x1a000a, wave=0xff3366, accent=0xff99cc

Requisitos:
    - ffmpeg-full (brew install ffmpeg-full) - requiere drawtext
    - Fuente Arial en /System/Library/Fonts/Supplemental/Arial.ttf

Pitfalls:
    - NO usar emojis (🎵 🔥) en títulos — renderizan como cuadrados
    - Usar caracteres Unicode: ♫ ♪ ★ ⚡ ✦
    - Siempre exportar MINIMAX_API_KEY antes de llamar a MiniMax
"""

import subprocess, os, tempfile, shutil

FFMPEG = "/opt/homebrew/opt/ffmpeg-full/bin/ffmpeg"
FONT = "/System/Library/Fonts/Supplemental/Arial.ttf"

def run_ffmpeg(args):
    p = subprocess.Popen(args, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    out, err = p.communicate(timeout=300)
    if p.returncode != 0:
        raise RuntimeError(f"FFmpeg: {err.decode()[:300]}")
    return out

def render(audio_in, video_out, title, palette, watermark):
    """Renderiza visualizer circle con marca de agua dinámica."""
    
    PALETTES = {
        "pop":    {"bg": "0x1a0033", "wave": "0xff1493", "accent": "0x00ffff"},
        "rock":   {"bg": "0x0a0500", "wave": "0xff5500", "accent": "0xaa3300"},
        "hiphop": {"bg": "0x0a001a", "wave": "0x9900ff", "accent": "0xff0066"},
        "latino": {"bg": "0x1a001a", "wave": "0xff00aa", "accent": "0x00ffcc"},
        "salsa":  {"bg": "0x1a0a00", "wave": "0xff6600", "accent": "0xffdd00"},
        "bachata":{"bg": "0x1a000a", "wave": "0xff3366", "accent": "0xff99cc"},
    }
    
    c = PALETTES.get(palette, PALETTES["pop"])
    tmp = tempfile.mkdtemp()
    
    try:
        # Paso 1: Video base con visualizer circle
        vf = (f"[0:a]showfreqs=mode=bar:rate=30:fscale=log:"
              f"colors={c['wave']}|{c['accent']}[freqs];"
              f"color=c={c['bg']}:s=1080x1920:r=30[bg];"
              f"[bg][freqs]overlay=(W-w)/2:(H-h)/2[v]")
        
        vis = os.path.join(tmp, "v.mp4")
        run_ffmpeg([FFMPEG, "-y", "-i", audio_in, "-filter_complex", vf,
                    "-map", "[v]", "-map", "0:a", "-c:v", "libx264",
                    "-preset", "fast", "-crf", "23", "-c:a", "aac",
                    "-b:a", "192k", "-shortest", vis])
        
        # Paso 2: Añadir título y marca de agua
        safe_title = title.replace("'", "\\'")
        safe_watermark = watermark.replace("'", "\\'")
        
        vf2 = (f"drawbox=x=0:y=1440:w=1080:h=480:color=black@0.5:t=fill,"
               f"drawtext=text='{safe_title}':fontfile={FONT}:fontcolor=white:"
               f"fontsize=60:x=(w-text_w)/2:y=1490:shadowx=2:shadowy=2:borderw=3:bordercolor=black@0.7,"
               f"drawtext=text='{safe_watermark}':fontfile={FONT}:fontcolor=white@0.7:"
               f"fontsize=24:x=w-text_w-20:y=h-text_h-20:shadowx=1:shadowy=1")
        
        run_ffmpeg([FFMPEG, "-y", "-i", vis, "-vf", vf2,
                    "-c:v", "libx264", "-preset", "fast", "-crf", "23",
                    "-c:a", "copy", "-movflags", "+faststart", video_out])
        
    finally:
        shutil.rmtree(tmp, ignore_errors=True)


if __name__ == "__main__":
    import argparse
    p = argparse.ArgumentParser()
    p.add_argument("audio")
    p.add_argument("-o", "--output", required=True)
    p.add_argument("-t", "--title", required=True)
    p.add_argument("-p", "--palette", default="pop")
    p.add_argument("-w", "--watermark", required=True)
    a = p.parse_args()
    render(a.audio, a.output, a.title, a.palette, a.watermark)
