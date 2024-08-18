use std::{collections::HashMap, fs::read_dir, io::Result};

use crate::{
    objects::{
        blob::get_blob_hash,
        commit::{get_hash_in_commit, get_head_commit},
        get_branch_name,
    },
    utils::fs_utils::{directory_exists, no_dir_string, read_lines},
};

/// Executes `vcs log` with `args` as arguments
///
/// Outputs the current status of the directory. Specifically, will log a message of the following
/// form:
///     On branch <CURRENT_BRANCH_NAME>
///     Changes to be commited:
///         <modified/deleted/new file>: <filename>
///
///     Changes not staged for commit:
///         modified: <filename>
///
///     Untracked files:
///         <filename>
///
/// based on the current state of the vcs directory. Within each section, the entries will be
/// sorted alphabetically
/// Will log `Not in an initialized vcs directory.` if no vcs dir was found, and will log
/// `Incorrect operands.` if more than 1 argument was supplied.
pub fn status(args: &Vec<String>) -> Result<String> {
    if !directory_exists(".vcs") {
        return Ok(String::from("Not in an initialized vcs directory."));
    } else if args.len() != 2 {
        return Ok(String::from("Incorrect operands."));
    }
    let mut output: Vec<String> = vec![];

    // Branch name line
    let branch_name = get_branch_name()?;
    output.push(format!("On branch {}", branch_name));

    // Changes to be committed section
    let mut to_be_committed: Vec<String> = vec![];
    let mut files_to_hashes: HashMap<String, FileStatus> = HashMap::new();
    for line in read_lines(".vcs/index")?.flatten() {
        let split_line: Vec<&str> = line.split(" ").collect();
        match split_line[0] {
            "blob" => {
                let line_filename = split_line[2];
                files_to_hashes.insert(
                    line_filename.to_string(),
                    FileStatus::Modified(split_line[1].to_string()),
                );
                let commit_hash = get_hash_in_commit(&get_head_commit()?, line_filename)?;
                if commit_hash == "DNE" {
                    to_be_committed.push(format!("new file: {}", line_filename));
                } else {
                    to_be_committed.push(format!("modified: {}", line_filename));
                }
            }
            "rm" => {
                let line_filename = split_line[1];
                files_to_hashes.insert(line_filename.to_string(), FileStatus::Removed);
                to_be_committed.push(format!("deleted: {}", line_filename));
            }
            _ => {
                panic!("Expected either `blob` or `rm`. Got {}", split_line[0])
            }
        }
    }
    if to_be_committed.len() > 0 {
        to_be_committed.sort();
        output.push(format!(
            "Changes to be committed:\n\t{}\n",
            to_be_committed.join("\n\t")
        ));
    }

    // Unadded changes section
    let mut not_staged: Vec<String> = vec![];
    let mut untracked: Vec<String> = vec![];
    for entry in read_dir(".")? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            continue;
        }
        let filename = no_dir_string(path);
        let (current_file_hash, _) = get_blob_hash(&filename)?;
        if files_to_hashes.contains_key(&filename) {
            let status = files_to_hashes.get(&filename).unwrap();
            match status {
                FileStatus::Modified(staged_hash) => {
                    if *staged_hash != current_file_hash {
                        not_staged.push(format!("modified: {}", filename));
                    }
                }
                FileStatus::Removed => {
                    not_staged.push(format!("modified: {}", filename));
                }
            }
        } else {
            let prev_hash = get_hash_in_commit(&get_head_commit()?, &filename)?;
            if prev_hash == "DNE" {
                untracked.push(filename);
            } else if prev_hash != current_file_hash {
                not_staged.push(format!("modified: {}", filename));
            }
        }
    }
    if not_staged.len() > 0 {
        not_staged.sort();
        output.push(format!(
            "Changes not staged for commit:\n\t{}\n",
            not_staged.join("\n\t")
        ));
    }
    if untracked.len() > 0 {
        untracked.sort();
        output.push(format!("Untracked files:\n\t{}\n", untracked.join("\n\t")));
    }

    // Return logic
    if output.len() == 1 {
        // This means there's just the branch name
        output.push(String::from("nothing to commit\n"));
    }
    Ok(output.join("\n"))
}

/// Auxiliary enum to help with remembering status of file in index
enum FileStatus {
    Modified(String),
    Removed,
}

