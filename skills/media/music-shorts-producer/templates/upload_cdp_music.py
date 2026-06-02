#!/usr/bin/env python3
"""
Upload NR Music shorts to YouTube via Playwright + Chrome CDP.
Connects to Chrome at --remote-debugging-port=9222.

Usage:
  python3 upload_cdp_music.py pop        # NR Pop
  python3 upload_cdp_music.py rock       # NR Rock
  python3 upload_cdp_music.py hiphop     # NR Hip-Hop
  python3 upload_cdp_music.py latino     # NR Latino
  python3 upload_cdp_music.py all        # all channels sequentially

Requirements:
  - Chrome running with: --remote-debugging-port=9222
  - User logged into YouTube Studio (all channels as editor)
  - playwright installed (pip install playwright)

Channel IDs:
  Pop:    UCoMn2PNx_wdhXkazzLeZJ4w
  Rock:   UCiimUjAf4EgmNaYNwtpTcTw
  HipHop: UCvjaeHB6AvJ4HVSjifOb76w
  Latino: UC2OvxQ76X6hA5nYV-xZdD0g
"""

import asyncio, json, os, sys, re
from datetime import datetime

CDP_PORT = 9222
CDP_URL = f"http://localhost:{CDP_PORT}"

CHANNELS = {
    "pop": {
        "channel_id": "UCoMn2PNx_wdhXkazzLeZJ4w",
        "dir": "/Users/nramos/nr-pop",
        "history": "/Users/nramos/nr-pop/published_history_nrpop.json",
        "emoji": "🎵",
        "name": "NR Pop",
        "description": "🎵 NR Pop - Música pop original creada con IA. Nuevos temas cada semana.\n\n#NRPop #MúsicaPop #IAMusic #NuevosTemas",
    },
    "rock": {
        "channel_id": "UCiimUjAf4EgmNaYNwtpTcTw",
        "dir": "/Users/nramos/nr-rock",
        "history": "/Users/nramos/nr-rock/published_history_nrrock.json",
        "emoji": "🎸",
        "name": "NR Rock",
        "description": "🎸 NR Rock - Rock original creado con inteligencia artificial. Nuevos temas cada semana.\n\n#NRRock #MúsicaRock #IAMusic #RockEspañol",
    },
    "hiphop": {
        "channel_id": "UCvjaeHB6AvJ4HVSjifOb76w",
        "dir": "/Users/nramos/nr-hiphop",
        "history": "/Users/nramos/nr-hiphop/published_history_nrhiphop.json",
        "emoji": "🎤",
        "name": "NR Hip-Hop",
        "description": "🎤 NR Hip-Hop - Hip hop y rap original creado con inteligencia artificial. Nuevos beats cada semana.\n\n#NRHipHop #HipHop #IAMusic #Rap",
    },
    "latino": {
        "channel_id": "UC2OvxQ76X6hA5nYV-xZdD0g",
        "dir": "/Users/nramos/nr-latino",
        "history": "/Users/nramos/nr-latino/published_history_nrlatino.json",
        "emoji": "💃",
        "name": "NR Latino",
        "description": "💃 NR Latino - Ritmos latinos originales creados con IA. Salsa, reguetón y más.\n\n#NRLatino #MúsicaLatina #IAMusic #Salsa #Reguetón",
    },
}

MONTHS_ES = ["enero","febrero","marzo","abril","mayo","junio",
             "julio","agosto","septiembre","octubre","noviembre","diciembre"]


async def get_pending_videos(channel_key):
    cfg = CHANNELS[channel_key]
    all_videos = sorted([f for f in os.listdir(cfg["dir"])
                         if f.startswith("short_") and f.endswith(".mp4")])
    published_videos = set()
    if os.path.exists(cfg["history"]):
        try:
            with open(cfg["history"]) as fh:
                for p in json.load(fh).get("published", []):
                    if p.get("video"):
                        published_videos.add(p["video"])
        except Exception:
            pass
    return [f for f in all_videos if f not in published_videos]


