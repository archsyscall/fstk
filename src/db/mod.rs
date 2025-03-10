mod item;
pub mod schema;
mod tag;

pub use item::{ItemManager, StackItem};
pub use tag::TagManager;

use anyhow::{anyhow, Result};
use rusqlite::Connection;
use std::path::PathBuf;

// Path operations
pub fn get_db_path() -> Result<PathBuf> {
    let home_dir = dirs::home_dir().ok_or_else(|| anyhow!("Could not determine home directory"))?;
    let fstk_dir = home_dir.join(".fstk");

    // Create directories if they don't exist
    std::fs::create_dir_all(&fstk_dir)?;
    std::fs::create_dir_all(fstk_dir.join(".data"))?;

    Ok(fstk_dir.join("fstk.db"))
}

pub fn establish_connection() -> Result<Connection> {
    let db_path = get_db_path()?;

    let conn = Connection::open(&db_path)?;

    // Enable foreign key constraints
    conn.execute("PRAGMA foreign_keys = ON", [])?;

    // Initialize schema if needed
    schema::initialize_schema(&conn)?;

    Ok(conn)
}

pub fn get_data_dir() -> Result<PathBuf> {
    let home_dir = dirs::home_dir().ok_or_else(|| anyhow!("Could not determine home directory"))?;
    let data_dir = home_dir.join(".fstk").join(".data");

    // Create directory if it doesn't exist
    std::fs::create_dir_all(&data_dir)?;

    Ok(data_dir)
}

pub fn get_stored_path(hash: &str) -> Result<PathBuf> {
    let data_dir = get_data_dir()?;
    Ok(data_dir.join(hash))
}
