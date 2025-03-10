use anyhow::{anyhow, Result};
use std::fs;

use crate::db::{establish_connection, get_stored_path, ItemManager};
use crate::utils::numbers::parse_number_range;

/// Remove items from the stack without restoring them.
pub fn remove(numbers: String, tags: Option<Vec<String>>) -> Result<()> {
    // Parse number range
    let number_list = parse_number_range(&numbers)?;

    // Connect to database
    let mut conn = establish_connection()?;

    let tag_vec = tags.unwrap_or_default();
    let filter_by_tags = !tag_vec.is_empty();

    // First, collect all the items to process based on the current state
    // This ensures we're working with a snapshot of the current display numbers
    let mut items_to_process = Vec::new();

    // Get list of all items with current display numbers
    let mut all_items = if filter_by_tags {
        ItemManager::list(&conn, &tag_vec)?
    } else {
        let empty_tags = Vec::new();
        ItemManager::list(&conn, &empty_tags)?
    };

    // Sort by pushed_at (descending) to match display order
    all_items.sort_by(|a, b| b.pushed_at.cmp(&a.pushed_at));

    // Map display numbers to database IDs
    for &number in &number_list {
        if number > 0 && number <= all_items.len() {
            // Convert display number to zero-based index
            let idx = number - 1;
            items_to_process.push((number, all_items[idx].clone()));
        } else {
            // Report invalid number
            if filter_by_tags {
                println!(
                    "No item found with number={} and tags=[{}]",
                    number,
                    tag_vec.join(", ")
                );
            } else {
                println!("No item found with number={}", number);
            }
        }
    }

    // Exit early if no valid items to process
    if items_to_process.is_empty() {
        return Err(anyhow!("No valid items to remove"));
    }

    // Track statistics
    let mut success_count = 0;
    let mut failed_count = 0;

    // Save items count for summary
    let items_count = items_to_process.len();

    // Now process all the collected items (atomically, based on the initial state)
    for (display_number, item) in items_to_process {
        // Get source path from the data directory
        let source_path = match get_stored_path(&item.stored_hash) {
            Ok(path) => path,
            Err(e) => {
                println!(
                    "Error getting stored path for item #{}: {}",
                    display_number, e
                );
                failed_count += 1;
                continue;
            }
        };

        // Delete the item from the database
        match ItemManager::delete(&mut conn, item.id) {
            Ok(true) => {
                // Delete the file or directory from storage if it exists
                if source_path.exists() {
                    let result = if item.item_type == "directory" {
                        fs::remove_dir_all(&source_path)
                    } else {
                        fs::remove_file(&source_path)
                    };

                    if result.is_ok() {
                        // Skip detailed success messages for batch operations

                        success_count += 1;
                    } else {
                        println!(
                            "Error removing file/directory for item #{}: {:?}",
                            display_number, result
                        );
                        failed_count += 1;
                    }
                } else {
                    // File/directory already removed from storage but entry was in DB
                    println!(
                        "Removed database entry for '{}' (#{}) (file was already removed)",
                        item.original_name, display_number
                    );
                    success_count += 1;
                }
            }
            Ok(false) => {
                println!("Error removing database entry for item #{}", display_number);
                failed_count += 1;
            }
            Err(e) => {
                println!("Database error for item #{}: {}", display_number, e);
                failed_count += 1;
            }
        }
    }

    // Print summary if multiple items were processed
    if items_count > 1 {
        println!(
            "Summary: {} item(s) removed successfully, {} failed",
            success_count, failed_count
        );
    }

    if success_count > 0 {
        Ok(())
    } else {
        Err(anyhow!("Failed to remove any items"))
    }
}