def generate_title(channel_key, video_file):
    """Título descriptivo: 🎸 NR Rock - Nuevo tema 28 de mayo de 2026 🎵"""
    cfg = CHANNELS[channel_key]
    m = re.match(r"short_(\d{4})(\d{2})(\d{2})_(?:\d{6})?\.mp4", video_file)
    if m:
        y, mth, d = m.group(1), int(m.group(2)), m.group(3)
        return f"{cfg['emoji']} {cfg['name']} - Nuevo tema {d} de {MONTHS_ES[mth-1]} de {y} {cfg['emoji']}"
    # Fallback: file mtime
    mtime = os.path.getmtime(os.path.join(cfg["dir"], video_file))
    dt = datetime.fromtimestamp(mtime)
    return f"{cfg['emoji']} {cfg['name']} - Nuevo tema {dt.day} de {MONTHS_ES[dt.month-1]} de {dt.year} {cfg['emoji']}"


def mark_published(channel_key, video_file, title):
    cfg = CHANNELS[channel_key]
    data = {"published": [], "used_titles": []}
    if os.path.exists(cfg["history"]):
        try:
            with open(cfg["history"]) as fh:
                data = json.load(fh)
        except Exception:
            pass
    data.setdefault("published", [])
    data.setdefault("used_titles", [])
    data["published"].append({"title": title, "video": video_file,
                               "timestamp": datetime.now().isoformat()})
    data["used_titles"].append(title)
    with open(cfg["history"], "w") as fh:
        json.dump(data, fh, indent=2, ensure_ascii=False)


async def upload_video(page, channel_key, video_file):
    """Upload one video using a fresh page in an existing CDP session."""
    cfg = CHANNELS[channel_key]
    video_path = os.path.join(cfg["dir"], video_file)
    title = generate_title(channel_key, video_file)

    if not os.path.exists(video_path):
        print(f"  ❌ No existe: {video_path}"); return False

    size_mb = os.path.getsize(video_path) / (1024 * 1024)
    print(f"\n  📤 {video_file} ({size_mb:.1f} MB) → {title}")

    try:
        # Navigate DIRECTLY to this channel's Studio (critical for multi-channel)
        studio_url = f"https://studio.youtube.com/channel/{cfg['channel_id']}/videos"
        print(f"  🌐 {cfg['name']}...")
        await page.goto(studio_url, wait_until="domcontentloaded")
        await asyncio.sleep(4)

        # Check session
        body = await page.locator("body").inner_text()
        if "Acceder" in body[:500] and "Correo electrónico" in body[:800]:
            print("  ❌ Sesión no iniciada (relogin needed)"); return False

        # Step 1: Click upload — try big "Subir vídeos" button first, then "Crear" menu
        btn = page.locator("button:has-text('Subir vídeos')").first
        if await btn.is_visible(timeout=3000):
            await btn.click(); await asyncio.sleep(2)
        else:
            crear = page.get_by_role("button", name="Crear").first
            if await crear.is_visible(timeout=5000):
                await crear.click(); await asyncio.sleep(1.5)
                menu = page.get_by_role("menuitem", name="Subir vídeos").first
                if await menu.is_visible(timeout=5000):
                    await menu.click(); await asyncio.sleep(2)
            else:
                print("  ❌ Sin botón para subir"); return False

        # Step 2: Upload file
        fi = page.locator('input[type="file"]').first
        await fi.set_input_files(video_path, timeout=30000)

        # Step 3: Wait for processing
        print("  ⏳ Procesando... (10s)")
        await asyncio.sleep(10)

        # Step 4: Title
        tb = page.get_by_role("textbox", name="Título").first
        if await tb.is_visible(timeout=15000):
            await tb.clear(); await tb.fill(title[:100])
            print(f"  ✅ Título: {title}")

        await asyncio.sleep(1)

        # Step 5: Description
        desc = cfg.get("description", "")
        if desc:
            db = page.get_by_role("textbox", name="Descripción").first
            if await db.is_visible(timeout=5000):
                await db.fill(desc); print("  ✅ Descripción con hashtags")

        await asyncio.sleep(1)

        # Step 6: "No es para niños" — MUST be correct or "Siguiente" stays disabled
        print("  👶 Marcando 'No es para niños'...")
        found = False
        # Strategy A: by name attribute (most reliable)
        try:
            radio = page.locator('tp-yt-paper-radio-button[name="VIDEO_MADE_FOR_KIDS_NOT_MFK"]').first
            if await radio.is_visible(timeout=3000):
                await radio.click(); found = True
        except:
            pass
        # Strategy B: by label text ("creado" not "hecha")
        if not found:
            try:
                lbl = page.get_by_label("No, no está creado para niños").first
                if await lbl.is_visible(timeout=2000):
                    await lbl.click(); found = True
            except:
                pass
        # Strategy C: iterate radios (NO match 'no' substring — picks "Sí...niños"!)
        if not found:
            for r in await page.locator("tp-yt-paper-radio-button").all():
                text = (await r.inner_text()).strip()
                if text.lower().startswith("no") and "niños" not in text.lower():
                    await r.click(); found = True; break
        print(f"  {'✅ No es para niños' if found else '⚠️ No encontrado'}")

        await asyncio.sleep(1)

        # Step 7: "Siguiente" × 3
        print("  ⏭️ Siguiente ×3...")
        for i in range(3):
            sig = page.get_by_role("button", name="Siguiente").first
            await sig.wait_for(state="visible", timeout=10000)
            disabled = await sig.get_attribute("disabled")
            if disabled:
                print(f"     Siguiente {i+1} disabled, esperando 3s...")
                await asyncio.sleep(3)
            await sig.click(timeout=15000)
            await asyncio.sleep(2)
            print(f"     ✅ {i+1}/3")

        await asyncio.sleep(1)

        # Step 8: Publish — try "Publicar" first, fallback "Guardar"
        print("  🚀 Publicando...")
        published = False
        for _ in range(12):  # wait up to ~120s for processing
            pub = page.get_by_role("button", name="Publicar").first
            if await pub.is_visible(timeout=5000) and not await pub.get_attribute("disabled"):
                await pub.click(); await asyncio.sleep(5)
                published = True; break
            # Check Guardar (enabled when processing done but button still says Guardar)
            g = page.get_by_role("button", name="Guardar").first
            if await g.is_visible(timeout=2000) and not await g.get_attribute("disabled"):
                await g.click(); await asyncio.sleep(5)
                published = True; break
            print(f"     Esperando procesamiento... ({_+1}/12)")
            await asyncio.sleep(10)

        if not published:
            print("  ❌ No se pudo publicar"); return False

        print(f"  ✅ ¡PUBLICADO! {title}")
        mark_published(channel_key, video_file, title)
        return True

    except Exception as e:
        print(f"  ❌ Error: {e}")
        return False


