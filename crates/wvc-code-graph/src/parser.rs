//! Tree-sitter parsing module.
//!
//! Provides functions to parse source code into tree-sitter syntax trees.

use std::path::Path;
use tree_sitter::{Parser, Tree};

use crate::language::detect_language;

/// Parse a string of source code into a tree-sitter Tree.
pub fn parse_str(source: &str, lang: tree_sitter::Language) -> Result<Tree, String> {
    let mut parser = Parser::new();
    parser
        .set_language(&lang)
        .map_err(|e| format!("failed to set language: {e}"))?;
    parser
        .parse(source, None)
        .ok_or_else(|| "failed to parse source code".to_string())
}

/// Parse a file from the filesystem.
///
/// Detects the language from the file extension and parses the contents.
pub fn parse_file(path: &Path) -> Result<Tree, String> {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_lowercase())
        .ok_or("file has no extension")?;

    let lang = detect_language(&ext)
        .ok_or_else(|| format!("unsupported file extension: .{ext}"))?;

    let source = std::fs::read_to_string(path)
        .map_err(|e| format!("failed to read file: {e}"))?;

    parse_str(&source, lang)
}
