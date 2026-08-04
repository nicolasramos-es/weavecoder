//! Core storage engine for the Code Knowledge Graph.
//!
//! Manages an embedded SQLite database with FTS5 indexing for fast text search.

use anyhow::{Context, Result};
use rusqlite::{Connection, params};
use std::path::{Path, PathBuf};

use crate::{SCHEMA_VERSION, symbols::*, relations::*, fts::*};

/// Errors that can occur during graph operations.
#[derive(Debug)]
pub enum CodeGraphError {
    Database(String),
    FtsQuery(String),
    NotFound(String),
}

impl std::fmt::Display for CodeGraphError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CodeGraphError::Database(msg) => write!(f, "Database error: {msg}"),
            CodeGraphError::FtsQuery(msg) => write!(f, "FTS query error: {msg}"),
            CodeGraphError::NotFound(msg) => write!(f, "Not found: {msg}"),
        }
    }
}

impl std::error::Error for CodeGraphError {}

impl From<rusqlite::Error> for CodeGraphError {
    fn from(err: rusqlite::Error) -> Self {
        CodeGraphError::Database(err.to_string())
    }
}

/// The main CodeGraph handle. Opens/creates a SQLite database file
/// and provides methods to insert, query, and search symbols and relations.
pub struct CodeGraph {
    conn: Option<Connection>,
    #[allow(dead_code)]
    db_path: Option<PathBuf>,
}

impl CodeGraph {
    /// Open (or create) a persistent SQLite database at the given path.
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        let conn = Connection::open(&path)
            .context("Failed to open SQLite database")?;

        conn.execute_batch("PRAGMA journal_mode=WAL;")?;
        conn.execute_batch("PRAGMA foreign_keys=ON;")?;
        conn.busy_timeout(std::time::Duration::from_secs(5))?;

