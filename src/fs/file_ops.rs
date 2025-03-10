use anyhow::{anyhow, Result};
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Move or copy a file or directory from source to destination.
/// If the move operation fails with EXDEV (cross-device) error, it will fallback to copy+delete.
pub fn move_or_copy<P: AsRef<Path>, Q: AsRef<Path>>(src: P, dst: Q) -> Result<()> {
    let src = src.as_ref();
    let dst = dst.as_ref();

    match fs::rename(src, dst) {
        Ok(_) => Ok(()),
        Err(e) if e.raw_os_error() == Some(libc::EXDEV) => {
            if src.is_dir() {
                copy_dir_recursive(src, dst)?;
                fs::remove_dir_all(src)?;
            } else {
                fs::copy(src, dst)?;
                fs::remove_file(src)?;
            }
            Ok(())
        }
        Err(e) => Err(anyhow!(
            "Failed to move '{}' to '{}': {}",
            src.display(),
            dst.display(),
            e
        )),
    }
}

/// Recursively copy a directory and all its contents.
pub fn copy_dir_recursive<P: AsRef<Path>, Q: AsRef<Path>>(src: P, dst: Q) -> Result<()> {
    let src = src.as_ref();
    let dst = dst.as_ref();

    if !dst.exists() {
        fs::create_dir_all(dst)?;
    }

    for entry in WalkDir::new(src) {
        let entry = entry?;
        let path = entry.path();

        if path == src {
            continue;
        }

        let relative_path = path.strip_prefix(src)?;
        let target_path = dst.join(relative_path);

        if path.is_dir() {
            fs::create_dir_all(&target_path)?;
        } else {
            if let Some(parent) = target_path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(path, &target_path)?;
        }
    }

    Ok(())
}

/// Generate a hash string from a file or directory path.
pub fn generate_hash(path: &Path, is_dir: bool) -> Result<String> {
    use sha2::{Digest, Sha256};

    let now = chrono::Local::now();
    let timestamp = now.timestamp_nanos_opt().unwrap_or(0);

    let mut hasher = Sha256::new();
    let path_str = path.to_string_lossy();

    hasher.update(path_str.as_bytes());
    hasher.update(timestamp.to_be_bytes());
    hasher.update(if is_dir { b"dir" } else { b"fil" });

    let hash = hasher.finalize();
    let hash_str = hex::encode(hash);

    Ok(hash_str[..16].to_string())
}

/// Check if a path exists and is accessible.
pub fn is_path_accessible(path: &Path) -> Result<bool> {
    if !path.exists() {
        return Err(anyhow!("Path does not exist: {}", path.display()));
    }

    match path.metadata() {
        Ok(_) => Ok(true),
        Err(e) => Err(anyhow!("Unable to access path '{}': {}", path.display(), e)),
    }
}

/// Check if a file or directory already exists at the destination path.
pub fn check_destination_conflict(path: &Path) -> bool {
    path.exists()
}

/// Get the file name from a path.
pub fn get_file_name(path: &Path) -> Result<String> {
    path.file_name()
        .ok_or_else(|| anyhow!("Invalid path, no filename component found"))
        .map(|name| name.to_string_lossy().to_string())
}

/// Get the absolute path of a path, optionally resolving symlinks.
pub fn get_absolute_path(path: &Path) -> Result<PathBuf> {
    match path.canonicalize() {
        Ok(p) => Ok(p),
        Err(e) => Err(anyhow!(
            "Could not resolve absolute path for '{}': {}",
            path.display(),
            e
        )),
    }
}

