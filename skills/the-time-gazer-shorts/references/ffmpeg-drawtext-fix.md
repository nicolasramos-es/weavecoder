# ffmpeg drawtext fix (24-May-2026)

## Problem

`produce_video_STABLE.py` failed on the second ffmpeg call (the `drawtext` overlay step) with exit code 8.

Root cause chain:
1. `/usr/bin/ffmpeg` didn't exist → fixed by using Homebrew path
2. Homebrew `ffmpeg` is the light formula, compiled **without** `--enable-libfreetype` or `--enable-libfontconfig`, so the `drawtext` filter doesn't exist
3. Even after installing `ffmpeg-full`, using `font='Arial'` (name-only, no fontfile) doesn't work because Homebrew's ffmpeg-full doesn't have macOS fontconfig integration

## Detection

```bash
# Check if drawtext filter exists
ffmpeg -filters 2>/dev/null | grep drawtext
# → empty = no drawtext

# Check ffmpeg-full
/opt/homebrew/opt/ffmpeg-full/bin/ffmpeg -hide_banner -filters 2>/dev/null | grep -c drawtext
# → 1 = drawtext available

# Check font support
ffmpeg -y -f lavfi -i "color=c=black:s=1080x1920:d=2" \
  -vf "drawtext=text='TEST':fontcolor=white:fontsize=38:fontfile=/System/Library/Fonts/Supplemental/Arial.ttf" \
  -c:v libx264 -pix_fmt yuv420p -an /tmp/test.mp4 2>&1
```

## Fix applied to `produce_video_STABLE.py`

```python
import shutil
FFMPEG = shutil.which("ffmpeg") or "/opt/homebrew/bin/ffmpeg"
FFPROBE = shutil.which("ffprobe") or "/opt/homebrew/bin/ffprobe"
FONT_PATH = "/System/Library/Fonts/Supplemental/Arial.ttf"
# Preferir ffmpeg-full (tiene drawtext) sobre el ffmpeg estándar
if os.path.exists("/opt/homebrew/opt/ffmpeg-full/bin/ffmpeg"):
    FFMPEG = "/opt/homebrew/opt/ffmpeg-full/bin/ffmpeg"
    FFPROBE = "/opt/homebrew/opt/ffmpeg-full/bin/ffprobe"
```

## Drawtext syntax for ffmpeg-full 8.1.1

**CORRECT (works):**
```
drawtext=text='Hello':fontcolor=white:fontsize=38:fontfile=/System/Library/Fonts/Supplemental/Arial.ttf
```

**WRONG (crashes with `Option not found`):**
```
drawtext=text='Hello':fontcolor=white:fontsize=38:fontname=Arial
drawtext=...:fontfile=...:fontname=Arial  # 'fontname' is not a parameter in this version
```

**Key rules:**
- ✅ `fontfile=` with **absolute path** to `.ttf`
- ❌ NO `:fontname=` parameter (it's not a valid option for this build)
- ❌ NO `font='Arial'` (name-only resolves via fontconfig which isn't available)

## What NOT to do

- Do NOT try to symlink or reinstall the regular `ffmpeg` — the regular Homebrew formula has never shipped with drawtext. `ffmpeg-full` is the correct formula.
- The `font='Name'` syntax works on some ffmpeg builds that have fontconfig. Our macOS build doesn't. Always specify `fontfile=` with the full .ttf path.
