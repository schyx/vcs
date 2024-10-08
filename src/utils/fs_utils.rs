use std::{
    fs::{metadata, File, OpenOptions},
    io::{BufRead, BufReader, Lines, Read, Result},
    path::{Path, PathBuf},
};

use crate::objects::object_exists;

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
pub fn get_file_contents(path: &str) -> Result<String> {
    assert!(file_exists(path));
    let mut file = File::open(path)?;
    let mut contents = String::new();
    let _ = file.read_to_string(&mut contents);
    Ok(contents)
}

/// Gets the line number in file. Throws an error if the line number doesn't exist
pub fn get_line_in_file(filename: &str, line_num: usize) -> Result<String> {
    let file = File::open(filename)?;
    let reader = BufReader::new(file);
    reader.lines().nth(line_num).expect(&format!(
        "{} is not {} lines long",
        filename,
        line_num.to_string()
    ))
}

/// Gets the line number in the object corresponding to hash. Throws an error if the line number doesn't exist
pub fn get_line_in_object(hash: &str, line_num: usize) -> Result<String> {
    assert!(object_exists(hash));
    let filename = format!(".vcs/objects/{}/{}", &hash[0..2], &hash[2..]);
    get_line_in_file(&filename, line_num)
}

/// Removes all contents from a file
pub fn clear_file_contents(path: &str) -> Result<()> {
    OpenOptions::new().write(true).truncate(true).open(path)?;
    Ok(())
}

/// Returns the lines of a file that can be flattened to an iterator
///
/// Code from https://doc.rust-lang.org/rust-by-example/std_misc/file/read_lines.html
pub fn read_lines<P>(filename: P) -> Result<Lines<BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(BufReader::new(file).lines())
}

/// Removes the `./` in a Pathbuf string
pub fn no_dir_string(path: PathBuf) -> String {
    path.file_name().unwrap().to_str().unwrap().to_string()
}
