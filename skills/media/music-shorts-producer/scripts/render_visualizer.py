#!/usr/bin/env python3
"""
NR Music Pop - Visualizer Renderer
Renderiza ondas de música animadas con FFmpeg.
Estilos: waves, spectrum, circle (⭐ preferido)
Paletas: kpop, latin, dance, ballad, electronic, pop

Uso:
  python3 render_visualizer.py cancion.mp3 --style circle --palette kpop --title "🎵 Tema" -o salida.mp4
  python3 render_visualizer.py cancion.mp3 --preview --title "Preview"  # compara todos los estilos
"""

import subprocess, os, sys, json, tempfile, shutil, re

FFMPEG = "/opt/homebrew/opt/ffmpeg-full/bin/ffmpeg"
FONTS = "/System/Library/Fonts/Supplemental"
FPS = 24

COLOR_PALETTES = {
    "kpop": {"bg": "0x1a0033", "wave": "0xff69b4", "accent": "0x00ffff", "text": "white"},
    "latin": {"bg": "0x1a0a00", "wave": "0xff4500", "accent": "0xffd700", "text": "white"},
    "ballad": {"bg": "0x0a0a2e", "wave": "0x87ceeb", "accent": "0xc0c0c0", "text": "white"},
    "dance": {"bg": "0x0d1117", "wave": "0x00ff7f", "accent": "0x8a2be2", "text": "white"},
    "pop": {"bg": "0x1a0033", "wave": "0xff1493", "accent": "0xffff00", "text": "white"},
    "electronic": {"bg": "0x000033", "wave": "0x00ffff", "accent": "0xff00ff", "text": "white"},
}

def run(cmd, timeout=300):
    p = subprocess.Popen(cmd, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    out, err = p.communicate(timeout=timeout)
    if p.returncode != 0:
        raise RuntimeError(f"FFmpeg error: {err.decode()[:500]}")
    return out

def get_duration(audio_path):
    out = run([FFMPEG, "-i", audio_path, "-f", "null", "-"], timeout=30)
    text = out.decode() if isinstance(out, bytes) else str(out)
    m = re.search(r"Duration: (\d+):(\d+):(\d+\.\d+)", text)
    if m:
        h, m2, s = m.groups()
        return float(h)*3600 + float(m2)*60 + float(s)
    return 30.0

def render(audio_path, output_path, title="NR Music Pop", subtitle="", style="circle",
           palette="pop", fps=FPS, bar_count=40):
    colors = COLOR_PALETTES.get(palette, COLOR_PALETTES["pop"])
    duration = get_duration(audio_path)
    tmp_dir = tempfile.mkdtemp()

    if style == "waves":
        vf = (f"[0:a]showwaves=mode=cline:rate={fps}:n={bar_count}:"
              f"colors={colors['wave']}|{colors['accent']}[waves];"
              f"color=c={colors['bg']}:s=1080x1920:r={fps}[bg];"
              f"[bg][waves]overlay=(W-w)/2:(H-h)/1.5[v]")
    elif style == "spectrum":
        vf = (f"[0:a]showspectrum=mode=combined:color=rainbow:slide=1:"
              f"rate={fps}:s=1080x800[spec];"
              f"color=c={colors['bg']}:s=1080x1920:r={fps}[bg];"
              f"[bg][spec]overlay=0:(H-h)/2[v]")
    else:  # circle (default)
        vf = (f"[0:a]showfreqs=mode=bar:rate={fps}:fscale=log:"
              f"colors={colors['wave']}|{colors['accent']}[freqs];"
              f"color=c={colors['bg']}:s=1080x1920:r={fps}[bg];"
              f"[bg][freqs]overlay=(W-w)/2:(H-h)/2[v]")

    visual = os.path.join(tmp_dir, "visual.mp4")
    run([FFMPEG, "-y", "-i", audio_path, "-filter_complex", vf,
         "-map", "[v]", "-map", "0:a",
         "-c:v", "libx264", "-preset", "fast", "-crf", "23",
         "-c:a", "aac", "-b:a", "192k",
         "-shortest", "-movflags", "+faststart", visual])

    overlays = [f"drawbox=x=0:y={int(1920*0.75)}:w=1080:h={int(1920*0.25)}:color=black@0.5:t=fill"]
    if title:
        overlays.append(f"drawtext=text='{title}':fontfile={FONTS}/Arial.ttf:"
                        f"fontcolor={colors['text']}:fontsize=48:"
                        f"x=(w-text_w)/2:y={int(1920*0.78)}:"
                        f"shadowcolor=black@0.6:shadowx=2:shadowy=2")
    if subtitle:
        overlays.append(f"drawtext=text='{subtitle}':fontfile={FONTS}/Arial.ttf:"
                        f"fontcolor={colors['accent']}:fontsize=28:"
                        f"x=(w-text_w)/2:y={int(1920*0.85)}:"
                        f"shadowcolor=black@0.6:shadowx=2:shadowy=2")
    overlays.append(f"drawtext=text='NR Music Pop':fontfile={FONTS}/Arial.ttf:"
                    f"fontcolor=white@0.4:fontsize=18:"
                    f"x=w-text_w-30:y=h-text_h-30")

    run([FFMPEG, "-y", "-i", visual, "-vf", ",".join(overlays),
         "-c:v", "libx264", "-preset", "fast", "-crf", "23",
         "-c:a", "copy", "-movflags", "+faststart", output_path])

    shutil.rmtree(tmp_dir, ignore_errors=True)
    size = os.path.getsize(output_path)
    print(f"✅ {output_path} ({size/1024/1024:.1f} MB)")

if __name__ == "__main__":
    import argparse
    p = argparse.ArgumentParser()
    p.add_argument("audio")
    p.add_argument("-o", "--output", default="")
    p.add_argument("-t", "--title", default="🎵 Nuevo Tema")
    p.add_argument("-s", "--subtitle", default="NR Music Pop")
    p.add_argument("--style", default="circle", choices=["waves", "spectrum", "circle"])
    p.add_argument("-p", "--palette", default="pop", choices=list(COLOR_PALETTES.keys()))
    p.add_argument("--preview", action="store_true")
    args = p.parse_args()
    if args.preview:
        for s in ["waves", "circle"]:
            out = os.path.join(os.path.dirname(args.audio) or ".", f"preview_{s}.mp4")
            try:
                render(args.audio, out, title=args.title, subtitle=args.subtitle, style=s)
            except Exception as e:
                print(f"❌ {s}: {e}")
    else:
        out = args.output or os.path.join(
            os.path.dirname(args.audio) or ".",
            f"viz_{os.path.splitext(os.path.basename(args.audio))[0]}.mp4")
        render(args.audio, out, args.title, args.subtitle, args.style, args.palette)
