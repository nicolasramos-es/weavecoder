#!/usr/bin/env python3
"""
Genera un short musical y lo sube a YouTube Shorts.

Uso:
  python3 scripts/generate_music_short.py --title "Neon Dreams" --style kpop --duration 30

Dependencias: ffmpeg (en PATH), google-api-python-client, google-auth
"""
import argparse, json, os, sys, urllib.request, base64, shutil

# ==== CONFIG ====
WORKDIR = "/Users/nramos/nr-pop"
TOKEN_FILE = "/Users/nramos/.openclaw/workspace/.credentials/youtube_token.json"
ENV_FILE = "/Users/nramos/.hermes/.env"

def get_minimax_key():
    with open(ENV_FILE) as f:
        for line in f:
            line = line.strip()
            if "=" in line and not line.startswith("#"):
                k, v = line.split("=", 1)
                if k == "MINIMAX_API_KEY":
                    return v
    raise RuntimeError("MINIMAX_API_KEY no encontrada en ~/.hermes/.env")

def generate_song(api_key, style, duration=30):
    """Genera canción con MiniMax. Devuelve path al MP3."""
    url = "https://api.minimax.io/v1/music_generation"
    styles = {
        "kpop": "High energy K-pop dance pop, 128 BPM, catchy 3-second hook, futuristic synth, female vocal",
        "pop": "Modern pop, 120 BPM, catchy hook, synth, female vocal, TikTok viral style",
        "latin": "Latin pop, reggaeton beat, 100 BPM, catchy hook, Spanish vocal",
        "ballad": "Emotional pop ballad, piano, strings, female vocal, 80 BPM"
    }
    lyrics = {
        "kpop": """[Intro]\nAh ah ah yeah\n\n[Chorus]\nPop pop pop we never stop\nClimbing to the very top\nFeel the beat drop never quit\nLa la la feel the light\n\n[Outro]\nYeah yeah yeah""",
        "pop": """[Intro]\nLa la la\n\n[Chorus]\nFeel the beat, dancing all night\nEvery moment feels so right\nWe don't stop, we keep going\n\n[Outro]\nYeah...""",
        "latin": """[Intro]\nVamos\n\n[Chorus]\nRitmo caliente, movemos juntos\nNight is calling, feeling good\nWe keep going, never stop\n\n[Outro]\nDale!""",
        "ballad": """[Intro]\nOh oh\n\n[Chorus]\nWhen the stars fall down on me\nYou're the only one I see\nEvery breath I take is free\n\n[Outro]\nForever..."""
    }
    prompt = styles.get(style, styles["pop"])
    lyrics_text = lyrics.get(style, lyrics["pop"])

    payload = {
        "model": "music-2.6",
        "prompt": prompt,
        "lyrics": lyrics_text,
        "duration": duration
    }
    data = json.dumps(payload).encode()
    req = urllib.request.Request(url, data=data, headers={
        "Authorization": f"Bearer {api_key}",
        "Content-Type": "application/json"
    })
    print(f"Generating {style} song ({duration}s)...")
    with urllib.request.urlopen(req, timeout=180) as resp:
        result = json.loads(resp.read())

    audio_hex = result["data"]["audio"]
    audio_bytes = bytes.fromhex(audio_hex)  # ← HEX, no base64!

    out = os.path.join(WORKDIR, f"song_{style}.mp3")
    with open(out, "wb") as f:
        f.write(audio_bytes)
    print(f"Song saved: {out}")
    return out

