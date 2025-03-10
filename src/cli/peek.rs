use anyhow::{anyhow, Result};
use owo_colors::OwoColorize;
use tabled::{settings::Style, Table, Tabled};

use crate::db::{establish_connection, ItemManager};

// A structure for displaying item metadata as key-value pairs
#[derive(Tabled)]
struct KeyValue {
    #[tabled(rename = "FIELD")]
    key: String,

    #[tabled(rename = "VALUE")]
    value: String,
}

/// Peek at an item's metadata without restoring it.
pub fn peek(number: Option<usize>, tags: Option<Vec<String>>) -> Result<()> {
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

            ItemManager::get_by_id(&conn, id)?
                .ok_or_else(|| anyhow!("No item found with number={}", num))?
        }
        (Some(num), _) => {
            // Get item by number from full list (no tag filtering)
            let empty_tags = Vec::new();
            let id = ItemManager::get_id_by_display_number(&conn, num, &empty_tags)?
                .ok_or_else(|| anyhow!("No item found with number={}", num))?;

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

    // Apply direct coloring in strings instead of using tabled's built-in coloring
    let is_directory = item.item_type == "directory";

    // Build key-value pairs for display with colors applied
    let rows = vec![
        KeyValue {
            key: "DATABASE ID".to_string(),
            value: item.id.to_string(),
        },
        KeyValue {
            key: "TYPE".to_string(),
            value: if is_directory {
                format!("{}", item.item_type.blue())
            } else {
                item.item_type.clone()
            },
        },
        KeyValue {
            key: "NAME".to_string(),
            value: if is_directory {
                format!("{}", item.original_name.blue())
            } else {
                item.original_name.clone()
            },
        },
        KeyValue {
            key: "PATH".to_string(),
            value: item.original_path.clone(),
        },
        KeyValue {
            key: "PUSHED_AT".to_string(),
            value: item.pushed_at.format("%Y-%m-%d %H:%M:%S").to_string(),
        },
        KeyValue {
            key: "TAGS".to_string(),
            value: if item.tags.is_empty() {
                "[]".to_string()
            } else {
                format!("[{}]", item.tags.join(", ").green())
            },
        },
        KeyValue {
            key: "STORAGE_HASH".to_string(),
            value: item.stored_hash.clone(),
        },
    ];

    // Format table with simple styling
    let mut table = Table::new(rows);
    table.with(Style::modern_rounded());

    // Print table
    println!("{}", table);

    Ok(())
}
