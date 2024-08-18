use std::io::Result;

use crate::{
    objects::write_object,
    utils::{fs_utils::get_file_contents, hash::sha2},
};

/// Creates a blob off of the given filename. Returns the hash of the blob.
///
/// Throws an error if the filename doesn't exist
pub fn create_blob(filename: &str) -> Result<String> {
    let (hash, contents) = get_blob_hash(filename)?;
    let _ = write_object(&hash, &contents);
    Ok(hash)
}

/// Gets the hash of a blob given a filename and the new contents written in the object.
///
/// Does not create the blob in the objects directory
pub fn get_blob_hash(filename: &str) -> Result<(String, String)> {
    let contents = get_file_contents(filename)?;
    let contents = String::from("blob\n") + &contents;
    Ok((sha2(&contents), contents))
}
