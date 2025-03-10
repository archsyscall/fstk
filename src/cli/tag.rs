use anyhow::{anyhow, Result};

use crate::db::{establish_connection, ItemManager, TagManager};
use crate::utils::display;

/// Add tags to an item in the stack.
pub fn add_tags(number: usize, tags: Vec<String>) -> Result<()> {
    // Connect to database
    let mut conn = establish_connection()?;

    // Get empty tags vector for display number lookup
    let empty_tags = Vec::new();

    // Important: For tag commands, always find item by number in the full list
    // because the --tags option is used for the tags to add
    let id = ItemManager::get_id_by_display_number(&conn, number, &empty_tags)?
        .ok_or_else(|| anyhow!("No item found with number={}", number))?;

    // Check if item exists (no need to store it since we removed the success message)
    ItemManager::get_by_id(&conn, id)?
        .ok_or_else(|| anyhow!("No item found with number={}", number))?;

    // Add tags
    let added = TagManager::add_to_item(&mut conn, id, &tags)?;

    // Only show message for error cases
    if added == 0 {
        println!("No new tags were added (all tags already exist)");
    }

    Ok(())
}

/// Remove tags from an item in the stack.
pub fn remove_tags(number: usize, tags: Vec<String>) -> Result<()> {
    // Connect to database
    let mut conn = establish_connection()?;

    // Get empty tags vector for display number lookup
    let empty_tags = Vec::new();

    // Important: For tag commands, always find item by number in the full list
    // because the --tags option is used for the tags to remove
    let id = ItemManager::get_id_by_display_number(&conn, number, &empty_tags)?
        .ok_or_else(|| anyhow!("No item found with number={}", number))?;

    // Check if item exists (no need to store it since we removed the success message)
    ItemManager::get_by_id(&conn, id)?
        .ok_or_else(|| anyhow!("No item found with number={}", number))?;

    // Remove tags
    let removed = TagManager::remove_from_item(&mut conn, id, &tags)?;

    // Only show message for error cases
    if removed == 0 {
        println!("No tags were removed (tags do not exist for this item)");
    }

    Ok(())
}

/// List all tags in the system with usage count.
pub fn list_tags() -> Result<()> {
    // Connect to database
    let conn = establish_connection()?;

    // Clean up unused tags silently
    TagManager::delete_unused_tags(&conn)?;

    // Get all tags
    let tags = TagManager::list_all(&conn)?;

    // Check if there are any tags
    if tags.is_empty() {
        println!("No tags found in the system.");
        return Ok(());
    }

    // Sort tags by usage count (highest usage first)
    let mut sorted_tags = tags.clone();
    sorted_tags.sort_by(|a, b| b.2.cmp(&a.2));

    // Display the tags table
    display::display_tags_table(&sorted_tags);

    Ok(())
}