def generate_thumbnail(api_key, style):
    """Genera thumbnail con MiniMax. Devuelve path al JPG."""
    prompts = {
        "kpop": "K-pop music visualizer, neon purple pink cyan waves, dark background, futuristic, music notes, 9:16 vertical",
        "pop": "Neon music visualizer, glowing colorful waves, dark background, vibrant, music notes, 9:16 vertical",
        "latin": "Latin music vibes, tropical colors, neon sunset, palm trees silhouette, 9:16 vertical",
        "ballad": "Dreamy music visualizer, soft pink purple gradient, floating lights, ethereal, 9:16 vertical"
    }
    url = "https://api.minimax.io/v1/image_generation"
    payload = {
        "model": "image-01",
        "prompt": prompts.get(style, prompts["pop"]),
        "image_size": "9:16",
        "response_format": "url"
    }
    data = json.dumps(payload).encode()
    req = urllib.request.Request(url, data=data, headers={
        "Authorization": f"Bearer {api_key}",
        "Content-Type": "application/json"
    })
    print(f"Generating thumbnail ({style})...")
    with urllib.request.urlopen(req, timeout=60) as resp:
        result = json.loads(resp.read())

    thumb_url = result["data"]["image_urls"][0]
    thumb_path = os.path.join(WORKDIR, f"thumb_{style}.jpg")

    req2 = urllib.request.Request(thumb_url, headers={"User-Agent": "Mozilla/5.0"})
    with urllib.request.urlopen(req2, timeout=30) as resp:
        with open(thumb_path, "wb") as f:
            f.write(resp.read())
    print(f"Thumbnail saved: {thumb_path}")
    return thumb_path

def assemble_video(thumb_path, audio_path, output_path):
    """Assembla video vertical 9:16 con FFmpeg."""
    ffmpeg_bin = shutil.which("ffmpeg") or "/opt/homebrew/bin/ffmpeg"
    cmd = [
        ffmpeg_bin, "-y",
        "-loop", "1", "-i", thumb_path,
        "-i", audio_path,
        "-vf", "scale=1080:1920:force_original_aspect_ratio=decrease,pad=1080:1920:(ow-iw)/2:(oh-ih)/2,setsar=1,format=yuv420p",
        "-c:v", "libx264", "-preset", "fast", "-crf", "23",
        "-c:a", "aac", "-b:a", "192k",
        "-shortest", "-movflags", "+faststart",
        output_path
    ]
    print(f"Assembling video...")
    result = os.system(" ".join(cmd))
    if result != 0:
        raise RuntimeError(f"FFmpeg failed with code {result}")
    print(f"Video saved: {output_path}")

def upload_to_youtube(video_path, title, description=""):
    """Sube video a YouTube Shorts."""
    from google.oauth2.credentials import Credentials
    from google.auth.transport.requests import Request
    from googleapiclient.discovery import build
    from googleapiclient.http import MediaFileUpload

    with open(TOKEN_FILE) as f:
        token_info = json.load(f)

    creds = Credentials(
        token=token_info["token"],
        refresh_token=token_info["refresh_token"],
        token_uri=token_info["token_uri"],
        client_id=token_info["client_id"],
        client_secret=token_info["client_secret"],
        scopes=token_info["scopes"]
    )
    if creds.expired:
        creds.refresh(Request())

    youtube = build("youtube", "v3", credentials=creds)
    body = {
        "snippet": {
            "title": title,
            "description": description or f"✨ #shorts #music #{args.style}",
            "tags": [args.style, "music", "shorts"],
            "categoryId": "10"
        },
        "status": {"privacyStatus": "public", "selfDeclaredMadeForKids": False}
    }
    media = MediaFileUpload(video_path, chunksize=-1, resumable=True)
    print("Uploading to YouTube...")
    response = youtube.videos().insert(
        part="snippet,status", body=body, media_body=media
    ).execute()
    vid = response["id"]
    print(f"✅ Uploaded! URL: https://youtube.com/shorts/{vid}")
    return vid

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Generate music short")
    parser.add_argument("--title", default="Neon Dreams", help="Video title")
    parser.add_argument("--style", default="kpop", choices=["kpop", "pop", "latin", "ballad"])
    parser.add_argument("--duration", type=int, default=30, help="Song duration in seconds (15-60)")
    parser.add_argument("--no-upload", action="store_true", help="Skip YouTube upload")
    args = parser.parse_args()

    os.makedirs(WORKDIR, exist_ok=True)
    api_key = get_minimax_key()

    song_path = generate_song(api_key, args.style, args.duration)
    thumb_path = generate_thumbnail(api_key, args.style)

    output = os.path.join(WORKDIR, f"short_{args.style}.mp4")
    assemble_video(thumb_path, song_path, output)

    if not args.no_upload:
        upload_to_youtube(output, args.title)