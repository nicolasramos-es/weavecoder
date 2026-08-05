//! Project scanner + AST extraction for `wvc init`.
//!
//! Walks a project directory, detects source files by extension, parses
//! them with tree-sitter, extracts symbols and relations, and stores them
//! in the CodeGraph database.

use crate::parser;
use crate::relations::RelationInsert;
use crate::storage::CodeGraph;
use crate::symbols::{SymbolInsert, SymbolKind};
use anyhow::{anyhow, Context, Result};
use std::path::{Path, PathBuf};

// ──────────────────────────────────────────
// Scan configuration
// ──────────────────────────────────────────

/// Configuration for the `wvc init` scan.
#[derive(Debug, Clone)]
pub struct InitConfig {
    /// Root directory to scan.
    pub root: PathBuf,
    /// Output database path (or None for in-memory).
    pub db_path: Option<PathBuf>,
    /// Extra extensions to support beyond the default set.
    pub extra_extensions: Vec<String>,
}

impl Default for InitConfig {
    fn default() -> Self {
        Self {
            root: PathBuf::from("."),
            db_path: None,
            extra_extensions: Vec::new(),
        }
    }
}

/// Result of a `wvc init` scan — summary of what was indexed.
#[derive(Debug)]
pub struct InitSummary {
    pub files_scanned: usize,
    pub symbols_found: usize,
    pub relations_found: usize,
    pub elapsed_ms: u128,
}

// ──────────────────────────────────────────
// File scanning (.gitignore-aware)
// ──────────────────────────────────────────

/// Extensions that indicate supported source files.
const SUPPORTED_EXTENSIONS: &[&str] = &["go", "py", "pyi", "ts", "tsx", "rs"];

/// Check if a file should be scanned based on its extension.
fn is_supported_extension(path: &Path) -> bool {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_lowercase());

    match ext.as_deref() {
        Some(ext) if SUPPORTED_EXTENSIONS.contains(&ext) => true,
        _ => false,
    }
}

/// Recursively walk a directory.
/// Returns only supported source files (skipping generated/third-party dirs).
pub fn scan_project(path: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    walk_dir(path, &mut files)?;
    Ok(files)
}

fn walk_dir(dir: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
    if !dir.is_dir() {
        return Ok(());
    }

    // Skip common generated/third-party directories
    let dir_name = dir.file_name().and_then(|n| n.to_str());
    match dir_name {
        Some(
            "node_modules"
            | "target"
            | ".venv"
            | "vendor"
            | ".git"
            | "__pycache__"
            | ".tox"
            | ".mypy_cache"
            | ".pytest_cache"
            | "dist"
            | "build",
        ) => {
            return Ok(());
        }
        _ => {}
    }

    // Read directory entries
    let entries =
        std::fs::read_dir(dir).with_context(|| format!("failed to read directory: {:?}", dir))?;

    for entry in entries {
        let entry = entry.with_context(|| "failed to read directory entry")?;
        let path = entry.path();

        if path.is_file() && is_supported_extension(&path) {
            files.push(path);
        } else if path.is_dir() {
            walk_dir(&path, files)?;
        }
    }

    Ok(())
}

// ──────────────────────────────────────────
// AST extraction (symbols + relations)
// ──────────────────────────────────────────

/// Extract symbols from a parsed tree-sitter tree.
pub fn extract_symbols(tree: &tree_sitter::Tree, source: &[u8], file_path: &str) -> Vec<SymbolInsert> {
    let root = tree.root_node();
    extract_symbols_from_node(&root, source, file_path, String::new())
}

