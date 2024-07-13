use std::{fs::OpenOptions, io::Write};

use crate::{
    objects::write_object,
    utils::{
        fs_utils::{directory_exists, file_exists, get_file_contents},
        hash::sha2,
    },
};
/// Executes `vcs add` with `args` as arguments
///
/// If there is one argument, adds the file in the argument to the .vcs index
/// If not in a vcs directory, log `Not in an initialized vcs directory.`
/// If incorrect number of commands, log `Incorrect operands.`
/// If file doesn't exist, log `File does not exist.`
/// Explicitly, if the file exists, this function updates the index file with a file's new hash,
/// and adds the text of the file to the objects directory. It also updates the parent trees'
/// hashes.
///
/// * `args` - arguments `init` was called with
pub fn add(args: &Vec<String>) -> String {
    if !directory_exists(".vcs") {
        return String::from("Not in an initialized vcs directory.");
    }

    match args.len() {
        3 => {
            let filename = &args[2];
            if !file_exists(filename) {
                return String::from("File does not exist.");
            }
            let contents = get_file_contents(filename);
            let hash = sha2(&contents);
            let _ = write_object(&hash, &contents);
            let mut file = OpenOptions::new()
                .append(true)
                .open(".vcs/index")
                .expect("Cannot open file");
            let to_index = format!("blob {} {}\n", hash, filename);
            let _ = file.write_all(to_index.as_bytes());
            String::from("")
        }
        _ => String::from("Incorrect operands."),
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
        objects::get_contents,
        operations::init::init,
        utils::{hash::sha2, test_dir::make_test_dir},
    };
    use std::{
        fs::{create_dir_all, File},
        io::Error,
    };

    #[test]
    fn not_in_vcs_dir() -> Result<(), Error> {
        let _test_dir = make_test_dir()?;
        let _ = File::create("test.txt");
        let test_args: Vec<String> = vec![
            "target/debug/vcs".to_string(),
            "add".to_string(),
            "test.txt".to_string(),
        ];
        assert_eq!("Not in an initialized vcs directory.", add(&test_args));
        Ok(())
    }

    #[test]
    fn incorrect_arg_number() -> Result<(), Error> {
        let _test_dir = make_test_dir()?;
        let _ = init(&vec!["target/debug/vcs".to_string(), "init".to_string()]);
        let _ = File::create("test.txt");
        let _ = File::create("test1.txt");
        let test_args: Vec<String> = vec![
            "target/debug/vcs".to_string(),
            "add".to_string(),
            "test.txt".to_string(),
            "test1.txt".to_string(),
        ];
        assert_eq!("Incorrect operands.", add(&test_args));
        Ok(())
    }

    #[test]
    fn file_does_not_exist() -> Result<(), Error> {
        let _test_dir = make_test_dir()?;
        let _ = init(&vec!["target/debug/vcs".to_string(), "init".to_string()]);
        let test_args: Vec<String> = vec![
            "target/debug/vcs".to_string(),
            "add".to_string(),
            "test.txt".to_string(),
        ];
        assert_eq!("File does not exist.", add(&test_args));
        Ok(())
    }

    #[test]
    fn correct_add_operation() -> Result<(), Error> {
        let _test_dir = make_test_dir()?;
        let _ = init(&vec!["target/debug/vcs".to_string(), "init".to_string()]);
        let _ = File::create("test.txt");
        let test_args: Vec<String> = vec![
            "target/debug/vcs".to_string(),
            "add".to_string(),
            "test.txt".to_string(),
        ];

        // Console output check
        assert_eq!("", add(&test_args));

        // Mutation of vcs dir check
        let empty_string_hash = sha2("");
        assert_eq!("", get_contents(&empty_string_hash));
        let index_contents = get_file_contents(".vcs/index");
        assert_eq!(
            format!("blob {} test.txt\n", empty_string_hash),
            index_contents
        );

        // Subdirectory
        let _ = create_dir_all("test_dir1/test_dir2/");
        let mut file = File::create("test_dir1/test_dir2/test.txt")?;
        let test_args: Vec<String> = vec![
            "target/debug/vcs".to_string(),
            "add".to_string(),
            "test_dir1/test_dir2/test.txt".to_string(),
        ];
        let file_text = "Test subdirectories!";
        let text_hash = sha2(file_text);
        let _ = file.write(file_text.as_bytes());
        assert_eq!("", add(&test_args));
        assert_eq!(file_text, get_contents(&text_hash));
        let index_contents = get_file_contents(".vcs/index");
        assert_eq!(
            format!(
                "blob {} test.txt\nblob {} test_dir1/test_dir2/test.txt\n",
                empty_string_hash, text_hash
            ),
            index_contents
        );
        Ok(())
    }

    #[test]
    fn same_as_commit_version() -> Result<(), Error> {
        panic!("Add behavior after commit has been added");
    }

    #[test]
    fn undoes_remove() -> Result<(), Error> {
        panic!("Add behavior after rm has been added")
    }
}
