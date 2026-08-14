//! Symbol embedding generation and similarity search.
//!
//! Generates embeddings for code symbols using the `wvc-embedding` crate
//! (all-MiniLM-L6-v2, 384-dim vectors) and enables cosine-similarity search
//! over persisted embeddings stored as SQLite BLOBs.
//!
//! ## Serialization format
//!
//! Embeddings are stored in SQLite as `Vec<u8>` — a little-endian
//! serialization of `f32` values (4 bytes each, 384 × 4 = 1536 bytes).
//! This avoids JSON overhead and keeps the BLOB fixed-width for efficient
//! I/O. The public API works with `Vec<f32>`; serialization is internal.

use anyhow::{Context, Result};
use std::path::Path;

use crate::{CodeGraph, Symbol};
use wvc_embedding::{self, Embedder, EmbeddingVec};

/// Dimensionality of the all-MiniLM-L6-v2 model.
pub const EMBEDDING_DIM: usize = 384;

/// Size in bytes of a serialized embedding (384 f32 × 4 bytes).
pub const SERIALIZED_EMBEDDING_SIZE: usize = EMBEDDING_DIM * 4;

// ---------------------------------------------------------------------------
// Serialization helpers (f32 ↔ Vec<u8> little-endian)
// ---------------------------------------------------------------------------

/// Serialize a `Vec<f32>` embedding to a little-endian byte vector.
pub fn serialize_embedding(vec: &[f32]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(vec.len() * 4);
    for &f in vec {
        bytes.extend_from_slice(&f.to_le_bytes());
    }
    bytes
}

