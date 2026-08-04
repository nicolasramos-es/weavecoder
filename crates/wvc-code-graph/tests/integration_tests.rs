use wvc_code_graph::{
    CodeGraph, SymbolInsert, SymbolQuery, RelationInsert, FtsQuery,
    SCHEMA_VERSION,
};

fn make_symbol(name: &str, kind: &str, file: &str, line: i64) -> SymbolInsert {
    SymbolInsert {
        name: name.to_string(),
        kind: kind.to_string(),
        file_path: file.to_string(),
        line,
        col: 0,
        language: Some("rust".to_string()),
        doc: Some(format!("Documentation for {name}")),
        embedding: None,
    }
}

fn make_relation(src: i64, tgt: i64, kind: &str) -> RelationInsert {
    RelationInsert {
        source_symbol_id: src,
        target_symbol_id: tgt,
        kind: kind.to_string(),
        metadata: None,
    }
}

#[test]
fn test_schema_version() {
    let graph = CodeGraph::open_memory().unwrap();
    assert_eq!(graph.schema_version(), SCHEMA_VERSION);
}

#[test]
fn test_tables_exist() {
    let graph = CodeGraph::open_memory().unwrap();
    assert_eq!(graph.symbol_count().unwrap(), 0);
    assert_eq!(graph.relation_count().unwrap(), 0);
}

#[test]
fn test_insert_symbol() {
    let mut graph = CodeGraph::open_memory().unwrap();
    let sym = make_symbol("my_function", "function", "src/main.rs", 10);
    let id = graph.insert_symbol(sym).unwrap();
    assert!(id > 0);
    assert_eq!(graph.symbol_count().unwrap(), 1);
}

#[test]
fn test_insert_multiple_symbols() {
    let mut graph = CodeGraph::open_memory().unwrap();
    let symbols = vec![
        make_symbol("func_a", "function", "src/a.rs", 1),
        make_symbol("func_b", "function", "src/b.rs", 5),
        make_symbol("MyClass", "class", "src/c.rs", 20),
    ];
    for sym in symbols {
        graph.insert_symbol(sym).unwrap();
    }
    assert_eq!(graph.symbol_count().unwrap(), 3);
}

#[test]
fn test_get_symbol() {
    let mut graph = CodeGraph::open_memory().unwrap();
    let sym = make_symbol("get_user", "function", "src/users.rs", 42);
    let id = graph.insert_symbol(sym).unwrap();

    let found = graph.get_symbol(id).unwrap().unwrap();
    assert_eq!(found.name, "get_user");
    assert_eq!(found.kind, "function");
    assert_eq!(found.file_path, "src/users.rs");
    assert_eq!(found.line, 42);
}

#[test]
fn test_get_symbol_not_found() {
    let graph = CodeGraph::open_memory().unwrap();
    assert!(graph.get_symbol(999).unwrap().is_none());
}

#[test]
fn test_upsert_insert() {
    let mut graph = CodeGraph::open_memory().unwrap();
    let sym = make_symbol("new_func", "function", "src/lib.rs", 100);
    let id = graph.upsert_symbol(sym).unwrap();
    assert!(id > 0);
    assert_eq!(graph.symbol_count().unwrap(), 1);
}

#[test]
fn test_upsert_update() {
    let mut graph = CodeGraph::open_memory().unwrap();
    let sym1 = make_symbol("existing", "function", "src/lib.rs", 10);
    let id = graph.upsert_symbol(sym1).unwrap();

    let sym2 = SymbolInsert {
        name: "existing".to_string(),
        kind: "method".to_string(),
        file_path: "src/lib.rs".to_string(),
        line: 20,
        col: 4,
        language: Some("rust".to_string()),
        doc: Some("Updated doc".to_string()),
        embedding: None,
    };
    let id2 = graph.upsert_symbol(sym2).unwrap();
    assert_eq!(id, id2);
    assert_eq!(graph.symbol_count().unwrap(), 1);

    let updated = graph.get_symbol(id).unwrap().unwrap();
    assert_eq!(updated.kind, "method");
    assert_eq!(updated.line, 20);
    assert_eq!(updated.doc, Some("Updated doc".to_string()));
}

#[test]
fn test_list_symbols_all() {
    let mut graph = CodeGraph::open_memory().unwrap();
    graph.insert_symbol(make_symbol("a", "function", "src/a.rs", 1)).unwrap();
    graph.insert_symbol(make_symbol("b", "class", "src/b.rs", 2)).unwrap();
    graph.insert_symbol(make_symbol("c", "function", "src/c.rs", 3)).unwrap();

    let all = graph.list_symbols(SymbolQuery::default()).unwrap();
    assert_eq!(all.len(), 3);
}

