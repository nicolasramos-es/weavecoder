//! In-memory code graph built on petgraph.
//!
//! Nodes are code symbols (`Symbol`), edges are relations (`RelationKind` —
//! calls, inherits, implements, imports/depends_on, ...). The graph is built
//! from the SQLite storage (`CodeGraph::list_symbols` + `list_relations`) so
//! that "who calls X", "what does Y depend on" and reachability queries run in
//! memory against the whole indexed project.
//!
//! This is the *code* graph (T5), distinct from the conversational memory
//! graph in `wvc-memory-types`.

use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::visit::EdgeRef;
use std::collections::{HashMap, HashSet, VecDeque};

use crate::{CodeGraph, Relation, RelationKind, Symbol};

/// Handle for a symbol node in the graph.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SymbolId(pub usize);

/// In-memory directed graph of code symbols.
pub struct SymbolGraph {
    graph: DiGraph<Symbol, RelationKind>,
    /// (file_path, name) -> SymbolId for dedup on insert.
    symbol_map: HashMap<(String, String), SymbolId>,
    /// storage symbol id -> SymbolId (for building from relations).
    id_map: HashMap<i64, SymbolId>,
}

impl Default for SymbolGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl SymbolGraph {
    pub fn new() -> Self {
        SymbolGraph {
            graph: DiGraph::new(),
            symbol_map: HashMap::new(),
            id_map: HashMap::new(),
        }
    }

    /// Add a symbol node. Dedup by (file_path, name); returns existing id on duplicate.
    pub fn add_symbol(&mut self, symbol: Symbol) -> SymbolId {
        let key = (symbol.file_path.clone(), symbol.name.clone());
        if let Some(&id) = self.symbol_map.get(&key) {
            return id;
        }
        let storage_id = symbol.id;
        let idx = self.graph.add_node(symbol);
        let id = SymbolId(idx.index());
        self.symbol_map.insert(key, id);
        if storage_id != 0 {
            self.id_map.insert(storage_id, id);
        }
        id
    }

    /// Add a directed edge between two symbols. Returns false if either id is invalid.
    pub fn add_edge(&mut self, from: SymbolId, to: SymbolId, kind: RelationKind) -> bool {
        let from_idx = NodeIndex::new(from.0);
        let to_idx = NodeIndex::new(to.0);
        if self.graph.node_weight(from_idx).is_some() && self.graph.node_weight(to_idx).is_some() {
            self.graph.add_edge(from_idx, to_idx, kind);
            true
        } else {
            false
        }
    }

    /// Look up a node's storage id by its SymbolId (useful after rebuilds).
    pub fn node_storage_id(&self, id: SymbolId) -> Option<i64> {
        self.graph.node_weight(NodeIndex::new(id.0)).map(|s| s.id)
    }

    /// Resolve a storage symbol id to a graph handle.
    pub fn resolve(&self, storage_id: i64) -> Option<SymbolId> {
        self.id_map.get(&storage_id).copied()
    }

    pub fn get_symbol(&self, id: SymbolId) -> Option<&Symbol> {
        self.graph.node_weight(NodeIndex::new(id.0))
    }

    pub fn symbol_count(&self) -> usize {
        self.graph.node_count()
    }

    pub fn edge_count(&self) -> usize {
        self.graph.edge_count()
    }

    /// Find symbols that point to `id` with a `Call` edge ("who calls X").
    pub fn callers_of(&self, id: SymbolId) -> Vec<&Symbol> {
        let target = NodeIndex::new(id.0);
        self.graph
            .edges_directed(target, petgraph::Direction::Incoming)
            .filter_map(|edge| {
                if edge.weight() == &RelationKind::Calls {
                    self.graph.node_weight(edge.source())
                } else {
                    None
                }
            })
            .collect()
    }

    /// Direct dependencies of `id`: outgoing edges that represent a dependency
    /// (calls, imports/depends_on, inherits, implements).
    pub fn dependencies_of(&self, id: SymbolId) -> Vec<&Symbol> {
        let source = NodeIndex::new(id.0);
        self.graph
            .edges_directed(source, petgraph::Direction::Outgoing)
            .filter_map(|edge| {
                if is_dependency_kind(edge.weight()) {
                    self.graph.node_weight(edge.target())
                } else {
                    None
                }
            })
            .collect()
    }

    /// Transitive dependencies of `id` via BFS over dependency edges.
    pub fn transitive_dependencies(&self, id: SymbolId) -> Vec<&Symbol> {
        let start = NodeIndex::new(id.0);
        let mut visited: HashSet<usize> = HashSet::new();
        let mut queue = VecDeque::new();

        for edge in self
            .graph
            .edges_directed(start, petgraph::Direction::Outgoing)
        {
            if is_dependency_kind(edge.weight()) {
                let target = edge.target().index();
                if !visited.contains(&target) {
                    visited.insert(target);
                    queue.push_back(edge.target());
                }
            }
        }

        let mut result = Vec::new();
        while let Some(node) = queue.pop_front() {
            if let Some(symbol) = self.graph.node_weight(node) {
                result.push(symbol);
            }
            for edge in self
                .graph
                .edges_directed(node, petgraph::Direction::Outgoing)
            {
                if is_dependency_kind(edge.weight()) {
                    let target = edge.target().index();
                    if !visited.contains(&target) {
                        visited.insert(target);
                        queue.push_back(edge.target());
                    }
                }
            }
        }

        result
    }

