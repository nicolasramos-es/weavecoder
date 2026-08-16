//! `wvc code-search` — hybrid search over the Code Knowledge Graph.
//!
//! Usage: `wvc code-search <query> [--db <path>] [--top-k N]`
//!
//! Combines FTS5 text match + semantic cosine similarity + graph traversal
//! (callers/dependencies) with Reciprocal Rank Fusion.
//!
//! In-memory session cache: same query + same cwd → instant LRU hit (max 50).
//! Invalidated on `wvc init` or cwd change.

use anyhow::Result;
use std::time::Instant;

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

    // Check session cache first (same query + cwd → instant hit).
    let session = wvc_session_cache::SessionCache::new();
    if let Some(cached) = session.code_search.get(query) {
        println!("🔎 [CACHE HIT] code graph ({path}) for: {query}");
        if cached.is_empty() {
            println!("  No results.");
            return Ok(());
        }
        for (i, r) in cached.iter().enumerate() {
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
        return Ok(());
    }

    println!("🔎 Searching code graph ({path}) for: {query}");

    let start = Instant::now();
    let storage = CodeGraph::open(path)?;
    let engine = HybridSearch::open(storage)?;
    let results = engine.search(query, None, top_k)?;
    let elapsed = start.elapsed();

    // Store in session cache for future hits.
    if !results.is_empty() {
        for r in &results {
            session.code_search.insert(query, r.clone());
        }
    }

    if results.is_empty() {
        println!("  No results.");
        return Ok(());
    }

    // Print results with timing info.
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
            "  {}. {} ({}) — {}:{} — score={:.3} [{}] ({}ms)",
            i + 1,
            r.symbol.name,
            r.symbol.kind,
            r.symbol.file_path,
            r.symbol.line,
            r.score,
            signals.join("+"),
            elapsed.as_millis()
        );
    }

    Ok(())
}
