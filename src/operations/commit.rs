use std::io::Error;

use chrono::Utc;

use crate::{
    objects::{
        commit::{get_head_commit, write_commit},
        get_object_contents,
        tree::write_tree,
    },
    utils::fs_utils::{
        clear_file_contents, directory_exists, get_file_contents, get_line_in_object,
    },
};

/// Executes `vcs commit`. Returns the string that is logged to the console, and the hash of the
/// commit object as well
///
/// If not in an initialized vcs directory, log `Not in an initialized vcs directory.`
/// If incorrect number of commands, log `Incorrect operands.`
/// If there was no commit message, log `Please enter a commit message.`
/// If there are no added files, log `No changes added to the commit`
/// If correct, we will update the current head/branch to point at the new commit, logging
/// information about time and author as well.
///
/// * `args` - arguments `commit` was called with
pub fn commit(args: &Vec<String>) -> Result<(String, String), Error> {
    if !directory_exists(".vcs") {
        return Ok((
            String::from("Not in an initialized vcs directory."),
            String::from(""),
        ));
    }
    match args.len() {
        2 => Ok((
            String::from("Please enter a commit message."),
            String::from(""),
        )),
        3 => {
            if args[2] == "" {
                return Ok((
                    String::from("Please enter a commit message."),
                    String::from(""),
                ));
            }

            let index_contents = get_file_contents(".vcs/index")?;
            if index_contents == "" {
                return Ok((
                    String::from("No changes added to the commit"),
                    String::from(""),
                ));
            }

            let parent = &get_line_in_object(&get_head_commit()?, 7)?;
            let parent_contents = get_object_contents(parent)?;
            let mut parent_lines: Vec<String> = parent_contents
                .split('\n')
                .filter(|line| *line != "Blobs" && *line != "Trees")
                .map(str::to_string)
                .collect();
            for change in index_contents.split('\n') {
                if change == "" {
                    break;
                }
                let split_change: Vec<&str> = change.split(' ').collect();
                match split_change[0] {
                    "blob" => {
                        let hash = split_change[1];
                        let filename = split_change[2];
                        let new_line = format!("{}: {}", filename, hash);
                        parent_lines.push(new_line);
                    }
                    _ => {
                        println!("{}", change);
                        panic!("Not implemented yet!")
                    }
                }
            }
            parent_lines.sort();
            let new_tree_hash = write_tree(&vec![], &parent_lines);
            let message = &args[2];
            let parent = &get_head_commit()?;
            let time = Utc::now().timestamp();
            let new_commit_hash = write_commit(message, parent, time, &new_tree_hash);
            let _ = clear_file_contents(".vcs/index");
            Ok((String::from(""), new_commit_hash))
        }
        _ => Ok((String::from("Incorrect operands."), String::from(""))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        objects::{commit::INITIAL_COMMIT_HASH, object_exists},
        operations::{add::add, init::init},
        utils::{hash::sha2, test_dir::make_test_dir},
    };
    use std::{
        fs::File,
        io::{Error, Write},
    };

    // Partitions for commit
    //      Failure cases: Not in directory, incorrect operands, no commit message, no changes or
    //          changes not added, correct
    //      If correct: Just adds, just removes, adds and removes
    #[test]
    fn not_in_vcs_dir() -> Result<(), Error> {
        let _test_dir = make_test_dir()?;
        let _ = File::create("test.txt");
        let test_args: Vec<String> = vec![
            String::from("target/debug/vcs"),
            String::from("commit"),
            String::from("\"commit test.txt\""),
        ];
        assert_eq!(
            "Not in an initialized vcs directory.",
            commit(&test_args)?.0
        );
        Ok(())
    }

    #[test]
    fn incorrect_operands() -> Result<(), Error> {
        // Setup
        let _test_dir = make_test_dir();
        let _ = init(&vec![
            String::from("target/debug/vcs"),
            String::from("init"),
        ]);
        let _ = File::create("test.txt");
        let _ = add(&vec![
            String::from("target/debug/vcs"),
            String::from("add"),
            String::from("test.txt"),
        ]);

        // Commit test
        let test_args: Vec<String> = vec![
            String::from("target/debug/vcs"),
            String::from("commit"),
            String::from("burn_arg1"),
            String::from("burn_arg2"),
        ];
        assert_eq!("Incorrect operands.", commit(&test_args)?.0);
        Ok(())
    }

    #[test]
    fn no_commit_message() -> Result<(), Error> {
        // Setup
        let _test_dir = make_test_dir();
        let _ = init(&vec![
            String::from("target/debug/vcs"),
            String::from("init"),
        ]);
        let _ = File::create("test.txt");
        let _ = add(&vec![
            String::from("target/debug/vcs"),
            String::from("add"),
            String::from("test.txt"),
        ]);

        // Commit test
        let test_args: Vec<String> = vec![String::from("target/debug/vcs"), String::from("commit")];
        assert_eq!("Please enter a commit message.", commit(&test_args)?.0);
        Ok(())
    }

    #[test]
    fn empty_commit_message() -> Result<(), Error> {
        // Setup
        let _test_dir = make_test_dir();
        let _ = init(&vec![
            String::from("target/debug/vcs"),
            String::from("init"),
        ]);
        let _ = File::create("test.txt");
        let _ = add(&vec![
            String::from("target/debug/vcs"),
            String::from("add"),
            String::from("test.txt"),
        ]);

        // Commit test
        let test_args: Vec<String> = vec![
            String::from("target/debug/vcs"),
            String::from("commit"),
            String::from(""),
        ];
        assert_eq!("Please enter a commit message.", commit(&test_args)?.0);
        Ok(())
    }

    #[test]
    fn no_changes() -> Result<(), Error> {
        // Setup
        let _test_dir = make_test_dir();
        let _ = init(&vec![
            String::from("target/debug/vcs"),
            String::from("init"),
        ]);
        let _ = File::create("test.txt");

        // Commit test
        let test_args: Vec<String> = vec![
            String::from("target/debug/vcs"),
            String::from("commit"),
            String::from("Add test.txt"),
        ];
        assert_eq!("No changes added to the commit", commit(&test_args)?.0);
        Ok(())
    }

    #[test]
    fn correct_just_adds() -> Result<(), Error> {
        let _test_dir = make_test_dir()?;

        // init vcs dir
        let _ = init(&vec![
            String::from("target/debug/vcs"),
            String::from("init"),
        ]);

        // mutate and add a file
        let mut file = File::create("test.txt")?;
        let file_text = "commit time!";
        let _ = file.write(file_text.as_bytes());
        let (_, file_hash) = add(&vec![
            String::from("target/debug/vcs"),
            String::from("add"),
            String::from("test.txt"),
        ])?;

        // test commmit
        let test_args: Vec<String> = vec![
            String::from("target/debug/vcs"),
            String::from("commit"),
            String::from("Add test.txt"),
        ];
        let (output_text, commit_hash) = commit(&test_args)?;
        assert_eq!("", output_text);
        let time = Utc::now().timestamp();
        let tree_text = format!("Trees\nBlobs\ntest.txt: {}", file_hash);
        let tree_hash = sha2(&tree_text);
        assert!(object_exists(&tree_hash));
        let commit_string = format!(
            "Message\n{}\nParent\n{}\nTime\n{}\nTree Hash\n{}",
            "Add test.txt",
            INITIAL_COMMIT_HASH,
            time.to_string(),
            tree_hash
        );
        assert_eq!(sha2(&commit_string), commit_hash);
        let index_contents_after_commit = get_file_contents(".vcs/index")?;
        assert_eq!(index_contents_after_commit, "");
        Ok(())
    }

    #[test]
    fn just_remove() -> Result<(), Error> {
        panic!("Not implemented");
    }

    #[test]
    fn add_and_remove() -> Result<(), Error> {
        panic!("Not implemented");
    }
}
