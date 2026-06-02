# Non-Vision Model Workaround for cua-driver

When the active model cannot process images (text-only models like DeepSeek V4 Flash, 
Llama, etc.), the `computer_use` tool's `capture` returns `0x0` dimensions and the 
SOM element tree appears empty/zeroed out. You must bypass the `computer_use` tool 
entirely and drive cua-driver via the terminal using `cua-driver call <tool> '<json>'`.

## Setup Check

Before any CLI calls, confirm the daemon is alive:

```bash
cua-driver status
# → "cua-driver daemon is running"
```

Also verify Hermes' MCP bridge is connected:

```bash
hermes mcp test cua-driver
# → ✓ Connected (…ms), ✓ Tools discovered: 29
```

If `hermes mcp test` fails, stop/restart the daemon:

```bash
cua-driver stop
cua-driver serve &
```

## Step-by-step Workflow (for a browser task)

### 1. Launch Safari (or any app) with a URL

```python
import subprocess, json
result = subprocess.run([
    'cua-driver', 'call', 'launch_app',
    json.dumps({"bundle_id": "com.apple.Safari", 
                "urls": ["https://example.com"]})
], capture_output=True, text=True, timeout=20)
data = json.loads(result.stdout)
pid = data["pid"]
# Pick the main content window (height > 100, is_on_screen=True)
content_window = [w for w in data["windows"] 
                  if w["bounds"]["height"] > 100 and w["is_on_screen"]][0]
window_id = content_window["window_id"]
```

### 2. Read the AX tree (find elements by label)

```python
result = subprocess.run([
    'cua-driver', 'call', 'get_window_state',
    json.dumps({"pid": pid, "window_id": window_id})
], capture_output=True, text=True, timeout=30)
data = json.loads(result.stdout)
tree = data["tree_markdown"]

# Search for the element you need
for line in tree.split('\n'):
    if 'Servicio' in line or 'Contacto' in line:
        print(line)
# → - [17] AXLink "Servicios" ...
```

### 3. Click by element index

```python
result = subprocess.run([
    'cua-driver', 'call', 'click',
    json.dumps({"pid": pid, "window_id": window_id, 
                "element_index": 17})
], capture_output=True, text=True, timeout=15)
# → ✅ Performed AXPress on [17] AXLink "Servicios"
```

### 4. Type text into the URL / search bar

Find the text field first via `get_window_state` (look for `WEB_BROWSER_ADDRESS_AND_SEARCH_FIELD` or `AXTextField`), then write:

```python
# set_value writes to AXValue (works for native text fields)
result = subprocess.run([
    'cua-driver', 'call', 'set_value',
    json.dumps({"pid": pid, "window_id": window_id,
                "element_index": 46, 
                "value": "https://example.com"})
], ...)

# Then press Return to navigate
result = subprocess.run([
    'cua-driver', 'call', 'press_key',
    json.dumps({"pid": pid, "key": "return"})
], ...)
```

### 5. Capture a screenshot and send it to the user

**Option A — Use `screencapture` with window bounds** (works every time):

```python
# Get window bounds from list_windows
result = subprocess.run(['cua-driver', 'call', 'list_windows', '{}'],
    capture_output=True, text=True, timeout=15)
data = json.loads(result.stdout)
for w in data["windows"]:
    if w["app_name"] == "Safari" and w["is_on_screen"] and w["bounds"]["height"] > 100:
        b = w["bounds"]
        # screencapture -R x,y,w,h
        subprocess.run([
            "screencapture", 
            f"-R{b['x']},{b['y']},{b['width']},{b['height']}",
            "-t", "png",
            "/tmp/screenshot.png"
        ])
# Then in your reply: MEDIA:/tmp/screenshot.png
```

**Option B — Try `screenshot` with `screenshot_out_file`** (sometimes doesn't save):

```bash
cua-driver call screenshot '{"window_id": 112, "screenshot_out_file": "/tmp/shot.png"}'
```

If the file isn't written, fall back to Option A.

## Key CLI tool names

| cua-driver tool | Purpose |
|---|---|
| `list_apps` | Find running apps by name, get PID |
| `list_windows` | Get all windows with bounds, on-screen status |
| `launch_app` | Start an app (pass `urls` for browsers) |
| `get_window_state` | Walk AX tree, get element indices |
| `click` | Click by element_index or [x, y] |
| `double_click` | Double-click (opens files in Finder, toggles fullscreen video) |
| `right_click` | Right-click by element_index or [x, y] |
| `press_key` | Single key (return, tab, escape, arrows, space) |
| `hotkey` | Key combo (cmd+s, cmd+shift+4) |
| `type_text` | Insert text into focused element |
| `set_value` | Set AXValue (dropdowns, sliders, native text fields) |
| `screenshot` | Capture window PNG (returns path or base64) |
| `scroll` | Scroll by page or line |
| `drag` | Drag from/to pixel coordinates |

## Known pitfalls

- **`set_value` on web text fields** — Safari WebKit often ignores AXValue writes. 
  Use `type_text` (character-by-key CGEvent fallback) instead.
- **`screenshot` returns text, not binary** — the `cua-driver call` CLI tool prints 
  a status message, not the raw PNG. Use `screencapture` for reliable file output.
- **`get_window_state` is slow** (~2-3s on modern web pages with 300+ AX elements). 
  Accept this latency — it's Safari's AX tree traversal, not the driver.
- **Element indices are per-snapshot** — call `get_window_state` fresh after every 
  page navigation or UI change. Stale indices cause silent no-ops.
- **`press_key` needs `window_id`** for NSMenu shortcuts (Cmd+S, Cmd+N) — 
  AppKit only routes menu-equivalent shortcuts to the active app. Without 
  `window_id`, the keystroke lands but the menu dispatch doesn't fire.
