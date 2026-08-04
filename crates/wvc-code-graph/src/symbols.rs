//! Symbol data types for the Code Knowledge Graph.

use serde::{Deserialize, Serialize};

/// Kind of code entity (function, class, module, etc.).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SymbolKind {
    Function,
    Class,
    Module,
    Variable,
    Constant,
    Method,
    Property,
    Interface,
    Enum,
    TypeAlias,
    Macro,
    Package,
    File,
    Directory,
    Other(String),
}

impl std::fmt::Display for SymbolKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SymbolKind::Function => write!(f, "function"),
            SymbolKind::Class => write!(f, "class"),
            SymbolKind::Module => write!(f, "module"),
            SymbolKind::Variable => write!(f, "variable"),
            SymbolKind::Constant => write!(f, "constant"),
            SymbolKind::Method => write!(f, "method"),
            SymbolKind::Property => write!(f, "property"),
            SymbolKind::Interface => write!(f, "interface"),
            SymbolKind::Enum => write!(f, "enum"),
            SymbolKind::TypeAlias => write!(f, "type_alias"),
            SymbolKind::Macro => write!(f, "macro"),
            SymbolKind::Package => write!(f, "package"),
            SymbolKind::File => write!(f, "file"),
            SymbolKind::Directory => write!(f, "directory"),
            SymbolKind::Other(kind) => write!(f, "{kind}"),
        }
    }
}

impl From<String> for SymbolKind {
    fn from(s: String) -> Self {
        match s.to_lowercase().as_str() {
            "function" => SymbolKind::Function,
            "class" => SymbolKind::Class,
            "module" => SymbolKind::Module,
            "variable" => SymbolKind::Variable,
            "constant" => SymbolKind::Constant,
            "method" => SymbolKind::Method,
            "property" => SymbolKind::Property,
            "interface" => SymbolKind::Interface,
            "enum" => SymbolKind::Enum,
            "type_alias" | "typealias" => SymbolKind::TypeAlias,
            "macro" => SymbolKind::Macro,
            "package" => SymbolKind::Package,
            "file" => SymbolKind::File,
            "directory" => SymbolKind::Directory,
            other => SymbolKind::Other(other.to_string()),
        }
    }
}

impl From<&str> for SymbolKind {
    fn from(s: &str) -> Self {
        s.to_string().into()
    }
}

/// A symbol to be inserted into the graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolInsert {
    pub name: String,
    pub kind: String,
    pub file_path: String,
    pub line: i64,
    pub col: i64,
    pub language: Option<String>,
    pub doc: Option<String>,
    pub embedding: Option<Vec<u8>>,
}

/// A fully loaded symbol from the database.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Symbol {
    pub id: i64,
    pub name: String,
    pub kind: String,
    pub file_path: String,
    pub line: i64,
    pub col: i64,
    pub language: Option<String>,
    pub doc: Option<String>,
    pub embedding: Option<Vec<u8>>,
}

/// Query filters for listing symbols.
#[derive(Debug, Clone, Default)]
pub struct SymbolQuery {
    pub kind: Option<String>,
    pub file_path: Option<String>,
    pub language: Option<String>,
    pub limit: Option<usize>,
}

/// A symbol returned from an FTS search, with ranking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolResult {
    pub id: i64,
    pub name: String,
    pub kind: String,
    pub file_path: String,
    pub line: i64,
    pub col: i64,
    pub language: Option<String>,
    pub doc: Option<String>,
    pub embedding: Option<Vec<u8>>,
}
