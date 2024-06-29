use std::{
    env::set_current_dir,
    fs::{create_dir, File},
    io::{Error, Write},
};

use crate::{
    objects::{commit::write_commit, tree::write_tree},
    utils::fs_utils::directory_exists,
};

/// Executes `vcs init` with `args` as arguments
///
/// If there are no arguments, executes `vcs init` in the current directory.
/// If there is one argument, executes `vcs init` in the directory named by the argument.
/// In the directory `init` is working in, creates the `.vcs` directory with `HEAD`, `branches`,
///     `objects` subfolders
/// If already in a vcs directory, logs `Already in a vcs directory.` to the console.
///
/// * `args` - arguments `init` was called with
pub fn init(args: &Vec<String>) -> String {
    match args.len() {
        2 => {
            if directory_exists(".vcs") {
                return String::from("Already in a vcs directory.");
            }
            let _ = create_empty_vcs_dir();
            return String::from("");
        }
        3 => {
            if !directory_exists(&args[2]) {
                let _ = create_dir(&args[2]);
            }

            if directory_exists(".vcs") {
                return String::from("Already in a vcs directory.");
            }

            let _ = set_current_dir(&args[2]);
            let _ = create_empty_vcs_dir();
            let _ = set_current_dir("..");
            return String::from("");
        }
        _ => String::from("Incorrect number of arguments. Expected 0 or 1 arguments."),
    }
}

/// Creates a commit with date Jan. 1, 1970, on branch `main`, and initial message `Initial commit`
///
/// Will throw error if not in a vcs directory
fn create_first_commit() -> String {
    assert!(directory_exists(".vcs"));
    let subtrees: Vec<(String, String)> = vec![];
    let subblobs: Vec<(String, String)> = vec![];
    let tree_hash = write_tree(&subtrees, &subblobs);
    write_commit("Initial commit", "No parent", 0, &tree_hash)
}

/// Assuming program is in the correct directory, create an empty `.vcs` directory
fn create_empty_vcs_dir() -> Result<(), Error> {
    let _ = create_dir(".vcs");
    let _ = create_dir(".vcs/objects");
    let _ = create_dir(".vcs/branches");
    let _ = File::create(".vcs/HEAD");
    let commit_hash = create_first_commit();
    let mut file = File::create(".vcs/branches/main")?;
    let _ = file.write_all(&commit_hash.as_bytes());

    Ok(())
}

#[cfg(test)]
mod tests {
    /*
     * Testing strategy for `init`:
     *      Partition on number of arguments: 0, 1, >1
     *      Partition on whether vcs was already initialized: yes, no
     */

    use super::*;
    use crate::utils::{fs_utils::file_exists, hash::sha2, test_dir::make_test_dir};
    use std::{env::set_current_dir, io::Read};

    #[test]
    fn more_than_one_argument() {
        let _test_dir = Result::expect(make_test_dir(), "Expected TestDir");
        let test_args: Vec<String> = vec![
            "target/debug/vcs".to_string(),
            "init".to_string(),
            "arg1".to_string(),
            "arg2".to_string(),
        ];
        assert_eq!(
            "Incorrect number of arguments. Expected 0 or 1 arguments.",
            init(&test_args)
        );
    }

    #[test]
    fn zero_arguments_not_in_vcs_dir() {
        let _test_dir = Result::expect(make_test_dir(), "Expected TestDir");

        let test_args: Vec<String> = vec!["target/debug/vcs".to_string(), "init".to_string()];

        assert_eq!("", init(&test_args));
        check_empty_vcs_directory_exists();

        assert_eq!("Already in a vcs directory.", init(&test_args));
    }

    #[test]
    fn one_argument_not_in_vcs_dir() {
        let _test_dir = Result::expect(make_test_dir(), "Expected TestDir");

        let test_args: Vec<String> = vec![
            "target/debug/vcs".to_string(),
            "init".to_string(),
            "test_dir".to_string(),
        ];

        assert_eq!("", init(&test_args));
        let _ = set_current_dir("test_dir");
        check_empty_vcs_directory_exists();
    }

    fn check_empty_vcs_directory_exists() {
        assert!(directory_exists(".vcs"));
        assert!(directory_exists(".vcs/branches"));
        assert!(directory_exists(".vcs/objects"));
        assert!(file_exists(".vcs/HEAD"));
        let empty_tree = "Trees\nBlobs";
        let empty_tree_hash = sha2(empty_tree);
        let tree_path = format!(
            ".vcs/objects/{}/{}",
            empty_tree_hash[0..2].to_string(),
            empty_tree_hash[2..].to_string()
        );
        assert!(file_exists(&tree_path));

        let first_commit = format!(
            "Message\nInitial commit\nParent\nNo parent\nTime\n0\nTree Hash\n{}",
            empty_tree_hash
        );
        let first_commit_hash = sha2(&first_commit);
        let tree_path = format!(
            ".vcs/objects/{}/{}",
            first_commit_hash[0..2].to_string(),
            first_commit_hash[2..].to_string()
        );
        assert!(file_exists(&tree_path));

        assert!(file_exists(".vcs/branches/main"));
        let mut file = Result::expect(File::open(".vcs/branches/main"), "");
        let mut contents = String::new();
        let _ = file.read_to_string(&mut contents);
        assert_eq!(first_commit_hash, contents);
    }
}
