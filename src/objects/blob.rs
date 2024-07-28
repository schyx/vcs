use std::io::Result;

use crate::{
    objects::write_object,
    utils::{fs_utils::get_file_contents, hash::sha2},
};

/// Creates a blob off of the given filename. Returns the hash of the blob.
///
/// Throws an error if the filename doesn't exist
pub fn create_blob(filename: &str) -> Result<String> {
    let contents = get_file_contents(filename)?;
    let contents = String::from("blob\n") + &contents;
    let hash = sha2(&contents);
    let _ = write_object(&hash, &contents);
    Ok(hash)
}
