use thiserror::Error;

#[derive(Error, Debug)]
#[allow(dead_code)]
pub enum FstkError {
    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("File system error: {0}")]
    FileSystemError(String),

    #[error("Item not found: {0}")]
    ItemNotFound(String),

    #[error("Tag error: {0}")]
    TagError(String),

    #[error("Destination conflict: {0}")]
    DestinationConflict(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Invalid argument: {0}")]
    InvalidArgument(String),

    #[error("IO error: {0}")]
    IoError(String),

    #[error("Other error: {0}")]
    Other(String),
}

impl From<std::io::Error> for FstkError {
    fn from(error: std::io::Error) -> Self {
        FstkError::IoError(error.to_string())
    }
}

impl From<rusqlite::Error> for FstkError {
    fn from(error: rusqlite::Error) -> Self {
        FstkError::DatabaseError(error.to_string())
    }
}

impl From<anyhow::Error> for FstkError {
    fn from(error: anyhow::Error) -> Self {
        FstkError::Other(error.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::anyhow;
    use rusqlite::Error as SqliteError;
    use std::io::{Error as IoError, ErrorKind};

    #[test]
    fn test_io_error_conversion() {
        let io_error = IoError::new(ErrorKind::NotFound, "file not found");
        let fstk_error: FstkError = io_error.into();

        match fstk_error {
            FstkError::IoError(msg) => assert!(msg.contains("file not found")),
            _ => panic!("Expected IoError variant"),
        }
    }

    #[test]
    fn test_sqlite_error_conversion() {
        let sqlite_error = SqliteError::QueryReturnedNoRows;
        let fstk_error: FstkError = sqlite_error.into();

        match fstk_error {
            FstkError::DatabaseError(msg) => assert!(msg.contains("no rows")),
            _ => panic!("Expected DatabaseError variant"),
        }
    }

    #[test]
    fn test_anyhow_error_conversion() {
        let anyhow_error = anyhow!("test error message");
        let fstk_error: FstkError = anyhow_error.into();

        match fstk_error {
            FstkError::Other(msg) => assert_eq!(msg, "test error message"),
            _ => panic!("Expected Other variant"),
        }
    }

    #[test]
    fn test_error_display() {
        let error = FstkError::ItemNotFound("test item".to_string());
        assert_eq!(format!("{}", error), "Item not found: test item");

        let error = FstkError::FileSystemError("permission denied".to_string());
        assert_eq!(format!("{}", error), "File system error: permission denied");
    }
}
