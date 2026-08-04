//! # wvc-code-graph
//!
//! Code Knowledge Graph for weavecoder: tree-sitter parsing and symbol
//! extraction (Go, Python, TypeScript, Rust) + embedded SQLite storage with
//! FTS5. Stores symbols (code entities) and relations (edges between them).

use tree_sitter::Tree;

// Language detection and parsing modules
mod language;
mod parser;

pub use language::{detect_language, Language};
pub use parser::{parse_file, parse_str};

// Storage modules
mod storage;
mod symbols;
mod relations;
mod fts;

pub use storage::{CodeGraph, CodeGraphError};
pub use symbols::{Symbol, SymbolKind, SymbolInsert, SymbolQuery, SymbolResult};
pub use relations::{Relation, RelationKind, RelationInsert};
pub use fts::{FtsSearchResult, FtsQuery};

pub const SCHEMA_VERSION: u32 = 1;

/// Parse a file's contents as a string and produce a tree-sitter Tree.
pub fn parse_source(source: &str, ext: &str) -> Result<Tree, String> {
    let language = detect_language(ext)
        .ok_or_else(|| format!("unsupported file extension: .{ext}"))?;
    parser::parse_str(source, language)
}

/// Detect the language from a file extension.
pub fn detect_ext(path: &str) -> Option<String> {
    std::path::Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_lowercase())
}

#[cfg(test)]
mod tests;
