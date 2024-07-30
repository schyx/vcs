use std::{
    fs::OpenOptions,
    io::{Result, Write},
};

use crate::{
    objects::blob::create_blob,
    utils::fs_utils::{directory_exists, file_exists},
};
/// Executes `vcs add` with `args` as arguments. Returns the string that should be logged to the
/// console and the hash of the added object if operation was successful.
///
/// If there is one argument, adds the file in the argument to the .vcs index
/// If not in a vcs directory, log `Not in an initialized vcs directory.`
/// If incorrect number of commands, log `Incorrect operands.`
/// If file doesn't exist, log `File does not exist.`
/// Explicitly, if the file exists, this function updates the index file with a file's new hash,
/// and adds the text of the file to the objects directory. It also updates the parent trees'
/// hashes.
///
/// * `args` - arguments `add` was called with
pub fn add(args: &Vec<String>) -> Result<(String, String)> {
    if !directory_exists(".vcs") {
        return Ok((
            String::from("Not in an initialized vcs directory."),
            String::from(""),
        ));
    }

    match args.len() {
        3 => {
            let filename = &args[2];
            if !file_exists(filename) {
                return Ok((String::from("File does not exist."), String::from("")));
            }
            let hash = create_blob(filename)?;
            let mut file = OpenOptions::new()
                .append(true)
                .open(".vcs/index")
                .expect("Cannot open file");
            let to_index = format!("blob {} {}\n", hash, filename);
            let _ = file.write_all(to_index.as_bytes());
            Ok((String::from(""), hash))
        }
        _ => Ok((String::from("Incorrect operands."), String::from(""))),
    }
}

#[cfg(test)]
pub mod tests {

    // Partitions for add
    // Partition on error condition:
    //      Not in VCS dir, incorrect number of operands, file doesn't exist, no error
    // Further partition on no error,
    //      no error no subdirectories, there are subdirectories, same version as commit version,
    //      file was removed

    use super::*;
    use crate::{
        objects::get_object_contents,
        operations::init::init,
        utils::{fs_utils::get_file_contents, hash::sha2, test_dir::make_test_dir},
    };
    use std::fs::{create_dir_all, File};

    #[test]
    fn not_in_vcs_dir() -> Result<()> {
        let _test_dir = make_test_dir()?;
        let _ = File::create("test.txt");
        let test_args: Vec<String> = vec![
            String::from("target/debug/vcs"),
            String::from("add"),
            String::from("test.txt"),
        ];
        assert_eq!("Not in an initialized vcs directory.", add(&test_args)?.0);
        Ok(())
    }

    #[test]
    fn incorrect_arg_number() -> Result<()> {
        let _test_dir = make_test_dir()?;
        let _ = init(&vec![
            String::from("target/debug/vcs"),
            String::from("init"),
        ]);
        let _ = File::create("test.txt");
        let _ = File::create("test1.txt");
        let test_args: Vec<String> = vec![
            String::from("target/debug/vcs"),
            String::from("add"),
            String::from("test.txt"),
            String::from("test1.txt"),
        ];
        assert_eq!("Incorrect operands.", add(&test_args)?.0);
        Ok(())
    }

    #[test]
    fn file_does_not_exist() -> Result<()> {
        let _test_dir = make_test_dir()?;
        let _ = init(&vec![
            String::from("target/debug/vcs"),
            String::from("init"),
        ]);
        let test_args: Vec<String> = vec![
            String::from("target/debug/vcs"),
            String::from("add"),
            String::from("test.txt"),
        ];
        assert_eq!("File does not exist.", add(&test_args)?.0);
        Ok(())
    }

    #[test]
    fn correct_add_operation() -> Result<()> {
        let _test_dir = make_test_dir()?;
        let _ = init(&vec![
            String::from("target/debug/vcs"),
            String::from("init"),
        ]);
        let _ = File::create("test.txt");
        let test_args: Vec<String> = vec![
            String::from("target/debug/vcs"),
            String::from("add"),
            String::from("test.txt"),
        ];

        // Console output check
        let (output_string, output_hash) = add(&test_args)?;
        assert_eq!("", output_string);

        // Mutation of vcs dir check
        let empty_string_hash = sha2("blob\n");
        assert_eq!("blob\n", get_object_contents(&empty_string_hash)?);
        assert_eq!(output_hash, empty_string_hash);
        let index_contents = get_file_contents(".vcs/index")?;
        assert_eq!(
            format!("blob {} test.txt\n", empty_string_hash),
            index_contents
        );

        // Subdirectory
        let _ = create_dir_all("test_dir1/test_dir2/");
        let mut file = File::create("test_dir1/test_dir2/test.txt")?;
        let test_args: Vec<String> = vec![
            String::from("target/debug/vcs"),
            String::from("add"),
            String::from("test_dir1/test_dir2/test.txt"),
        ];
        let file_text = "Test subdirectories!";
        let blob_text = String::from("blob\n") + file_text;
        let blob_hash = sha2(&blob_text);
        let _ = file.write(file_text.as_bytes());
        let (output_text, output_hash) = add(&test_args)?;
        assert_eq!("", output_text);
        assert_eq!(output_hash, sha2(&blob_text));
        assert_eq!(blob_text, get_object_contents(&blob_hash)?);
        let index_contents = get_file_contents(".vcs/index")?;
        assert_eq!(
            format!(
                "blob {} test.txt\nblob {} test_dir1/test_dir2/test.txt\n",
                empty_string_hash, blob_hash
            ),
            index_contents
        );
        Ok(())
    }

    #[test]
    fn same_as_commit_version() -> Result<()> {
        panic!("Add behavior after commit has been added");
    }

    #[test]
    fn undoes_remove() -> Result<()> {
        panic!("Add behavior after rm has been added")
    }
}
