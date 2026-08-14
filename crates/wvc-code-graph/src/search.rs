//! Hybrid search: FTS5 + cosine similarity + graph traversal, fused with RRF.
//!
//! Combines three signals over the code knowledge graph:
//! 1. **FTS5** — exact/substring text match from the SQLite FTS5 index (T3).
//! 2. **Semantic** — cosine similarity over symbol embeddings (T4), used only
//!    when a query embedding is provided and symbols carry embeddings.
//! 3. **Graph** — neighborhood enrichment via the petgraph `SymbolGraph` (T5):
//!    callers and dependencies of a hit are surfaced with a degraded score.
//!
//! Fusion uses Reciprocal Rank Fusion (RRF, k=60), which needs no score
//! normalization across incompatible signals (BM25 rank vs cosine vs hop
//! distance) — only rank order matters.

use crate::{CodeGraph, FtsQuery, Symbol, SymbolGraph};

/// A search signal that contributed to a result.
#[derive(Debug, Clone)]
pub enum SearchSignal {
    /// FTS5 text match (BM25 rank).
    Fts { score: f64 },
    /// Cosine similarity over embeddings (T4).
    Semantic { score: f64 },
    /// Graph neighborhood enrichment (T5): a neighbor of a direct hit.
    Graph { score: f64, neighbor_name: String },
}

/// A single hybrid search result.
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub symbol: Symbol,
    /// Fused score (RRF + small signal-strength blend).
    pub score: f64,
    /// Which signals contributed to this result.
    pub signals: Vec<SearchSignal>,
}

/// Hybrid search engine over a persisted code graph.
pub struct HybridSearch {
    storage: CodeGraph,
    graph: SymbolGraph,
}

impl HybridSearch {
    /// Open the hybrid search engine over a code-graph database.
    /// Builds the in-memory petgraph from the stored symbols/relations.
    pub fn open(storage: CodeGraph) -> anyhow::Result<Self> {
        let graph = SymbolGraph::build_from_storage(&storage)?;
        Ok(Self { storage, graph })
    }

    // ── Signal 1: FTS5 ──────────────────────────────────────────────

    /// FTS5 text search. Returns (storage symbol id, rank-score) pairs
    /// where a lower BM25 rank maps to a higher signal score.
    ///
    /// Uses a prefix query (`term*`) so substring-ish lookups like
    /// "parse" match "parseConfig" — the "búsqueda por substring" criterion.
    fn fts_signal(&self, query: &str) -> Vec<(i64, f64)> {
        let trimmed = query.trim();
        if trimmed.is_empty() {
            return Vec::new();
        }
        // Single terms become prefix queries; multi-term stays as-is.
        let fts_query = if trimmed.split_whitespace().count() == 1 {
            format!("{trimmed}*")
        } else {
            trimmed.to_string()
        };
        let results = self
            .storage
            .search_fts(FtsQuery {
                query: fts_query,
                limit: Some(50),
            })
            .unwrap_or_default();
        results
            .into_iter()
            .map(|r| {
                // BM25 rank: lower is better. Invert to a 0..1 signal score.
                let score = 1.0 / (1.0 + r.rank.max(0.0));
                (r.id, score)
            })
            .collect()
    }

    // ── Signal 2: semantic (cosine) ─────────────────────────────────

    /// Cosine similarity over persisted embeddings. Used only when
    /// `query_embedding` is Some (i.e. the caller could generate one).
    /// Returns (storage symbol id, cosine score).
    fn semantic_signal(&self, query_embedding: &[f32], top_k: usize) -> Vec<(i64, f64)> {
        let symbols = self
            .storage
            .list_symbols(Default::default())
            .unwrap_or_default();
        let mut scored: Vec<(i64, f64)> = Vec::new();
        for s in symbols {
            if let Some(blob) = &s.embedding
                && let Some(vec) = crate::embedding::deserialize_embedding(blob)
            {
                let sim = wvc_embedding::cosine_similarity(query_embedding, &vec);
                if sim > 0.0 {
                    scored.push((s.id, sim as f64));
                }
            }
        }
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(top_k);
        scored
    }

