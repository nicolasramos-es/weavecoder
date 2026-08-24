//! # wvc-session-cache
//!
//! In-memory session cache for weavecoder: LRU caching for code-search and chat completions.
//!
//! Thread-safe, max 50 entries per cache. Invalidates on cwd change or `wvc init`.

use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;

/// Maximum number of entries per cache.
const MAX_ENTRIES: usize = 50;

// ── Internal LRU structure ──────────────────────────────────────────────

/// An entry in the LRU cache.
#[derive(Debug, Clone)]
struct CacheEntry<V> {
    value: V,
    /// Insertion order (monotonically increasing). Used for LRU eviction.
    access_order: u64,
}

/// A generic thread-safe LRU cache with a configurable max size.
#[derive(Debug)]
struct LruCache<V> {
    map: HashMap<String, CacheEntry<V>>,
    order_counter: u64,
    max_entries: usize,
}

impl<V> LruCache<V> {
    fn new(max_entries: usize) -> Self {
        Self {
            map: HashMap::new(),
            order_counter: 0,
            max_entries,
        }
    }

    /// Insert or update a value. Returns the old value if evicted.
    fn insert(&mut self, key: String, value: V) -> Option<V> {
        self.order_counter += 1;
        let order = self.order_counter;

        // If key already exists, update it and move to front
        if self.map.contains_key(&key) {
            let entry = self.map.get_mut(&key).unwrap();
            entry.value = value;
            entry.access_order = order;
            return None;
        }

        // Evict LRU entry if at capacity (but always insert the new entry after).
        let mut evicted_value: Option<V> = None;
        if self.map.len() >= self.max_entries {
            let lru_key = self
                .map
                .iter()
                .min_by_key(|(_, e)| e.access_order)
                .map(|(k, _)| k.clone());

            if let Some(evict_key) = lru_key {
                let evicted = self.map.remove(&evict_key);
                if let Some(entry) = evicted {
                    evicted_value = Some(entry.value);
                }
            }
        }

        self.map.insert(
            key,
            CacheEntry {
                value,
                access_order: order,
            },
        );
        evicted_value
    }

    /// Get a value by key, moving it to the front (most recently used).
    fn get(&mut self, key: &str) -> Option<&V> {
        if let Some(entry) = self.map.get_mut(key) {
            self.order_counter += 1;
            entry.access_order = self.order_counter;
            Some(&entry.value)
        } else {
            None
        }
    }

    /// Clear all entries.
    fn clear(&mut self) {
        self.map.clear();
    }

    /// Current number of entries.
    fn len(&self) -> usize {
        self.map.len()
    }
}

// ── Code search cache ───────────────────────────────────────────────────

use wvc_code_graph::{HybridSearch, SearchResult};

/// Cache key for code-search: hash of (query, cwd).
fn code_search_key(query: &str, cwd: &PathBuf) -> String {
    let mut hasher = Sha256::new();
    hasher.update(query.as_bytes());
    hasher.update(b"\x00");
    hasher.update(cwd.as_os_str().as_encoded_bytes());
    hex::encode(hasher.finalize())
}

/// Thread-safe in-memory session cache for code-search results.
#[derive(Debug)]
pub struct CodeSearchCache {
    inner: Mutex<LruCache<SearchResult>>,
    /// Current working directory snapshot. Cache invalidates when cwd changes.
    cwd_snapshot: Mutex<PathBuf>,
}

