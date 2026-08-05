//! `wvc init` — project scanner + AST extraction → SQLite storage.
//!
//! Usage: `wvc init <project-path>`
//!
//! Scans source files by extension (Go, Python, TypeScript, Rust), parses
//! their ASTs with tree-sitter, extracts symbols and call relations, then
//! persists everything to an embedded SQLite database with FTS5 indexing.

use anyhow::Result;
use std::path::PathBuf;

/// Run the init command: scan a project directory, parse source files,
/// extract symbols/relations, and store them in the CodeGraph database.
pub async fn run_init(project_path: &str) -> Result<()> {
    use wvc_code_graph::{InitConfig, run_init as graph_init};

    let root = PathBuf::from(project_path);

    if !root.exists() {
        eprintln!("Error: path does not exist: {}", project_path);
        return Ok(());
    }

    println!("🔍 Initializing code graph for: {}", project_path);

    let config = InitConfig {
        root,
        db_path: None, // in-memory by default
        extra_extensions: vec![],
    };

    let summary = graph_init(config)?;

    println!("✅ Code graph initialized:");
    println!("   Files scanned: {}", summary.files_scanned);
    println!("   Symbols found: {}", summary.symbols_found);
    println!("   Relations found: {}", summary.relations_found);
    println!("   Time: {}ms", summary.elapsed_ms);

    Ok(())
}
