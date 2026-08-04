//! Code Knowledge Graph: tree-sitter parsing and symbol extraction.
//!
//! Provides language detection by file extension and tree-sitter parsing
//! for Go, Python, TypeScript, and Rust source files.

use tree_sitter::Tree;

// Language detection and parsing modules
mod language;
mod parser;

pub use language::{detect_language, Language};
pub use parser::{parse_file, parse_str};

#[cfg(test)]
mod tests;

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
