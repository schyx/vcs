use std::{
    fs::{metadata, File},
    io::Read,
    path::Path,
};

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

/// Gets the text in file
pub fn get_file_contents(path: &str) -> String {
    let mut file = Result::expect(File::open(path), "");
    let mut contents = String::new();
    let _ = file.read_to_string(&mut contents);
    contents
}