    /// Is `to` transitively reachable from `from` via dependency edges?
    pub fn is_reachable(&self, from: SymbolId, to: SymbolId) -> bool {
        if from == to {
            return true;
        }
        let start = NodeIndex::new(from.0);
        let target_idx = to.0;
        let mut visited: HashSet<usize> = HashSet::new();
        let mut queue = VecDeque::new();

        queue.push_back(start);
        visited.insert(start.index());

        while let Some(node) = queue.pop_front() {
            for edge in self
                .graph
                .edges_directed(node, petgraph::Direction::Outgoing)
            {
                if is_dependency_kind(edge.weight()) {
                    let t = edge.target().index();
                    if t == target_idx {
                        return true;
                    }
                    if !visited.contains(&t) {
                        visited.insert(t);
                        queue.push_back(edge.target());
                    }
                }
            }
        }

        false
    }

    /// Detect cycles over ALL edges (not just dependency kinds). Each cycle is
    /// reported as the ordered list of symbol names forming it.
    pub fn detect_cycles(&self) -> Vec<Vec<String>> {
        let mut cycles = Vec::new();
        let mut visited: HashSet<usize> = HashSet::new();
        let mut rec_stack: Vec<usize> = Vec::new();
        let mut rec_set: HashSet<usize> = HashSet::new();

        for node in self.graph.node_indices() {
            if !visited.contains(&node.index()) {
                Self::cycle_dfs(
                    &self.graph,
                    node,
                    &mut visited,
                    &mut rec_stack,
                    &mut rec_set,
                    &mut cycles,
                );
            }
        }

        cycles
    }

    fn cycle_dfs(
        graph: &DiGraph<Symbol, RelationKind>,
        node: NodeIndex,
        visited: &mut HashSet<usize>,
        rec_stack: &mut Vec<usize>,
        rec_set: &mut HashSet<usize>,
        cycles: &mut Vec<Vec<String>>,
    ) {
        visited.insert(node.index());
        rec_stack.push(node.index());
        rec_set.insert(node.index());

        for edge in graph.edges_directed(node, petgraph::Direction::Outgoing) {
            let target = edge.target();
            if rec_set.contains(&target.index()) {
                if let Some(pos) = rec_stack.iter().position(|&n| n == target.index()) {
                    let cycle_nodes: Vec<String> = rec_stack[pos..]
                        .iter()
                        .filter_map(|&n| graph.node_weight(NodeIndex::new(n)))
                        .map(|s| s.name.clone())
                        .collect();
                    cycles.push(cycle_nodes);
                }
            } else if !visited.contains(&target.index()) {
                Self::cycle_dfs(graph, target, visited, rec_stack, rec_set, cycles);
            }
        }

        rec_stack.pop();
        rec_set.remove(&node.index());
    }

    /// Build a graph from storage: loads all symbols and relations and wires
    /// the edges. Uses the storage ids to connect relations to nodes.
    pub fn build_from_storage(storage: &CodeGraph) -> anyhow::Result<Self> {
        let mut graph = Self::new();
        let symbols = storage.list_symbols(Default::default())?;
        for symbol in symbols {
            graph.add_symbol(symbol);
        }
        let relations = storage.list_relations()?;
        for rel in relations {
            graph.add_relation(&rel);
        }
        Ok(graph)
    }

    /// Add a relation read from storage. Unknown ids are skipped.
    pub fn add_relation(&mut self, rel: &Relation) -> bool {
        let from = match self.id_map.get(&rel.source_symbol_id) {
            Some(&id) => id,
            None => return false,
        };
        let to = match self.id_map.get(&rel.target_symbol_id) {
            Some(&id) => id,
            None => return false,
        };
        let kind = RelationKind::from(rel.kind.clone());
        self.add_edge(from, to, kind)
    }
}

