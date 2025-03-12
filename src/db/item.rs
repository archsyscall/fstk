use anyhow::{anyhow, Result};
use chrono::{DateTime, Local};
use rusqlite::{params, Connection, Row};

use crate::db::tag::{find_or_create_tag, TagManager};

#[derive(Debug, Clone)]
pub struct StackItem {
    pub id: i64,
    pub original_name: String,
    pub original_path: String,
    pub stored_hash: String,
    pub item_type: String, // "file" or "directory"
    pub pushed_at: DateTime<Local>,
    pub tags: Vec<String>,
}

impl StackItem {
    pub fn from_row(row: &Row) -> Result<Self> {
        let id = row.get(0)?;
        let original_name = row.get(1)?;
        let original_path = row.get(2)?;
        let stored_hash = row.get(3)?;
        let item_type = row.get(4)?;

        let pushed_at_str: String = row.get(5)?;
        // The date in SQLite is stored as UTC without timezone info, so we need to parse it as UTC
        // and then convert to local time
        let naive_dt = chrono::NaiveDateTime::parse_from_str(&pushed_at_str, "%Y-%m-%d %H:%M:%S")
            .map_err(|e| anyhow!("Error parsing date: {}", e))?;
        // First interpret as UTC, then convert to local time
        let pushed_at = chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(naive_dt, chrono::Utc)
            .with_timezone(&Local);

        Ok(StackItem {
            id,
            original_name,
            original_path,
            stored_hash,
            item_type,
            pushed_at,
            tags: Vec::new(), // We'll populate tags later
        })
    }
}

pub struct ItemManager;

impl ItemManager {
    pub fn insert(
        conn: &mut Connection,
        original_name: &str,
        original_path: &str,
        stored_hash: &str,
        item_type: &str,
        tags: &[String],
    ) -> Result<i64> {
        // Start a transaction for atomicity
        let tx = conn.transaction()?;

        // Insert the stack item
        tx.execute(
            "INSERT INTO stack_items (original_name, original_path, stored_hash, type) VALUES (?, ?, ?, ?)",
            params![original_name, original_path, stored_hash, item_type],
        )?;

        let item_id = tx.last_insert_rowid();

        // Process tags if provided
        if !tags.is_empty() {
            for tag in tags {
                let tag = tag.trim();
                if tag.is_empty() {
                    continue;
                }

                // Find or create tag
                let tag_id = find_or_create_tag(&tx, tag)?;

                // Associate tag with stack item
                tx.execute(
                    "INSERT OR IGNORE INTO item_tags (item_id, tag_id) VALUES (?, ?)",
                    params![item_id, tag_id],
                )?;
            }
        }

        // Commit the transaction
        tx.commit()?;

        Ok(item_id)
    }

    pub fn get_by_id(conn: &Connection, id: i64) -> Result<Option<StackItem>> {
        let mut stmt = conn.prepare(
            "SELECT id, original_name, original_path, stored_hash, type, pushed_at 
             FROM stack_items WHERE id = ?",
        )?;

        let mut rows = stmt.query(params![id])?;

        if let Some(row) = rows.next()? {
            let mut item = StackItem::from_row(row)?;
            item.tags = TagManager::get_for_item(conn, id)?;
            Ok(Some(item))
        } else {
            Ok(None)
        }
    }

    pub fn get_latest(conn: &Connection) -> Result<Option<StackItem>> {
        let mut stmt = conn.prepare(
            "SELECT id, original_name, original_path, stored_hash, type, pushed_at 
             FROM stack_items ORDER BY pushed_at DESC LIMIT 1",
        )?;

        let mut rows = stmt.query([])?;

        if let Some(row) = rows.next()? {
            let mut item = StackItem::from_row(row)?;
            item.tags = TagManager::get_for_item(conn, item.id)?;
            Ok(Some(item))
        } else {
            Ok(None)
        }
    }