#[cfg(test)]
pub mod tests {

    // Partitions for status
    // Partition on error modes
    //      Not in vcs dir, too many operands, no error
    // On branch: main, not main
    // No changes at all, there exist changes
    // On changes to be committed: Empty, just modify, just delete, just new file, multiple mixed
    // On changes not staged for commit: there is file modified, there's not a file modified
    // On Untracked files: empty, nonempty

    use std::{
        env::set_current_dir,
        fs::{create_dir, File},
        io::Write,
    };

    use crate::{
        operations::{add::add, commit::commit, init::init, rm::rm},
        utils::{fs_utils::clear_file_contents, test_dir::make_test_dir},
    };

    use super::*;

    #[test]
    fn not_in_vcs_dir() -> Result<()> {
        let _test_dir = make_test_dir()?;
        create_dir("test_dir")?;
        set_current_dir("test_dir")?;
        let test_args: Vec<String> = vec![String::from("target/debug/vcs"), String::from("status")];
        assert_eq!("Not in an initialized vcs directory.", status(&test_args)?);
        Ok(())
    }

    #[test]
    fn incorrect_arg_number() -> Result<()> {
        let _test_dir = make_test_dir()?;
        create_dir("test_dir")?;
        set_current_dir("test_dir")?;
        let _ = init(&vec![
            String::from("target/debug/vcs"),
            String::from("init"),
        ]);
        let test_args: Vec<String> = vec![
            String::from("target/debug/vcs"),
            String::from("status"),
            String::from("test.txt"),
        ];
        assert_eq!("Incorrect operands.", status(&test_args)?);
        Ok(())
    }

    #[test]
    fn not_on_main() -> Result<()> {
        todo!("Implement after adding branching");
    }

    #[test]
    fn no_changes() -> Result<()> {
        let _test_dir = make_test_dir()?;
        create_dir("test_dir")?;
        set_current_dir("test_dir")?;
        let _ = init(&vec![
            String::from("target/debug/vcs"),
            String::from("init"),
        ]);
        assert_eq!(
            "On branch main\nnothing to commit\n",
            status(&vec![
                String::from("target/debug/vcs"),
                String::from("status")
            ])?
        );
        Ok(())
    }

    #[test]
    fn covers_empty_no_modification_nonempty() -> Result<()> {
        let _test_dir = make_test_dir()?;
        create_dir("test_dir")?;
        set_current_dir("test_dir")?;
        let _ = init(&vec![
            String::from("target/debug/vcs"),
            String::from("init"),
        ]);
        let _ = File::create("test.txt")?;
        assert_eq!(
            "On branch main\nUntracked files:\n\ttest.txt\n",
            status(&vec![
                String::from("target/debug/vcs"),
                String::from("status")
            ])?
        );
        Ok(())
    }

    #[test]
    fn covers_empty_modification_empty() -> Result<()> {
        let _test_dir = make_test_dir()?;
        create_dir("test_dir")?;
        set_current_dir("test_dir")?;
        let _ = init(&vec![
            String::from("target/debug/vcs"),
            String::from("init"),
        ]);
        let mut file = File::create("test.txt")?;
        let _ = add(&vec![
            String::from("target/debug/vcs"),
            String::from("add"),
            String::from("test.txt"),
        ])?;
        let _ = commit(&vec![
            String::from("target/debug/vcs"),
            String::from("commit"),
            String::from("Add test.txt"),
        ])?;
        file.write_all("Test modification".as_bytes())?;
        assert_eq!(
            "On branch main\nChanges not staged for commit:\n\tmodified: test.txt\n",
            status(&vec![
                String::from("target/debug/vcs"),
                String::from("status")
            ])?
        );
        Ok(())
    }

