//! Relation data types for the Code Knowledge Graph.

use serde::{Deserialize, Serialize};

/// Kind of relationship between two symbols.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RelationKind {
    Calls,
    Inherits,
    Implements,
    DependsOn,
    Contains,
    References,
    Defines,
    Uses,
    Other(String),
}

impl std::fmt::Display for RelationKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RelationKind::Calls => write!(f, "calls"),
            RelationKind::Inherits => write!(f, "inherits"),
            RelationKind::Implements => write!(f, "implements"),
            RelationKind::DependsOn => write!(f, "depends_on"),
            RelationKind::Contains => write!(f, "contains"),
            RelationKind::References => write!(f, "references"),
            RelationKind::Defines => write!(f, "defines"),
            RelationKind::Uses => write!(f, "uses"),
            RelationKind::Other(kind) => write!(f, "{kind}"),
        }
    }
}

impl From<String> for RelationKind {
    fn from(s: String) -> Self {
        match s.to_lowercase().as_str() {
            "calls" => RelationKind::Calls,
            "inherits" => RelationKind::Inherits,
            "implements" => RelationKind::Implements,
            "depends_on" => RelationKind::DependsOn,
            "contains" => RelationKind::Contains,
            "references" => RelationKind::References,
            "defines" => RelationKind::Defines,
            "uses" => RelationKind::Uses,
            other => RelationKind::Other(other.to_string()),
        }
    }
}

/// A relation to be inserted into the graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationInsert {
    pub source_symbol_id: i64,
    pub target_symbol_id: i64,
    pub kind: String,
    pub metadata: Option<String>,
}

/// A fully loaded relation from the database.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relation {
    pub source_symbol_id: i64,
    pub target_symbol_id: i64,
    pub kind: String,
    pub metadata: Option<String>,
    pub source_name: String,
    pub target_name: String,
}
