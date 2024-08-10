use std::{
    fs::{remove_file, File},
    io::{Result, Write},
};

use crate::{
    objects::commit::{get_hash_in_commit, get_head_commit},
    utils::fs_utils::{clear_file_contents, directory_exists, get_file_contents},
};

/// Executes `vcs rm` with `args` as arguments. Returns the string that should be logged to the
/// console.
///
/// If there is one argument, stage the file for removal. If the file is tracked in the current
///     commit, stage it for removal and remove the file from the working directory if the user
///     has not already done so (do not remove it unless it is tracked in the current commit).
/// If not in a vcs directory, log `Not in an initialized vcs directory.`
/// If incorrect number of commands, log `Incorrect operands.`
/// If file is neither staged nor tracked by the head commit, log `No reason to remove the file.`
/// Explicitly, this function either removes a file from the index if the file was previously
///     staged, or removes the file from the directory and adds a line in the index file to remove
///     the file on the next commit.
///
/// * `args` - arguments `rm` was called with
pub fn rm(args: &Vec<String>) -> Result<String> {
    if !directory_exists(".vcs") {
        return Ok(String::from("Not in an initialized vcs directory."));
    }
    match args.len() {
        3 => {
            let prev_commit_hash_of_file = get_hash_in_commit(&get_head_commit()?, &args[2])?;
            let seen_file = remove_from_index(&args[2])?;
            if prev_commit_hash_of_file == "DNE" {
                if seen_file {
                    return Ok(String::from(""));
                } else {
                    return Ok(String::from("No reason to remove the file."));
                }
            } else {
                remove_file(&args[2])?;
                return Ok(String::from(""));
            }
        }
        _ => Ok(String::from("Incorrect operands.")),
    }
}

/// Removes a line from the `.vcs/index`
///
/// Returns true iff the file existed in the index file
fn remove_from_index(rm_filename: &str) -> Result<bool> {
    let mut seen_file = false;
    let mut new_index: Vec<String> = vec![];
    for line in get_file_contents(".vcs/index")?.lines() {
        let line_split = line.split(' ').collect::<Vec<&str>>();
        match line_split[0] {
            "rm" => {
                if line_split[1] != rm_filename {
                    new_index.push(line.to_string());
                } else {
                    panic!("Should not be removing {} twice in a row", rm_filename);
                }
            }
            "blob" => {
                let line_filename = line_split[2];
                if rm_filename == line_filename {
                    seen_file = true;
                    continue;
                }
                new_index.push(line.to_string());
            }
            _ => panic!("Expected either `rm` or `blob`, but got {}", line_split[0]),
        }
    }
    if !seen_file {
        new_index.push(format!("rm {}", rm_filename));
    }
    clear_file_contents(".vcs/index")?;
    let mut index_file = File::create(".vcs/index")?;
    let index_contents = new_index.join("\n");
    index_file.write_all(&index_contents.into_bytes())?;
    Ok(seen_file)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        operations::{add::add, commit::commit, init::init},
        utils::{
            fs_utils::{file_exists, get_file_contents},
            test_dir::make_test_dir,
        },
    };
    use std::{fs::File, io::Write};

    // Partitions for rm
    //      Failure cases: Not in directory, incorrect operands, file is neither staged nor tracked
    //          by head commit, correct
    //      If correct: File trac
    #[test]
    fn not_in_vcs_dir() -> Result<()> {
        let _test_dir = make_test_dir()?;
        let _ = File::create("test.txt");
        let test_args: Vec<String> = vec![
            String::from("target/debug/vcs"),
            String::from("rm"),
            String::from("test.txt"),
        ];
        assert_eq!("Not in an initialized vcs directory.", rm(&test_args)?);
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

        // rm test
        let test_args: Vec<String> = vec![String::from("target/debug/vcs"), String::from("rm")];
        assert_eq!("Incorrect operands.", rm(&test_args)?);
        let test_args: Vec<String> = vec![
            String::from("target/debug/vcs"),
            String::from("rm"),
            String::from("burn_arg 1"),
            String::from("burn_arg 2"),
        ];
        assert_eq!("Incorrect operands.", rm(&test_args)?);
        Ok(())
    }

    #[test]
    fn file_not_tracked_or_staged() -> Result<()> {
        // Setup
        let _test_dir = make_test_dir();
        let _ = init(&vec![
            String::from("target/debug/vcs"),
            String::from("init"),
        ]);
        let _ = File::create("test.txt");

        // rm test
        let test_args: Vec<String> = vec![
            String::from("target/debug/vcs"),
            String::from("rm"),
            String::from("test.txt"),
        ];
        assert_eq!("No reason to remove the file.", rm(&test_args)?);
        Ok(())
    }

    #[test]
    fn correct_file_tracked_by_head_commit() -> Result<()> {
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

        // Commit the file
        let (_, _) = commit(&vec![
            String::from("target/debug/vcs"),
            String::from("commit"),
            String::from("Add test.txt"),
        ])?;

        // Test rm
        let test_args = vec![
            String::from("target/debug/vcs"),
            String::from("rm"),
            String::from("test.txt"),
        ];
        assert_eq!("", rm(&test_args)?);
        assert!(!file_exists("test.txt"));
        let index_contents = get_file_contents(".vcs/index")?;
        assert_eq!("rm test.txt", index_contents);

        // Commit the remove
        let (_, commit_hash) = commit(&vec![
            String::from("target/debug/vcs"),
            String::from("commit"),
            String::from("Remove test.txt"),
        ])?;
        assert_eq!(get_hash_in_commit(&commit_hash, "test.txt")?, "DNE");

        Ok(())
    }

    #[test]
    fn correct_file_not_tracked_by_head_commit() -> Result<()> {
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

        // Test rm
        let test_args = vec![
            String::from("target/debug/vcs"),
            String::from("rm"),
            String::from("test.txt"),
        ];
        assert_eq!("", rm(&test_args)?);
        assert!(file_exists("test.txt"));
        let index_contents = get_file_contents(".vcs/index")?;
        assert_eq!("", index_contents);
        Ok(())
    }
}