async def upload_channel(channel_key, browser, context):
    cfg = CHANNELS[channel_key]
    print(f"\n{'='*50}\n🎬 {cfg['emoji']} {cfg['name']}\n{'='*50}")
    pending = await get_pending_videos(channel_key)
    if not pending:
        print(f"✅ No hay pendientes"); return 0
    print(f"📋 {len(pending)} pendientes")
    ok_count = 0
    for i, vf in enumerate(pending, 1):
        print(f"\n--- [{i}/{len(pending)}] {vf} ---")
        page = await context.new_page()
        try:
            if await upload_video(page, channel_key, vf):
                ok_count += 1
                await asyncio.sleep(3)
        finally:
            await page.close()
    print(f"\n📊 {cfg['emoji']} {cfg['name']}: {ok_count}/{len(pending)}")
    return ok_count


async def main():
    from playwright.async_api import async_playwright
    args = sys.argv[1:]
    if not args or "all" in args:
        channels = ["pop", "rock", "hiphop", "latino"]
    else:
        channels = [a for a in args if a.lower() in CHANNELS]
    if not channels:
        print(__doc__); return

    print(f"🌐 Conectando a Chrome CDP :{CDP_PORT}...")
    async with async_playwright() as p:
        browser = await p.chromium.connect_over_cdp(CDP_URL)
        context = browser.contexts[0] or await browser.new_context()
        total_ok = total_pend = 0
        for ch in channels:
            pend = await get_pending_videos(ch)
            total_pend += len(pend)
            total_ok += await upload_channel(ch, browser, context)
        print(f"\n{'='*50}\n📊 TOTAL: {total_ok}/{total_pend}\n{'='*50}")

if __name__ == "__main__":
    asyncio.run(main())
