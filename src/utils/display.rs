use crate::db::StackItem;
use tabled::{
    settings::{Alignment, Padding, Style},
    Table, Tabled,
};

#[derive(Tabled)]
pub struct DisplayItem {
    #[tabled(rename = "NO")]
    pub display_number: usize,

    #[tabled(rename = "T")]
    pub item_type: String,

    #[tabled(rename = "NAME")]
    pub name: String,

    #[tabled(rename = "TAGS")]
    pub tags: String,

    #[tabled(rename = "PUSHED AT")]
    pub pushed_at: String,
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.chars().count() <= max_len {
        return s.to_string();
    }

    let visible: String = s.chars().take(max_len - 3).collect();
    format!("{}...", visible)
}

/// Create a DisplayItem from a database StackItem and a display number
pub fn create_display_item(item: &StackItem, number: usize) -> DisplayItem {
    let type_indicator = if item.item_type == "directory" {
        "d"
    } else {
        "f"
    };
    let name = truncate(&item.original_name, 18);
    let item_type = type_indicator.to_string();

    let tags_str = if item.tags.is_empty() {
        String::new()
    } else {
        let tags_joined = item.tags.join(", ");
        truncate(&tags_joined, 18)
    };

    DisplayItem {
        display_number: number,
        item_type,
        name,
        tags: tags_str,
        pushed_at: item.pushed_at.format("%Y-%m-%d %H:%M:%S").to_string(),
    }
}

/// Create and display a table of stack items
pub fn display_items_table(items: &[StackItem]) {
    if items.is_empty() {
        return;
    }

    let display_items: Vec<DisplayItem> = items
        .iter()
        .enumerate()
        .map(|(index, item)| create_display_item(item, index + 1))
        .collect();

    let mut table = Table::new(display_items);

    table
        .with(Style::modern_rounded())
        .with(Padding::new(1, 1, 0, 0))
        .with(Alignment::left());

    println!("{}", table);
}

/// Create a display-ready tag for the tag list command
#[derive(Tabled)]
pub struct DisplayTag {
    #[tabled(rename = "ID")]
    pub id: i64,

    #[tabled(rename = "NAME")]
    pub name: String,

    #[tabled(rename = "COUNT")]
    pub count: i64,
}

/// Create and display a table of tags
pub fn display_tags_table(tags: &[(i64, String, i64)]) {
    if tags.is_empty() {
        return;
    }

    let display_tags: Vec<DisplayTag> = tags
        .iter()
        .map(|(id, name, count)| DisplayTag {
            id: *id,
            name: truncate(name, 18),
            count: *count,
        })
        .collect();

    let mut table = Table::new(display_tags);

    table
        .with(Style::modern_rounded())
        .with(Padding::new(1, 1, 0, 0))
        .with(Alignment::left());

    println!("{}", table);
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Local;

    fn create_test_item() -> StackItem {
        StackItem {
            id: 1,
            original_name: "test_file.txt".to_string(),
            original_path: "/path/to/test_file.txt".to_string(),
            stored_hash: "abcdef1234567890".to_string(),
            item_type: "file".to_string(),
            pushed_at: Local::now(),
            tags: vec!["tag1".to_string(), "tag2".to_string()],
        }
    }

    #[test]
    fn test_truncate() {
        assert_eq!(truncate("short", 10), "short");
        assert_eq!(truncate("this is a long string", 10), "this is...");
        assert_eq!(truncate("exactly15chars", 15), "exactly15chars");

        // Test string that needs truncation
        let long_string = "abcdefghijklmnopqrstuvwxyz";
        let result = truncate(long_string, 10);
        assert!(result.len() <= 13); // 10 + "..."
        assert!(result.ends_with("..."));
        assert_eq!(result, "abcdefg...");
    }

    #[test]
    fn test_create_display_item() {
        // Test file item
        let item = create_test_item();
        let display_item = create_display_item(&item, 1);

        assert_eq!(display_item.display_number, 1);
        assert_eq!(display_item.item_type, "f");
        assert_eq!(display_item.name, "test_file.txt");
        assert_eq!(display_item.tags, "tag1, tag2");

        // Create directory item
        let mut dir_item = create_test_item();
        dir_item.item_type = "directory".to_string();
        let display_dir = create_display_item(&dir_item, 2);

        assert_eq!(display_dir.item_type, "d");

        // Test long name truncation
        let mut long_name_item = create_test_item();
        long_name_item.original_name = "this_is_a_very_long_filename.txt".to_string();
        let display_long = create_display_item(&long_name_item, 3);

        // Check truncation occurred and has ... at the end
        assert!(display_long.name.len() < long_name_item.original_name.len());
        assert!(display_long.name.ends_with("..."));

        // Test long tags truncation
        let mut long_tags_item = create_test_item();
        long_tags_item.tags = vec![
            "tag1".to_string(),
            "tag2".to_string(),
            "tag3".to_string(),
            "very_long_tag_name".to_string(),
        ];
        let display_long_tags = create_display_item(&long_tags_item, 4);

        // Check it truncates and has ...
        assert!(display_long_tags.tags.contains("tag1"));
        assert!(display_long_tags.tags.contains("tag2"));
        assert!(display_long_tags.tags.ends_with("..."));
    }
}