/// Whether an edge kind represents a dependency for traversal purposes.
fn is_dependency_kind(kind: &RelationKind) -> bool {
    matches!(
        kind,
        RelationKind::Calls
            | RelationKind::DependsOn
            | RelationKind::Inherits
            | RelationKind::Implements
            | RelationKind::Uses
            | RelationKind::References
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CodeGraph, SymbolInsert};

    fn insert_symbol(graph: &mut CodeGraph, name: &str, file: &str) -> i64 {
        graph
            .insert_symbol(SymbolInsert {
                name: name.to_string(),
                kind: "function".to_string(),
                file_path: file.to_string(),
                line: 1,
                col: 0,
                language: Some("go".to_string()),
                doc: None,
                embedding: None,
            })
            .unwrap()
    }

    fn insert_relation(graph: &mut CodeGraph, from: i64, to: i64, kind: &str) {
        graph
            .insert_relation(crate::RelationInsert {
                source_symbol_id: from,
                target_symbol_id: to,
                kind: kind.to_string(),
                metadata: None,
            })
            .unwrap();
    }

    fn sample_graph() -> SymbolGraph {
        let mut storage = CodeGraph::open_memory().unwrap();
        let main = insert_symbol(&mut storage, "main", "main.go");
        let http_get = insert_symbol(&mut storage, "http.Get", "http.go");
        let parse_json = insert_symbol(&mut storage, "parseJSON", "json.go");
        let utils = insert_symbol(&mut storage, "utils", "utils.go");

        insert_relation(&mut storage, main, http_get, "calls");
        insert_relation(&mut storage, main, parse_json, "calls");
        insert_relation(&mut storage, parse_json, utils, "depends_on");
        insert_relation(&mut storage, main, utils, "calls");

        SymbolGraph::build_from_storage(&storage).unwrap()
    }

    #[test]
    fn test_build_from_storage_counts() {
        let graph = sample_graph();
        assert_eq!(graph.symbol_count(), 4);
        assert_eq!(graph.edge_count(), 4);
    }

    #[test]
    fn test_callers_of() {
        let graph = sample_graph();
        let main = graph.resolve(1).unwrap();
        let http_get = graph.resolve(2).unwrap();
        let callers: Vec<String> = graph
            .callers_of(http_get)
            .iter()
            .map(|s| s.name.clone())
            .collect();
        assert_eq!(callers, vec!["main"]);
        // nobody calls main
        assert!(graph.callers_of(main).is_empty());
    }

    #[test]
    fn test_dependencies_of_direct() {
        let graph = sample_graph();
        let main = graph.resolve(1).unwrap();
        let deps: Vec<String> = graph
            .dependencies_of(main)
            .iter()
            .map(|s| s.name.clone())
            .collect();
        assert!(deps.contains(&"http.Get".to_string()));
        assert!(deps.contains(&"parseJSON".to_string()));
        assert!(deps.contains(&"utils".to_string()));
        assert_eq!(deps.len(), 3);
    }

    #[test]
    fn test_transitive_dependencies() {
        let graph = sample_graph();
        let main = graph.resolve(1).unwrap();
        let transitive: Vec<String> = graph
            .transitive_dependencies(main)
            .iter()
            .map(|s| s.name.clone())
            .collect();
        // http.Get, parseJSON, utils (and utils again via parseJSON — deduped by visit)
        assert!(transitive.contains(&"http.Get".to_string()));
        assert!(transitive.contains(&"parseJSON".to_string()));
        assert!(transitive.contains(&"utils".to_string()));
        // utils is reachable transitively via parseJSON even though also direct
        let utils = graph.resolve(4).unwrap();
        assert!(graph.is_reachable(main, utils));
    }

    #[test]
    fn test_is_reachable_false() {
        let graph = sample_graph();
        let parse_json = graph.resolve(3).unwrap();
        let main = graph.resolve(1).unwrap();
        // parseJSON does not depend on main
        assert!(!graph.is_reachable(parse_json, main));
    }

    #[test]
    fn test_detect_cycles() {
        let mut storage = CodeGraph::open_memory().unwrap();
        let a = insert_symbol(&mut storage, "a", "a.go");
        let b = insert_symbol(&mut storage, "b", "b.go");
        let c = insert_symbol(&mut storage, "c", "c.go");
        insert_relation(&mut storage, a, b, "calls");
        insert_relation(&mut storage, b, c, "calls");
        insert_relation(&mut storage, c, a, "calls");
        let graph = SymbolGraph::build_from_storage(&storage).unwrap();
        let cycles = graph.detect_cycles();
        assert_eq!(cycles.len(), 1);
        assert_eq!(cycles[0].len(), 3);
    }

    #[test]
    fn test_build_under_100ms_for_10k_symbols() {
        // ~10K symbols with a handful of relations — build must be <100ms.
        let mut storage = CodeGraph::open_memory().unwrap();
        let mut symbols = Vec::new();
        for i in 0..10_000 {
            symbols.push(SymbolInsert {
                name: format!("fn_{i}"),
                kind: "function".to_string(),
                file_path: format!("mod_{}.go", i % 100),
                line: 1,
                col: 0,
                language: Some("go".to_string()),
                doc: None,
                embedding: None,
            });
        }
        storage.batch_insert_symbols(symbols).unwrap();
        for i in 0..10_000 {
            insert_relation(
                &mut storage,
                i as i64 + 1,
                ((i + 1) % 10_000) as i64 + 1,
                "calls",
            );
        }
        let start = std::time::Instant::now();
        let graph = SymbolGraph::build_from_storage(&storage).unwrap();
        let elapsed = start.elapsed();
        assert_eq!(graph.symbol_count(), 10_000);
        assert!(elapsed.as_millis() < 100, "build took {:?}", elapsed);
    }
}
