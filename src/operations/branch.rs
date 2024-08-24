use std::{
    fs::{read_dir, remove_file, File},
    io::{Result, Write},
};

use crate::{
    objects::{commit::get_head_commit, get_branch_name},
    utils::fs_utils::{directory_exists, file_exists, no_dir_string},
};

/// Executes `vcs branch` with `args` as arguments. returns the string that should be logged to the
/// console.
///
/// There are three possible uses of this function:
///     1. `vcs branch`: Lists the branches in alphabetical order, with a * to the right of the
///        current branch
///     2. `vcs branch <BRANCH_NAME>`: Creates a new branch with name <BRANCH_NAME>. Will log `A
///        branch named <BRANCH_NAME> already exist.` if trying to create a new branch of the same
///        name.
///     3. `vcs branch -d <BRANCH_NAME>`: Deletes the branch named <BRANCH_NAME>. Will log `Deleted
///        branch <BRANCH_NAME>.` if successful, `Branch <BRANCH_NAME> was not found.` if the
///        requested branch doesn't exist, and `Cannot delete branch <BRANCH_NAME>. Switch to a
///        different branch to delete.` if on the same branch as the one requested for deletion.
///
/// If there are an incorrect number of arguments, log `Incorrect operands.`, and if not in an
/// initialized vcs directory, log `Not in an initialized vcs directory.`.
pub fn branch(args: &Vec<String>) -> Result<String> {
    if !directory_exists(".vcs") {
        return Ok(String::from("Not in an initialized vcs directory."));
    }
    assert_eq!(args[1], "branch");
    match args.len() {
        2 => {
            let mut branches: Vec<String> = vec![];
            let current_branch = get_branch_name()?;
            for entry in read_dir(".vcs/branches")? {
                let entry = entry?;
                let path = entry.path();
                assert!(path.is_file());
                let branchname = no_dir_string(path);
                if branchname == current_branch {
                    branches.push(format!("{} *", branchname));
                } else {
                    branches.push(branchname);
                }
            }
            branches.sort();
            return Ok(branches.join("\n"));
        }
        3 => {
            let new_branchname = &args[2];
            let filename = format!(".vcs/branches/{}", new_branchname);
            if file_exists(&filename) {
                return Ok(format!("A branch named {} already exists.", new_branchname));
            }
            let mut file = File::create(filename)?;
            let current_commit = get_head_commit()?;
            file.write_all(current_commit.as_bytes())?;
            return Ok(String::from(""));
        }
        4 => {
            let filename = format!(".vcs/branches/{}", args[3]);
            if args[2] != "-d" {
                return Ok(String::from("Incorrect operands"));
            } else if args[3] == get_branch_name()? {
                return Ok(format!(
                    "Cannot delete branch {}. Switch to a different branch to delete.",
                    args[3]
                ));
            } else if !file_exists(&filename) {
                return Ok(format!("Branch {} was not found.", args[3]));
            }
            remove_file(format!(".vcs/branches/{}", args[3]))?;
            return Ok(format!("Deleted branch {}.", args[3]));
        }
        _ => return Ok(String::from("Incorrect operands.")),
    }
}

#[cfg(test)]
pub mod tests {

    // Partitions for branch
    // Partition on which of three:
    //      Error not in vcs, error incorrect operands, list branches, create branch, delete
    //      branch.
    //  Further partition on number of branches to list: 1, >1
    //  Further partition on creation of branches: no error, one already exists
    //  Further partition on deleting branches: no error, doesn't exist, same branch

    use super::*;

    use crate::{
        objects::commit::{get_head_commit, INITIAL_COMMIT_HASH},
        operations::init::init,
        utils::{
            fs_utils::{file_exists, get_file_contents},
            test_dir::make_test_dir,
        },
    };

    #[test]
    fn not_in_vcs_dir() -> Result<()> {
        let _test_dir = make_test_dir()?;
        let test_args: Vec<String> = vec![String::from("target/debug/vcs"), String::from("branch")];
        assert_eq!("Not in an initialized vcs directory.", branch(&test_args)?);
        Ok(())
    }