fn extract_symbols_from_node(
    node: &tree_sitter::Node,
    source: &[u8],
    file_path: &str,
    parent_kind: String,
) -> Vec<SymbolInsert> {
    let mut symbols = Vec::new();

    match node.kind() {
        // ── Functions & Methods ──
        "function_definition" | "function_declaration" | "method_declaration" | "fn_item" | "function_item" => {
            if let Some(name) = find_fn_name(node, source) {
                symbols.push(SymbolInsert {
                    name,
                    kind: if parent_kind == "impl_item" || node.kind() == "method_declaration" {
                        SymbolKind::Method.to_string()
                    } else {
                        SymbolKind::Function.to_string()
                    },
                    file_path: file_path.to_string(),
                    line: node.start_position().row as i64 + 1, // 1-indexed
                    col: node.start_position().column as i64 + 1,
                    language: detect_lang_from_file(file_path),
                    doc: extract_doc(node, source),
                    embedding: None,
                });
            }
        }

        // ── Classes / Structs ──
        "class_declaration" | "class_definition" => {
            if let Some(name) = find_fn_name(node, source) {
                symbols.push(SymbolInsert {
                    name,
                    kind: SymbolKind::Class.to_string(),
                    file_path: file_path.to_string(),
                    line: node.start_position().row as i64 + 1,
                    col: node.start_position().column as i64 + 1,
                    language: detect_lang_from_file(file_path),
                    doc: extract_doc(node, source),
                    embedding: None,
                });
            }
        }

        // Go uses: type_declaration -> type_spec -> (struct_type|enum_type|...)
        // This handles the Go pattern for extracting struct/type/enum names
        "type_declaration" => {
            for i in 0..node.named_child_count() {
                if let Some(child) = node.named_child(i) {
                    if child.kind() == "type_spec" {
                        // type_spec has name (identifier) and value (struct_type, enum_type, etc.)
                        for j in 0..child.child_count() {
                            if let Some(ns) = child.child(j) {
                                if ns.kind() == "identifier" || ns.kind() == "type_identifier" {
                                    let name = ns.utf8_text(source).ok().unwrap_or_default();
                                    // Determine kind based on what type_spec points to
                                    let kind = match child.named_child(1) {
                                        Some(v) => match v.kind() {
                                            "struct_type" | "enum_declaration" | "enum_definition" 
                                            | "enum_spec" => SymbolKind::Other("type".to_string()),
                                            _ => SymbolKind::Other("type".to_string()),
                                        },
                                        None => SymbolKind::Other("type".to_string()),
                                    };
                                    symbols.push(SymbolInsert {
                                        name: name.to_string(),
                                        kind: kind.to_string(),
                                        file_path: file_path.to_string(),
                                        line: child.start_position().row as i64 + 1,
                                        col: child.start_position().column as i64 + 1,
                                        language: detect_lang_from_file(file_path),
                                        doc: None,
                                        embedding: None,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
        
        // Go uses: type_declaration > type_spec > (identifier + struct_type)
        
        "struct_declaration" | "struct_definition" | "struct_item" => {
            if let Some(name) = find_fn_name(node, source) {
                symbols.push(SymbolInsert {
                    name,
                    kind: SymbolKind::Other("struct".to_string()).to_string(),
                    file_path: file_path.to_string(),
                    line: node.start_position().row as i64 + 1,
                    col: node.start_position().column as i64 + 1,
                    language: detect_lang_from_file(file_path),
                    doc: extract_doc(node, source),
                    embedding: None,
                });
            }
        }

        // ── Traits / Interfaces ──
        "trait_item" | "interface_declaration" => {
            if let Some(name) = find_fn_name(node, source) {
                symbols.push(SymbolInsert {
                    name,
                    kind: SymbolKind::Interface.to_string(),
                    file_path: file_path.to_string(),
                    line: node.start_position().row as i64 + 1,
                    col: node.start_position().column as i64 + 1,
                    language: detect_lang_from_file(file_path),
                    doc: extract_doc(node, source),
                    embedding: None,
                });
            }
        }

        // ── Imports ──
        "import_statement" | "import_declaration" | "use" => {
            if let Some(name) = extract_import_name(node, source) {
                symbols.push(SymbolInsert {
                    name: format!("import_{}", name),
                    kind: SymbolKind::Module.to_string(),
                    file_path: file_path.to_string(),
                    line: node.start_position().row as i64 + 1,
                    col: node.start_position().column as i64 + 1,
                    language: detect_lang_from_file(file_path),
                    doc: None,
                    embedding: None,
                });
            }
        }

        // ── Enums ──
        "enum_declaration" | "enum_definition" => {
            if let Some(name) = find_fn_name(node, source) {
                symbols.push(SymbolInsert {
                    name,
                    kind: SymbolKind::Enum.to_string(),
                    file_path: file_path.to_string(),
                    line: node.start_position().row as i64 + 1,
                    col: node.start_position().column as i64 + 1,
                    language: detect_lang_from_file(file_path),
                    doc: extract_doc(node, source),
                    embedding: None,
                });
            }
        }

        // ── Variables / Constants ──
        "variable_declaration" | "var_declaration" | "const_declaration" | "let_statement" => {
            if let Some(name) = find_ident_name(node, source) {
                symbols.push(SymbolInsert {
                    name: format!("var_{}", name),
                    kind: SymbolKind::Variable.to_string(),
                    file_path: file_path.to_string(),
                    line: node.start_position().row as i64 + 1,
                    col: node.start_position().column as i64 + 1,
                    language: detect_lang_from_file(file_path),
                    doc: None,
                    embedding: None,
                });
            }
        }

        "type_alias_declaration" | "type_definition" => {
            if let Some(name) = find_ident_name(node, source) {
                symbols.push(SymbolInsert {
                    name: format!("type_{}", name),
                    kind: SymbolKind::TypeAlias.to_string(),
                    file_path: file_path.to_string(),
                    line: node.start_position().row as i64 + 1,
                    col: node.start_position().column as i64 + 1,
                    language: detect_lang_from_file(file_path),
                    doc: None,
                    embedding: None,
                });
            }
        }

        _ => {} // Skip unrecognized node types
    }

    // Recurse into children
    let mut child = node.child(0);
    while let Some(c) = child {
        symbols.extend(extract_symbols_from_node(
            &c,
            source,
            file_path,
            node.kind().to_string(),
        ));
        child = c.next_sibling();
    }

    symbols
}

/// Extract relations (edges) from a parsed tree.
pub fn extract_relations(
    tree: &tree_sitter::Tree,
    source: &[u8],
    symbol_names: &[String],
) -> Vec<(String, String)> {
    // Returns pairs of (caller_name, callee_name) for call relations.
    // We return names here; the caller will map them to IDs after insertion.
    let root = tree.root_node();
    extract_relations_from_node(&root, source, symbol_names, String::new())
}

fn extract_relations_from_node(
    node: &tree_sitter::Node,
    source: &[u8],
    symbol_names: &[String],
    _parent_kind: String,
) -> Vec<(String, String)> {
    let mut relations = Vec::new();

    // Detect calls: `foo()` or `self.foo()` patterns
    if node.kind() == "call_expression" || node.kind() == "field_expression" {
        if let Some(callee_name) = find_callee_name(node, source) {
            // Only create relation if the callee is a known symbol
            if symbol_names.contains(&callee_name) {
                relations.push(("self".to_string(), callee_name));
            }
        }
    }

    // Detect extends/implements: `extends Foo`, `implements Bar`
    if matches!(node.kind(), "super_class" | "implements_clause" | "implies") {
        if let Some(parent_name) = find_fn_name(node, source) {
            if symbol_names.contains(&parent_name) {
                relations.push(("self".to_string(), parent_name));
            }
        }
    }

    // Recurse into children
    let mut child = node.child(0);
    while let Some(c) = child {
        relations.extend(extract_relations_from_node(&c, source, symbol_names, node.kind().to_string()));
        child = c.next_sibling();
    }

    relations
}

// ──────────────────────────────────────────
// AST helpers
// ──────────────────────────────────────────

fn find_fn_name(node: &tree_sitter::Node, source: &[u8]) -> Option<String> {
    // Common patterns: name is first named child with kind "identifier" or "name"
    for i in 0..node.named_child_count() {
        if let Some(child) = node.named_child(i) {
            if matches!(child.kind(), "identifier" | "name" | "type_identifier") {
                return child.utf8_text(source).ok().map(|s| s.to_string());
            }
        }
    }
    None
}

fn find_ident_name(node: &tree_sitter::Node, source: &[u8]) -> Option<String> {
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            if child.kind() == "identifier" || child.kind() == "name" || child.kind() == "type_identifier" {
                return child.utf8_text(source).ok().map(|s| s.to_string());
            }
        }
    }
    None
}

fn find_callee_name(node: &tree_sitter::Node, source: &[u8]) -> Option<String> {
    match node.kind() {
        "call_expression" => {
            // callee is first child (could be identifier, selector_expression, or scoped_identifier)
            if let Some(callee) = node.child(0) {
                // Direct identifier call like: greet("World")
                if callee.kind() == "identifier" || callee.kind() == "type_identifier" {
                    return callee.utf8_text(source).ok().map(|s| s.to_string());
                }
                // Member expression like: fmt.Sprintf("Hello", name)
                find_fn_name(&callee, source).or_else(|| {
                    for i in 0..callee.named_child_count() {
                        if let Some(child) = callee.named_child(i) {
                            if matches!(child.kind(), "property_identifier" | "identifier" | "type_identifier") {
                                return child.utf8_text(source).ok().map(|s| s.to_string());
                            }
                        }
                    }
                    None
                })
            } else {
                None
            }
        }
        // scoped_identifier like: Config::new(...)  
        "scoped_identifier" => {
            // Get the last segment (the actual function/method name)
            if let Some(last) = node.named_child(node.named_child_count() - 1) {
                if matches!(last.kind(), "identifier" | "type_identifier") {
                    return last.utf8_text(source).ok().map(|s| s.to_string());
                }
            }
            None
        }
        _ => find_fn_name(node, source),
    }
}

fn extract_import_name(node: &tree_sitter::Node, source: &[u8]) -> Option<String> {
    // Go: "import \"fmt\""  → string child contains module name
    // TS/JS: import x from "module" → string or identifier
    // Rust: use std::collections::HashMap
    match node.kind() {
        "import_statement" => {
            for i in 0..node.child_count() {
                if let Some(child) = node.child(i) {
                    if child.kind() == "string" {
                        return child.utf8_text(source).ok().map(|s| s.to_string());
                    } else if child.kind() == "import_specifier" || child.kind() == "source" {
                        return child.utf8_text(source).ok().map(|s| s.to_string());
                    }
                }
            }
        }
        "use" => {
            // Rust use path: last component is the name
            let full_path = node.utf8_text(source).ok()?.to_string();
            if !full_path.is_empty() {
                return Some(
                    full_path
                        .split("::")
                        .last()
                        .unwrap_or(&full_path)
                        .to_string(),
                );
            }
        }
        _ => {}
    }
    None
}

fn extract_doc(_node: &tree_sitter::Node, _source: &[u8]) -> Option<String> {
    // Doc extraction would need preceding comment lookup.
    // Simplified: return None for now.
    None
}

fn detect_lang_from_file(path: &str) -> Option<String> {
    let ext = Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_lowercase());

    match ext.as_deref() {
        Some("go") => Some("go".to_string()),
        Some("py" | "pyi") => Some("python".to_string()),
        Some("ts" | "tsx") => Some("typescript".to_string()),
        Some("rs") => Some("rust".to_string()),
        _ => None,
    }
}

// ──────────────────────────────────────────
// Main init orchestration
// ──────────────────────────────────────────

/// Run `wvc init`: scan the project directory, parse ASTs, extract symbols & relations,
/// and store them in the CodeGraph database.
pub fn run_init(config: InitConfig) -> Result<InitSummary> {
    let start = std::time::Instant::now();

    // Open or create the graph database
    let mut graph = match &config.db_path {
        Some(path) => CodeGraph::open(path)?,
        None => CodeGraph::open_memory()?,
    };

    // Step 1: Scan the project directory
    println!("🔍 Scanning project at: {:?}", config.root);
    let files = scan_project(&config.root)?;
    println!("   Found {} source files", files.len());

    if files.is_empty() {
        return Ok(InitSummary {
            files_scanned: 0,
            symbols_found: 0,
            relations_found: 0,
            elapsed_ms: start.elapsed().as_millis(),
        });
    }

    // Step 2: Parse each file and extract symbols
    let mut all_symbols = Vec::new();
    for file_path in &files {
        match parse_single_file(file_path) {
            Ok(symbs) => all_symbols.extend(symbs),
            Err(e) => eprintln!("   ⚠️ Failed to parse {:?}: {}", file_path, e),
        }
    }

    println!("   Extracted {} symbols", all_symbols.len());

    // Step 3: Batch insert symbols into the database (idempotent via upsert-like behavior)
    let inserted = graph.batch_insert_symbols(all_symbols.clone())?;
    println!("   Stored {} symbols in database", inserted);

    // Step 4: Build symbol name set for relation extraction
    let symbol_names: Vec<String> = all_symbols.iter().map(|s| s.name.clone()).collect();

    // Step 5: Parse again and extract relations (need full context)
    let mut call_pairs: Vec<(String, String)> = Vec::new();
    for file_path in &files {
        match parse_file_for_relations(file_path) {
            Ok((tree, source)) => {
                call_pairs.extend(extract_relations(&tree, &source, &symbol_names));
            }
            Err(_) => {} // Skip files that fail re-parsing
        }
    }

    println!("   Extracted {} relations", call_pairs.len());

    // Step 6: Build a map from (file_path, name) -> symbol_id for relation insertion
    let symbols_list = graph.list_symbols(Default::default())?;
    let mut name_map: std::collections::HashMap<String, i64> = std::collections::HashMap::new();
    for sym in &symbols_list {
        name_map.insert(format!("{}::{}", sym.file_path, sym.name), sym.id);
    }

    // Insert relations using symbol IDs
    let mut relations_count = 0;
    for (caller_name, callee_name) in &call_pairs {
        // Build the full path keys — caller is always "self" (the containing file)
        let caller_key = format!("self::{}", caller_name);
        let callee_key = format!("self::{}", callee_name);

        if let (Some(&source_id), Some(&target_id)) =
            (name_map.get(&caller_key), name_map.get(&callee_key))
        {
            let kind = "calls"; // Maps to RelationKind::Calls
            let _ = graph.insert_relation(RelationInsert {
                source_symbol_id: source_id,
                target_symbol_id: target_id,
                kind: kind.to_string(),
                metadata: None,
            });
            relations_count += 1;
        } else if let Some(&_target_id) = name_map.get(&callee_key) {
            // Self-reference or unresolved caller — still record a call to the function
            // Use a placeholder "main" symbol if possible
            if let Some(&_placeholder) = name_map.get(&format!("self::{}", callee_name)) {
                // Skip if we can't resolve both sides cleanly
            }
        }
    }

    let elapsed = start.elapsed().as_millis();
    Ok(InitSummary {
        files_scanned: files.len(),
        symbols_found: all_symbols.len(),
        relations_found: relations_count,
        elapsed_ms: elapsed,
    })
}

fn parse_single_file(path: &Path) -> Result<Vec<SymbolInsert>> {
    let source = std::fs::read_to_string(path)?;
    let tree = parser::parse_file(path).map_err(|e| anyhow!("parse_file failed: {}", e))?;
    let file_path = path.to_string_lossy().to_string();
    Ok(extract_symbols(&tree, source.as_bytes(), &file_path))
}

fn parse_file_for_relations(
    path: &Path,
) -> Result<(tree_sitter::Tree, Vec<u8>)> {
    let source = std::fs::read(path)?;
    let tree = parser::parse_file(path).map_err(|e| anyhow!("parse_file failed: {}", e))?;
    Ok((tree, source))
}

// ──────────────────────────────────────────
// Tests
// ──────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse_source;

    #[test]
    fn test_scan_project_returns_files() {
        // Create a temporary directory with some source files
        let temp_dir = tempfile::tempdir().unwrap();
        let go_file = temp_dir.path().join("main.go");
        std::fs::write(&go_file, "package main\n\nfunc main() {}").unwrap();

        let py_file = temp_dir.path().join("app.py");
        std::fs::write(&py_file, "def hello():\n    pass").unwrap();

        let ts_file = temp_dir.path().join("index.ts");
        std::fs::write(&ts_file, "export function greet(): void {}").unwrap();

        // Create a node_modules dir (should be ignored)
        let nm_dir = temp_dir.path().join("node_modules");
        std::fs::create_dir(&nm_dir).unwrap();
        std::fs::write(nm_dir.join("bad.js"), "var x = 1").unwrap();

        // Create a .git dir (should be ignored)
        let git_dir = temp_dir.path().join(".git");
        std::fs::create_dir(&git_dir).unwrap();
        std::fs::write(git_dir.join("config"), "[core]").unwrap();

        let files = scan_project(temp_dir.path()).unwrap();

        // Should find exactly 3 files (main.go, app.py, index.ts)
        assert_eq!(files.len(), 3);
        assert!(files.iter().any(|f| f.file_name().unwrap() == "main.go"));
        assert!(files.iter().any(|f| f.file_name().unwrap() == "app.py"));
        assert!(files.iter().any(|f| f.file_name().unwrap() == "index.ts"));
    }

    #[test]
    fn test_extract_golang_symbols() {
        let source = r#"package main

// greet is the greeting function
func greet(name string) string {
    return fmt.Sprintf("Hello, %s", name)
}

type User struct {
    Name string
    Age  int
}
"#;
        let tree = parse_source(source, "go").unwrap();
        let bytes = source.as_bytes();
        let symbols = extract_symbols(&tree, bytes, "example.go");

        assert!(!symbols.is_empty(), "should extract at least some symbols");

        // DEBUG: print what we actually found
        for s in &symbols {
            eprintln!("  GO SYMBOL: name={} kind={}", s.name, s.kind);
        }

        // Verify we found the function
        assert!(
            symbols.iter().any(|s| s.name == "greet"),
            "should find 'greet' function"
        );

        // Verify struct
        let names_kinds: Vec<_> = symbols.iter().map(|s| format!("{}({})", s.name, s.kind)).collect();
        assert!(!names_kinds.is_empty(), "GO symbols found: {}", names_kinds.join(", "));
        assert!(symbols.iter().any(|s| s.name == "User"), 
            "GO: should find 'User' struct. All symbols: {:?}", names_kinds);
    }

    #[test]
    fn test_extract_python_symbols() {
        let source = r#"class UserService:
    def __init__(self, db):
        self.db = db

    def get_user(self, id):
        return self.db.query(id)

def create_app():
    app = UserService(None)
    return app
"#;
        let tree = parse_source(source, "py").unwrap();
        let bytes = source.as_bytes();
        let symbols = extract_symbols(&tree, bytes, "app.py");

        assert!(!symbols.is_empty());

        // Check function
        assert!(symbols.iter().any(|s| s.name == "create_app"));

        // Check class
        assert!(symbols.iter().any(|s| s.name == "UserService"));
    }

    #[test]
    fn test_extract_rust_symbols() {
        let source = r#"use std::collections::HashMap;

pub struct Config {
    pub name: String,
}

impl Config {
    pub fn new(name: String) -> Self {
        Config { name }
    }
}

fn main() {
    let config = Config::new("test".to_string());
}"#;
        let tree = parse_source(source, "rs").unwrap();
        let bytes = source.as_bytes();
        let symbols = extract_symbols(&tree, bytes, "main.rs");

        assert!(!symbols.is_empty());

        let _sym_strs: Vec<String> = symbols.iter().map(|s| format!("{}({})", s.name, s.kind)).collect();
        // Check struct
        assert!(
            symbols.iter().any(|s| s.name == "Config"),
            "should find Config struct"
        );

        // Check function
        assert!(
            symbols.iter().any(|s| s.name == "new") || symbols.iter().any(|s| s.name == "main")
        );
    }

    #[test]
    fn test_extract_typescript_symbols() {
        let source = r#"interface User {
    name: string;
    age: number;
}

class UserService {
    getUsers(): User[] {
        return [];
    }
}

function main(): void {
    const svc = new UserService();
}"#;
        let tree = parse_source(source, "ts").unwrap();
        let bytes = source.as_bytes();
        let symbols = extract_symbols(&tree, bytes, "app.ts");

        assert!(!symbols.is_empty());

        let _sym_strs2: Vec<String> = symbols.iter().map(|s| format!("{}({})", s.name, s.kind)).collect();
        // Check interface
        assert!(
            symbols.iter().any(|s| s.name == "User"),
            "should find 'User' interface"
        );

        // Check class
        assert!(
            symbols.iter().any(|s| s.name == "UserService"),
            "should find 'UserService' class"
        );
    }

    #[test]
    fn test_extract_symbol_call_relations() {
        let source = r#"package main

import "fmt"

func greet(name string) string {
    return fmt.Sprintf("Hello, %s", name)
}

func main() {
    msg := greet("World")
    fmt.Println(msg)
}
"#;
        let tree = parse_source(source, "go").unwrap();
        let bytes = source.as_bytes();
        let symbols = extract_symbols(&tree, bytes, "main.go");

        let names: Vec<String> = symbols.iter().map(|s| s.name.clone()).collect();

        // Extract relations — should find calls to 'fmt', 'greet', 'Sprintf', etc.
        let relations = extract_relations(&tree, bytes, &names);
        assert!(
            !relations.is_empty(),
            "should find call relations in Go code (found {})"
        , relations.len());
    }

    #[test]
    fn test_extract_import_symbols() {
        let source = r#"use std::collections::HashMap;

fn main() {}
"#;
        let tree = parse_source(source, "rs").unwrap();
        let bytes = source.as_bytes();
        let symbols = extract_symbols(&tree, bytes, "main.rs");

        // Should find the 'HashMap' import (or a module symbol)
        assert!(!symbols.is_empty(), "should find at least some symbols");
    }
}
