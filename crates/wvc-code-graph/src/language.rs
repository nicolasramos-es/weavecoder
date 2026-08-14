//! Language detection by file extension.

/// Supported programming languages.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    Go,
    Python,
    Typescript,
    Rust,
}

/// Map file extensions to tree-sitter languages.
///
/// Returns the tree-sitter language function and the language enum.
pub fn detect_language(ext: &str) -> Option<tree_sitter::Language> {
    match ext {
        "go" => Some(tree_sitter_go::LANGUAGE.into()),
        "py" | "pyi" => Some(tree_sitter_python::LANGUAGE.into()),
        "ts" | "tsx" => Some(tree_sitter_typescript::LANGUAGE_TSX.into()),
        "rs" => Some(tree_sitter_rust::LANGUAGE.into()),
        _ => None,
    }
}

/// Get a human-readable name for a language.
#[allow(dead_code)]
pub fn language_name(lang: Language) -> &'static str {
    match lang {
        Language::Go => "go",
        Language::Python => "python",
        Language::Typescript => "typescript",
        Language::Rust => "rust",
    }
}

/// Detect language from extension with a Language enum for downstream use.
#[allow(dead_code)]
pub fn detect_language_enum(ext: &str) -> Option<Language> {
    match ext {
        "go" => Some(Language::Go),
        "py" | "pyi" => Some(Language::Python),
        "ts" | "tsx" => Some(Language::Typescript),
        "rs" => Some(Language::Rust),
        _ => None,
    }
}
