//! FTS5 search query builder and results.

use serde::{Deserialize, Serialize};

/// FTS search query parameters.
#[derive(Debug, Clone, Default)]
pub struct FtsQuery {
    /// The search terms (space-separated for FTS5 AND, OR for alternatives).
    pub query: String,
    /// Maximum number of results.
    pub limit: Option<usize>,
}

impl FtsQuery {
    /// Build the FTS5 query string from the search parameters.
    ///
    /// Supports:
    /// - Simple term: `fn_name` → matches "fn_name"
    /// - Prefix search: `fn_name*` → matches "fn_name", "fn_named", etc.
    /// - Phrase: `"my function"` → exact phrase match
    /// - OR: `func OR method` → matches either
    /// - Column filter: `name:fn_name` → search only name column
    pub fn build_fts_query(&self) -> String {
        self.query.clone()
    }
}

/// A single result from an FTS search, including BM25 rank.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FtsSearchResult {
    pub id: i64,
    pub name: String,
    pub kind: String,
    pub file_path: String,
    pub line: i64,
    pub col: i64,
    pub language: Option<String>,
    pub doc: Option<String>,
    pub embedding: Option<Vec<u8>>,
    pub rank: f64,
}
