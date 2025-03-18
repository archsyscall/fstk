use anyhow::{anyhow, Result};
use std::path::PathBuf;

use crate::db::{establish_connection, get_stored_path, ItemManager};
use crate::fs;

/// Restore an item from the stack to its original location and remove it from the stack.
pub fn restore(number: Option<usize>, tags: Option<Vec<String>>) -> Result<()> {
    let tag_vec = tags.unwrap_or_default();
    let filter_by_tags = !tag_vec.is_empty();

    // Connect to database
    let mut conn = establish_connection()?;

    // Get item based on provided criteria
    let item = match number {
        Some(num) => {
            // Get item by number with optional tag filtering
            let id = if filter_by_tags {
                ItemManager::get_id_by_display_number(&conn, num, &tag_vec)?.ok_or_else(|| {
                    anyhow!(
                        "No item found with number={} and tags=[{}]",
                        num,
                        tag_vec.join(", ")
                    )
                })?
            } else {
                let empty_tags = Vec::new();
                ItemManager::get_id_by_display_number(&conn, num, &empty_tags)?
                    .ok_or_else(|| anyhow!("No item found with number={}", num))?
            };

            // Get item by DB ID
            ItemManager::get_by_id(&conn, id)?
                .ok_or_else(|| anyhow!("No item found with number={}", num))?
        }
        None => {
            // Get latest item
            if filter_by_tags {
                ItemManager::get_latest_by_tags(&conn, &tag_vec)?
                    .ok_or_else(|| anyhow!("No items found with tags=[{}]", tag_vec.join(", ")))?
            } else {
                ItemManager::get_latest(&conn)?.ok_or_else(|| anyhow!("No items in the stack"))?
            }
        }
    };

    // Construct destination path using the original path and filename
    let mut dest_path = PathBuf::from(&item.original_path);
    dest_path.push(&item.original_name);

    // Check if destination already exists
    if fs::check_destination_conflict(&dest_path) {
        return Err(anyhow!(
            "Original destination already exists: {}. Use 'pop' with a custom destination to avoid conflicts.",
            dest_path.display()
        ));
    }

    // Get source path from the data directory
    let source_path = get_stored_path(&item.stored_hash)?;

    // Ensure source exists
    if !source_path.exists() {
        return Err(anyhow!(
            "Error: Source file missing from storage: {}",
            source_path.display()
        ));
    }

    // Ensure parent directory exists
    if let Some(parent) = dest_path.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent)?;
        }
    }

    // Move the item to its original location
    fs::move_or_copy(&source_path, &dest_path)?;

    // Remove from database
    ItemManager::delete(&mut conn, item.id)?;

    Ok(())
}
