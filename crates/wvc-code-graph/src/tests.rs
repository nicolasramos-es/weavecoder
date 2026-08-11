//! Unit tests for language detection and parsing.

use crate::{detect_language, parse_str, Language};
use crate::language::detect_language_enum;

// ── Language detection tests ──────────────────────────────────────────

#[test]
fn test_detect_language_go() {
    let lang = detect_language("go");
    assert!(lang.is_some());
}

#[test]
fn test_detect_language_python() {
    let lang = detect_language("py");
    assert!(lang.is_some());
    let lang2 = detect_language("pyi");
    assert!(lang2.is_some());
}

#[test]
fn test_detect_language_typescript() {
    let lang_ts = detect_language("ts");
    assert!(lang_ts.is_some());
    let lang_tsx = detect_language("tsx");
    assert!(lang_tsx.is_some());
}

#[test]
fn test_detect_language_rust() {
    let lang = detect_language("rs");
    assert!(lang.is_some());
}

#[test]
fn test_detect_language_unsupported() {
    let lang = detect_language("js");
    assert!(lang.is_none());
    let lang2 = detect_language("java");
    assert!(lang2.is_none());
    let lang3 = detect_language("rb");
    assert!(lang3.is_none());
}

#[test]
fn test_detect_language_enum() {
    assert_eq!(detect_language_enum("go"), Some(Language::Go));
    assert_eq!(detect_language_enum("py"), Some(Language::Python));
    assert_eq!(detect_language_enum("pyi"), Some(Language::Python));
    assert_eq!(detect_language_enum("ts"), Some(Language::Typescript));
    assert_eq!(detect_language_enum("tsx"), Some(Language::Typescript));
    assert_eq!(detect_language_enum("rs"), Some(Language::Rust));
    assert!(detect_language_enum("js").is_none());
    assert!(detect_language_enum("txt").is_none());
}

// ── Parsing tests ─────────────────────────────────────────────────────

#[test]
fn test_parse_go_snippet() {
    let source = "package main\n\nimport \"fmt\"\n\nfunc main() {\n    fmt.Println(\"hello\")\n}\n";
    let lang = detect_language("go").unwrap();
    let tree = parse_str(source, lang);
    assert!(tree.is_ok(), "Go parsing should succeed");
    let tree = tree.unwrap();
    let root = tree.root_node();
    assert!(!root.has_error(), "Go parse tree should have no errors");
}

#[test]
fn test_parse_python_snippet() {
    let source = "def hello(name: str) -> str:\n    return f\"Hello, {name}\"\n\nclass Greeter:\n    def __init__(self, msg: str):\n        self.msg = msg\n";
    let lang = detect_language("py").unwrap();
    let tree = parse_str(source, lang);
    assert!(tree.is_ok(), "Python parsing should succeed");
    let tree = tree.unwrap();
    let root = tree.root_node();
    assert!(!root.has_error(), "Python parse tree should have no errors");
}

#[test]
fn test_parse_typescript_snippet() {
    let source = "interface User {\n    name: string;\n    age: number;\n}\n\nfunction greet(user: User): string {\n    return `Hello, ${user.name}`;\n}\n";
    let lang = detect_language("ts").unwrap();
    let tree = parse_str(source, lang);
    assert!(tree.is_ok(), "TypeScript parsing should succeed");
    let tree = tree.unwrap();
    let root = tree.root_node();
    assert!(!root.has_error(), "TypeScript parse tree should have no errors");
}

#[test]
fn test_parse_rust_snippet() {
    let source = "fn main() {\n    let message = String::from(\"hello\");\n    println!(\"{}\", message);\n}\n\nstruct Counter {\n    count: u32,\n}\n\nimpl Counter {\n    fn new() -> Self {\n        Counter { count: 0 }\n    }\n}\n";
    let lang = detect_language("rs").unwrap();
    let tree = parse_str(source, lang);
    assert!(tree.is_ok(), "Rust parsing should succeed");
    let tree = tree.unwrap();
    let root = tree.root_node();
    assert!(!root.has_error(), "Rust parse tree should have no errors");
}

#[test]
fn test_parse_unsupported_extension() {
    let lang = detect_language("js");
    assert!(lang.is_none(), "JavaScript should not be supported");
}

#[test]
fn test_parse_empty_source() {
    let source = "";
    let lang = detect_language("py").unwrap();
    let tree = parse_str(source, lang);
    assert!(tree.is_ok(), "Empty source should parse");
}

#[test]
fn test_parse_go_invalid_syntax() {
    let source = "package main\nfunc broken {";
    let lang = detect_language("go").unwrap();
    let tree = parse_str(source, lang);
    assert!(tree.is_ok(), "tree-sitter is tolerant of syntax errors");
    let tree = tree.unwrap();
    let root = tree.root_node();
    let _ = root.has_error();
}

#[test]
fn test_parse_python_invalid_syntax() {
    let source = "def broken(";
    let lang = detect_language("py").unwrap();
    let tree = parse_str(source, lang);
    assert!(tree.is_ok(), "tree-sitter is tolerant of syntax errors");
}

#[test]
fn test_parse_typescript_invalid_syntax() {
    let source = "const x = ;";
    let lang = detect_language("ts").unwrap();
    let tree = parse_str(source, lang);
    assert!(tree.is_ok(), "tree-sitter is tolerant of syntax errors");
}

#[test]
fn test_parse_rust_invalid_syntax() {
    let source = "fn broken {";
    let lang = detect_language("rs").unwrap();
    let tree = parse_str(source, lang);
    assert!(tree.is_ok(), "tree-sitter is tolerant of syntax errors");
}
