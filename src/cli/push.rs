use anyhow::{anyhow, Result};
use std::path::PathBuf;

use crate::db::{establish_connection, get_data_dir, ItemManager};
use crate::fs;

/// Push a file or directory to the stack.
pub fn push(path_str: &str, tags: Option<Vec<String>>) -> Result<i64> {
    let path = PathBuf::from(path_str);

    if !fs::is_path_accessible(&path)? {
        return Err(anyhow!("Path is not accessible: {}", path.display()));
    }

    let abs_path = fs::get_absolute_path(&path)?;
    let name = fs::get_file_name(&abs_path)?;
    let parent = match abs_path.parent() {
        Some(p) => p.to_string_lossy().to_string(),
        None => String::from("/"),
    };

    let is_dir = abs_path.is_dir();
    let item_type = if is_dir { "directory" } else { "file" };
    let hash = fs::generate_hash(&abs_path, is_dir)?;

    let data_dir = get_data_dir()?;
    let target_path = data_dir.join(&hash);

    fs::move_or_copy(&abs_path, &target_path)?;

    let mut conn = establish_connection()?;
    let tags_vec = tags.unwrap_or_default();
    let item_id = ItemManager::insert(&mut conn, &name, &parent, &hash, item_type, &tags_vec)?;

    Ok(item_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::schema;
    use rusqlite::Connection;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    #[allow(dead_code)] // 테스트 코드에서 필요한 헬퍼 함수들이므로 dead_code 경고 무시
                        // Mocked versions of DB functions for testing
    fn test_establish_connection() -> Result<Connection> {
        let conn = Connection::open_in_memory()?;
        conn.execute("PRAGMA foreign_keys = ON", [])?;
        schema::initialize_schema(&conn)?;
        Ok(conn)
    }

    #[allow(dead_code)] // 테스트 코드에서 필요한 헬퍼 함수들이므로 dead_code 경고 무시
    fn test_get_data_dir() -> Result<PathBuf> {
        Ok(tempdir()?.path().to_path_buf())
    }

    // Helper to create a temporary test file
    fn create_test_file(content: &str) -> Result<(tempfile::TempDir, PathBuf)> {
        let temp_dir = tempdir()?;
        let file_path = temp_dir.path().join("test.txt");

        let mut file = File::create(&file_path)?;
        write!(file, "{}", content)?;

        Ok((temp_dir, file_path))
    }

    #[test]
    #[ignore] // This test requires mocking which we're simulating but not actually implementing
    fn test_push_file() -> Result<()> {
        // This is a sample test that demonstrates how the push function would be tested
        // In a real implementation, we would use test doubles or a testing framework

        let (_dir, file_path) = create_test_file("Test file content")?;

        // We would need to mock fs::move_or_copy, establish_connection, get_data_dir, etc.
        // For example, by using a testing framework or dependency injection

        // Simplified test structure
        let tags = Some(vec!["tag1".to_string(), "tag2".to_string()]);
        let _item_id = push(file_path.to_str().unwrap(), tags)?;

        // In a real test we would verify:
        // 1. The file was moved/copied to the target location
        // 2. An item was properly inserted in the database
        // 3. Tags were properly associated

        Ok(())
    }
}
