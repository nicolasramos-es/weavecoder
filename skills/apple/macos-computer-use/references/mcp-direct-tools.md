# MCP-Level cua-driver Tools (Alternative to `computer_use`)

When the `computer_use` tool is available but your model cannot process images (returns `0x0` on capture), you can bypass it entirely and use the **MCP-level cua-driver tools** directly. These are first-class Hermes tools named `mcp_cua_driver_*` and are **much more ergonomic** than the terminal-based workaround (`cua-driver call ...`).

## Available MCP tools

| Tool | Purpose | Best for |
|---|---|---|
| `mcp_cua_driver_get_window_state` | Walk AX tree + screenshot of one window | Finding element indices, reading page structure |
| `mcp_cua_driver_page` | JS execution, innerText, DOM queries in browser | Extracting text content, running JavaScript |
| `mcp_cua_driver_click` | Click by element index or [x, y] | Interacting with UI elements |
| `mcp_cua_driver_type_text` | Insert text at cursor | Filling forms, typing URLs |
| `mcp_cua_driver_press_key` | Single keypress (return, tab, escape) | Navigation, form submission |
| `mcp_cua_driver_hotkey` | Key combo (cmd+s, cmd+shift+4) | Shortcuts |
| `mcp_cua_driver_scroll` | Scroll viewport by page or line | Revealing content |
| `mcp_cua_driver_set_value` | Set AXValue (dropdowns, sliders) | Selecting from popups |
| `mcp_cua_driver_screenshot` | Capture window PNG | Getting a visual (stored as base64 or file) |
| `mcp_cua_driver_list_apps` | Get running apps with PIDs | Finding the target app |
| `mcp_cua_driver_list_windows` | Get all windows with bounds | Identifying windows by title |
| `mcp_cua_driver_launch_app` | Start an app in background | Opening browsers, native apps |
| `mcp_cua_driver_zoom` | Zoom into a screenshot region at full resolution | Reading small text in screenshots |

## Key difference from `computer_use`

The `computer_use` tool is a single high-level abstraction that manages capture → click → verify cycles. The MCP tools are lower-level — you manage the cycle yourself:

1. `launch_app` or `list_windows` → get PID + window_id
2. `get_window_state(pid, window_id)` → get element indices + AX tree text
3. `click(pid, window_id, element_index=N)` → interact
4. Repeat step 2 to re-snapshot after state changes

The **element index cache is scoped per (pid, window_id)** — indices from one window do NOT resolve against another. Always `get_window_state` fresh for the correct pair.

## Browser interaction (the `page` tool)

The `mcp_cua_driver_page` tool is unique to MCP tools — there is no equivalent in `computer_use`. Three actions:

| Action | What it does | When to use |
|---|---|---|
| `get_text` | Returns `document.body.innerText` | Extracting the full text content of a page (articles, chat histories, search results) |
| `execute_javascript` | Runs JS in page context, returns result | Navigating, reading page state, clicking via JS, extracting data |
| `query_dom` | `querySelectorAll(css)` → JSON array of matches | Getting structured data (table rows, links, attributes) |

### Prerequisite: "Allow JavaScript from Apple Events"

Chrome blocks JS execution by default. Enable it once per Chrome session:

```
mcp_cua_driver_page(
  action="enable_javascript_apple_events",
  bundle_id="com.google.Chrome",
  user_has_confirmed_enabling=true
)
```

⚠️ This **quits and relaunches Chrome** — the PID changes, so you must re-discover windows with `list_windows(pid=new_pid)`.

### Navigation pattern (SPA apps like Gemini)

For single-page apps that change content via URL routing:

```python
# Navigate via JS
mcp_cua_driver_page(pid=pid, window_id=wid,
  action="execute_javascript",
  javascript="window.location.href = 'https://gemini.google.com/u/1/app/xxxxx'")

# Wait briefly, then get the text
mcp_cua_driver_page(pid=pid, window_id=wid, action="get_text")
```

**Pitfall:** SPAs may load asynchronously. If `get_text` returns empty or stale content, wait 2-3 seconds before reading. You can use `execute_javascript` with a timeout or poll `document.readyState`.

## Complete workflow for extracting data from a web app (e.g., Gemini chat history)

This is the pattern we used to extract 30 Gemini conversations.

### ⚠️ Google Takeout does NOT include Gemini conversations

Google Takeout's "Gemini" option only exports activity data (usage logs), not actual conversation transcripts. The user confirmed this explicitly: **Option B (manual browser extraction) is the only way** to get full Gemini conversation content. Accept this and proceed with browser-based extraction — do not suggest Takeout again for Gemini data unless Google changes their export policy.

### Phase 1: Setup

```python
# 1. Launch Chrome to the target URL
result = mcp_cua_driver_launch_app(
  bundle_id="com.google.Chrome",
  urls=["https://gemini.google.com/u/1/app"])
pid = result["pid"]
wid = result["windows"][-1]["window_id"]  # Main content window

# 2. Enable JavaScript from Apple Events
mcp_cua_driver_page(
  action="enable_javascript_apple_events",
  bundle_id="com.google.Chrome",
  user_has_confirmed_enabling=True)
# → Chrome relaunches, new PID!

# 3. Re-find windows
new_pid = ... # From list_apps
windows = mcp_cua_driver_list_windows(pid=new_pid)
wid = ... # The "Google Gemini" window
```

### Phase 2: Get list of conversations

```javascript
// Run in the browser
document.querySelectorAll('a[href*="/app/"]').forEach(a => {
  const text = a.textContent?.trim();
  if (text && text.length > 3 && !text.includes('Nueva') && !text.includes('Buscar')) {
    convs.push({title: text, url: a.href});
  }
});
```

### Phase 3: Extract each conversation

```python
for url in conversation_urls:
    # Navigate
    mcp_cua_driver_page(pid=pid, window_id=wid,
      action="execute_javascript",
      javascript=f"window.location.href = '{url}'")
    
    # Wait for load
    time.sleep(3)
    
    # Get content
    text = mcp_cua_driver_page(pid=pid, window_id=wid, action="get_text")
    
    # Save to file
    write_file(f"/path/to/{filename}.md", text)
```

### Phase 4: Account management

If your user has multiple Google accounts:
- Account 0 (default): `gemini.google.com/app/...`
- Account 1 (switched): `gemini.google.com/u/1/app/...`
- Account 2: `gemini.google.com/u/2/app/...`

The URL prefix `/u/N/` controls which account is active. Navigating to a URL **without** the prefix switches back to the default account. Always include the `/u/N/` prefix when working with a non-default account.

### Phase 5: Parallel extraction via delegate_task

For large numbers of conversations (30+), delegate the navigation + extraction to a subagent to avoid exhausting your own context window:

```
delegate_task(
  goal="Navigate to each Gemini conversation URL using cua-driver page tools and extract the text",
  context="Chrome PID X, window_id Y. URLs with /u/1/ prefix for personal account.",
  toolsets=["computer_use"]
)
```

The subagent needs the `computer_use` toolset to access `mcp_cua_driver_*` tools.

### Phase 6: Process with a local model (privacy-preserving)

If the user wants processing/summarization to stay local (no data leaves their network):
1. Configure a `custom_providers` entry pointing to their local inference server
2. After extraction, use that provider to process the saved files
3. See `hermes-provider-configuration` skill for setup details