    pub fn get_latest_by_tags(conn: &Connection, tags: &[String]) -> Result<Option<StackItem>> {
        if tags.is_empty() {
            return Self::get_latest(conn);
        }

        // Build a query that finds items with ALL the specified tags
        let placeholders = std::iter::repeat("?")
            .take(tags.len())
            .collect::<Vec<_>>()
            .join(",");
        let sql = format!(
            "SELECT si.id, si.original_name, si.original_path, si.stored_hash, si.type, si.pushed_at
             FROM stack_items si
             WHERE si.id IN (
                 SELECT item_id 
                 FROM item_tags it
                 JOIN tags t ON it.tag_id = t.id
                 WHERE t.name IN ({})
                 GROUP BY item_id
                 HAVING COUNT(DISTINCT t.name) = ?
             )
             ORDER BY si.pushed_at DESC
             LIMIT 1",
            placeholders
        );

        let mut stmt = conn.prepare(&sql)?;

        // Prepare params: all tag names followed by the count of tags
        let mut params: Vec<rusqlite::types::Value> = tags
            .iter()
            .map(|t| rusqlite::types::Value::Text(t.clone()))
            .collect();
        params.push(rusqlite::types::Value::Integer(tags.len() as i64));

        let mut rows = stmt.query(rusqlite::params_from_iter(params))?;

        if let Some(row) = rows.next()? {
            let mut item = StackItem::from_row(row)?;
            item.tags = TagManager::get_for_item(conn, item.id)?;
            Ok(Some(item))
        } else {
            Ok(None)
        }
    }

    pub fn list(conn: &Connection, tags: &[String]) -> Result<Vec<StackItem>> {
        let mut items = Vec::new();

        let sql = if tags.is_empty() {
            // No tag filtering, get all items without sorting
            "SELECT id, original_name, original_path, stored_hash, type, pushed_at 
             FROM stack_items"
                .to_string()
        } else {
            // Filter by tags
            let placeholders = std::iter::repeat("?")
                .take(tags.len())
                .collect::<Vec<_>>()
                .join(",");
            format!(
                "SELECT si.id, si.original_name, si.original_path, si.stored_hash, si.type, si.pushed_at
                 FROM stack_items si
                 WHERE si.id IN (
                     SELECT item_id 
                     FROM item_tags it
                     JOIN tags t ON it.tag_id = t.id
                     WHERE t.name IN ({})
                     GROUP BY item_id
                     HAVING COUNT(DISTINCT t.name) = ?
                 )",
                placeholders
            )
        };

        let mut stmt = conn.prepare(&sql)?;

        let rows = if tags.is_empty() {
            stmt.query([])?
        } else {
            // Prepare params: all tag names followed by the count of tags
            let mut params: Vec<rusqlite::types::Value> = tags
                .iter()
                .map(|t| rusqlite::types::Value::Text(t.clone()))
                .collect();
            params.push(rusqlite::types::Value::Integer(tags.len() as i64));

            stmt.query(rusqlite::params_from_iter(params))?
        };

        let mut rows = rows;

        while let Some(row) = rows.next()? {
            let mut item = StackItem::from_row(row)?;
            item.tags = TagManager::get_for_item(conn, item.id)?;
            items.push(item);
        }

        Ok(items)
    }

    /// Get database ID by display number
    pub fn get_id_by_display_number(
        conn: &Connection,
        display_number: usize,
        tags: &[String],
    ) -> Result<Option<i64>> {
        // Get all items, sort them and find the ID by display number
        let mut items = Self::list(conn, tags)?;

        // No items found
        if items.is_empty() {
            return Ok(None);
        }

        // Sort by pushed_at descending (newest first)
        items.sort_by(|a, b| b.pushed_at.cmp(&a.pushed_at));

        // Find item by display number (display numbers start at 1)
        if display_number <= items.len() && display_number > 0 {
            let item = &items[display_number - 1];
            Ok(Some(item.id))
        } else {
            Ok(None)
        }
    }

    /// Delete an item from the stack and clean up any orphaned tags
    pub fn delete(conn: &mut Connection, id: i64) -> Result<bool> {
        // First, identify tags associated with this item for cleanup later
        let tag_ids = Self::get_tag_ids_for_item(conn, id)?;

        // Start a transaction
        let tx = conn.transaction()?;

        // Delete the item
        let result = tx.execute("DELETE FROM stack_items WHERE id = ?", params![id])?;

        // The foreign key constraints will automatically delete from item_tags

        // Clean up any orphaned tags
        if result > 0 && !tag_ids.is_empty() {
            TagManager::cleanup_orphaned_tags(&tx, &tag_ids)?;
        }

        // Commit the transaction
        tx.commit()?;

        Ok(result > 0)
    }