impl CodeSearchCache {
    /// Create a new code-search cache.
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(LruCache::new(MAX_ENTRIES)),
            cwd_snapshot: Mutex::new(std::env::current_dir().unwrap_or_default()),
        }
    }

    /// Try to get a cached result. Returns Some if cache hit, None if miss.
    pub fn get(&self, query: &str) -> Option<SearchResult> {
        let cwd = {
            let cwd_guard = self.cwd_snapshot.lock().unwrap();
            cwd_guard.clone()
        };

        let key = code_search_key(query, &cwd);
        let mut cache = self.inner.lock().unwrap();
        cache.get(&key).cloned()
    }

    /// Insert a result into the cache.
    pub fn insert(&self, query: &str, result: SearchResult) {
        let cwd = {
            let cwd_guard = self.cwd_snapshot.lock().unwrap();
            cwd_guard.clone()
        };

        let key = code_search_key(query, &cwd);
        let mut cache = self.inner.lock().unwrap();
        cache.insert(key, result);
    }

    /// Invalidate the cache when cwd changes. Returns true if invalidated.
    pub fn invalidate_on_cwd_change(&self) -> bool {
        let new_cwd = std::env::current_dir().unwrap_or_default();
        let mut cwd_guard = self.cwd_snapshot.lock().unwrap();
        if *cwd_guard != new_cwd {
            *cwd_guard = new_cwd.clone();
            let mut cache = self.inner.lock().unwrap();
            cache.clear();
            return true;
        }
        false
    }

    /// Clear the entire cache.
    pub fn clear(&self) {
        let mut cache = self.inner.lock().unwrap();
        cache.clear();
    }

    /// Current number of cached entries.
    pub fn len(&self) -> usize {
        let cache = self.inner.lock().unwrap();
        cache.len()
    }
}

impl Default for CodeSearchCache {
    fn default() -> Self {
        Self::new()
    }
}

// ── Chat completion cache ───────────────────────────────────────────────

/// Cache key for chat completions: hash of (question, context, model).
fn chat_completion_key(question: &str, context: &str, model: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(question.as_bytes());
    hasher.update(b"\x00");
    hasher.update(context.as_bytes());
    hasher.update(b"\x00");
    hasher.update(model.as_bytes());
    hex::encode(hasher.finalize())
}

/// Thread-safe in-memory session cache for chat completions.
#[derive(Debug)]
pub struct ChatCompletionCache {
    inner: Mutex<LruCache<String>>,
}

impl ChatCompletionCache {
    /// Create a new chat completion cache.
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(LruCache::new(MAX_ENTRIES)),
        }
    }

    /// Try to get a cached response. Returns Some if cache hit, None if miss.
    pub fn get(&self, question: &str, context: &str, model: &str) -> Option<String> {
        let key = chat_completion_key(question, context, model);
        let mut cache = self.inner.lock().unwrap();
        cache.get(&key).cloned()
    }

    /// Insert a response into the cache.
    pub fn insert(&self, question: &str, context: &str, model: &str, response: String) {
        let key = chat_completion_key(question, context, model);
        let mut cache = self.inner.lock().unwrap();
        cache.insert(key, response);
    }

    /// Clear the entire cache.
    pub fn clear(&self) {
        let mut cache = self.inner.lock().unwrap();
        cache.clear();
    }

    /// Current number of cached entries.
    pub fn len(&self) -> usize {
        let cache = self.inner.lock().unwrap();
        cache.len()
    }
}

impl Default for ChatCompletionCache {
    fn default() -> Self {
        Self::new()
    }
}

// ── Session cache (combined) ────────────────────────────────────────────

/// Combined session cache: code-search + chat completion caches.
#[derive(Debug)]
pub struct SessionCache {
    pub code_search: CodeSearchCache,
    pub chat_completion: ChatCompletionCache,
}

impl SessionCache {
    /// Create a new session cache.
    pub fn new() -> Self {
        Self {
            code_search: CodeSearchCache::new(),
            chat_completion: ChatCompletionCache::new(),
        }
    }

    /// Invalidate all caches on `wvc init` (re-indexation).
    pub fn invalidate_on_init(&self) {
        self.code_search.clear();
        self.chat_completion.clear();
    }

    /// Invalidate code-search cache on cwd change.
    pub fn invalidate_on_cwd_change(&self) -> bool {
        self.code_search.invalidate_on_cwd_change()
    }