        let mut graph = Self {
            conn: Some(conn),
            db_path: Some(path),
        };
        graph.ensure_schema()?;
        Ok(graph)
    }

    /// Open an in-memory database (useful for tests).
    pub fn open_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()
            .context("Failed to open in-memory SQLite database")?;

        conn.execute_batch("PRAGMA journal_mode=WAL;")?;
        conn.execute_batch("PRAGMA foreign_keys=ON;")?;
        conn.busy_timeout(std::time::Duration::from_secs(5))?;

        let mut graph = Self {
            conn: Some(conn),
            db_path: None,
        };
        graph.ensure_schema()?;
        Ok(graph)
    }

    fn ensure_schema(&mut self) -> Result<()> {
        let conn = self.conn.as_mut().unwrap();
        let current_version: u32 = conn
            .query_row("PRAGMA user_version;", [], |row| row.get(0))
            .unwrap_or(0);

        if current_version == 0 {
            Self::create_schema(conn)?;
            conn.execute_batch(&format!(
                "PRAGMA user_version = {};",
                SCHEMA_VERSION
            ))?;
        } else if current_version != SCHEMA_VERSION {
            return Err(anyhow::anyhow!(
                "Schema version mismatch: database has v{}, expected v{}",
                current_version,
                SCHEMA_VERSION
            ));
        }

        Ok(())
    }

    fn create_schema(conn: &Connection) -> Result<()> {
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS symbols (
                id            INTEGER PRIMARY KEY AUTOINCREMENT,
                name          TEXT NOT NULL,
                kind          TEXT NOT NULL,
                file_path     TEXT NOT NULL,
                line          INTEGER NOT NULL,
                col           INTEGER NOT NULL DEFAULT 0,
                language      TEXT,
                doc           TEXT,
                embedding     BLOB
            );

            CREATE TABLE IF NOT EXISTS relations (
                source_symbol_id INTEGER NOT NULL REFERENCES symbols(id) ON DELETE CASCADE,
                target_symbol_id INTEGER NOT NULL REFERENCES symbols(id) ON DELETE CASCADE,
                kind             TEXT NOT NULL,
                metadata         TEXT,
                PRIMARY KEY (source_symbol_id, target_symbol_id, kind)
            );

            CREATE VIRTUAL TABLE IF NOT EXISTS symbols_fts USING fts5(
                name, doc,
                content='symbols',
                content_rowid='id'
            );

            CREATE TRIGGER IF NOT EXISTS symbols_ai AFTER INSERT ON symbols BEGIN
                INSERT INTO symbols_fts(rowid, name, doc)
                    VALUES (new.id, new.name, new.doc);
            END;

            CREATE TRIGGER IF NOT EXISTS symbols_ad AFTER DELETE ON symbols BEGIN
                INSERT INTO symbols_fts(symbols_fts, rowid, name, doc)
                    VALUES ('delete', old.id, old.name, old.doc);
            END;

            CREATE TRIGGER IF NOT EXISTS symbols_au AFTER UPDATE ON symbols BEGIN
                INSERT INTO symbols_fts(symbols_fts, rowid, name, doc)
                    VALUES ('delete', old.id, old.name, old.doc);
                INSERT INTO symbols_fts(rowid, name, doc)
                    VALUES (new.id, new.name, new.doc);
            END;
            ",
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_symbols_file_path ON symbols(file_path);",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_symbols_kind ON symbols(kind);",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_relations_source ON relations(source_symbol_id);",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_relations_target ON relations(target_symbol_id);",
            [],
        )?;

        Ok(())
    }

    pub fn insert_symbol(&mut self, symbol: SymbolInsert) -> Result<i64> {
        let conn = self.conn.as_mut().unwrap();
        let _id = conn.execute(
            "INSERT INTO symbols (name, kind, file_path, line, col, language, doc, embedding)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                symbol.name,
                symbol.kind,
                symbol.file_path,
                symbol.line,
                symbol.col,
                symbol.language,
                symbol.doc,
                symbol.embedding,
            ],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn upsert_symbol(&mut self, symbol: SymbolInsert) -> Result<i64> {
        let conn = self.conn.as_mut().unwrap();
        let existing: Option<i64> = conn.query_row(
            "SELECT id FROM symbols WHERE name = ?1 AND file_path = ?2",
            params![&symbol.name, &symbol.file_path],
            |row| row.get(0),
        ).ok();

        if let Some(id) = existing {
            conn.execute(
                "UPDATE symbols SET kind = ?1, line = ?2, col = ?3, language = ?4, doc = ?5, embedding = ?6
                 WHERE id = ?7",
                params![
                    symbol.kind,
                    symbol.line,
                    symbol.col,
                    symbol.language,
                    symbol.doc,
                    symbol.embedding,
                    id,
                ],
            )?;
            Ok(id)
        } else {
            self.insert_symbol(symbol)
        }
    }

    pub fn get_symbol(&self, id: i64) -> Result<Option<Symbol>> {
        let conn = self.conn.as_ref().unwrap();
        match conn.query_row(
            "SELECT id, name, kind, file_path, line, col, language, doc, embedding
             FROM symbols WHERE id = ?1",
            params![id],
            |row| {
                Ok(Symbol {
                    id,
                    name: row.get(1)?,
                    kind: row.get(2)?,
                    file_path: row.get(3)?,
                    line: row.get(4)?,
                    col: row.get(5)?,
                    language: row.get(6)?,
                    doc: row.get(7)?,
                    embedding: row.get::<_, Option<Vec<u8>>>(8).ok().flatten(),
                })
            },
        ) {
            Ok(sym) => Ok(Some(sym)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    pub fn list_symbols(&self, query: SymbolQuery) -> Result<Vec<Symbol>> {
        let conn = self.conn.as_ref().unwrap();
        let mut sql = String::from("SELECT id, name, kind, file_path, line, col, language, doc, embedding FROM symbols");
        let mut conditions = Vec::new();
        let mut sql_params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

        if let Some(kind) = &query.kind {
            conditions.push("kind = ?".to_string());
            sql_params.push(Box::new(kind.clone()));
        }
        if let Some(file_path) = &query.file_path {
            conditions.push("file_path = ?".to_string());
            sql_params.push(Box::new(file_path.clone()));
        }
        if let Some(language) = &query.language {
            conditions.push("language = ?".to_string());
            sql_params.push(Box::new(language.clone()));
        }

        if !conditions.is_empty() {
            sql.push_str(" WHERE ");
            sql.push_str(&conditions.join(" AND "));
        }

        // Append LIMIT after WHERE clause (not inside it)
        if let Some(limit) = query.limit {
            sql.push_str(" LIMIT ");
            sql.push_str(&limit.to_string());
        }

        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt.query_map(
            rusqlite::params_from_iter(sql_params.iter().map(|p| p.as_ref())),
            |row| {
                Ok(Symbol {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    kind: row.get(2)?,
                    file_path: row.get(3)?,
                    line: row.get(4)?,
                    col: row.get(5)?,
                    language: row.get(6)?,
                    doc: row.get(7)?,
                    embedding: row.get::<_, Option<Vec<u8>>>(8).ok().flatten(),
                })
            },
        )?;

        Ok(rows.collect::<Result<Vec<_>, _>>()?)
    }

    pub fn insert_relation(&mut self, rel: RelationInsert) -> Result<()> {
        let conn = self.conn.as_mut().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO relations (source_symbol_id, target_symbol_id, kind, metadata)
             VALUES (?1, ?2, ?3, ?4)",
            params![
                rel.source_symbol_id,
                rel.target_symbol_id,
                rel.kind,
                rel.metadata,
            ],
        )?;
        Ok(())
    }

    pub fn get_relations(&self, symbol_id: i64) -> Result<Vec<Relation>> {
        let conn = self.conn.as_ref().unwrap();
        let mut stmt = conn.prepare(
            "SELECT r.source_symbol_id, r.target_symbol_id, r.kind, r.metadata,
                    s1.name AS source_name, s2.name AS target_name
             FROM relations r
             JOIN symbols s1 ON r.source_symbol_id = s1.id
             JOIN symbols s2 ON r.target_symbol_id = s2.id
             WHERE r.source_symbol_id = ?1",
        )?;
        let rows = stmt.query_map(params![symbol_id], |row| {
            Ok(Relation {
                source_symbol_id: row.get(0)?,
                target_symbol_id: row.get(1)?,
                kind: row.get(2)?,
                metadata: row.get(3)?,
                source_name: row.get(4)?,
                target_name: row.get(5)?,
            })
        })?;

        Ok(rows.collect::<Result<Vec<_>, _>>()?)
    }

    pub fn search_fts(&self, query: FtsQuery) -> Result<Vec<FtsSearchResult>> {
        let conn = self.conn.as_ref().unwrap();
        let fts_query = query.build_fts_query();

        let sql = format!(
            "SELECT s.id, s.name, s.kind, s.file_path, s.line, s.col, s.language, s.doc,
                    s.embedding, rank
             FROM symbols_fts f
             JOIN symbols s ON s.id = f.rowid
             WHERE symbols_fts MATCH ?1
             ORDER BY rank",
        );

        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt.query_map(params![fts_query], |row| {
            Ok(FtsSearchResult {
                id: row.get(0)?,
                name: row.get(1)?,
                kind: row.get(2)?,
                file_path: row.get(3)?,
                line: row.get(4)?,
                col: row.get(5)?,
                language: row.get(6)?,
                doc: row.get(7)?,
                embedding: row.get::<_, Option<Vec<u8>>>(8).ok().flatten(),
                rank: row.get::<_, f64>(9)?,
            })
        })?;

        let mut results: Vec<FtsSearchResult> = rows.collect::<Result<Vec<_>, _>>()?;
        results.sort_by(|a, b| a.rank.partial_cmp(&b.rank).unwrap_or(std::cmp::Ordering::Equal));

        if let Some(limit) = query.limit {
            results.truncate(limit);
        }

        Ok(results)
    }

    pub fn schema_version(&self) -> u32 {
        let conn = self.conn.as_ref().unwrap();
        conn.query_row("PRAGMA user_version;", [], |row| row.get(0)).unwrap_or(0)
    }

    pub fn symbol_count(&self) -> Result<usize> {
        let conn = self.conn.as_ref().unwrap();
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM symbols", [], |row| row.get(0)
        )?;
        Ok(count as usize)
    }

    pub fn relation_count(&self) -> Result<usize> {
        let conn = self.conn.as_ref().unwrap();
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM relations", [], |row| row.get(0)
        )?;
        Ok(count as usize)
    }

    pub fn batch_insert_symbols(&mut self, symbols: Vec<SymbolInsert>) -> Result<usize> {
        let conn = self.conn.as_mut().unwrap();
        let tx = conn.transaction()?;
        let mut count = 0usize;

        for symbol in symbols {
            tx.execute(
                "INSERT INTO symbols (name, kind, file_path, line, col, language, doc, embedding)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![
                    symbol.name,
                    symbol.kind,
                    symbol.file_path,
                    symbol.line,
                    symbol.col,
                    symbol.language,
                    symbol.doc,
                    symbol.embedding,
                ],
            )?;
            count += 1;
        }

        tx.commit()?;
        Ok(count)
    }

    pub fn close(&mut self) -> Result<()> {
        if let Some(conn) = self.conn.take() {
            conn.close().map_err(|(conn, err)| {
                self.conn = Some(conn);
                anyhow::anyhow!("Failed to close database: {}", err)
            })?;
        }
        Ok(())
    }
}

impl Drop for CodeGraph {
    fn drop(&mut self) {
        if let Some(conn) = self.conn.take() {
            let _ = conn.close();
        }
    }
}
