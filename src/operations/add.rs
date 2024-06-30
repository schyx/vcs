use std::fs::File;
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
    "Not Implemented Yet".to_string()
}

#[cfg(test)]
pub mod tests {

    // Partitions for add
    // Partition on error condition:
    //      Not in VCS dir, incorrect number of operands, file doesn't exist, no error

    use super::*;
    use crate::{
        objects::get_contents,
        operations::init::init,
        utils::{hash::sha2, test_dir::make_test_dir},
    };
    use std::io::{Error, Read};

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
        let mut index_contents = String::new();
        let mut index_file = File::open(".vcs/index")?;
        let _ = index_file.read_to_string(&mut index_contents);
        assert_eq!(
            format!("blob {} test.txt", empty_string_hash),
            index_contents
        );
        Ok(())
    }
}
