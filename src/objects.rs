use std::{
    fs::{create_dir_all, File},
    io::{Result, Write},
    path::Path,
};

use crate::utils::fs_utils::{directory_exists, file_exists, get_file_contents};

pub mod blob;
pub mod commit;
pub mod tree;

/// Writes the object with hash `hash` and text `text` into the .vcs/objects directory
///
/// Will throw an error if `.vcs` directory doesn't exist
pub fn write_object(hash: &str, text: &str) -> Result<()> {
    assert!(directory_exists(".vcs"));
    let dir_name = &hash[0..2];
    let file_name = &hash[2..];
    let path = format!(".vcs/objects/{}/{}", dir_name, file_name);

    // Create parent directories if they do not exist
    let parent_dir = Path::new(&path).parent().unwrap();
    if !parent_dir.exists() {
        create_dir_all(parent_dir)?;
    }

    let mut file = File::create(path)?;
    let _ = file.write_all(text.as_bytes());

    Ok(())
}

/// Given a hash of the object, returns the contents of the file
///
/// Will panic if the hash does not exist in the objects dir
pub fn get_object_contents(hash: &str) -> Result<String> {
    let file_name = format!(
        ".vcs/objects/{}/{}",
        hash[0..2].to_string(),
        hash[2..].to_string()
    );
    if !file_exists(&file_name) {
        panic!("No object with hash of {} exists.", hash);
    }

    get_file_contents(&file_name)
}

/// Returns true iff a vcs object with the given hash exists
pub fn object_exists(hash: &str) -> bool {
    let file_name = format!(
        ".vcs/objects/{}/{}",
        hash[0..2].to_string(),
        hash[2..].to_string()
    );
    return file_exists(&file_name);
}

#[cfg(test)]
mod tests {
    /*
     * tests that an object is created at the correct place
     *
     * tests that get_object_contents returns the correct contents if file exists, and that it panics when
     * file doesn't exist
     */

    use std::fs::create_dir;

    use super::*;
    use crate::utils::{fs_utils::file_exists, test_dir::make_test_dir};

    #[test]
    fn test_write_object() -> Result<()> {
        let _test_dir = make_test_dir()?;
        let _ = create_dir(".vcs");
        let hash = "1234567890";
        let text = "test text";
        let _ = write_object(hash, text);

        // tests that write_object has the correct side effects
        let filename = ".vcs/objects/12/34567890";
        assert!(file_exists(filename));
        let contents = get_file_contents(filename)?;
        assert_eq!(text, contents);

        // test that get_object_contents gets the right contents
        assert_eq!("test text", get_object_contents(hash)?);

        Ok(())
    }

    #[test]
    #[should_panic(expected = "No object with hash of 1234567890 exists.")]
    fn panics_correctly() {
        let _test_dir = make_test_dir();
        let _ = create_dir(".vcs");
        let hash = "1234567890";
        let _ = get_object_contents(hash);
    }
}