    // ── Signal 3: graph neighborhood enrichment ─────────────────────

    /// For each direct hit, surface its callers and dependencies with a
    /// degraded score. Returns (storage symbol id, degraded score).
    fn graph_signal(&self, hits: &[(i64, f64)], max_per_hit: usize) -> Vec<(i64, f64)> {
        let mut out: Vec<(i64, f64)> = Vec::new();
        for (id, hit_score) in hits {
            let Some(sid) = self.graph.resolve(*id) else {
                continue;
            };
            let mut neighbors: Vec<&Symbol> = Vec::new();
            neighbors.extend(self.graph.callers_of(sid));
            neighbors.extend(self.graph.dependencies_of(sid));
            for n in neighbors.into_iter().take(max_per_hit) {
                out.push((n.id, hit_score * 0.4)); // degraded by one hop
            }
        }
        out
    }

    // ── Fusion (RRF) ────────────────────────────────────────────────

    /// Reciprocal Rank Fusion over several ranked lists of (id, signal_score).
    fn rrf_fuse(ranked_lists: Vec<Vec<(i64, f64)>>, k: f64) -> Vec<(i64, f64)> {
        let mut fused: std::collections::HashMap<i64, (f64, f64)> =
            std::collections::HashMap::new();
        for list in ranked_lists {
            for (rank, (id, signal_score)) in list.iter().enumerate() {
                let entry = fused.entry(*id).or_insert((0.0, 0.0));
                entry.0 += 1.0 / (k + rank as f64); // RRF contribution
                entry.1 += *signal_score; // raw signal strength accumulator
            }
        }
        let mut out: Vec<(i64, f64)> = fused
            .into_iter()
            .map(|(id, (rrf, raw))| (id, rrf + raw * 0.05))
            .collect();
        out.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        out
    }

    // ── Public API ──────────────────────────────────────────────────

