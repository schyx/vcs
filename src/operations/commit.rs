use std::{
    collections::HashMap,
    fs::File,
    io::{Result, Write},
};

use chrono::Utc;

use crate::{
    objects::{
        commit::{get_commit_tree, get_head_commit, write_commit},
        get_object_contents,
        tree::write_tree,
    },
    utils::fs_utils::{clear_file_contents, directory_exists, file_exists, get_file_contents},
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
pub fn commit(args: &Vec<String>) -> Result<(String, String)> {
    assert!(args[1] == "commit");
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

            let parent = &get_commit_tree(&get_head_commit()?)?;
            let parent_contents: Vec<String> = get_object_contents(parent)?
                .split('\n')
                .filter(|line| *line != "Blobs" && *line != "Trees")
                .map(str::to_string)
                .collect();
            let mut parent_lines: HashMap<String, String> = HashMap::new();
            for line in parent_contents {
                if let Some((object_name, object_hash)) = line.split_once(": ") {
                    parent_lines.insert(object_name.to_string(), object_hash.to_string());
                }
            }
            for change in index_contents.split('\n') {
                let split_change: Vec<&str> = change.split(' ').collect();
                match split_change[0] {
                    "blob" => {
                        let hash = split_change[1];
                        let filename = split_change[2];
                        parent_lines.insert(filename.to_string(), hash.to_string());
                    }
                    "rm" => {
                        let filename = split_change[1];
                        parent_lines.remove(filename);
                    }
                    _ => {
                        panic!(
                            "Expected change to be either `rm` or `blob`, got {}.",
                            split_change[0]
                        );
                    }
                }
            }
            let mut parent_contents: Vec<String> = vec![];
            for (object_name, object_hash) in &parent_lines {
                parent_contents.push(format!("{}: {}", object_name, object_hash));
            }
            parent_contents.sort();
            let new_tree_hash = write_tree(&vec![], &parent_contents);
            let message = &args[2];
            let parent = &get_head_commit()?;
            let time = Utc::now().timestamp();
            let new_commit_hash = write_commit(message, parent, time, &new_tree_hash);
            update_head(new_commit_hash.clone())?;
            let _ = clear_file_contents(".vcs/index");
            Ok((String::from(""), new_commit_hash))
        }
        _ => Ok((String::from("Incorrect operands."), String::from(""))),
    }
}

/// Updates the commit that the current branch is pointing at.
///
/// Will throw an error if the current checked out commit is not on a branch
fn update_head(commit_hash: String) -> Result<()> {
    let head = get_file_contents(".vcs/HEAD")?;
    let branch_file_name = format!(".vcs/branches/{}", head);
    assert!(file_exists(&branch_file_name));
    clear_file_contents(&branch_file_name)?;
    let mut branch_file = File::create(branch_file_name)?;
    branch_file.write_all(&commit_hash.into_bytes())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        objects::{
            commit::{get_commit_tree, get_hash_in_commit, INITIAL_COMMIT_HASH},
            object_exists,
        },
        operations::{add::add, init::init, rm::rm},
        utils::{hash::sha2, test_dir::make_test_dir},
    };
    use std::{fs::File, io::Write};

    // Partitions for commit
    //      Failure cases: Not in directory, incorrect operands, no commit message, no changes or
    //          changes not added, correct
    //      If correct: Just adds, just removes, adds and removes
    #[test]
    fn not_in_vcs_dir() -> Result<()> {
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
    fn incorrect_operands() -> Result<()> {
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
    fn no_commit_message() -> Result<()> {
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
    fn empty_commit_message() -> Result<()> {
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
    fn no_changes() -> Result<()> {
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
    fn correct_just_adds() -> Result<()> {
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
            "Parent\n{}\nTime\n{}\nTree Hash\n{}\nMessage\n{}",
            INITIAL_COMMIT_HASH,
            time.to_string(),
            tree_hash,
            "Add test.txt",
        );
        assert_eq!(sha2(&commit_string), commit_hash);
        let index_contents_after_commit = get_file_contents(".vcs/index")?;
        assert_eq!(index_contents_after_commit, "");
        assert_eq!(commit_hash, get_head_commit()?);
        Ok(())
    }

    #[test]
    fn just_remove() -> Result<()> {
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
        let (_, _) = add(&vec![
            String::from("target/debug/vcs"),
            String::from("add"),
            String::from("test.txt"),
        ])?;
        let mut file = File::create("test2.txt")?;
        let file_text = "";
        let _ = file.write(file_text.as_bytes());
        let (_, _) = add(&vec![
            String::from("target/debug/vcs"),
            String::from("add"),
            String::from("test2.txt"),
        ])?;

        // Commit
        let (_, _) = commit(&vec![
            String::from("target/debug/vcs"),
            String::from("commit"),
            String::from("Add test.txt and test2.txt"),
        ])?;

        let rm_text = rm(&vec![
            String::from("target/debug/vcs"),
            String::from("rm"),
            String::from("test.txt"),
        ])?;
        assert_eq!("", rm_text);
        let _ = rm(&vec![
            String::from("target/debug/vcs"),
            String::from("rm"),
            String::from("test2.txt"),
        ])?;
        assert!(!file_exists("test.txt"));
        assert!(!file_exists("test2.txt"));
        let (_, _) = commit(&vec![
            String::from("target/debug/vcs"),
            String::from("commit"),
            String::from("Remove test.txt and test2.txt"),
        ])?;
        let head_commit = get_head_commit()?;
        assert_eq!("DNE", get_hash_in_commit(&head_commit, "test2.txt")?);
        assert_eq!("DNE", get_hash_in_commit(&head_commit, "test.txt")?);

        Ok(())
    }

    #[test]
    fn add_and_remove() -> Result<()> {
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
        let (_, _) = add(&vec![
            String::from("target/debug/vcs"),
            String::from("add"),
            String::from("test.txt"),
        ])?;

        // Commit
        let (_, _) = commit(&vec![
            String::from("target/debug/vcs"),
            String::from("commit"),
            String::from("Add test.txt"),
        ])?;

        let _ = rm(&vec![
            String::from("target/debug/vcs"),
            String::from("rm"),
            String::from("test.txt"),
        ])?;
        let mut file = File::create("test2.txt")?;
        let file_text = "";
        let _ = file.write(file_text.as_bytes());
        let (_, _) = add(&vec![
            String::from("target/debug/vcs"),
            String::from("add"),
            String::from("test2.txt"),
        ])?;
        let (_, commit_hash) = commit(&vec![
            String::from("target/debug/vcs"),
            String::from("commit"),
            String::from("Add test2 and remove test1"),
        ])?;
        assert!(!file_exists("test.txt"));
        let expected_tree = format!("Trees\nBlobs\ntest2.txt: {}", sha2("blob\n"));
        let tree_hash = get_commit_tree(&commit_hash)?;
        assert_eq!(expected_tree, get_object_contents(&tree_hash)?);

        Ok(())
    }
}