/// Create parent directories for a file if they don't exist.
#[allow(dead_code)]
pub fn ensure_parent_dirs(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_move_or_copy_file() {
        let temp_dir = tempdir().unwrap();
        let source_path = temp_dir.path().join("source.txt");
        let dest_path = temp_dir.path().join("dest.txt");

        let mut file = File::create(&source_path).unwrap();
        writeln!(file, "Test content").unwrap();

        move_or_copy(&source_path, &dest_path).unwrap();

        assert!(!source_path.exists());
        assert!(dest_path.exists());

        let content = std::fs::read_to_string(&dest_path).unwrap();
        assert_eq!(content, "Test content\n");
    }

    #[test]
    fn test_copy_dir_recursive() {
        let temp_dir = tempdir().unwrap();

        // Create source directory with files and subdirectories
        let src_dir = temp_dir.path().join("src_dir");
        fs::create_dir(&src_dir).unwrap();

        // Create a file in the source directory
        let src_file = src_dir.join("file.txt");
        let mut file = File::create(&src_file).unwrap();
        writeln!(file, "Test content").unwrap();

        // Create a subdirectory
        let src_subdir = src_dir.join("subdir");
        fs::create_dir(&src_subdir).unwrap();

        // Create a file in the subdirectory
        let src_subfile = src_subdir.join("subfile.txt");
        let mut subfile = File::create(&src_subfile).unwrap();
        writeln!(subfile, "Subdir test content").unwrap();

        // Destination directory
        let dst_dir = temp_dir.path().join("dst_dir");

        // Copy the directory recursively
        copy_dir_recursive(&src_dir, &dst_dir).unwrap();

        // Check if all files and directories were copied
        assert!(dst_dir.exists());
        assert!(dst_dir.join("file.txt").exists());
        assert!(dst_dir.join("subdir").exists());
        assert!(dst_dir.join("subdir/subfile.txt").exists());

        // Check file contents
        let content = std::fs::read_to_string(&dst_dir.join("file.txt")).unwrap();
        assert_eq!(content, "Test content\n");

        let subcontent = std::fs::read_to_string(&dst_dir.join("subdir/subfile.txt")).unwrap();
        assert_eq!(subcontent, "Subdir test content\n");
    }

    #[test]
    fn test_generate_hash() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");

        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "Test content").unwrap();

        let hash1 = generate_hash(&file_path, false).unwrap();
        assert_eq!(hash1.len(), 16);

        let other_path = dir.path().join("other.txt");
        let mut file2 = File::create(&other_path).unwrap();
        writeln!(file2, "Test content").unwrap();

        let hash2 = generate_hash(&other_path, false).unwrap();
        assert_ne!(hash1, hash2);

        // Test directory hash
        let dir_hash = generate_hash(dir.path(), true).unwrap();
        assert_eq!(dir_hash.len(), 16);
        assert_ne!(hash1, dir_hash);
    }

    #[test]
    fn test_is_path_accessible() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");

        // Path doesn't exist yet
        let result = is_path_accessible(&file_path);
        assert!(result.is_err());

        // Create the file
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "Test content").unwrap();

        // Path exists and is accessible
        let result = is_path_accessible(&file_path);
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_check_destination_conflict() {
        let dir = tempdir().unwrap();
        let existing_path = dir.path().join("existing.txt");
        let nonexistent_path = dir.path().join("nonexistent.txt");

        // Create a file
        let mut file = File::create(&existing_path).unwrap();
        writeln!(file, "Test content").unwrap();

        // Check for conflict
        assert!(check_destination_conflict(&existing_path));
        assert!(!check_destination_conflict(&nonexistent_path));
    }

    #[test]
    fn test_get_file_name() {
        let file_path = PathBuf::from("/path/to/file.txt");
        let result = get_file_name(&file_path);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "file.txt");

        // Test with path that has no filename
        let invalid_path = PathBuf::from("/");
        let result = get_file_name(&invalid_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_ensure_parent_dirs() {
        let dir = tempdir().unwrap();
        let nested_path = dir.path().join("parent/child/file.txt");

        // Parent directories don't exist yet
        assert!(!nested_path.parent().unwrap().exists());

        // Create parent directories
        ensure_parent_dirs(&nested_path).unwrap();

        // Parent directories should now exist
        assert!(nested_path.parent().unwrap().exists());
    }
}
