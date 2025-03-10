use anyhow::Result;
use rusqlite::{params, Connection};

pub fn find_or_create_tag(conn: &Connection, tag_name: &str) -> Result<i64> {
    let mut stmt = conn.prepare("SELECT id FROM tags WHERE name = ?")?;
    let mut rows = stmt.query(params![tag_name])?;

    if let Some(row) = rows.next()? {
        return Ok(row.get(0)?);
    }

    conn.execute("INSERT INTO tags (name) VALUES (?)", params![tag_name])?;
    Ok(conn.last_insert_rowid())
}

pub struct TagManager;

impl TagManager {
    pub fn get_for_item(conn: &Connection, item_id: i64) -> Result<Vec<String>> {
        let mut stmt = conn.prepare(
            "SELECT t.name 
             FROM tags t
             JOIN item_tags it ON t.id = it.tag_id
             WHERE it.item_id = ?
             ORDER BY t.name",
        )?;

        let rows = stmt.query_map(params![item_id], |row| row.get::<_, String>(0))?;

        let mut tags = Vec::new();
        for tag_result in rows {
            tags.push(tag_result?);
        }

        Ok(tags)
    }

    pub fn add_to_item(conn: &mut Connection, item_id: i64, tags: &[String]) -> Result<usize> {
        let tx = conn.transaction()?;

        let mut total_added = 0;

        for tag in tags {
            let tag = tag.trim();
            if tag.is_empty() {
                continue;
            }

            let tag_id = find_or_create_tag(&tx, tag)?;

            let affected = tx.execute(
                "INSERT OR IGNORE INTO item_tags (item_id, tag_id) VALUES (?, ?)",
                params![item_id, tag_id],
            )?;

            total_added += affected;
        }

        tx.commit()?;

        Ok(total_added)
    }

    /// Remove tags from an item and clean up orphaned tags
    pub fn remove_from_item(conn: &mut Connection, item_id: i64, tags: &[String]) -> Result<usize> {
        let tx = conn.transaction()?;
        let mut total_removed = 0;
        let mut removed_tag_ids = Vec::new();

        for tag in tags {
            let tag = tag.trim();
            if tag.is_empty() {
                continue;
            }

            let mut stmt = tx.prepare("SELECT id FROM tags WHERE name = ?")?;
            let mut rows = stmt.query(params![tag])?;

            if let Some(row) = rows.next()? {
                let tag_id: i64 = row.get(0)?;
                let affected = tx.execute(
                    "DELETE FROM item_tags WHERE item_id = ? AND tag_id = ?",
                    params![item_id, tag_id],
                )?;

                if affected > 0 {
                    total_removed += affected;
                    removed_tag_ids.push(tag_id);
                }
            }
        }

        Self::cleanup_orphaned_tags(&tx, &removed_tag_ids)?;
        tx.commit()?;
        Ok(total_removed)
    }

    /// Clean up orphaned tags - tags that no longer have any items associated with them
    pub fn cleanup_orphaned_tags(conn: &Connection, tag_ids: &[i64]) -> Result<usize> {
        let mut cleaned_up = 0;

        if tag_ids.is_empty() {
            return Ok(0);
        }

        for &tag_id in tag_ids {
            let mut stmt = conn.prepare("SELECT COUNT(*) FROM item_tags WHERE tag_id = ?")?;
            let count: i64 = stmt.query_row(params![tag_id], |row| row.get(0))?;

            if count == 0 {
                let affected = conn.execute("DELETE FROM tags WHERE id = ?", params![tag_id])?;
                cleaned_up += affected;
            }
        }

        Ok(cleaned_up)
    }

    // This function is a utility for maintenance operations
    // and has been temporarily disabled as it's not needed for regular operation
    /*
    /// Clean up all orphaned tags in the database
    fn cleanup_all_orphaned_tags(conn: &Connection) -> Result<usize> {
        // Delete tags that don't have any associated items
        let result = conn.execute(
            "DELETE FROM tags WHERE id NOT IN (SELECT DISTINCT tag_id FROM item_tags WHERE tag_id IS NOT NULL)",
            [],
        )?;

        Ok(result)
    }
    */

    /// Delete all unused tags
    pub fn delete_unused_tags(conn: &Connection) -> Result<usize> {
        let result = conn.execute(
            "DELETE FROM tags WHERE (SELECT COUNT(*) FROM item_tags WHERE item_tags.tag_id = tags.id) = 0",
            [],
        )?;

        Ok(result)
    }