    /// Helper function to get tag IDs for an item
    fn get_tag_ids_for_item(conn: &Connection, item_id: i64) -> Result<Vec<i64>> {
        let mut stmt = conn.prepare("SELECT tag_id FROM item_tags WHERE item_id = ?")?;

        let rows = stmt.query_map(params![item_id], |row| row.get::<_, i64>(0))?;

        let mut tag_ids = Vec::new();
        for tag_id_result in rows {
            tag_ids.push(tag_id_result?);
        }

        Ok(tag_ids)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::schema;
    use std::thread::sleep;
    use std::time::Duration;

    fn setup_test_db() -> Result<Connection> {
        let conn = Connection::open_in_memory()?;
        // Enable foreign key constraints
        conn.execute("PRAGMA foreign_keys = ON", [])?;
        // Initialize schema
        schema::initialize_schema(&conn)?;
        Ok(conn)
    }

    #[test]
    fn test_stack_item_from_row() -> Result<()> {
        let conn = setup_test_db()?;

        // Insert a test item
        conn.execute(
            "INSERT INTO stack_items (original_name, original_path, stored_hash, type) VALUES (?, ?, ?, ?)",
            params!["test.txt", "/path/to/test.txt", "abcdef1234567890", "file"],
        )?;

        // Retrieve the row directly to a StackItem
        let mut stmt = conn.prepare(
            "SELECT id, original_name, original_path, stored_hash, type, pushed_at 
             FROM stack_items LIMIT 1",
        )?;

        // Use map_row to avoid lifetime issues
        let item = stmt.query_row([], |row| {
            // Convert anyhow::Result to rusqlite::Result by using a match
            match StackItem::from_row(row) {
                Ok(item) => Ok(item),
                Err(_e) => Err(rusqlite::Error::InvalidQuery), // Convert to a rusqlite error
            }
        })?;

        // Verify fields
        assert_eq!(item.original_name, "test.txt");
        assert_eq!(item.original_path, "/path/to/test.txt");
        assert_eq!(item.stored_hash, "abcdef1234567890");
        assert_eq!(item.item_type, "file");
        assert!(item.tags.is_empty());

        Ok(())
    }

    #[test]
    fn test_insert_and_get_item() -> Result<()> {
        let mut conn = setup_test_db()?;

        // Insert test item
        let item_id = ItemManager::insert(
            &mut conn,
            "test.txt",
            "/path/to/test.txt",
            "abcdef1234567890",
            "file",
            &["test-tag".to_string()],
        )?;

        // Retrieve item
        let item = ItemManager::get_by_id(&conn, item_id)?.expect("Item should exist");

        assert_eq!(item.original_name, "test.txt");
        assert_eq!(item.original_path, "/path/to/test.txt");
        assert_eq!(item.stored_hash, "abcdef1234567890");
        assert_eq!(item.item_type, "file");
        assert_eq!(item.tags.len(), 1);
        assert_eq!(item.tags[0], "test-tag");

        Ok(())
    }

    #[test]
    fn test_get_latest() -> Result<()> {
        let mut conn = setup_test_db()?;

        // Insert multiple items
        ItemManager::insert(
            &mut conn,
            "older.txt",
            "/path/to/older.txt",
            "hash1",
            "file",
            &[],
        )?;

        // Simulate delay between inserts
        sleep(Duration::from_millis(10));

        ItemManager::insert(
            &mut conn,
            "newer.txt",
            "/path/to/newer.txt",
            "hash2",
            "file",
            &[],
        )?;

        // Get latest item
        let latest = ItemManager::get_latest(&conn)?.expect("Item should exist");

        assert_eq!(latest.original_name, "newer.txt");
        assert_eq!(latest.stored_hash, "hash2");

        Ok(())
    }

    #[test]
    fn test_get_latest_by_tags() -> Result<()> {
        let mut conn = setup_test_db()?;

        // Insert items with different tags
        ItemManager::insert(
            &mut conn,
            "file1.txt",
            "/path/to/file1.txt",
            "hash1",
            "file",
            &["tag1".to_string(), "tag2".to_string()],
        )?;

        ItemManager::insert(
            &mut conn,
            "file2.txt",
            "/path/to/file2.txt",
            "hash2",
            "file",
            &["tag2".to_string(), "tag3".to_string()],
        )?;

        // Get latest with specific tag
        let item = ItemManager::get_latest_by_tags(&conn, &["tag1".to_string()])?
            .expect("Item should exist");

        assert_eq!(item.original_name, "file1.txt");

        // Get latest with multiple tags
        let item =
            ItemManager::get_latest_by_tags(&conn, &["tag2".to_string(), "tag3".to_string()])?
                .expect("Item should exist");

        assert_eq!(item.original_name, "file2.txt");

        Ok(())
    }

    #[test]
    fn test_list_and_filter() -> Result<()> {
        let mut conn = setup_test_db()?;

        // Insert items with different tags
        ItemManager::insert(
            &mut conn,
            "file1.txt",
            "/path/to/file1.txt",
            "hash1",
            "file",
            &["tag1".to_string(), "common".to_string()],
        )?;

        ItemManager::insert(
            &mut conn,
            "file2.txt",
            "/path/to/file2.txt",
            "hash2",
            "file",
            &["tag2".to_string(), "common".to_string()],
        )?;

        // List all items
        let items = ItemManager::list(&conn, &[])?;
        assert_eq!(items.len(), 2);

        // Filter by tag
        let items = ItemManager::list(&conn, &["tag1".to_string()])?;
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].original_name, "file1.txt");