    #[test]
    fn covers_modified_no_empty() -> Result<()> {
        let _test_dir = make_test_dir()?;
        create_dir("test_dir")?;
        set_current_dir("test_dir")?;
        let _ = init(&vec![
            String::from("target/debug/vcs"),
            String::from("init"),
        ]);
        let mut file = File::create("test.txt")?;
        let _ = add(&vec![
            String::from("target/debug/vcs"),
            String::from("add"),
            String::from("test.txt"),
        ])?;
        let _ = commit(&vec![
            String::from("target/debug/vcs"),
            String::from("commit"),
            String::from("Add test.txt"),
        ])?;
        file.write_all("Test modification".as_bytes())?;
        let _ = add(&vec![
            String::from("target/debug/vcs"),
            String::from("add"),
            String::from("test.txt"),
        ])?;
        assert_eq!(
            "On branch main\nChanges to be committed:\n\tmodified: test.txt\n",
            status(&vec![
                String::from("target/debug/vcs"),
                String::from("status")
            ])?
        );
        Ok(())
    }

    #[test]
    fn covers_deleted_no_empty() -> Result<()> {
        let _test_dir = make_test_dir()?;
        create_dir("test_dir")?;
        set_current_dir("test_dir")?;
        let _ = init(&vec![
            String::from("target/debug/vcs"),
            String::from("init"),
        ]);
        let _ = File::create("test.txt")?;
        let _ = add(&vec![
            String::from("target/debug/vcs"),
            String::from("add"),
            String::from("test.txt"),
        ])?;
        let _ = commit(&vec![
            String::from("target/debug/vcs"),
            String::from("commit"),
            String::from("Add test.txt"),
        ])?;
        let _ = rm(&vec![
            String::from("target/debug/vcs"),
            String::from("rm"),
            String::from("test.txt"),
        ])?;
        assert_eq!(
            "On branch main\nChanges to be committed:\n\tdeleted: test.txt\n",
            status(&vec![
                String::from("target/debug/vcs"),
                String::from("status")
            ])?
        );
        Ok(())
    }

    #[test]
    fn covers_new_file_no_empty() -> Result<()> {
        let _test_dir = make_test_dir()?;
        create_dir("test_dir")?;
        set_current_dir("test_dir")?;
        let _ = init(&vec![
            String::from("target/debug/vcs"),
            String::from("init"),
        ]);
        let _ = File::create("test.txt")?;
        let _ = add(&vec![
            String::from("target/debug/vcs"),
            String::from("add"),
            String::from("test.txt"),
        ])?;
        assert_eq!(
            "On branch main\nChanges to be committed:\n\tnew file: test.txt\n",
            status(&vec![
                String::from("target/debug/vcs"),
                String::from("status")
            ])?
        );
        Ok(())
    }

    #[test]
    fn covers_mix_modification_nonempty() -> Result<()> {
        let _test_dir = make_test_dir()?;
        create_dir("test_dir")?;
        set_current_dir("test_dir")?;
        let _ = init(&vec![
            String::from("target/debug/vcs"),
            String::from("init"),
        ]);
        let _ = File::create("test.txt")?;
        let mut test2 = File::create("test2.txt")?;
        let mut test3 = File::create("test3.txt")?;
        let _ = add(&vec![
            String::from("target/debug/vcs"),
            String::from("add"),
            String::from("test2.txt"),
        ])?;
        let _ = add(&vec![
            String::from("target/debug/vcs"),
            String::from("add"),
            String::from("test.txt"),
        ])?;
        let _ = commit(&vec![
            String::from("target/debug/vcs"),
            String::from("commit"),
            String::from("Add test2.txt"),
        ]);
        test2.write_all("big test!".as_bytes())?;
        let _ = add(&vec![
            String::from("target/debug/vcs"),
            String::from("add"),
            String::from("test2.txt"),
        ])?;
        test3.write_all("big test for test3!".as_bytes())?;
        let _ = add(&vec![
            String::from("target/debug/vcs"),
            String::from("add"),
            String::from("test3.txt"),
        ])?;
        let _ = rm(&vec![
            String::from("target/debug/vcs"),
            String::from("rm"),
            String::from("test.txt"),
        ]);
        clear_file_contents("test2.txt")?;
        test2.write_all("big test for test 2!".as_bytes())?;
        let _ = File::create("test4.txt")?;
        assert_eq!(
            "On branch main\nChanges to be committed:\n\tdeleted: test.txt\n\tmodified: test2.txt\n\tnew file: test3.txt\n\nChanges not staged for commit:\n\tmodified: test2.txt\n\nUntracked files:\n\ttest4.txt\n",
            status(&vec![
                String::from("target/debug/vcs"),
                String::from("status")
            ])?
        );
        Ok(())
    }
}
