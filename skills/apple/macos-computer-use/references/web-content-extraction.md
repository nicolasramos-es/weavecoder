# Web Content Extraction via cua-driver MCP tools

Pattern for extracting content from web apps (Gemini chat history, SaaS dashboards, etc.) using `mcp_cua_driver_page` tools — no vision required.

## Prerequisites

- Chrome with **"Allow JavaScript from Apple Events"** enabled (one-time)
- Session persistence: Chrome stays logged in if you use the user's main browser window

## Enable JS from Apple Events

```python
# ONE-TIME SETUP per Chrome process
mcp_cua_driver_page(
    action="enable_javascript_apple_events",
    bundle_id="com.google.Chrome",
    user_has_confirmed_enabling=True,
    pid=..., window_id=...
)
```

**⚠️ CRITICAL:** This action **relaunches Chrome** with a new PID. After calling it:
1. `list_apps()` to get Chrome's new PID
2. `list_windows(pid=new_pid)` to find the window
3. `get_window_state(pid=new_pid, window_id=...)` to inspect

## Extraction strategy

### Step 1 — Discover all conversation/page URLs

```javascript
// Execute in the page context
const links = document.querySelectorAll('a[href*="/app/"]');
const convs = [];
links.forEach(a => {
  const text = a.textContent?.trim();
  const href = a.getAttribute('href');
  if (text && text.length > 3 && href && !text.includes('Nueva') && !text.includes('Buscar')) {
    convs.push({title: text.substring(0, 100), url: href});
  }
});
return JSON.stringify(convs, null, 2);
```

Save the JSON list locally as a `.conversations_index.json` or similar for tracking extraction progress.

### Step 2 — Handle multi-account/webapp scenarios

Some web apps use URL-based account switching:
- Account A: `gemini.google.com/app/...` (default)
- Account B: `gemini.google.com/u/1/app/...` (secondary)
- Always verify which account you're in by checking the sidebar for the account name

**Pattern for checking account context:**
```python
mcp_cua_driver_get_window_state(pid=..., window_id=..., query="Cuenta de Google email")
# Look for the account link element in the response
```

If you need to switch accounts, prefer using `/u/N/` URL prefix in navigation, NOT clicking the account switcher UI (which can trigger logout flows).

### Step 3 — Navigate to each page

```javascript
window.location.href = 'https://webapp.example.com/u/1/app/CONVERSATION_ID';
```

Wait for load by calling `get_window_state()` and confirming the window title changed.

### Step 4 — Extract page content

**For text-heavy pages (conversations, articles):**
```python
result = mcp_cuda_driver_page(action="get_text", pid=..., window_id=...)
```

**For structured data (tables, lists):**
```python
result = mcp_cua_driver_page(
    action="query_dom",
    css_selector="table tr",
    attributes=["data-id"],
    pid=..., window_id=...
)
```

### Step 5 — Save to files

```python
from hermes_tools import write_file
write_file("/path/to/conversations/001-title.md", content)
```

## Multi-conversation batch extraction

For extracting N conversations:

1. Get the full list of URLs (Step 1)
2. Track progress with a `todo` list
3. For each URL:
   a. Navigate via `execute_javascript`
   b. Wait for page load (check title changed)
   c. Extract text via `get_text()`
   d. Save to file
4. If N is large (>20), delegate to a subagent for parallel work

**Known working pattern:** This technique was used to extract 30+ Gemini conversations from a personal account across a session. The key was using `/u/1/` URL prefix to stay in the right Google account.

## Multi-account web scraping (Gemini-specific)

Some web apps like Gemini support Google account switching via URL prefix:

- **Default account** (no prefix): `gemini.google.com/app/CONV_ID`
- **Secondary account** (`/u/1/`): `gemini.google.com/u/1/app/CONV_ID`

**Critical rule:** Navigating to a conversation URL **without** the `/u/N/` prefix while logged into the secondary account silently switches to the default account. Always verify which account you're in by checking the AX tree:

```python
result = mcp_cua_driver_get_window_state(
    pid=..., window_id=..., 
    query="Cuenta de Google"
)
# Look for Cuenta de Google: Nombre (email) in the response
```