        // Filter by common tag
        let items = ItemManager::list(&conn, &["common".to_string()])?;
        assert_eq!(items.len(), 2);

        Ok(())
    }

    #[test]
    fn test_delete_item() -> Result<()> {
        let mut conn = setup_test_db()?;

        // Insert test item
        let item_id = ItemManager::insert(
            &mut conn,
            "delete_me.txt",
            "/path/to/delete_me.txt",
            "hash_delete",
            "file",
            &["temp-tag".to_string()],
        )?;

        // Verify item exists
        assert!(ItemManager::get_by_id(&conn, item_id)?.is_some());

        // Delete item
        let deleted = ItemManager::delete(&mut conn, item_id)?;
        assert!(deleted);

        // Verify item no longer exists
        assert!(ItemManager::get_by_id(&conn, item_id)?.is_none());

        Ok(())
    }

    #[test]
    fn test_get_id_by_display_number() -> Result<()> {
        let conn = setup_test_db()?;

        // Insert items with unique timestamps in the database directly to avoid
        // any timing issues with the test
        let mut stmt = conn.prepare(
            "INSERT INTO stack_items (original_name, original_path, stored_hash, type, pushed_at) 
             VALUES (?, ?, ?, ?, datetime('now', '-1 minute'))",
        )?;

        stmt.execute(params![
            "file1.txt",
            "/path/to/file1.txt",
            "hash_display_1",
            "file"
        ])?;

        let item1_id = conn.last_insert_rowid();

        let mut stmt = conn.prepare(
            "INSERT INTO stack_items (original_name, original_path, stored_hash, type, pushed_at) 
             VALUES (?, ?, ?, ?, datetime('now'))",
        )?;

        stmt.execute(params![
            "file2.txt",
            "/path/to/file2.txt",
            "hash_display_2",
            "file"
        ])?;

        let item2_id = conn.last_insert_rowid();

        // Add a test tag to each item
        let tag_id = find_or_create_tag(&conn, "test")?;

        conn.execute(
            "INSERT INTO item_tags (item_id, tag_id) VALUES (?, ?)",
            params![item1_id, tag_id],
        )?;

        conn.execute(
            "INSERT INTO item_tags (item_id, tag_id) VALUES (?, ?)",
            params![item2_id, tag_id],
        )?;

        // Item 2 should be display number 1 (newest first)
        let id =
            ItemManager::get_id_by_display_number(&conn, 1, &[])?.expect("Should find an item");

        assert_eq!(id, item2_id);

        // Item 1 should be display number 2 (oldest second)
        let id =
            ItemManager::get_id_by_display_number(&conn, 2, &[])?.expect("Should find an item");

        assert_eq!(id, item1_id);

        // Test invalid display number
        let result = ItemManager::get_id_by_display_number(&conn, 999, &[])?;
        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_get_tag_ids_for_item() -> Result<()> {
        let mut conn = setup_test_db()?;

        // Create item with tags
        let item_id = ItemManager::insert(
            &mut conn,
            "test.txt",
            "/path/to/test.txt",
            "hash_test",
            "file",
            &["tag1".to_string(), "tag2".to_string()],
        )?;

        // Get tag IDs
        let tag_ids = ItemManager::get_tag_ids_for_item(&conn, item_id)?;

        // Should have 2 tag IDs
        assert_eq!(tag_ids.len(), 2);

        Ok(())
    }
}
