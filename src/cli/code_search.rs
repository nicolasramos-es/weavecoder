//! `wvc code-search` — hybrid search over the Code Knowledge Graph.
//!
//! Usage: `wvc code-search <query> [--db <path>] [--top-k N]`
//!
//! Combines FTS5 text match + semantic cosine similarity + graph traversal
//! (callers/dependencies) with Reciprocal Rank Fusion.

use anyhow::Result;

/// Run the code-search command against an initialized code-graph database.
pub async fn run_code_search(query: &str, db_path: Option<&str>, top_k: usize) -> Result<()> {
    use wvc_code_graph::{CodeGraph, HybridSearch};

    let path = db_path.unwrap_or("code-graph.db");
    if !std::path::Path::new(path).exists() {
        eprintln!(
            "Error: code graph database not found at {path}. Run `wvc init <project> --db {path}` first."
        );
        return Ok(());
    }

    println!("🔎 Searching code graph ({path}) for: {query}");

    let storage = CodeGraph::open(path)?;
    let engine = HybridSearch::open(storage)?;
    let results = engine.search(query, None, top_k)?;

    if results.is_empty() {
        println!("  No results.");
        return Ok(());
    }

    for (i, r) in results.iter().enumerate() {
        let signals: Vec<&str> = r
            .signals
            .iter()
            .map(|s| match s {
                wvc_code_graph::SearchSignal::Fts { .. } => "fts",
                wvc_code_graph::SearchSignal::Semantic { .. } => "semantic",
                wvc_code_graph::SearchSignal::Graph { .. } => "graph",
            })
            .collect();
        println!(
            "  {}. {} ({}) — {}:{} — score={:.3} [{}]",
            i + 1,
            r.symbol.name,
            r.symbol.kind,
            r.symbol.file_path,
            r.symbol.line,
            r.score,
            signals.join("+")
        );
    }

    Ok(())
}
