use anyhow::{anyhow, Result};
use std::env;

use crate::db::{establish_connection, get_stored_path, ItemManager};
use crate::fs::{check_destination_conflict, copy_dir_recursive};

/// Restore an item from the stack without removing it.
pub fn restore(number: Option<usize>, tags: Option<Vec<String>>) -> Result<()> {
    // Connect to database
    let conn = establish_connection()?;

    // Get item based on provided criteria
    let item = match (number, tags.as_ref()) {
        (Some(num), Some(tag_vec)) if !tag_vec.is_empty() => {
            // Get item by number within filtered tags
            let id =
                ItemManager::get_id_by_display_number(&conn, num, tag_vec)?.ok_or_else(|| {
                    anyhow!(
                        "No item found with number={} and tags=[{}]",
                        num,
                        tag_vec.join(", ")
                    )
                })?;

            // Get item by DB ID
            ItemManager::get_by_id(&conn, id)?
                .ok_or_else(|| anyhow!("No item found with number={}", num))?
        }
        (Some(num), _) => {
            // Get item by number from full list (no tag filtering)
            let empty_tags = Vec::new();
            let id = ItemManager::get_id_by_display_number(&conn, num, &empty_tags)?
                .ok_or_else(|| anyhow!("No item found with number={}", num))?;

            // Get item by DB ID
            ItemManager::get_by_id(&conn, id)?
                .ok_or_else(|| anyhow!("No item found with number={}", num))?
        }
        (None, Some(tags)) => {
            // Get latest item by tags
            ItemManager::get_latest_by_tags(&conn, tags)?
                .ok_or_else(|| anyhow!("No items found with tags=[{}]", tags.join(", ")))?
        }
        (None, None) => {
            // Get latest item
            ItemManager::get_latest(&conn)?.ok_or_else(|| anyhow!("No items in the stack"))?
        }
    };

    // Get current directory
    let current_dir = env::current_dir()?;

    // Construct destination path in current directory
    let dest_path = current_dir.join(&item.original_name);

    // Check if destination already exists
    if check_destination_conflict(&dest_path) {
        return Err(anyhow!(
            "Destination already exists: {}. Choose a different destination to avoid conflicts.",
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

    // Copy the item to the current directory (do not remove from stack)
    if item.item_type == "directory" {
        copy_dir_recursive(&source_path, &dest_path)?;
    } else {
        std::fs::copy(&source_path, &dest_path)?;
    }

    // Skip success messages for better CLI silence

    Ok(())
}