    #[test]
    pub fn incorrect_arguments() -> Result<()> {
        let _test_dir = make_test_dir()?;
        let _ = init(&vec![
            String::from("target/debug/vcs"),
            String::from("init"),
        ]);
        assert_eq!(
            "Incorrect operands.",
            branch(&vec![
                String::from("target/debug/vcs"),
                String::from("branch"),
                String::from("b"),
                String::from("c"),
                String::from("a"),
            ])?
        );
        Ok(())
    }

    #[test]
    pub fn list_one_branch() -> Result<()> {
        let _test_dir = make_test_dir()?;
        let _ = init(&vec![
            String::from("target/debug/vcs"),
            String::from("init"),
        ]);
        assert_eq!(
            "main *",
            branch(&vec![
                String::from("target/debug/vcs"),
                String::from("branch"),
            ])?
        );
        Ok(())
    }

    #[test]
    pub fn list_many_branches() -> Result<()> {
        let _test_dir = make_test_dir()?;
        let _ = init(&vec![
            String::from("target/debug/vcs"),
            String::from("init"),
        ]);
        assert_eq!(
            "",
            branch(&vec![
                String::from("target/debug/vcs"),
                String::from("branch"),
                String::from("test_branch"),
            ])?
        );
        assert_eq!(
            "main *\ntest_branch",
            branch(&vec![
                String::from("target/debug/vcs"),
                String::from("branch"),
            ])?
        );
        Ok(())
    }

    #[test]
    pub fn test_branch_creation() -> Result<()> {
        let _test_dir = make_test_dir()?;
        let _ = init(&vec![
            String::from("target/debug/vcs"),
            String::from("init"),
        ]);
        assert_eq!(
            "",
            branch(&vec![
                String::from("target/debug/vcs"),
                String::from("branch"),
                String::from("test_branch"),
            ])?
        );
        assert!(file_exists(".vcs/branches/test_branch"));
        assert_eq!("main", get_branch_name()?);
        assert_eq!(
            INITIAL_COMMIT_HASH,
            get_file_contents(".vcs/branches/test_branch")?
        );
        Ok(())
    }

    #[test]
    pub fn test_creating_already_existing_branch() -> Result<()> {
        let _test_dir = make_test_dir()?;
        let _ = init(&vec![
            String::from("target/debug/vcs"),
            String::from("init"),
        ]);
        let _ = branch(&vec![
            String::from("target/debug/vcs"),
            String::from("branch"),
            String::from("test_branch"),
        ])?;
        assert_eq!(
            "A branch named test_branch already exists.",
            branch(&vec![
                String::from("target/debug/vcs"),
                String::from("branch"),
                String::from("test_branch"),
            ])?
        );
        Ok(())
    }

    #[test]
    pub fn test_delete_branch() -> Result<()> {
        let _test_dir = make_test_dir()?;
        let _ = init(&vec![
            String::from("target/debug/vcs"),
            String::from("init"),
        ]);
        let _ = branch(&vec![
            String::from("target/debug/vcs"),
            String::from("branch"),
            String::from("test_branch"),
        ])?;
        assert_eq!(
            "Deleted branch test_branch.",
            branch(&vec![
                String::from("target/debug/vcs"),
                String::from("branch"),
                String::from("-d"),
                String::from("test_branch"),
            ])?
        );
        assert!(!file_exists(".vcs/branches/test_branch"));
        assert_eq!(INITIAL_COMMIT_HASH, get_head_commit()?);
        Ok(())
    }

    #[test]
    pub fn test_delete_nonexistent_branch() -> Result<()> {
        let _test_dir = make_test_dir()?;
        let _ = init(&vec![
            String::from("target/debug/vcs"),
            String::from("init"),
        ]);
        assert_eq!(
            "Branch test_branch was not found.",
            branch(&vec![
                String::from("target/debug/vcs"),
                String::from("branch"),
                String::from("-d"),
                String::from("test_branch"),
            ])?
        );
        Ok(())
    }

    #[test]
    pub fn test_delete_current_branch() -> Result<()> {
        let _test_dir = make_test_dir()?;
        let _ = init(&vec![
            String::from("target/debug/vcs"),
            String::from("init"),
        ]);
        assert_eq!(
            "Cannot delete branch main. Switch to a different branch to delete.",
            branch(&vec![
                String::from("target/debug/vcs"),
                String::from("branch"),
                String::from("-d"),
                String::from("main"),
            ])?
        );
        Ok(())
    }
}