#[test]
fn test_list_symbols_filter_kind() {
    let mut graph = CodeGraph::open_memory().unwrap();
    graph.insert_symbol(make_symbol("func", "function", "src/a.rs", 1)).unwrap();
    graph.insert_symbol(make_symbol("MyClass", "class", "src/b.rs", 2)).unwrap();

    let mut query = SymbolQuery::default();
    query.kind = Some("function".to_string());
    let results = graph.list_symbols(query).unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].name, "func");
}

#[test]
fn test_list_symbols_filter_file() {
    let mut graph = CodeGraph::open_memory().unwrap();
    graph.insert_symbol(make_symbol("a", "function", "src/a.rs", 1)).unwrap();
    graph.insert_symbol(make_symbol("b", "function", "src/b.rs", 2)).unwrap();

    let mut query = SymbolQuery::default();
    query.file_path = Some("src/a.rs".to_string());
    let results = graph.list_symbols(query).unwrap();
    assert_eq!(results.len(), 1);
}

#[test]
fn test_list_symbols_limit() {
    let mut graph = CodeGraph::open_memory().unwrap();
    for i in 0..10 {
        graph.insert_symbol(make_symbol(&format!("sym_{i}"), "function", "src/a.rs", i as i64)).unwrap();
    }

    let mut query = SymbolQuery::default();
    query.limit = Some(3);
    let results = graph.list_symbols(query).unwrap();
    assert_eq!(results.len(), 3);
}

#[test]
fn test_insert_relation() {
    let mut graph = CodeGraph::open_memory().unwrap();
    let src = graph.insert_symbol(make_symbol("caller", "function", "src/a.rs", 1)).unwrap();
    let tgt = graph.insert_symbol(make_symbol("callee", "function", "src/b.rs", 5)).unwrap();

    graph.insert_relation(make_relation(src, tgt, "calls")).unwrap();
    assert_eq!(graph.relation_count().unwrap(), 1);
}

#[test]
fn test_get_relations() {
    let mut graph = CodeGraph::open_memory().unwrap();
    let src = graph.insert_symbol(make_symbol("parent", "class", "src/a.rs", 1)).unwrap();
    let child1 = graph.insert_symbol(make_symbol("child1", "class", "src/b.rs", 10)).unwrap();
    let child2 = graph.insert_symbol(make_symbol("child2", "class", "src/c.rs", 20)).unwrap();

    graph.insert_relation(make_relation(src, child1, "inherits")).unwrap();
    graph.insert_relation(make_relation(src, child2, "inherits")).unwrap();

    let rels = graph.get_relations(src).unwrap();
    assert_eq!(rels.len(), 2);
    assert_eq!(rels[0].kind, "inherits");
    assert_eq!(rels[0].target_name, "child1");
    assert_eq!(rels[1].target_name, "child2");
}

#[test]
fn test_relation_with_metadata() {
    let mut graph = CodeGraph::open_memory().unwrap();
    let src = graph.insert_symbol(make_symbol("mod_a", "module", "src/a.rs", 1)).unwrap();
    let tgt = graph.insert_symbol(make_symbol("fn_b", "function", "src/b.rs", 5)).unwrap();

    graph.insert_relation(RelationInsert {
        source_symbol_id: src,
        target_symbol_id: tgt,
        kind: "contains".to_string(),
        metadata: Some(r#"{"visibility": "pub"}"#.to_string()),
    }).unwrap();

    let rels = graph.get_relations(src).unwrap();
    assert_eq!(rels.len(), 1);
    assert_eq!(rels[0].metadata, Some(r#"{"visibility": "pub"}"#.to_string()));
}

#[test]
fn test_fts_basic_search() {
    let mut graph = CodeGraph::open_memory().unwrap();
    graph.insert_symbol(make_symbol("get_user", "function", "src/users.rs", 10)).unwrap();
    graph.insert_symbol(make_symbol("create_user", "function", "src/users.rs", 20)).unwrap();
    graph.insert_symbol(make_symbol("delete_user", "function", "src/users.rs", 30)).unwrap();

    let results = graph.search_fts(FtsQuery {
        query: "get_user".to_string(),
        limit: None,
    }).unwrap();

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].name, "get_user");
}

#[test]
fn test_fts_prefix_search() {
    let mut graph = CodeGraph::open_memory().unwrap();
    graph.insert_symbol(make_symbol("fn_name", "function", "src/a.rs", 1)).unwrap();
    graph.insert_symbol(make_symbol("fn_named", "function", "src/b.rs", 2)).unwrap();
    graph.insert_symbol(make_symbol("other_fn", "function", "src/c.rs", 3)).unwrap();

    let results = graph.search_fts(FtsQuery {
        query: "fn_name*".to_string(),
        limit: None,
    }).unwrap();

    assert!(results.len() >= 2);
}

#[test]
fn test_fts_substring_search() {
    let mut graph = CodeGraph::open_memory().unwrap();
    graph.insert_symbol(make_symbol("calculate_total", "function", "src/math.rs", 1)).unwrap();
    graph.insert_symbol(make_symbol("total_price", "function", "src/math.rs", 10)).unwrap();
    graph.insert_symbol(make_symbol("subtotal", "function", "src/math.rs", 20)).unwrap();

    let results = graph.search_fts(FtsQuery {
        query: "total".to_string(),
        limit: None,
    }).unwrap();

    assert!(results.len() >= 2);
}

#[test]
fn test_fts_with_doc_search() {
    let mut graph = CodeGraph::open_memory().unwrap();
    let _ = graph.insert_symbol(SymbolInsert {
        name: "process_data".to_string(),
        kind: "function".to_string(),
        file_path: "src/processor.rs".to_string(),
        line: 50,
        col: 0,
        language: Some("rust".to_string()),
        doc: Some("Processes data from the input stream".to_string()),
        embedding: None,
    });

    let results = graph.search_fts(FtsQuery {
        query: "input stream".to_string(),
        limit: None,
    }).unwrap();

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].name, "process_data");
}

