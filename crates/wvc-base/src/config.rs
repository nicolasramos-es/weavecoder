//! Configuration file support for wvc
//!
//! Config is loaded from `~/.wvc/config.toml` (or `$WVC_HOME/config.toml`)
//! Environment variables override config file settings.

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::hash::{Hash, Hasher};
use std::path::PathBuf;
use std::sync::{LazyLock, RwLock};
use std::time::{Duration, Instant, SystemTime};
pub use wvc_config_types::{
    AgentsConfig, AmbientConfig, AuthConfig, AutoJudgeConfig, AutoReviewConfig, CompactionConfig,
    CompactionMode, CrossProviderFailoverMode, DiagramDisplayMode, DiagramPanePosition,
    DiffDisplayMode, DisplayConfig, FeatureConfig, GatewayConfig, HooksConfig, KeybindingsConfig,
    LatexRenderingMode, LaunchHotkeyEntry, LaunchHotkeysConfig, MarkdownSpacingMode,
    NamedProviderAuth, NamedProviderConfig, NamedProviderModelConfig, NamedProviderType,
    NativeScrollbarConfig, NotificationsConfig, OverscrollStatusMode, PowerConfig, ProviderConfig,
    ReasoningDisplayMode, SafetyConfig, SessionPickerResumeAction, SponsorsConfig, SwarmSpawnMode,
    SwarmStripLayout, TerminalConfig, UpdateChannel, WebSearchConfig, WebSearchEngine,
};

const CONFIG_CACHE_CHECK_INTERVAL: Duration = if cfg!(test) {
    Duration::ZERO
} else {
    Duration::from_millis(500)
};

