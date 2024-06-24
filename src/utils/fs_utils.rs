use std::{fs::metadata, path::Path};

/// Returns true iff `path` is a directory that exists
pub fn directory_exists(path: &str) -> bool {
    let path = Path::new(path);
    metadata(path)
        .map(|metadata| metadata.is_dir())
        .unwrap_or(false)
}

/// Returns true iff `path` is a directory that exists
pub fn file_exists(path: &str) -> bool {
    let path = Path::new(path);
    metadata(path)
        .map(|metadata| metadata.is_file())
        .unwrap_or(false)
}