**Prefixed navigation pattern:**
```javascript
window.location.href = 'https://gemini.google.com/u/1/app/CONVERSATION_ID';
```

## Finding ALL conversations (not just recent)

**Problem:** The sidebar only shows ~30 recent conversations. To find the full history, navigate to the library view.

**For conversation list:**
```javascript
window.location.href = 'https://gemini.google.com/u/1/library';
```
Then query all conversation links:
```javascript
document.querySelectorAll('a[href*="/app/"]')
```

**For generated documents (Deep Research, etc.):**
```javascript
window.location.href = 'https://gemini.google.com/u/1/library/documents';
```

**For generated images/media:**
Navigate to the library and find the "Contenido multimedia" section. Images are lazy-loaded and may not be directly downloadable via JS — consider using Google Takeout for bulk media export.

**Counting unique conversations vs sidebar noise:**
The sidebar renders multiple DOM elements per conversation (the title link + a "More options" button). Count **unique href values** to get the real number:
```javascript
const unique = new Set();
document.querySelectorAll('a[href*="/app/"]').forEach(a => {
  const text = a.textContent?.trim();
  if (text && text.length > 2 && /* filter UI elements */) {
    unique.add(a.getAttribute('href').split('?')[0]);
  }
});
// unique.size is the real conversation count
```

## Filtering `get_text()` output noise

Gemini's `get_text()` returns the **entire visible text** including sidebar items (Nueva conversación, Buscar conversaciones, Bibliotecha, etc.), account info, and every conversation title in the sidebar mixed with the active conversation's content.

**To identify the active conversation's content:** Look for the headers `"Has dicho"` (user said) and `"Gemini ha dicho"` (Gemini said) as delimiters between user and AI turns.

## Using `get_window_state(query=...)` to find specific elements

The `query` parameter filters the AX tree to matching lines + their ancestor chain, saving context when you only need to find one button/link:

```python
# Find account info in the sidebar
result = mcp_cua_driver_get_window_state(
    pid=..., window_id=...,
    query="Cuenta de Google Nicolás"
)
# tree_markdown is filtered to relevant lines only

# Find a specific conversation title
result = mcp_cua_driver_get_window_state(
    pid=..., window_id=...,
    query="Spacebot Configuración"
)
```

## Batch extraction at scale (30+ pages)

For extracting 30+ pages, the sequential navigation pattern (`navigate → wait → get_text → save → next`) works but is slow (50+ tool calls). Strategies to accelerate:

1. **Delegate to a subagent** with `delegate_task()` — the subagent gets its own context window and can make up to 50 tool calls without exhausting the parent's context.
2. **Pass ALL conversation URLs in context** so the subagent doesn't need to rediscover them.
3. **Use Playwright + CDP** if the user's Chrome has remote debugging enabled (port 9222) — dramatically faster for batch page scraping via `page.content()` or `page.evaluate()`.

## Pitfalls

- **URL prefix matters** — Some web apps use `/u/N/` for account switching. Navigating to a conversation URL without the prefix switches to the default account. Always include the correct prefix and verify with `query="Cuenta de Google"`.
- **Page load timing** — After `window.location.href = url`, the JS returns immediately but the page hasn't loaded yet. Call `get_window_state()` to confirm the new page is rendered before calling `get_text()`.
- **"Conversación con Gemini" in get_text()** — The conversation heading is always "Conversación con Gemini" regardless of the actual title. The actual title is only in the sidebar link, not in the page content.
- **Rate limiting** — Gemini may rate-limit rapid navigation. Space out navigations with short waits between them.
- **Session timeout** — Long extraction sessions (>50 pages) may trigger re-authentication. Save progress every ~20 conversations.
- **`get_window_state` stalls on large trees** — Gemini pages can have 2000+ AX elements, causing high latency and large responses. Use `query=...` to filter, or use `page(action="get_text")` instead for text extraction.
- **Subagent cannot see the user's browser** — Subagents spawned via `delegate_task()` have isolated contexts and no access to the parent's Chrome/cua-driver session. The parent MUST do all browser interaction; the subagent can only process already-extracted data.