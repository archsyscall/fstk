use anyhow::Result;

use crate::db::{establish_connection, ItemManager};
use crate::utils::display;

/// List items in the stack, optionally filtered by tags.
pub fn list(tags: Option<Vec<String>>) -> Result<()> {
    // Connect to database
    let conn = establish_connection()?;

    // Get items with optional tag filtering
    let tags_vec = tags.unwrap_or_default();
    let mut items = ItemManager::list(&conn, &tags_vec)?;

    // Check if there are any items
    if items.is_empty() {
        if tags_vec.is_empty() {
            println!("No items in the stack.");
        } else {
            println!("No items found with tags=[{}].", tags_vec.join(", "));
        }
        return Ok(());
    }

    // Sort items by pushed_at in descending order (newest first)
    items.sort_by(|a, b| b.pushed_at.cmp(&a.pushed_at));

    // Display the items as a formatted table
    display::display_items_table(&items);

    Ok(())
}
