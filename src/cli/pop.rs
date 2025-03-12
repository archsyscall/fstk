use anyhow::{anyhow, Result};
use std::env;
use std::io::{self, Write};

use crate::db::{establish_connection, get_stored_path, ItemManager};
use crate::fs;
use crate::utils::numbers::parse_number_range;

/// Pop items from the stack and restore them to the current directory or a specified output directory.
pub fn pop(numbers: Option<String>, tags: Option<Vec<String>>, output: Option<String>) -> Result<()> {
    let tag_vec = tags.unwrap_or_default();
    let filter_by_tags = !tag_vec.is_empty();
    
    // Determine output directory (default to current directory if not specified)
    let output_dir = match &output {
        Some(path) => {
            let dir_path = std::path::PathBuf::from(path);
            // Check if the output directory exists and is a directory
            if !dir_path.exists() {
                return Err(anyhow!("Output directory does not exist: {}", dir_path.display()));
            }
            if !dir_path.is_dir() {
                return Err(anyhow!("Specified output path is not a directory: {}", dir_path.display()));
            }
            dir_path
        },
        None => env::current_dir()?
    };

    // Connect to database
    let mut conn = establish_connection()?;

    // If no numbers are specified, pop the latest item
    if numbers.is_none() {
        let item = if filter_by_tags {
            // Get latest item by tags
            ItemManager::get_latest_by_tags(&conn, &tag_vec)?
                .ok_or_else(|| anyhow!("No items found with tags=[{}]", tag_vec.join(", ")))?
        } else {
            // Get latest item
            ItemManager::get_latest(&conn)?.ok_or_else(|| anyhow!("No items in the stack"))?
        };

        // Construct destination path using output_dir
        let dest_path = output_dir.join(&item.original_name);

        // Check if destination already exists
        if fs::check_destination_conflict(&dest_path) {
            return Err(anyhow!(
                "Destination already exists: {}. Use 'restore' with a different destination to avoid conflicts.",
                dest_path.display()
            ));
        }

        // Get source path
        let source_path = get_stored_path(&item.stored_hash)?;

        // Ensure source exists
        if !source_path.exists() {
            return Err(anyhow!(
                "Error: Source file missing from storage: {}",
                source_path.display()
            ));
        }

        // Move the item
        fs::move_or_copy(&source_path, &dest_path)?;

        // Remove from database
        ItemManager::delete(&mut conn, item.id)?;

        // Skip success message for better CLI silence

        return Ok(());
    }

    // Parse the number range
    let number_list = parse_number_range(&numbers.unwrap())?;

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
        return Err(anyhow!("No valid items to pop"));
    }

    // Ask for confirmation before batch processing
    if items_to_process.len() > 1 {
        println!(
            "You are about to pop {} items from the stack.",
            items_to_process.len()
        );
        print!("Do you want to continue? [y/N]: ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim().to_lowercase();

        if input != "y" && input != "yes" {
            println!("Operation cancelled.");
            return Ok(());
        }
    }

    // Output directory is already determined above

    // Track statistics
    let mut success_count = 0;
    let mut skipped_count = 0;
    let mut failed_count = 0;

    // Save items count for summary
    let items_count = items_to_process.len();

    // Process all items atomically (based on the initial state)
    for (display_number, item) in items_to_process {
        // Construct destination path in output directory
        let dest_path = output_dir.join(&item.original_name);

        // Check if destination already exists
        if fs::check_destination_conflict(&dest_path) {
            println!("Destination already exists: {}", dest_path.display());

            if items_count > 1 {
                print!("Skip this item? [Y/n]: ");
                io::stdout().flush()?;

                let mut input = String::new();
                io::stdin().read_line(&mut input)?;
                let input = input.trim().to_lowercase();

                if input != "n" && input != "no" {
                    println!("Skipping item #{}", display_number);
                    skipped_count += 1;
                    continue;
                }

                println!(
                    "Cannot continue with item #{} due to destination conflict",
                    display_number
                );
                failed_count += 1;
                continue;
            } else {
                return Err(anyhow!(
                    "Destination already exists: {}. Use 'restore' with a different destination to avoid conflicts.",
                    dest_path.display()
                ));
            }
        }

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

        // Ensure source exists
        if !source_path.exists() {
            println!(
                "Source file missing for item #{}: {}",
                display_number,
                source_path.display()
            );
            failed_count += 1;
            continue;
        }

        // Move the item to the current directory
        match fs::move_or_copy(&source_path, &dest_path) {
            Ok(_) => {
                // Remove item from database
                match ItemManager::delete(&mut conn, item.id) {
                    Ok(true) => {
                        // Skip detailed success messages for batch operations
                        success_count += 1;
                    }
                    _ => {
                        println!("Error removing database entry for item #{}", display_number);
                        // Try to undo the file operation
                        let _ = fs::move_or_copy(&dest_path, &source_path);
                        failed_count += 1;
                    }
                }
            }
            Err(e) => {
                println!("Error moving item #{}: {}", display_number, e);
                failed_count += 1;
            }
        }
    }

    // Print summary if multiple items were processed
    if items_count > 1 {
        println!(
            "Summary: {} item(s) popped successfully, {} skipped, {} failed",
            success_count, skipped_count, failed_count
        );
    }

    if success_count > 0 {
        Ok(())
    } else {
        Err(anyhow!("Failed to pop any items"))
    }
}