const CONFIG_ENV_KEYS: &[&str] = &[
    "HOME",
    "WVC_ACP_PROFILE",
    "WVC_ACP_TOOL_PROFILE",
    "WVC_ACTIVE_SESSIONS_MANAGER",
    "WVC_EXTERNAL_SESSIONS",
    "WVC_AMBIENT_ENABLED",
    "WVC_AMBIENT_MAX_INTERVAL",
    "WVC_AMBIENT_MIN_INTERVAL",
    "WVC_AMBIENT_MODEL",
    "WVC_AMBIENT_PROACTIVE",
    "WVC_AMBIENT_PROVIDER",
    "WVC_AMBIENT_VISIBLE",
    "WVC_ANIMATION_FPS",
    "WVC_AUTO_POKE",
    "WVC_AUTOJUDGE_ENABLED",
    "WVC_AUTOJUDGE_MODEL",
    "WVC_AUTOREVIEW_ENABLED",
    "WVC_AUTOREVIEW_MODEL",
    "WVC_AUTO_POKE",
    "WVC_AUTO_SERVER_RELOAD",
    "WVC_BING_API_KEY",
    "WVC_BING_API_KEY_ENV",
    "WVC_BING_MARKET",
    "WVC_CENTERED_TOGGLE_KEY",
    "WVC_CHAT_NATIVE_SCROLLBAR",
    "WVC_COMPACT_NOTIFICATIONS",
    "WVC_COPY_BADGE_ALT_LABEL",
    "WVC_COPY_SELECTION_TOGGLE_KEY",
    "WVC_COPILOT_PREMIUM",
    "WVC_CROSS_PROVIDER_FAILOVER",
    "WVC_DEBUG_SOCKET",
    "WVC_DEFAULT_REASONING_DISPLAY",
    "WVC_DICTATION_COMMAND",
    "WVC_DICTATION_KEY",
    "WVC_DICTATION_MODE",
    "WVC_DICTATION_TIMEOUT_SECS",
    "WVC_DIFF_LINE_WRAP",
    "WVC_DIFF_MODE",
    "WVC_DIFF_MODE_CYCLE_KEY",
    "WVC_DIAGRAM_PANE_TOGGLE_KEY",
    "WVC_DISABLE_BASE_TOOLS",
    "WVC_DISABLED_ANIMATIONS",
    "WVC_DISABLED_TOOLS",
    "WVC_DISCORD_BOT_TOKEN",
    "WVC_DISCORD_BOT_USER_ID",
    "WVC_DISCORD_CHANNEL_ID",
    "WVC_DISCORD_REPLY_ENABLED",
    "WVC_DISPLAY_CENTERED",
    "WVC_EFFORT_DECREASE_KEY",
    "WVC_EFFORT_INCREASE_KEY",
    "WVC_EMAIL_REPLY_ENABLED",
    "WVC_EMAIL_TO",
    "WVC_FOCUS_HOOK",
    "WVC_GATEWAY_BIND_ADDR",
    "WVC_GATEWAY_ENABLED",
    "WVC_GATEWAY_PORT",
    "WVC_HOME",
    "WVC_HOOK_PRE_TOOL",
    "WVC_HOOK_PRE_TOOL_TIMEOUT_MS",
    "WVC_HOOK_POST_TOOL",
    "WVC_HOOK_SESSION_END",
    "WVC_HOOK_SESSION_START",
    "WVC_HOOK_TURN_END",
    "WVC_HOOK_TURN_START",
    "WVC_IDLE_ANIMATION",
    "WVC_IMAP_HOST",
    "WVC_INFO_WIDGET_TOGGLE_KEY",
    "WVC_JADE_RELAY_API_BASE",
    "WVC_JADE_RELAY_ENABLED",
    "WVC_JADE_RELAY_LAUNCH_ENABLED",
    "WVC_JADE_RELAY_LAUNCH_WORKING_DIR",
    "WVC_JADE_RELAY_REPLY_ENABLED",
    "WVC_JADE_RELAY_SESSION_ID",
    "WVC_JADE_RELAY_TOKEN",
    "WVC_JADE_RELAY_TOKEN_ID",
    "WVC_JADE_RELAY_USER_ID",
    "WVC_KV_CACHE_MISS_NOTICES",
    "WVC_LATEX_RENDERING",
    "WVC_MARKDOWN_SPACING",
    "WVC_MEMORY_EMBEDDING_BACKEND",
    "WVC_MEMORY_EMBEDDING_BASE_URL",
    "WVC_MEMORY_EMBEDDING_DIM",
    "WVC_MEMORY_EMBEDDING_MODEL",
    "WVC_MEMORY_ENABLED",
    "WVC_ENABLE_MERMAID",
    "WVC_MEMORY_MODEL",
    "WVC_MEMORY_SIDECAR_ENABLED",
    "WVC_PERSIST_MEMORY_INJECTIONS",
    "WVC_MESSAGE_TIMESTAMPS",
    "WVC_MODEL",
    "WVC_MODEL_SWITCH_KEY",
    "WVC_MODEL_SWITCH_PREV_KEY",
    "WVC_MOUSE_CAPTURE",
    "WVC_NEW_TERMINAL_KEY",
    "WVC_NO_EMOJI",
    "WVC_NTFY_SERVER",
    "WVC_NTFY_TOPIC",
    "WVC_OPENAI_NATIVE_COMPACTION_MODE",
    "WVC_OPENAI_NATIVE_COMPACTION_THRESHOLD_TOKENS",
    "WVC_OPENAI_REASONING_EFFORT",
    "WVC_OPENAI_SERVICE_TIER",
    "WVC_OPENAI_TRANSPORT",
    "WVC_ANTHROPIC_REASONING_EFFORT",
    "WVC_PRESERVE_REASONING_CONTEXT",
    "WVC_PERFORMANCE",
    "WVC_PIN_IMAGES",
    "WVC_PIN_TODOS",
    "WVC_PREVENT_SLEEP_WHILE_STREAMING",
    "WVC_PROVIDER",
    "WVC_PROMPT_ENTRY_ANIMATION",
    "WVC_QUEUE_MODE",
    "WVC_REASONING_DISPLAY",
    "WVC_REDRAW_FPS",
    "WVC_SAME_PROVIDER_ACCOUNT_FAILOVER",
    "WVC_SCROLL_BOOKMARK_KEY",
    "WVC_SCROLL_DOWN_FALLBACK_KEY",
    "WVC_SCROLL_DOWN_KEY",
    "WVC_SCROLL_PAGE_DOWN_KEY",
    "WVC_SCROLL_PAGE_UP_KEY",
    "WVC_SCROLL_PROMPT_DOWN_KEY",
    "WVC_SCROLL_PROMPT_UP_KEY",
    "WVC_SCROLL_UP_FALLBACK_KEY",
    "WVC_SCROLL_UP_KEY",
    "WVC_SEARXNG_URL",
    "WVC_SHOW_AGENTGREP_OUTPUT",
    "WVC_SHOW_DIFFS",
    "WVC_SHOW_THINKING",
    "WVC_SIDE_PANEL_TOGGLE_KEY",
    "WVC_SIDE_PANEL_NATIVE_SCROLLBAR",
    "WVC_SMTP_PASSWORD",
    "WVC_SPAWN_HOOK",
    "WVC_STREAM_IDLE_TIMEOUT_SECS",
    "WVC_SWARM_ENABLED",
    "WVC_SWARM_MODEL",
    "WVC_SWARM_MAX_CONCURRENT_AGENTS",
    "WVC_SWARM_SPAWN_MODE",
    "WVC_SWARM_STRIP_LAYOUT",
    "WVC_TELEGRAM_BOT_TOKEN",
    "WVC_TELEGRAM_CHAT_ID",
    "WVC_TELEGRAM_REPLY_ENABLED",
    "WVC_TOOL_CALL_DETAILS",
    "WVC_TOOL_PROFILE",
    "WVC_TOOLS",
    "WVC_TRUSTED_EXTERNAL_AUTH_SOURCES",
    "WVC_TYPING_SCROLL_LOCK_TOGGLE_KEY",
    "WVC_UPDATE_CHANNEL",
    "WVC_WEBSEARCH_ENGINE",
    "WVC_WEBSEARCH_FALLBACK_ENGINES",
    "WVC_WORKSPACE_DOWN_KEY",
    "WVC_WORKSPACE_LEFT_KEY",
    "WVC_WORKSPACE_RIGHT_KEY",
    "WVC_WORKSPACE_UP_KEY",
    "XDG_CONFIG_HOME",
];