/// Deserialize a little-endian byte slice back to `Vec<f32>`.
/// Accepts any multiple-of-4 byte length; returns None if not divisible by 4.
pub fn deserialize_embedding(bytes: &[u8]) -> Option<Vec<f32>> {
    if bytes.is_empty() || !bytes.len().is_multiple_of(4) {
        return None;
    }
    let floats: Vec<f32> = bytes
        .chunks_exact(4)
        .map(|chunk| f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
        .collect();
    Some(floats)
}

// ---------------------------------------------------------------------------
// SymbolEmbedder — generates embeddings from symbol text representation
// ---------------------------------------------------------------------------

/// Builds a textual representation from a symbol for embedding.
///
/// The representation is `kind: name — doc` (doc only if present), which
/// gives the model enough context to distinguish symbols with similar names
/// but different purposes.
pub fn symbol_text(symbol: &Symbol) -> String {
    let doc = symbol.doc.as_deref().unwrap_or("");
    if doc.is_empty() {
        format!("{}: {}", symbol.kind, symbol.name)
    } else {
        // Truncate doc to avoid exceeding model's max sequence length.
        let doc = if doc.len() > 512 { &doc[..512] } else { doc };
        format!("{}: {} — {}", symbol.kind, symbol.name, doc)
    }
}

/// Embedder wrapper that generates embeddings for symbols.
pub struct SymbolEmbedder {
    embedder: Embedder,
}

impl SymbolEmbedder {
    /// Load the embedding model from a directory (downloads if missing).
    pub fn new(model_dir: &Path) -> Result<Self> {
        let embedder =
            Embedder::load_from_dir(model_dir).context("Failed to load embedding model")?;
        Ok(Self { embedder })
    }

    /// Generate an embedding for a single symbol.
    pub fn embed_symbol(&self, symbol: &Symbol) -> Result<EmbeddingVec> {
        let text = symbol_text(symbol);
        self.embedder
            .embed(&text)
            .context("Failed to generate embedding for symbol")
    }

    /// Generate embeddings for a batch of symbols.
    pub fn embed_batch(&self, symbols: &[Symbol]) -> Result<Vec<EmbeddingVec>> {
        symbols.iter().map(|s| self.embed_symbol(s)).collect()
    }

    /// Update a single symbol's embedding in the database.
    pub fn update_symbol_embedding(&self, graph: &mut CodeGraph, symbol_id: i64) -> Result<()> {
        let symbol = graph
            .get_symbol(symbol_id)?
            .ok_or_else(|| anyhow::anyhow!("symbol {} not found", symbol_id))?;
        let embedding = self.embed_symbol(&symbol)?;
        let serialized = serialize_embedding(&embedding);
        graph.update_symbol_embedding(symbol_id, Some(serialized))?;
        Ok(())
    }

    /// Generate and persist embeddings for all symbols that lack one.
    pub fn embed_all_missing(&self, graph: &mut CodeGraph) -> Result<usize> {
        let all_symbols = graph.list_symbols(Default::default())?;
        let missing: Vec<Symbol> = all_symbols
            .into_iter()
            .filter(|s| s.embedding.is_none())
            .collect();

        if missing.is_empty() {
            return Ok(0);
        }

        let embeddings = self.embed_batch(&missing)?;
        let mut count = 0usize;

        for (symbol, embedding) in missing.iter().zip(embeddings) {
            let serialized = serialize_embedding(&embedding);
            graph.update_symbol_embedding(symbol.id, Some(serialized))?;
            count += 1;
        }

        Ok(count)
    }
}

// ---------------------------------------------------------------------------
// EmbeddingSearch — cosine similarity search over persisted embeddings
// ---------------------------------------------------------------------------

/// Result of an embedding similarity search.
#[derive(Debug, Clone)]
pub struct EmbeddingSearchResult {
    /// The matched symbol.
    pub symbol: Symbol,
    /// Cosine similarity score (0.0–1.0 for normalized vectors).
    pub similarity: f32,
}

/// Search engine for embedding-based similarity queries.
pub struct EmbeddingSearch {
    graph: CodeGraph,
}

impl EmbeddingSearch {
    /// Create a new search engine backed by the given graph.
    pub fn new(graph: CodeGraph) -> Self {
        Self { graph }
    }

    /// Search for the top-K symbols most similar to the given embedding.
    pub fn search(
        &self,
        query_embedding: &[f32],
        top_k: usize,
        threshold: f32,
    ) -> Result<Vec<EmbeddingSearchResult>> {
        let all_symbols = self.graph.list_symbols(Default::default())?;

        // Filter out symbols without embeddings.
        let candidates: Vec<(usize, &Symbol)> = all_symbols
            .iter()
            .enumerate()
            .filter_map(|(i, s)| s.embedding.as_ref().map(|_| (i, s)))
            .collect();

        if candidates.is_empty() {
            return Ok(Vec::new());
        }

        // Deserialize all candidate embeddings.
        let candidate_vecs: Vec<EmbeddingVec> = candidates
            .iter()
            .filter_map(|(_, s)| s.embedding.as_ref().and_then(|e| deserialize_embedding(e)))
            .collect();

        if candidate_vecs.is_empty() {
            return Ok(Vec::new());
        }

        // Use wvc-embedding's find_similar for top-K.
        let hits = wvc_embedding::find_similar(query_embedding, &candidate_vecs, threshold, top_k);

        // Map back to symbols.
        let results: Vec<EmbeddingSearchResult> = hits
            .into_iter()
            .filter_map(|(idx, score)| {
                candidates
                    .get(idx)
                    .map(|(_, symbol)| EmbeddingSearchResult {
                        symbol: (*symbol).clone(),
                        similarity: score,
                    })
            })
            .collect();

        Ok(results)
    }

    /// Search using a symbol's text representation directly.
    /// Convenience method: generates the query embedding then searches.
    pub fn search_by_symbol(
        &self,
        query_symbol: &Symbol,
        embedder: &SymbolEmbedder,
        top_k: usize,
        threshold: f32,
    ) -> Result<Vec<EmbeddingSearchResult>> {
        let query_embedding = embedder.embed_symbol(query_symbol)?;
        self.search(&query_embedding, top_k, threshold)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Symbol;

    // --- Serialization tests ---

    #[test]
    fn test_serialize_deserialize_roundtrip() {
        let original: EmbeddingVec = vec![0.1, 0.5, -0.3, 0.99, 0.0];
        let serialized = serialize_embedding(&original);
        let deserialized = deserialize_embedding(&serialized).unwrap();
        assert_eq!(original.len(), deserialized.len());
        for (a, b) in original.iter().zip(deserialized.iter()) {
            assert!((a - b).abs() < 1e-6, "f32 mismatch: {a} vs {b}");
        }
    }

    #[test]
    fn test_serialize_full_dimension() {
        let full: EmbeddingVec = (0..EMBEDDING_DIM)
            .map(|i| (i as f32) * 0.01 - 1.0)
            .collect();
        let serialized = serialize_embedding(&full);
        assert_eq!(serialized.len(), SERIALIZED_EMBEDDING_SIZE);
        let deserialized = deserialize_embedding(&serialized).unwrap();
        assert_eq!(deserialized.len(), EMBEDDING_DIM);
        for (a, b) in full.iter().zip(deserialized.iter()) {
            assert!((a - b).abs() < 1e-6);
        }
    }

    #[test]
    fn test_deserialize_wrong_size_returns_none() {
        // 100 bytes is divisible by 4, so it would deserialize fine.
        // Use a non-multiple-of-4 size instead.
        let too_short = vec![0u8; 101];
        assert!(deserialize_embedding(&too_short).is_none());

        // Non-multiple-of-4 should return None.
        let odd = vec![0u8; SERIALIZED_EMBEDDING_SIZE + 1];
        assert!(deserialize_embedding(&odd).is_none());
    }

    #[test]
    fn test_serialize_empty_vec() {
        let empty: EmbeddingVec = vec![];
        let serialized = serialize_embedding(&empty);
        assert_eq!(serialized.len(), 0);
        // Empty serialization should not deserialize.
        assert!(deserialize_embedding(&serialized).is_none());
    }

    // --- Symbol text representation tests ---

    #[test]
    fn test_symbol_text_no_doc() {
        let symbol = Symbol {
            id: 1,
            name: "my_function".to_string(),
            kind: "function".to_string(),
            file_path: "/path/to/file.py".to_string(),
            line: 10,
            col: 4,
            language: Some("python".to_string()),
            doc: None,
            embedding: None,
        };
        let text = symbol_text(&symbol);
        assert_eq!(text, "function: my_function");
    }

    #[test]
    fn test_symbol_text_with_doc() {
        let symbol = Symbol {
            id: 2,
            name: "process_data".to_string(),
            kind: "function".to_string(),
            file_path: "/path/to/file.go".to_string(),
            line: 25,
            col: 0,
            language: Some("go".to_string()),
            doc: Some("Processes the input data and returns results.".to_string()),
            embedding: None,
        };
        let text = symbol_text(&symbol);
        assert!(text.starts_with("function: process_data — Processes"));
        assert!(text.contains("input data"));
    }

    #[test]
    fn test_symbol_text_truncates_long_doc() {
        let long_doc = "a".repeat(1000);
        let symbol = Symbol {
            id: 3,
            name: "big_func".to_string(),
            kind: "function".to_string(),
            file_path: "/path.rs".to_string(),
            line: 1,
            col: 0,
            language: Some("rust".to_string()),
            doc: Some(long_doc),
            embedding: None,
        };
        let text = symbol_text(&symbol);
        // Should be truncated to ~512 chars for the doc part.
        assert!(text.len() < 700); // "function: big_func — " + 512
    }

    // --- Cosine similarity tests (reuse wvc-embedding logic) ---

    #[test]
    fn test_cosine_identical_vectors() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        let sim = wvc_embedding::cosine_similarity(&a, &b);
        assert!(
            (sim - 1.0).abs() < 0.001,
            "identical vectors should have similarity 1.0, got {sim}"
        );
    }

    #[test]
    fn test_cosine_orthogonal_vectors() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![0.0, 1.0, 0.0];
        let sim = wvc_embedding::cosine_similarity(&a, &b);
        assert!(
            (sim - 0.0).abs() < 0.001,
            "orthogonal vectors should have similarity 0.0, got {sim}"
        );
    }

    #[test]
    fn test_cosine_opposite_vectors() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![-1.0, 0.0, 0.0];
        let sim = wvc_embedding::cosine_similarity(&a, &b);
        assert!(
            (sim - (-1.0)).abs() < 0.001,
            "opposite vectors should have similarity -1.0, got {sim}"
        );
    }

    #[test]
    fn test_find_similar_returns_sorted_top_k() {
        let query = vec![1.0, 0.0, 0.0];
        let candidates = vec![
            vec![0.2, 0.0, 0.0],
            vec![0.9, 0.0, 0.0],
            vec![0.7, 0.0, 0.0],
            vec![0.8, 0.0, 0.0],
        ];
        let hits = wvc_embedding::find_similar(&query, &candidates, 0.0, 3);
        assert_eq!(hits.len(), 3);
        // Should be sorted by descending similarity.
        assert!(hits[0].1 >= hits[1].1);
        assert!(hits[1].1 >= hits[2].1);
    }

    #[test]
    fn test_find_similar_threshold_filtering() {
        let query = vec![1.0, 0.0, 0.0];
        let candidates = vec![
            vec![0.9, 0.0, 0.0],
            vec![0.3, 0.0, 0.0],
            vec![0.1, 0.0, 0.0],
        ];
        // High threshold should filter out low-similarity candidates.
        let hits = wvc_embedding::find_similar(&query, &candidates, 0.5, 10);
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].0, 0); // index of [0.9, 0, 0]
    }

    // --- Determinism test ---

    #[test]
    fn test_embedding_determinism() {
        // Same input text should produce the same embedding.
        // We can't easily load the full model in unit tests, but we can
        // verify that our text representation is deterministic.
        let symbol1 = Symbol {
            id: 1,
            name: "test_func".to_string(),
            kind: "function".to_string(),
            file_path: "/test.py".to_string(),
            line: 1,
            col: 0,
            language: Some("python".to_string()),
            doc: Some("A test function.".to_string()),
            embedding: None,
        };
        let symbol2 = Symbol {
            id: 2,
            name: "test_func".to_string(),
            kind: "function".to_string(),
            file_path: "/test.py".to_string(),
            line: 10,
            col: 0,
            language: Some("python".to_string()),
            doc: Some("A test function.".to_string()),
            embedding: None,
        };
        // Same name, kind, doc → same text representation.
        assert_eq!(symbol_text(&symbol1), symbol_text(&symbol2));
    }
}