    pub fn list_all(conn: &Connection) -> Result<Vec<(i64, String, i64)>> {
        let mut stmt = conn.prepare(
            "SELECT t.id, t.name, COUNT(it.item_id) as usage_count
             FROM tags t
             LEFT JOIN item_tags it ON t.id = it.tag_id
             GROUP BY t.id
             ORDER BY t.name",
        )?;

        let rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, i64>(2)?,
            ))
        })?;

        let mut tags = Vec::new();
        for tag_result in rows {
            tags.push(tag_result?);
        }

        Ok(tags)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::schema;
    use rusqlite::Connection;

    fn setup_test_db() -> Result<Connection> {
        let conn = Connection::open_in_memory()?;
        conn.execute("PRAGMA foreign_keys = ON", [])?;
        schema::initialize_schema(&conn)?;
        Ok(conn)
    }

    fn setup_test_item_with_tags(conn: &mut Connection, tags: &[String]) -> Result<i64> {
        // Create a test item
        let mut stmt = conn.prepare(
            "INSERT INTO stack_items (original_name, original_path, stored_hash, type) VALUES (?, ?, ?, ?)"
        )?;

        stmt.execute(params![
            "test_file.txt",
            "/path/to/test_file.txt",
            "test_hash",
            "file"
        ])?;

        let item_id = conn.last_insert_rowid();

        // Add tags
        for tag in tags {
            let tag_id = find_or_create_tag(conn, tag)?;
            conn.execute(
                "INSERT INTO item_tags (item_id, tag_id) VALUES (?, ?)",
                params![item_id, tag_id],
            )?;
        }

        Ok(item_id)
    }

    #[test]
    fn test_find_or_create_tag() -> Result<()> {
        let conn = setup_test_db()?;

        // Create new tag
        let tag_id1 = find_or_create_tag(&conn, "test_tag")?;
        assert!(tag_id1 > 0);

        // Find existing tag
        let tag_id2 = find_or_create_tag(&conn, "test_tag")?;
        assert_eq!(
            tag_id1, tag_id2,
            "Should return the same ID for existing tag"
        );

        // Create another tag
        let tag_id3 = find_or_create_tag(&conn, "another_tag")?;
        assert_ne!(tag_id1, tag_id3, "Different tags should have different IDs");

        Ok(())
    }

    #[test]
    fn test_get_for_item() -> Result<()> {
        let mut conn = setup_test_db()?;
        let item_id =
            setup_test_item_with_tags(&mut conn, &["tag1".to_string(), "tag2".to_string()])?;

        let tags = TagManager::get_for_item(&conn, item_id)?;

        assert_eq!(tags.len(), 2);
        assert!(tags.contains(&"tag1".to_string()));
        assert!(tags.contains(&"tag2".to_string()));

        Ok(())
    }

    #[test]
    fn test_add_to_item() -> Result<()> {
        let mut conn = setup_test_db()?;
        let item_id = setup_test_item_with_tags(&mut conn, &["existing".to_string()])?;

        // Add two new tags
        let added = TagManager::add_to_item(
            &mut conn,
            item_id,
            &["new_tag1".to_string(), "new_tag2".to_string()],
        )?;

        assert_eq!(added, 2, "Should have added 2 tags");

        // Verify tags were added
        let tags = TagManager::get_for_item(&conn, item_id)?;
        assert_eq!(tags.len(), 3);
        assert!(tags.contains(&"existing".to_string()));
        assert!(tags.contains(&"new_tag1".to_string()));
        assert!(tags.contains(&"new_tag2".to_string()));

        // Try adding duplicate tag
        let added = TagManager::add_to_item(&mut conn, item_id, &["existing".to_string()])?;
        assert_eq!(added, 0, "Should not add duplicate tag");

        Ok(())
    }

    #[test]
    fn test_remove_from_item() -> Result<()> {
        let mut conn = setup_test_db()?;
        let item_id = setup_test_item_with_tags(
            &mut conn,
            &["tag1".to_string(), "tag2".to_string(), "tag3".to_string()],
        )?;

        // Remove one tag
        let removed = TagManager::remove_from_item(&mut conn, item_id, &["tag2".to_string()])?;
        assert_eq!(removed, 1, "Should have removed 1 tag");

        // Verify tag was removed
        let tags = TagManager::get_for_item(&conn, item_id)?;
        assert_eq!(tags.len(), 2);
        assert!(tags.contains(&"tag1".to_string()));
        assert!(tags.contains(&"tag3".to_string()));
        assert!(!tags.contains(&"tag2".to_string()));

        // Try removing non-existent tag
        let removed =
            TagManager::remove_from_item(&mut conn, item_id, &["nonexistent".to_string()])?;
        assert_eq!(removed, 0, "Should not remove non-existent tag");

        Ok(())
    }

    #[test]
    fn test_cleanup_orphaned_tags() -> Result<()> {
        let mut conn = setup_test_db()?;

        // Create tags
        let tag_id1 = find_or_create_tag(&conn, "tag1")?;
        let tag_id2 = find_or_create_tag(&conn, "tag2")?;

        // Only use tag1 with an item
        let _item_id = setup_test_item_with_tags(&mut conn, &["tag1".to_string()])?;

        // Cleanup tag2 (unused)
        let cleaned = TagManager::cleanup_orphaned_tags(&conn, &[tag_id2])?;
        assert_eq!(cleaned, 1, "Should have cleaned 1 orphaned tag");

        // Verify tag1 still exists but tag2 is gone
        let all_tags = TagManager::list_all(&conn)?;
        assert_eq!(all_tags.len(), 1);
        assert_eq!(all_tags[0].0, tag_id1);
        assert_eq!(all_tags[0].1, "tag1");

        Ok(())
    }

    #[test]
    fn test_delete_unused_tags() -> Result<()> {
        let mut conn = setup_test_db()?;

        // Create tags but don't use them
        find_or_create_tag(&conn, "unused1")?;
        find_or_create_tag(&conn, "unused2")?;

        // Create and use one tag
        let tag_id = find_or_create_tag(&conn, "used")?;
        let _item_id = setup_test_item_with_tags(&mut conn, &["used".to_string()])?;

        // Delete unused tags
        let deleted = TagManager::delete_unused_tags(&conn)?;
        assert_eq!(deleted, 2, "Should have deleted 2 unused tags");

        // Verify only the used tag remains
        let tags = TagManager::list_all(&conn)?;
        assert_eq!(tags.len(), 1);
        assert_eq!(tags[0].0, tag_id);
        assert_eq!(tags[0].1, "used");

        Ok(())
    }

    #[test]
    fn test_list_all() -> Result<()> {
        let conn = setup_test_db()?;

        // Create two items with unique hashes to avoid constraint failure
        conn.execute(
            "INSERT INTO stack_items (original_name, original_path, stored_hash, type) VALUES (?, ?, ?, ?)",
            params!["file1.txt", "/path/to/file1.txt", "hash1_for_tagtest", "file"],
        )?;
        let item1_id = conn.last_insert_rowid();

        conn.execute(
            "INSERT INTO stack_items (original_name, original_path, stored_hash, type) VALUES (?, ?, ?, ?)",
            params!["file2.txt", "/path/to/file2.txt", "hash2_for_tagtest", "file"],
        )?;
        let item2_id = conn.last_insert_rowid();

        // Create tags and associate with items
        let tag1_id = find_or_create_tag(&conn, "tag1")?;
        let tag2_id = find_or_create_tag(&conn, "tag2")?;
        let common_id = find_or_create_tag(&conn, "common")?;

        // Associate tags with items
        conn.execute(
            "INSERT INTO item_tags (item_id, tag_id) VALUES (?, ?)",
            params![item1_id, tag1_id],
        )?;
        conn.execute(
            "INSERT INTO item_tags (item_id, tag_id) VALUES (?, ?)",
            params![item1_id, common_id],
        )?;
        conn.execute(
            "INSERT INTO item_tags (item_id, tag_id) VALUES (?, ?)",
            params![item2_id, tag2_id],
        )?;
        conn.execute(
            "INSERT INTO item_tags (item_id, tag_id) VALUES (?, ?)",
            params![item2_id, common_id],
        )?;

        // Get all tags
        let tags = TagManager::list_all(&conn)?;

        assert_eq!(tags.len(), 3);

        // Find common tag
        let common_tag = tags
            .iter()
            .find(|t| t.1 == "common")
            .expect("Common tag should exist");
        assert_eq!(common_tag.2, 2, "Common tag should be used by 2 items");

        // Find unique tags
        let tag1 = tags
            .iter()
            .find(|t| t.1 == "tag1")
            .expect("tag1 should exist");
        let tag2 = tags
            .iter()
            .find(|t| t.1 == "tag2")
            .expect("tag2 should exist");
        assert_eq!(tag1.2, 1, "tag1 should be used by 1 item");
        assert_eq!(tag2.2, 1, "tag2 should be used by 1 item");

        Ok(())
    }
}