#[derive(Debug, Clone, PartialEq, Eq)]
struct ConfigCacheFingerprint {
    path: Option<PathBuf>,
    modified: Option<SystemTime>,
    len: Option<u64>,
    env: Vec<(String, String)>,
}

impl ConfigCacheFingerprint {
    fn current() -> Self {
        let path = Config::path();
        let metadata = path.as_ref().and_then(|path| std::fs::metadata(path).ok());
        Self {
            path,
            modified: metadata
                .as_ref()
                .and_then(|metadata| metadata.modified().ok()),
            len: metadata.as_ref().map(std::fs::Metadata::len),
            env: config_env_fingerprint(),
        }
    }
}

struct ConfigCache {
    config: &'static Config,
    fingerprint: ConfigCacheFingerprint,
    last_checked: Instant,
    force_reload: bool,
}

static CONFIG_CACHE: LazyLock<RwLock<ConfigCache>> = LazyLock::new(|| {
    let config = leak_config(Config::load());
    // Fingerprint after the load: applying env overrides may set env vars
    // (e.g. copilot_premium -> WVC_COPILOT_PREMIUM), and fingerprinting
    // first would guarantee a spurious full reload on the next check.
    let fingerprint = ConfigCacheFingerprint::current();
    // Seed the global context-limit cache from named provider configs on first
    // load so every codepath (TUI info widget, compaction budget, model
    // switching) sees user-configured `context_window` values from the start.
    // Read from the loaded config directly to avoid recursing into config(),
    // which would deadlock on the still-initializing CONFIG_CACHE.
    populate_context_limits_from_config_ref(config);
    RwLock::new(ConfigCache {
        config,
        fingerprint,
        last_checked: Instant::now(),
        force_reload: false,
    })
});

