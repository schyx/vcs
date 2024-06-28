use std::{
    fs::{create_dir_all, File},
    io::{Error, Write},
    path::Path,
};

use crate::utils::fs_utils::directory_exists;

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

#[cfg(test)]
mod tests {
    /*
     * tests that an object is created at the correct place
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

        assert!(file_exists(".vcs/objects/12/34567890"));

        let mut file = File::open(".vcs/objects/12/34567890")?;
        let mut contents = String::new();

        file.read_to_string(&mut contents)?;
        assert_eq!(text, contents);

        Ok(())
    }
}
