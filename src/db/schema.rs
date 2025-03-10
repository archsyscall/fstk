use anyhow::Result;
use rusqlite::Connection;

pub const SCHEMA_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS stack_items (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    original_name TEXT NOT NULL,
    original_path TEXT NOT NULL,
    stored_hash TEXT NOT NULL UNIQUE,
    type TEXT NOT NULL,
    pushed_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS tags (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE
);

CREATE TABLE IF NOT EXISTS item_tags (
    item_id INTEGER,
    tag_id INTEGER,
    PRIMARY KEY(item_id, tag_id),
    FOREIGN KEY(item_id) REFERENCES stack_items(id) ON DELETE CASCADE,
    FOREIGN KEY(tag_id) REFERENCES tags(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_stack_items_pushed_at ON stack_items(pushed_at);
CREATE INDEX IF NOT EXISTS idx_stack_items_stored_hash ON stack_items(stored_hash);
CREATE INDEX IF NOT EXISTS idx_tags_name ON tags(name);
"#;

pub fn initialize_schema(conn: &Connection) -> Result<()> {
    conn.execute_batch(SCHEMA_SQL)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    #[test]
    fn test_schema_initialization() -> Result<()> {
        // Create in-memory database
        let conn = Connection::open_in_memory()?;

        // Initialize schema
        initialize_schema(&conn)?;

        // Verify tables exist
        let tables = get_tables(&conn)?;
        assert!(tables.contains(&"stack_items".to_string()));
        assert!(tables.contains(&"tags".to_string()));
        assert!(tables.contains(&"item_tags".to_string()));

        // Verify indices exist
        let indices = get_indices(&conn)?;
        assert!(indices.contains(&"idx_stack_items_pushed_at".to_string()));
        assert!(indices.contains(&"idx_stack_items_stored_hash".to_string()));
        assert!(indices.contains(&"idx_tags_name".to_string()));

        // Test foreign key constraints are enabled
        let foreign_keys_enabled: bool =
            conn.query_row("PRAGMA foreign_keys", [], |row| row.get(0))?;

        assert!(foreign_keys_enabled, "Foreign keys should be enabled");

        Ok(())
    }

    fn get_tables(conn: &Connection) -> Result<Vec<String>> {
        let mut stmt = conn.prepare(
            "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%'",
        )?;

        let table_iter = stmt.query_map([], |row| row.get::<_, String>(0))?;

        let mut tables = Vec::new();
        for table in table_iter {
            tables.push(table?);
        }

        Ok(tables)
    }

    fn get_indices(conn: &Connection) -> Result<Vec<String>> {
        let mut stmt = conn.prepare(
            "SELECT name FROM sqlite_master WHERE type='index' AND name NOT LIKE 'sqlite_%'",
        )?;

        let index_iter = stmt.query_map([], |row| row.get::<_, String>(0))?;

        let mut indices = Vec::new();
        for index in index_iter {
            indices.push(index?);
        }

        Ok(indices)
    }
}