fn leak_config(config: Config) -> &'static Config {
    Box::leak(Box::new(config))
}

/// Seed the global context-limit cache from a config reference directly.
///
/// Used during CONFIG_CACHE initialization (where calling config() would
/// deadlock) and shares its logic with
/// `crate::provider::populate_context_limits_from_config`.
fn populate_context_limits_from_config_ref(cfg: &Config) {
    crate::provider::populate_context_limits_from_config_value(cfg);
}

/// Get the global config instance.
///
/// The returned reference is backed by a reloadable process cache. Calls check
/// the config file path/metadata and relevant environment overrides on a short
/// throttle, not every frame. When those inputs change, the next checked call
/// reloads config.toml and invalidates dependent auth/model caches. Older
/// references remain valid for the duration of any in-flight operation.
pub fn config() -> &'static Config {
    let now = Instant::now();
    if let Ok(cache) = CONFIG_CACHE.read()
        && !cache.force_reload
        && now.duration_since(cache.last_checked) < CONFIG_CACHE_CHECK_INTERVAL
    {
        return cache.config;
    }

    let mut reload_reason = None;
    let config = {
        let mut cache = CONFIG_CACHE
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner());

        let now = Instant::now();
        if !cache.force_reload
            && now.duration_since(cache.last_checked) < CONFIG_CACHE_CHECK_INTERVAL
        {
            return cache.config;
        }

        let fingerprint = ConfigCacheFingerprint::current();
        cache.last_checked = now;
        if cache.force_reload || cache.fingerprint != fingerprint {
            reload_reason = Some(describe_config_reload(
                cache.force_reload,
                &cache.fingerprint,
                &fingerprint,
            ));
            cache.config = leak_config(Config::load());
            // Loading applies env overrides that can themselves set env vars
            // (e.g. copilot_premium propagates config -> WVC_COPILOT_PREMIUM).
            // Re-fingerprint after the load so those self-inflicted env changes
            // don't trigger a guaranteed second reload on the next check.
            cache.fingerprint = ConfigCacheFingerprint::current();
            cache.force_reload = false;
        }
        cache.config
    };

    if let Some(reason) = reload_reason {
        crate::logging::info(&format!("CONFIG_RELOAD {}", reason));
        // A config reload can change config-derived system prompt sections
        // (feature toggles, sponsors, ...), which legitimately invalidates the
        // KV cache prefix of warm sessions. Document it so a subsequent
        // harness-attributed cache miss is surfaced with this cause instead of
        // as an unexplained prompt mutation.
        crate::cache_invalidation::record("config reload", &reason);
        notify_config_reloaded();
        // Re-seed the global context-limit cache so user edits to named
        // provider `context_window` values take effect without a restart.
        crate::provider::populate_context_limits_from_config();
    }

    config
}

fn describe_config_reload(
    forced: bool,
    previous: &ConfigCacheFingerprint,
    next: &ConfigCacheFingerprint,
) -> String {
    let mut parts = Vec::new();
    if forced {
        parts.push("forced=true".to_string());
    }
    if previous.path != next.path {
        parts.push(format!(
            "path={:?}->{:?}",
            previous.path.as_ref().map(|p| p.display().to_string()),
            next.path.as_ref().map(|p| p.display().to_string())
        ));
    }
    if previous.modified != next.modified {
        parts.push("modified_changed=true".to_string());
    }
    if previous.len != next.len {
        parts.push(format!("len={:?}->{:?}", previous.len, next.len));
    }
    let env_changes = describe_env_changes(&previous.env, &next.env);
    if !env_changes.is_empty() {
        parts.push(format!("env=[{}]", env_changes.join(", ")));
    }
    if parts.is_empty() {
        "unchanged".to_string()
    } else {
        parts.join(" ")
    }
}