    /// Total cached entries across both caches.
    pub fn total_len(&self) -> usize {
        self.code_search.len() + self.chat_completion.len()
    }
}

impl Default for SessionCache {
    fn default() -> Self {
        Self::new()
    }
}

// ── Tests ───────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_code_search_cache_hit() {
        let cache = CodeSearchCache::new();
        let tmp = TempDir::new().unwrap();
        let cwd = tmp.path().canonicalize().unwrap();

        // Set cwd snapshot
        {
            let mut cwd_guard = cache.cwd_snapshot.lock().unwrap();
            *cwd_guard = cwd.clone();
        }

        // Insert a result
        let result = SearchResult {
            symbol: wvc_code_graph::Symbol {
                id: 1,
                name: "test_func".to_string(),
                kind: "function".to_string(),
                file_path: "test.rs".to_string(),
                line: 10,
                col: 0,
                language: Some("rust".to_string()),
                doc: None,
                embedding: None,
            },
            score: 0.95,
            signals: vec![wvc_code_graph::SearchSignal::Fts { score: 0.9 }],
        };

        cache.insert("test query", result.clone());

        // Should hit
        let cached = cache.get("test query");
        assert!(cached.is_some());
        assert_eq!(cached.unwrap().symbol.name, "test_func");
    }

    #[test]
    fn test_code_search_cache_miss() {
        let cache = CodeSearchCache::new();
        let tmp = TempDir::new().unwrap();
        let cwd = tmp.path().canonicalize().unwrap();

        {
            let mut cwd_guard = cache.cwd_snapshot.lock().unwrap();
            *cwd_guard = cwd.clone();
        }

        cache.insert(
            "test query",
            SearchResult {
                symbol: wvc_code_graph::Symbol {
                    id: 1,
                    name: "test_func".to_string(),
                    kind: "function".to_string(),
                    file_path: "test.rs".to_string(),
                    line: 10,
                    col: 0,
                    language: Some("rust".to_string()),
                    doc: None,
                    embedding: None,
                },
                score: 0.95,
                signals: vec![wvc_code_graph::SearchSignal::Fts { score: 0.9 }],
            },
        );

        // Different query should miss
        assert!(cache.get("different query").is_none());
    }

    #[test]
    fn test_code_search_cwd_invalidation() {
        let cache = CodeSearchCache::new();
        let tmp1 = TempDir::new().unwrap();
        let tmp2 = TempDir::new().unwrap();
        let cwd1 = tmp1.path().canonicalize().unwrap();
        let cwd2 = tmp2.path().canonicalize().unwrap();

        {
            let mut cwd_guard = cache.cwd_snapshot.lock().unwrap();
            *cwd_guard = cwd1.clone();
        }

        cache.insert(
            "test query",
            SearchResult {
                symbol: wvc_code_graph::Symbol {
                    id: 1,
                    name: "test_func".to_string(),
                    kind: "function".to_string(),
                    file_path: "test.rs".to_string(),
                    line: 10,
                    col: 0,
                    language: Some("rust".to_string()),
                    doc: None,
                    embedding: None,
                },
                score: 0.95,
                signals: vec![wvc_code_graph::SearchSignal::Fts { score: 0.9 }],
            },
        );

        // Simulate cwd change
        {
            let mut cwd_guard = cache.cwd_snapshot.lock().unwrap();
            *cwd_guard = cwd2.clone();
        }

        // Should miss after invalidation
        assert!(cache.get("test query").is_none());
    }

    #[test]
    fn test_chat_completion_cache_hit() {
        let cache = ChatCompletionCache::new();

        cache.insert(
            "What is fn main?",
            "project: weavecoder",
            "qwen3.6-35b",
            "fn main is the entry point of a Rust program.".to_string(),
        );

        let response = cache.get("What is fn main?", "project: weavecoder", "qwen3.6-35b");
        assert!(response.is_some());
        assert_eq!(
            response.unwrap(),
            "fn main is the entry point of a Rust program."
        );
    }

    #[test]
    fn test_chat_completion_cache_miss_different_model() {
        let cache = ChatCompletionCache::new();

        cache.insert(
            "What is fn main?",
            "project: weavecoder",
            "qwen3.6-35b",
            "fn main is the entry point of a Rust program.".to_string(),
        );

        // Different model should miss
        assert!(cache
            .get("What is fn main?", "project: weavecoder", "gpt-4")
            .is_none());
    }

    #[test]
    fn test_lru_eviction() {
        let cache = CodeSearchCache::new();
        let tmp = TempDir::new().unwrap();
        let cwd = tmp.path().canonicalize().unwrap();

        {
            let mut cwd_guard = cache.cwd_snapshot.lock().unwrap();
            *cwd_guard = cwd.clone();
        }

        // Insert 51 entries (max is 50)
        for i in 0..51 {
            cache.insert(
                &format!("query {}", i),
                SearchResult {
                    symbol: wvc_code_graph::Symbol {
                        id: i,
                        name: format!("symbol_{}", i),
                        kind: "function".to_string(),
                        file_path: format!("file_{}.rs", i),
                        line: i,
                        col: 0,
                        language: Some("rust".to_string()),
                        doc: None,
                        embedding: None,
                    },
                    score: 0.95,
                    signals: vec![wvc_code_graph::SearchSignal::Fts { score: 0.9 }],
                },
            );
        }

        // Cache should be at max capacity
        assert_eq!(cache.len(), 50);

        // First entry should have been evicted
        assert!(cache.get("query 0").is_none());

        // Last entry should be present
        assert!(cache.get("query 50").is_some());
    }

    #[test]
    fn test_session_cache_combined() {
        let session = SessionCache::new();

        // Insert into code search cache
        session.code_search.insert(
            "test",
            SearchResult {
                symbol: wvc_code_graph::Symbol {
                    id: 1,
                    name: "test".to_string(),
                    kind: "function".to_string(),
                    file_path: "test.rs".to_string(),
                    line: 1,
                    col: 0,
                    language: Some("rust".to_string()),
                    doc: None,
                    embedding: None,
                },
                score: 0.95,
                signals: vec![wvc_code_graph::SearchSignal::Fts { score: 0.9 }],
            },
        );

        // Insert into chat completion cache
        session
            .chat_completion
            .insert("q", "ctx", "model", "response".to_string());

        assert_eq!(session.total_len(), 2);

        // Invalidate on init
        session.invalidate_on_init();
        assert_eq!(session.total_len(), 0);
    }

    #[test]
    fn test_code_search_different_cwd_different_keys() {
        let cache = CodeSearchCache::new();
        let tmp1 = TempDir::new().unwrap();
        let tmp2 = TempDir::new().unwrap();
        let cwd1 = tmp1.path().canonicalize().unwrap();
        let cwd2 = tmp2.path().canonicalize().unwrap();

        // Insert with cwd1
        {
            let mut cwd_guard = cache.cwd_snapshot.lock().unwrap();
            *cwd_guard = cwd1.clone();
        }

        cache.insert(
            "query",
            SearchResult {
                symbol: wvc_code_graph::Symbol {
                    id: 1,
                    name: "test".to_string(),
                    kind: "function".to_string(),
                    file_path: "test.rs".to_string(),
                    line: 1,
                    col: 0,
                    language: Some("rust".to_string()),
                    doc: None,
                    embedding: None,
                },
                score: 0.95,
                signals: vec![wvc_code_graph::SearchSignal::Fts { score: 0.9 }],
            },
        );

        // Change cwd to cwd2
        {
            let mut cwd_guard = cache.cwd_snapshot.lock().unwrap();
            *cwd_guard = cwd2.clone();
        }

        // Should miss after invalidation
        assert!(cache.get("query").is_none());
    }
}