#[test]
fn test_fts_limit_results() {
    let mut graph = CodeGraph::open_memory().unwrap();
    for i in 0..20 {
        graph.insert_symbol(make_symbol(&format!("func_{i}"), "function", "src/a.rs", i as i64)).unwrap();
    }

    let results = graph.search_fts(FtsQuery {
        query: "func_*".to_string(),
        limit: Some(5),
    }).unwrap();

    assert_eq!(results.len(), 5);
}

#[test]
fn test_fts_no_match() {
    let mut graph = CodeGraph::open_memory().unwrap();
    graph.insert_symbol(make_symbol("alpha", "function", "src/a.rs", 1)).unwrap();

    let results = graph.search_fts(FtsQuery {
        query: "nonexistent_xyz".to_string(),
        limit: None,
    }).unwrap();

    assert!(results.is_empty());
}

#[test]
fn test_batch_insert() {
    let mut graph = CodeGraph::open_memory().unwrap();
    let symbols: Vec<SymbolInsert> = (0..100)
        .map(|i| make_symbol(&format!("batch_{i}"), "function", "src/batch.rs", i as i64))
        .collect();

    let count = graph.batch_insert_symbols(symbols).unwrap();
    assert_eq!(count, 100);
    assert_eq!(graph.symbol_count().unwrap(), 100);
}

#[test]
fn test_symbol_with_embedding() {
    let mut graph = CodeGraph::open_memory().unwrap();
    let embedding = vec![0.1f32, 0.2, 0.3, 0.4, 0.5];
    let bytes: Vec<u8> = unsafe {
        std::slice::from_raw_parts(
            embedding.as_ptr() as *const u8,
            embedding.len() * std::mem::size_of::<f32>(),
        ).to_vec()
    };

    let sym = SymbolInsert {
        name: "embedded_func".to_string(),
        kind: "function".to_string(),
        file_path: "src/embed.rs".to_string(),
        line: 1,
        col: 0,
        language: Some("rust".to_string()),
        doc: Some("Has embedding".to_string()),
        embedding: Some(bytes.clone()),
    };
    let id = graph.insert_symbol(sym).unwrap();

    let found = graph.get_symbol(id).unwrap().unwrap();
    assert_eq!(found.embedding, Some(bytes));
}

#[test]
fn test_persistent_database() {
    let tmp = tempfile::tempdir().unwrap();
    let db_path = tmp.path().join("test.db");

    {
        let mut graph = CodeGraph::open(&db_path).unwrap();
        graph.insert_symbol(make_symbol("persisted", "function", "src/main.rs", 1)).unwrap();
        assert_eq!(graph.symbol_count().unwrap(), 1);
    }

    {
        let graph = CodeGraph::open(&db_path).unwrap();
        assert_eq!(graph.symbol_count().unwrap(), 1);
        let sym = graph.get_symbol(1).unwrap().unwrap();
        assert_eq!(sym.name, "persisted");
    }
}

#[test]
fn test_schema_version_mismatch() {
    let tmp = tempfile::tempdir().unwrap();
    let db_path = tmp.path().join("version_test.db");

    {
        let conn = rusqlite::Connection::open(&db_path).unwrap();
        conn.execute_batch("PRAGMA user_version = 99;").unwrap();
    }

    let result = CodeGraph::open(&db_path);
    assert!(result.is_err());
}