fn describe_env_changes(previous: &[(String, String)], next: &[(String, String)]) -> Vec<String> {
    let previous_map: BTreeMap<&str, &str> = previous
        .iter()
        .map(|(key, value)| (key.as_str(), value.as_str()))
        .collect();
    let next_map: BTreeMap<&str, &str> = next
        .iter()
        .map(|(key, value)| (key.as_str(), value.as_str()))
        .collect();
    let keys: BTreeSet<&str> = previous_map
        .keys()
        .chain(next_map.keys())
        .copied()
        .collect();

    keys.into_iter()
        .filter_map(|key| match (previous_map.get(key), next_map.get(key)) {
            (Some(previous), Some(next)) if previous != next => Some(format!(
                "{}:changed({}->{})",
                key,
                env_value_fingerprint(previous),
                env_value_fingerprint(next)
            )),
            (None, Some(next)) => Some(format!("{}:added({})", key, env_value_fingerprint(next))),
            (Some(previous), None) => Some(format!(
                "{}:removed({})",
                key,
                env_value_fingerprint(previous)
            )),
            _ => None,
        })
        .collect()
}

fn env_value_fingerprint(value: &str) -> String {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    value.hash(&mut hasher);
    format!("len:{} hash:{:016x}", value.len(), hasher.finish())
}