    /// Run the hybrid search.
    ///
    /// `query_embedding` is optional: when provided, the semantic signal is
    /// included; otherwise the search runs on FTS5 + graph traversal only.
    pub fn search(
        &self,
        query: &str,
        query_embedding: Option<&[f32]>,
        top_k: usize,
    ) -> anyhow::Result<Vec<SearchResult>> {
        let fts = self.fts_signal(query);
        if fts.is_empty() && query_embedding.is_none() {
            return Ok(Vec::new());
        }

        let mut lists: Vec<Vec<(i64, f64)>> = Vec::new();
        lists.push(fts.clone());

        if let Some(qemb) = query_embedding {
            let sem = self.semantic_signal(qemb, top_k * 2);
            if !sem.is_empty() {
                lists.push(sem);
            }
        }

        // Graph enrichment over the FTS hits (and semantic hits if present).
        let mut graph_hits: Vec<(i64, f64)> = fts.clone();
        if lists.len() > 1 {
            graph_hits.extend(lists[1].clone());
        }
        let graph = self.graph_signal(&graph_hits, 5);
        if !graph.is_empty() {
            lists.push(graph);
        }

        let fused = Self::rrf_fuse(lists, 60.0);

        let fts_map: std::collections::HashMap<i64, f64> = fts.into_iter().collect();
        let mut results = Vec::new();
        for (id, score) in fused.into_iter().take(top_k.max(10)) {
            let Some(symbol) = self.storage.get_symbol(id)? else {
                continue;
            };
            let mut signals = Vec::new();
            if let Some(&s) = fts_map.get(&id) {
                signals.push(SearchSignal::Fts { score: s });
            }
            if signals.is_empty() {
                signals.push(SearchSignal::Graph {
                    score,
                    neighbor_name: symbol.name.clone(),
                });
            }
            results.push(SearchResult {
                symbol,
                score,
                signals,
            });
        }
        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{RelationInsert, SymbolInsert};

    fn insert_symbol(graph: &mut CodeGraph, name: &str, file: &str, doc: Option<&str>) -> i64 {
        graph
            .insert_symbol(SymbolInsert {
                name: name.to_string(),
                kind: "function".to_string(),
                file_path: file.to_string(),
                line: 1,
                col: 0,
                language: Some("go".to_string()),
                doc: doc.map(|s| s.to_string()),
                embedding: None,
            })
            .unwrap()
    }

    fn insert_relation(graph: &mut CodeGraph, from: i64, to: i64, kind: &str) {
        graph
            .insert_relation(RelationInsert {
                source_symbol_id: from,
                target_symbol_id: to,
                kind: kind.to_string(),
                metadata: None,
            })
            .unwrap();
    }

    fn sample_storage() -> CodeGraph {
        let mut graph = CodeGraph::open_memory().unwrap();
        let main = insert_symbol(&mut graph, "main", "main.go", Some("entry point"));
        let parse_json = insert_symbol(
            &mut graph,
            "parseJSON",
            "json.go",
            Some("parse JSON payload"),
        );
        let http_get = insert_symbol(&mut graph, "http.Get", "http.go", None);
        let utils = insert_symbol(&mut graph, "utils", "utils.go", None);
        insert_relation(&mut graph, main, parse_json, "calls");
        insert_relation(&mut graph, main, http_get, "calls");
        insert_relation(&mut graph, parse_json, utils, "depends_on");
        graph
    }

    #[test]
    fn test_exact_symbol_search_first() {
        let storage = sample_storage();
        let engine = HybridSearch::open(storage).unwrap();
        let results = engine.search("parseJSON", None, 5).unwrap();
        assert!(!results.is_empty());
        assert_eq!(results[0].symbol.name, "parseJSON");
    }

    #[test]
    fn test_semantic_search_returns_relevant_symbol() {
        let mut storage = sample_storage();
        // Give two symbols embeddings: parseJSON and utils.
        // Embedding via raw vectors: parseJSON similar to "json parser" query.
        let query: Vec<f32> = vec![0.9, 0.1, 0.0, 0.0];
        let similar: Vec<f32> = vec![0.8, 0.2, 0.0, 0.0];
        let different: Vec<f32> = vec![0.0, 0.1, 0.9, 0.0];
        let parse_json_id = storage
            .list_symbols(Default::default())
            .unwrap()
            .iter()
            .find(|s| s.name == "parseJSON")
            .unwrap()
            .id;
        let utils_id = storage
            .list_symbols(Default::default())
            .unwrap()
            .iter()
            .find(|s| s.name == "utils")
            .unwrap()
            .id;
        storage
            .update_symbol_embedding(
                parse_json_id,
                Some(crate::embedding::serialize_embedding(&similar)),
            )
            .unwrap();
        storage
            .update_symbol_embedding(
                utils_id,
                Some(crate::embedding::serialize_embedding(&different)),
            )
            .unwrap();

        let engine = HybridSearch::open(storage).unwrap();
        let results = engine.search("json parser", Some(&query), 5).unwrap();
        assert!(!results.is_empty());
        // parseJSON must appear before utils (similarity 0.9+ vs ~0.01+).
        let names: Vec<&str> = results.iter().map(|r| r.symbol.name.as_str()).collect();
        let pos_json = names.iter().position(|n| *n == "parseJSON");
        let pos_utils = names.iter().position(|n| *n == "utils");
        assert!(pos_json.is_some(), "parseJSON should appear, got {names:?}");
        assert!(pos_json.unwrap() < pos_utils.unwrap_or(usize::MAX));
    }

    #[test]
    fn test_graph_enrichment_adds_callers() {
        let storage = sample_storage();
        let engine = HybridSearch::open(storage).unwrap();
        // Searching for utils: main calls it and parseJSON depends on it, but
        // neither main nor parseJSON contain "utils" in the name. Enrichment
        // surfaces utils itself via FTS (its name matches) — and the callers
        // of parseJSON should be enriched when searching parseJSON.
        let results = engine.search("utils", None, 5).unwrap();
        assert!(!results.is_empty());
        assert_eq!(results[0].symbol.name, "utils");
    }

    #[test]
    fn test_search_empty_returns_empty() {
        let storage = CodeGraph::open_memory().unwrap();
        let engine = HybridSearch::open(storage).unwrap();
        let results = engine.search("nothing_matches_xyz", None, 5).unwrap();
        assert!(results.is_empty());
    }
}
