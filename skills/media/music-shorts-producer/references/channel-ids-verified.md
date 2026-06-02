# NR Music — Channel IDs (verificados 2026-05-31)

IDs obtenidos desde la metadata de cada página de canal en YouTube.

| Canal | Handle | Channel ID |
|-------|--------|-----------|
| 🎵 NR Pop | @NRMusicPop | `UCoMn2PNx_wdhXkazzLeZJ4w` |
| 🎸 NR Rock | @NRMusicRock | `UCiimUjAf4EgmNaYNwtpTcTw` |
| 🎤 NR Hip-Hop | @NRMusicHipHop | `UCvjaeHB6AvJ4HVSjifOb76w` |
| 💃 NR Latino | @NRMusicLatino | `UC2OvxQ76X6hA5nYV-xZdD0g` |

## Cómo navegar al Studio de cada canal

```python
studio_url = f"https://studio.youtube.com/channel/{channel_id}/videos"
await page.goto(studio_url, wait_until="domcontentloaded")
```

## Cómo obtener un channel_id desde un handle

```python
await page.goto(f"https://www.youtube.com/{handle}", wait_until="domcontentloaded")
await asyncio.sleep(5)
channel_id = await page.evaluate('''
    () => {
        const scripts = document.querySelectorAll("script");
        for (const s of scripts) {
            const m = s.textContent.match(/"channelId":"(UC[\w-]+)"/);
            if (m) return m[1];
            const m2 = s.textContent.match(/"externalId":"(UC[\w-]+)"/);
            if (m2) return m2[1];
        }
        return null;
    }
''')
```

## Directorios de trabajo

| Canal | Carpeta | Script orquestador |
|-------|--------|-------------------|
| Pop | `~/nr-pop/` | `orchestrate_nrpop_v2.py` |
| Rock | `~/nr-rock/` | `orchestrate_nrrock_v2.py` |
| Hip-Hop | `~/nr-hiphop/` | `orchestrate_nrhiphop_v2.py` |
| Latino | `~/nr-latino/` | `orchestrate_nrlatino_v2.py` |
