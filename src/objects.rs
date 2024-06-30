use std::{
    fs::{create_dir_all, File},
    io::{Error, Read, Write},
    path::Path,
};

use crate::utils::fs_utils::{directory_exists, file_exists};

pub mod blob;
pub mod commit;
pub mod tree;

/// Writes the object with hash `hash` and text `text` into the .vcs/objects directory
///
/// Will throw an error if `.vcs` directory doesn't exist
pub fn write_object(hash: &str, text: &str) -> Result<(), Error> {
    assert!(directory_exists(".vcs"));
    let dir_name = &hash[0..2];
    let file_name = &hash[2..];
    let mut path = String::from(".vcs/objects/");
    path.push_str(dir_name);
    path.push_str("/");
    path.push_str(file_name);

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
pub fn get_contents(hash: &str) -> String {
    let file_name: String = format!(
        ".vcs/objects/{}/{}",
        hash[0..2].to_string(),
        hash[2..].to_string()
    );
    if !file_exists(&file_name) {
        panic!("No object with hash of {} exists.", hash);
    }

    let mut file = Result::expect(File::open(file_name), "");
    let mut contents = String::new();
    let _ = file.read_to_string(&mut contents);
    contents
}

#[cfg(test)]
mod tests {
    /*
     * tests that an object is created at the correct place
     *
     * tests that get_contents returns the correct contents if file exists, and that it panics when
     * file doesn't exist
     */

    use std::{fs::create_dir, io::Read};

    use super::*;
    use crate::utils::{fs_utils::file_exists, test_dir::make_test_dir};

    #[test]
    fn test_write_object() -> Result<(), Error> {
        let _test_dir = make_test_dir();
        let _ = create_dir(".vcs");
        let hash = "1234567890";
        let text = "test text";
        let _ = write_object(hash, text);

        // tests that write_object has the correct side effects
        assert!(file_exists(".vcs/objects/12/34567890"));
        let mut file = File::open(".vcs/objects/12/34567890")?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        assert_eq!(text, contents);

        // test that get_contents gets the right contents
        assert_eq!("test text", get_contents(hash));

        Ok(())
    }

    #[test]
    #[should_panic(expected = "No object with hash of 1234567890 exists.")]
    fn panics_correctly() {
        let _test_dir = make_test_dir();
        let _ = create_dir(".vcs");
        let hash = "1234567890";
        get_contents(hash);
    }
}
