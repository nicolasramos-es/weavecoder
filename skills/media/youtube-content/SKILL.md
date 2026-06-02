---
name: youtube-content
description: "YouTube transcripts to summaries, threads, blogs."
platforms: [linux, macos, windows]
---

# YouTube Content Tool

## When to use

Use when the user shares a YouTube URL or video link, asks to summarize a video, requests a transcript, or wants to extract and reformat content from any YouTube video. Transforms transcripts into structured content (chapters, summaries, threads, blog posts).

Extract transcripts from YouTube videos and convert them into useful formats.

## Setup

```bash
pip install youtube-transcript-api
```

**⚠️ Python version pitfall (macOS):** The script uses `/usr/bin/python3` (Apple's 3.9). A `pip install` from a different Python (Homebrew 3.11/3.12) won't make the package available to the script. Install for the right Python: `python3 -m pip install youtube-transcript-api`. If you get an import error after installing, check `which python3` and install against that specific binary. Prefer running the script through Hermes' managed Python (the venv in `~/.hermes/`) for consistency.

## Helper Script

`SKILL_DIR` is the directory containing this SKILL.md file. The script accepts any standard YouTube URL format, short links (youtu.be), shorts, embeds, live links, or a raw 11-character video ID.

```bash
# JSON output with metadata
python3 SKILL_DIR/scripts/fetch_transcript.py "https://youtube.com/watch?v=VIDEO_ID"

# Plain text (good for piping into further processing)
python3 SKILL_DIR/scripts/fetch_transcript.py "URL" --text-only

# With timestamps
python3 SKILL_DIR/scripts/fetch_transcript.py "URL" --timestamps

# Specific language with fallback chain
python3 SKILL_DIR/scripts/fetch_transcript.py "URL" --language tr,en
```

## Output Formats

After fetching the transcript, format it based on what the user asks for:

- **Chapters**: Group by topic shifts, output timestamped chapter list
- **Summary**: Concise 5-10 sentence overview of the entire video
- **Chapter summaries**: Chapters with a short paragraph summary for each
- **Thread**: Twitter/X thread format — numbered posts, each under 280 chars
- **Blog post**: Full article with title, sections, and key takeaways
- **Quotes**: Notable quotes with timestamps
- **Audio traducido (TTS)**: Full transcript translation to another language + TTS audio delivery. See "Translate + TTS Audio Pipeline" below.

### Example — Chapters Output

```
00:00 Introduction — host opens with the problem statement
03:45 Background — prior work and why existing solutions fall short
12:20 Core method — walkthrough of the proposed approach
24:10 Results — benchmark comparisons and key takeaways
31:55 Q&A — audience questions on scalability and next steps
```

## Workflow

1. **Fetch** the transcript using the helper script with `--text-only --timestamps`.
2. **Validate**: confirm the output is non-empty and in the expected language. If empty, retry without `--language` to get any available transcript. If still empty, tell the user the video likely has transcripts disabled.
3. **Chunk if needed**: if the transcript exceeds ~50K characters, split into overlapping chunks (~40K with 2K overlap) and summarize each chunk before merging.
4. **Transform** into the requested output format. If the user did not specify a format, default to a summary.
5. **Verify**: re-read the transformed output to check for coherence, correct timestamps, and completeness before presenting.

## Error Handling

- **Transcript disabled**: tell the user; suggest they check if subtitles are available on the video page.
- **Private/unavailable video**: relay the error and ask the user to verify the URL.
- **No matching language**: retry without `--language` to fetch any available transcript, then note the actual language to the user.
- **Dependency missing**: run `pip install youtube-transcript-api` and retry.

## Translate + TTS Audio Pipeline

Use this when the user asks for a translated audio version of a video. The full pipeline: fetch transcript → translate to target language → generate voice audio → deliver to user.

### Steps

1. **Fetch** the transcript (plain text, no timestamps needed): `python3 SKILL_DIR/scripts/fetch_transcript.py "URL" --language en --text-only`.

2. **Translate** via `delegate_task` — do NOT translate inline. Transcripts this long (25K–50K chars) will blow context limits. Delegate the translation:
   - Pass the file path (e.g. `/tmp/transcript_raw.txt`) in `context`.
   - Goal: "Traduce completamente al español el contenido de /tmp/transcript_raw.txt. Traducción natural y completa, sin resumir. Guarda en /tmp/transcript_espanol.txt."
   - Use toolsets: `["terminal", "file"]`.
   - Verify the output file exists and has comparable size to the original.

3. **Split into TTS-friendly chunks** (~3000-3500 chars each). A 32-min video transcript (~38K chars) typically yields ~12-15 chunks. Use a Python helper:
   ```python
   paragraphs = text.strip().split('\n\n')
   chunks = []
   current = ""
   for para in paragraphs:
       if len(current) + len(para) + 2 > 3500:
           chunks.append(current.strip())
           current = para
       else:
           current = (current + "\n\n" + para).strip()
   if current: chunks.append(current)
   ```

4. **Generate TTS** for each chunk using `text_to_speech(output_path=f"/tmp/tts_chunk_{i:02d}.ogg")`. The provider voice is user-configured (default for Spanish: Edge voice es-ES-ElviraNeural). Fire as many parallel TTS calls as possible.

5. **Deliver** all chunk audio files to the user via `MEDIA:/tmp/tts_chunk_XX.ogg` tags in the response message. Label them clearly (e.g. "Fragmento 1/15", "Fragmento 2/15", etc.) with the video title and language.

### Pitfalls

- **Python version mismatch on macOS**: The fetch script uses `/usr/bin/python3` (3.9). If TTS or text splitting uses the system pip, they won't see `youtube-transcript-api`. Install with `python3 -m pip install ...` or use the Hermes venv.
- **Transcript too large for inline translation**: Always delegate. 38K chars = ~32 min video. Never try to translate more than ~8K chars in a single agent turn.
- **TTS character limits**: Edge TTS handles ~3000-3500 chars per chunk comfortably. Don't push past 4000.
- **Telegram file size**: .ogg files from Edge TTS are ~800K-1.7MB each for 3000-3500 chars text. Well within Telegram limits.
- **Parallel TTS generation**: The tool can handle multiple simultaneous calls — use this to speed up the pipeline.