fn config_env_fingerprint() -> Vec<(String, String)> {
    let mut values = std::env::vars_os()
        .filter_map(|(key, value)| {
            let key = key.to_string_lossy().to_string();
            if CONFIG_ENV_KEYS.contains(&key.as_str()) {
                Some((key, value.to_string_lossy().to_string()))
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    values.sort_by(|left, right| left.0.cmp(&right.0));
    values
}

pub fn invalidate_config_cache() {
    let mut cache = CONFIG_CACHE
        .write()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    cache.force_reload = true;
    drop(cache);
    notify_config_reloaded();
}

fn notify_config_reloaded() {
    CONFIG_RELOAD_GENERATION.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    for listener in CONFIG_RELOAD_LISTENERS
        .read()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .iter()
    {
        listener();
    }
}

/// Monotonic counter bumped every time the config cache reloads.
///
/// Callers that snapshot config-derived state (e.g. the TUI's parsed
/// keybindings) can poll this cheaply and re-derive their snapshot when the
/// generation changes, giving instant hot-reload of config edits without a
/// restart.
static CONFIG_RELOAD_GENERATION: std::sync::atomic::AtomicU64 =
    std::sync::atomic::AtomicU64::new(0);

/// Current config reload generation. Increments after every cache reload.
pub fn config_reload_generation() -> u64 {
    CONFIG_RELOAD_GENERATION.load(std::sync::atomic::Ordering::Relaxed)
}

/// Listeners invoked after the config cache reloads.
///
/// Config is a foundational module, so instead of reaching up into higher-level
/// subsystems (auth cache, event bus) on reload, those subsystems register a
/// reaction here at startup. This keeps config free of upward dependencies and
/// breaks the config -> auth / config -> bus cycle edges.
/// Type of a config reload listener callback.
type ConfigReloadListener = fn();

static CONFIG_RELOAD_LISTENERS: LazyLock<RwLock<Vec<ConfigReloadListener>>> =
    LazyLock::new(|| RwLock::new(Vec::new()));

/// Register a callback to run after the config cache reloads.
///
/// Callbacks must be cheap and non-blocking; they run on whichever thread
/// triggers the reload. Intended to be called once per subsystem during
/// process startup.
pub fn on_config_reloaded(listener: fn()) {
    CONFIG_RELOAD_LISTENERS
        .write()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .push(listener);
}

/// Main configuration struct
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct Config {
    /// Keybinding configuration
    pub keybindings: KeybindingsConfig,

    /// External dictation / speech-to-text integration
    pub dictation: DictationConfig,

    /// Display/UI configuration
    pub display: DisplayConfig,

    /// Feature toggles
    pub features: FeatureConfig,

    /// Web search tool configuration
    pub websearch: WebSearchConfig,

    /// Built-in tool exposure configuration
    pub tools: ToolConfig,

    /// Agent Client Protocol adapter configuration
    pub acp: AcpConfig,

    /// Auth trust / consent configuration
    pub auth: AuthConfig,

    /// Provider configuration
    pub provider: ProviderConfig,

    /// Named provider profiles, keyed by profile name.
    ///
    /// Example:
    /// [providers.my-gateway]
    /// type = "openai-compatible"
    /// base_url = "https://llm.example.com/v1"
    /// api_key_env = "MY_GATEWAY_API_KEY"
    pub providers: BTreeMap<String, NamedProviderConfig>,

    /// Agent-specific model defaults
    pub agents: AgentsConfig,

    /// Terminal window/pane spawning configuration
    pub terminal: TerminalConfig,

    /// Lifecycle hooks (external commands at turn/session/tool boundaries)
    pub hooks: HooksConfig,

    /// Ambient mode configuration
    pub ambient: AmbientConfig,

    /// Safety / notification configuration
    pub safety: SafetyConfig,

    /// Desktop notifications for interactive sessions (e.g. turn completion)
    pub notifications: NotificationsConfig,

    /// WebSocket gateway configuration (for iOS/web clients)
    pub gateway: GatewayConfig,

    /// Compaction configuration
    pub compaction: CompactionConfig,

    /// Power-management configuration (prevent sleep while streaming)
    pub power: PowerConfig,

    /// Auto-review configuration
    pub autoreview: AutoReviewConfig,

    /// Auto-judge configuration
    pub autojudge: AutoJudgeConfig,

    /// Partner discovery configuration. Skipped when it matches the shipped
    /// default so saving config never bakes today's default into the file (see
    /// [`sponsors_is_default`]).
    #[serde(skip_serializing_if = "sponsors_is_default")]
    pub sponsors: SponsorsConfig,

    /// Global "launch a new wvc" hotkeys (macOS). Baked once by auto-import.
    pub launch_hotkeys: LaunchHotkeysConfig,
}

/// Agent Client Protocol adapter configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AcpConfig {
    /// Client compatibility profile: "standard" (default), "extended", or "full".
    pub profile: String,
    /// Tool profile to request when `wvc acp` starts a daemon itself.
    pub tool_profile: String,
}

impl Default for AcpConfig {
    fn default() -> Self {
        Self {
            profile: "standard".to_string(),
            tool_profile: "acp".to_string(),
        }
    }
}

/// Controls which tools are sent to the model.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct ToolConfig {
    /// Tool profile: "full" (default), "acp", "minimal"/"lite", or "none".
    pub profile: String,
    /// Explicit allow-list. When set, only these tools are exposed.
    /// Use "*" or "all" to expose all tools without an allow-list.
    pub enabled: Vec<String>,
    /// Tools to remove after applying profile/enabled.
    pub disabled: Vec<String>,
    /// Disable all built-in tools unless `enabled` is provided.
    pub disable_base_tools: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ToolSelection {
    pub allowed_tools: Option<HashSet<String>>,
    pub disabled_tools: HashSet<String>,
}

impl ToolConfig {
    pub fn selection(&self) -> ToolSelection {
        let mut allowed_tools = self.base_allowed_tools();
        let disabled_tools: HashSet<String> = self
            .disabled
            .iter()
            .map(|name| normalize_tool_name(name))
            .filter(|name| !name.is_empty())
            .collect();

        if let Some(allowed) = allowed_tools.as_mut() {
            for name in &disabled_tools {
                allowed.remove(name);
            }
        }

        ToolSelection {
            allowed_tools,
            disabled_tools,
        }
    }

    pub fn allowed_tools(&self) -> Option<HashSet<String>> {
        self.selection().allowed_tools
    }

    pub fn apply_to_allowed_set(&self, allowed: &mut HashSet<String>) {
        let selection = self.selection();
        if let Some(global_allowed) = selection.allowed_tools {
            allowed.retain(|name| global_allowed.contains(name));
        }
        for disabled in selection.disabled_tools {
            allowed.remove(&disabled);
        }
    }

    fn base_allowed_tools(&self) -> Option<HashSet<String>> {
        let (explicit, enables_all_tools) = self.normalized_enabled_tools();

        let profile = self.profile.trim().to_ascii_lowercase();
        if enables_all_tools {
            None
        } else if !explicit.is_empty() {
            Some(explicit)
        } else if self.disable_base_tools || matches!(profile.as_str(), "none" | "off" | "disabled")
        {
            Some(HashSet::new())
        } else if matches!(profile.as_str(), "acp") {
            Some(
                [
                    "bash",
                    "read",
                    "write",
                    "edit",
                    "multiedit",
                    "apply_patch",
                    "patch",
                    "agentgrep",
                    "ls",
                    "batch",
                ]
                .into_iter()
                .map(|name| name.to_string())
                .collect(),
            )
        } else if matches!(profile.as_str(), "minimal" | "lite" | "small") {
            Some(
                [
                    "bash",
                    "read",
                    "write",
                    "edit",
                    "multiedit",
                    "apply_patch",
                    "patch",
                    "agentgrep",
                    "ls",
                ]
                .into_iter()
                .map(|name| name.to_string())
                .collect(),
            )
        } else {
            None
        }
    }

    fn normalized_enabled_tools(&self) -> (HashSet<String>, bool) {
        let mut enabled = HashSet::new();
        let mut enables_all_tools = false;

        for name in &self.enabled {
            let normalized = normalize_tool_name(name);
            if normalized.is_empty() {
                continue;
            }
            if normalized == "*" || normalized.eq_ignore_ascii_case("all") {
                enables_all_tools = true;
            } else {
                enabled.insert(normalized);
            }
        }

        (enabled, enables_all_tools)
    }
}

fn normalize_tool_name(name: &str) -> String {
    let trimmed = name.trim().trim_matches('"');
    wvc_tool_types::resolve_tool_name(trimmed).to_string()
}

/// External dictation / speech-to-text integration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct DictationConfig {
    /// Shell command to run. Must print the transcript to stdout.
    pub command: String,
    /// How to apply the resulting transcript.
    pub mode: crate::protocol::TranscriptMode,
    /// Optional in-app hotkey to trigger dictation.
    pub key: String,
    /// Maximum time to wait for the command to finish (0 = no timeout).
    pub timeout_secs: u64,
}

impl Default for DictationConfig {
    fn default() -> Self {
        Self {
            command: String::new(),
            mode: crate::protocol::TranscriptMode::Send,
            key: "off".to_string(),
            timeout_secs: 90,
        }
    }
}

mod config_file;
mod default_file;
mod display_summary;
mod env_overrides;

#[cfg(test)]
#[path = "config_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "config_color_tests.rs"]
mod color_tests;

/// Whether integration discovery settings carry no information beyond the shipped
/// default, so `[sponsors]` can be left out of written config files.
///
/// Discovery originally shipped opt-in with `enabled = false`, and because
/// config saves serialize the whole struct, any save during that window froze
/// the old default into the user's file and permanently disabled discovery even
/// after the default flipped. Omitting default sections prevents a repeat.
fn sponsors_is_default(sponsors: &SponsorsConfig) -> bool {
    sponsors.enabled && is_default_discovery_endpoint(&sponsors.endpoint)
}

/// Endpoints that only ever came from a shipped default, never a user choice.
fn is_default_discovery_endpoint(endpoint: &str) -> bool {
    matches!(
        endpoint.trim_end_matches('/'),
        "https://api.weavecoder.sh/v1/discovery" | "https://api.solosystems.dev/v1/discovery"
    )
}
