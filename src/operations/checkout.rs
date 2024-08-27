use std::{
    collections::HashSet,
    fs::{read_dir, remove_file, File},
    io::{Result, Write},
};

use crate::{
    objects::{
        commit::{get_commit_tree, get_hash_in_commit, get_head_commit},
        get_branch_name, get_object_contents, object_exists,
        tree::serialize_tree,
    },
    utils::{
        fs_utils::{
            clear_file_contents, directory_exists, file_exists, get_file_contents, no_dir_string,
        },
        hash::sha2,
    },
};

/// Executes `vcs checkout` with `args` as arguments. Returns the string taht should be locked to
/// the console.
///
/// There are three possible use cases of this function:
///     1. `vcs checkout <BRANCH_NAME>`: Takes all files in the commit at the head of the given
///         branch, and puts them in the working directory, overwriting the versions of the files
///         that are already there if they exist. Also, at the end of this command, the given
///         branch will now be considered the current branch (HEAD). Any files that are tracked in
///         the current branch but are not present in the checked-out branch are deleted. Logs the
///         text `Switched to branch <BRANCH_NAME>.` if it's a different branch. If the
///         branch to check out is the current branch, log `Already on <BRANCH_NAME>.` If the
///         branch does not exist, log `<BRANCH_NAME> does not exist.`
///     2. `vcs checkout [commit_id] -- <FILE_NAME>`: Takes the version of the file as it exists in
///         the commit with the given id, and puts it in the working directory, overwriting the
///         version of the file that’s already there if there is one. The new version of the file is
///         not staged. If the commit_id is omitted, use the version of the file from the head
///         commit instead. If the commit id doesn't exist, logs `No commit with ID <COMMIT_ID>
///         exists.`
///     3. `vcs checkout [commit_id]`: Takes all files in the commit specified, and puts them in
///        the working directory, overwriting the version of the files that are already there if
///        they exist. Any files not tracked in the commit will be deleted. Logs `Switched to
///        commit <COMMIT_ID>.` The new head will be detached, and any modications to the vcs
///        directory (via `add`, `rm`, or `commit`) will return the message `Currently in a detached
///        HEAD state, check out a branch to modify the directory.`
///
/// If there are an incorrect number of arguments, log `Incorrect operands.`, and if not in an
/// initialized vcs directory, log `Not in an initialized vcs directory.`.
pub fn checkout(args: &Vec<String>) -> Result<String> {
    assert!(args[1] == "checkout");
    if !directory_exists(".vcs") {
        return Ok(String::from("Not in an initialized vcs directory."));
    }
    match args.len() {
        3 => {
            if args[2] == get_branch_name()? {
                return Ok(format!("Already on {}.", args[2]));
            }
            if file_exists(&format!(".vcs/branches/{}", args[2])) {
                clear_file_contents(".vcs/HEAD")?;
                // Modify HEAD file
                let mut head_file = File::create(".vcs/HEAD")?;
                head_file.write_all(args[2].as_bytes())?;
                // // Modify directory state
                let commit_hash = get_file_contents(&format!(".vcs/branches/{}", args[2]))?;
                update_dir_state(commit_hash)?;
                return Ok(format!("Switched to branch {}.", args[2]));
            } else if object_exists(&args[2]) {
                clear_file_contents(".vcs/HEAD")?;
                // Modify HEAD file
                let mut head_file = File::create(".vcs/HEAD")?;
                head_file.write_all(args[2].as_bytes())?;
                // // Modify directory state
                update_dir_state(args[2].clone())?;
                return Ok(format!("Switched to commit {}.", args[2]));
            } else {
                return Ok(format!("{} does not exist.", args[2]));
            }
        }
        4 => {
            if args[2] != "--" {
                return Ok(String::from("Incorrect operands."));
            }
            let new_args = vec![
                args[0].clone(),
                args[1].clone(),
                get_head_commit()?,
                args[2].clone(),
                args[3].clone(),
            ];
            return checkout(&new_args);
        }
        5 => {
            if args[3] != "--" {
                return Ok(String::from("Incorrect operands."));
            } else if !object_exists(&args[2]) {
                return Ok(format!("No commit with ID {} exists.", args[2]));
            }
            let hash = get_hash_in_commit(&args[2], &args[4])?;
            if hash == "DNE" {
                if file_exists(&args[4]) {
                    remove_file(args[4].clone())?;
                }
            } else {
                write_file_given_hash(args[4].clone(), hash)?;
            }
            return Ok(String::from(""));
        }
        _ => return Ok(String::from("Incorrect operands.")),
    }
}

/// Changes the directory to the state at the given commit hash
fn update_dir_state(commit_hash: String) -> Result<()> {
    assert!(file_exists(&format!(
        ".vcs/objects/{}/{}",
        &commit_hash[0..2],
        &commit_hash[2..]
    )));
    let mut current_dir: HashSet<String> = HashSet::new();
    for entry in read_dir("./")? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            continue;
        }
        current_dir.insert(no_dir_string(path));
    }
    let tree_contents = get_object_contents(&get_commit_tree(&commit_hash)?)?;
    let serialized_tree = serialize_tree(&tree_contents);
    for (filename, filehash) in serialized_tree {
        if current_dir.remove(&filename) {
            if sha2(&format!("blob\n{}", get_file_contents(&filename)?)) != filehash {
                write_file_given_hash(filename, filehash)?;
            }
        } else {
            write_file_given_hash(filename, filehash)?;
        }
    }
    for filename in current_dir {
        remove_file(filename)?;
    }
    Ok(())
}

/// Given a filename and a blob hash, create a new file with the contents of the blob in the file
fn write_file_given_hash(filename: String, hash: String) -> Result<()> {
    if file_exists(&filename) {
        clear_file_contents(&filename)?;
    }
    let mut new_file = File::create(filename)?;
    let mut blob_contents = get_object_contents(&hash)?;
    // drains the first 5 characters since that's `blob\n`
    blob_contents.drain(0..5);
    new_file.write_all(blob_contents.as_str().as_bytes())?;
    Ok(())
}

#[cfg(test)]
pub mod tests {

    // Partitions for checkout:
    // Partition on which happens:
    //     Error not in vcs, error incorrect operands, checkout branch, checkout file in
    //     commit, checkout commit by id.
    // Further partition on checking out branches: same branch, different branches, branch doesn't
    //      exist
    // Further partition on checkout file in commit:
    //      On whether commit is specified: yes, no
    //  For checkout commit: check that `add`, `rm`, and `commit` log correct response.

    use std::{
        env::set_current_dir,
        fs::{create_dir, remove_file, File},
        io::Write,
    };

    use crate::{
        objects::commit::{get_head_commit, INITIAL_COMMIT_HASH},
        operations::{add::add, branch::branch, commit::commit, init::init, rm::rm},
        utils::{
            fs_utils::{clear_file_contents, file_exists, get_file_contents},
            test_dir::make_test_dir,
        },
    };

    use super::*;

    #[test]
    fn not_in_vcs_dir() -> Result<()> {
        let _test_dir = make_test_dir()?;
        let test_args: Vec<String> =
            vec![String::from("target/debug/vcs"), String::from("checkout")];
        assert_eq!(
            "Not in an initialized vcs directory.",
            checkout(&test_args)?
        );
        Ok(())
    }

    #[test]
    fn incorrect_operands() -> Result<()> {
        let _test_dir = make_test_dir()?;
        let _ = init(&vec![
            String::from("target/debug/vcs"),
            String::from("init"),
        ]);
        assert_eq!(
            "Incorrect operands.",
            checkout(&vec![
                String::from("target/debug/vcs"),
                String::from("checkout"),
            ])?
        );
        assert_eq!(
            "Incorrect operands.",
            checkout(&vec![
                String::from("target/debug/vcs"),
                String::from("checkout"),
                INITIAL_COMMIT_HASH.to_string(),
                String::from("--"),
            ])?
        );
        Ok(())
    }

    #[test]
    fn test_checkout_same_branch() -> Result<()> {
        let _test_dir = make_test_dir()?;
        let _ = init(&vec![
            String::from("target/debug/vcs"),
            String::from("init"),
        ]);
        assert_eq!(
            "Already on main.",
            checkout(&vec![
                String::from("target/debug/vcs"),
                String::from("checkout"),
                String::from("main"),
            ])?
        );
        Ok(())
    }

    #[test]
    fn test_checkout_nonexistent_branch() -> Result<()> {
        let _test_dir = make_test_dir()?;
        let _ = init(&vec![
            String::from("target/debug/vcs"),
            String::from("init"),
        ]);
        assert_eq!(
            "test does not exist.",
            checkout(&vec![
                String::from("target/debug/vcs"),
                String::from("checkout"),
                String::from("test"),
            ])?
        );
        Ok(())
    }

    #[test]
    fn test_checkout_diff_branch() -> Result<()> {
        let _test_dir = make_test_dir()?;
        create_dir("test_dir")?;
        set_current_dir("test_dir")?;
        let _ = init(&vec![
            String::from("target/debug/vcs"),
            String::from("init"),
        ]);
        let _ = branch(&vec![
            String::from("target/debug/vcs"),
            String::from("branch"),
            String::from("test_branch"),
        ]);
        let mut first_file = File::create("f1.txt")?;
        first_file.write_all("file one text".as_bytes())?;
        let _ = add(&vec![
            String::from("target/debug/vcs"),
            String::from("add"),
            String::from("f1.txt"),
        ])?;
        let mut modified_file = File::create("f2.txt")?;
        modified_file.write_all("to be changed".as_bytes())?;
        let _ = add(&vec![
            String::from("target/debug/vcs"),
            String::from("add"),
            String::from("f2.txt"),
        ])?;
        let (_, first_commit_hash) = commit(&vec![
            String::from("target/debug/vcs"),
            String::from("commit"),
            String::from("Add two files"),
        ])?;
        assert_eq!(
            "Switched to branch test_branch.",
            checkout(&vec![
                String::from("target/debug/vcs"),
                String::from("checkout"),
                String::from("test_branch"),
            ])?
        );
        assert!(!file_exists("f1.txt"));
        assert!(!file_exists("f2.txt"));
        assert_eq!(INITIAL_COMMIT_HASH, get_head_commit()?);
        let mut second_modified_file = File::create("f2.txt")?;
        second_modified_file.write_all("to this!".as_bytes())?;
        let _ = add(&vec![
            String::from("target/debug/vcs"),
            String::from("add"),
            String::from("f2.txt"),
        ])?;
        let _ = commit(&vec![
            String::from("target/debug/vcs"),
            String::from("commit"),
            String::from("Switch f2.txt text"),
        ])?;
        assert_eq!(
            "Switched to branch main.",
            checkout(&vec![
                String::from("target/debug/vcs"),
                String::from("checkout"),
                String::from("main"),
            ])?
        );
        assert_eq!(first_commit_hash, get_head_commit()?);
        assert!(file_exists("f1.txt"));
        assert_eq!("file one text", get_file_contents("f1.txt")?);
        assert!(file_exists("f2.txt"));
        assert_eq!("to be changed", get_file_contents("f2.txt")?);
        Ok(())
    }

    #[test]
    fn test_checkout_nonexistent_id() -> Result<()> {
        let _test_dir = make_test_dir()?;
        create_dir("test_dir")?;
        set_current_dir("test_dir")?;
        let _ = init(&vec![
            String::from("target/debug/vcs"),
            String::from("init"),
        ]);
        assert_eq!(
            "No commit with ID dne exists.",
            checkout(&vec![
                String::from("target/debug/vcs"),
                String::from("checkout"),
                String::from("dne"),
                String::from("--"),
                String::from("f3.txt"),
            ])?
        );
        Ok(())
    }

    #[test]
    fn test_checkout_file_no_id() -> Result<()> {
        let _test_dir = make_test_dir()?;
        create_dir("test_dir")?;
        set_current_dir("test_dir")?;
        let _ = init(&vec![
            String::from("target/debug/vcs"),
            String::from("init"),
        ]);
        let mut file_one = File::create("f1.txt")?;
        file_one.write_all("file 1 text".as_bytes())?;
        let mut file_two = File::create("f2.txt")?;
        file_two.write_all("file 2 text".as_bytes())?;
        let _ = add(&vec![
            String::from("target/debug/vcs"),
            String::from("add"),
            String::from("f1.txt"),
        ])?;
        let _ = add(&vec![
            String::from("target/debug/vcs"),
            String::from("add"),
            String::from("f2.txt"),
        ])?;
        let _ = commit(&vec![
            String::from("target/debug/vcs"),
            String::from("commit"),
            String::from("Set up first commit"),
        ]);
        checkout(&vec![
            String::from("target/debug/vcs"),
            String::from("checkout"),
            String::from("--"),
            String::from("f3.txt"),
        ])?;
        assert!(!file_exists("f3.txt"));
        remove_file("f1.txt")?;
        assert!(!file_exists("f1.txt"));
        checkout(&vec![
            String::from("target/debug/vcs"),
            String::from("checkout"),
            String::from("--"),
            String::from("f1.txt"),
        ])?;
        assert!(file_exists("f1.txt"));
        assert_eq!("file 1 text", get_file_contents("f1.txt")?);
        clear_file_contents("f2.txt")?;
        assert_eq!("", get_file_contents("f2.txt")?);
        checkout(&vec![
            String::from("target/debug/vcs"),
            String::from("checkout"),
            String::from("--"),
            String::from("f2.txt"),
        ])?;
        assert_eq!("file 2 text", get_file_contents("f2.txt")?);
        checkout(&vec![
            String::from("target/debug/vcs"),
            String::from("checkout"),
            String::from("--"),
            String::from("f2.txt"),
        ])?;
        assert_eq!("file 2 text", get_file_contents("f2.txt")?);
        Ok(())
    }

    #[test]
    fn test_checkout_file_with_id() -> Result<()> {
        let _test_dir = make_test_dir()?;
        create_dir("test_dir")?;
        set_current_dir("test_dir")?;
        let _ = init(&vec![
            String::from("target/debug/vcs"),
            String::from("init"),
        ]);
        let mut file_one = File::create("f1.txt")?;
        file_one.write_all("file 1 text".as_bytes())?;
        let mut file_two = File::create("f2.txt")?;
        file_two.write_all("file 2 text".as_bytes())?;
        let _ = add(&vec![
            String::from("target/debug/vcs"),
            String::from("add"),
            String::from("f1.txt"),
        ])?;
        let _ = add(&vec![
            String::from("target/debug/vcs"),
            String::from("add"),
            String::from("f2.txt"),
        ])?;
        let (_, commit_hash) = commit(&vec![
            String::from("target/debug/vcs"),
            String::from("commit"),
            String::from("Set up first commit"),
        ])?;
        rm(&vec![
            String::from("target/debug/vcs"),
            String::from("rm"),
            String::from("f2.txt"),
        ])?;
        commit(&vec![
            String::from("target/debug/vcs"),
            String::from("commit"),
            String::from("Remove f2.txt"),
        ])?;
        checkout(&vec![
            String::from("target/debug/vcs"),
            String::from("checkout"),
            commit_hash.clone(),
            String::from("--"),
            String::from("f3.txt"),
        ])?;
        assert!(!file_exists("f3.txt"));
        clear_file_contents("f1.txt")?;
        checkout(&vec![
            String::from("target/debug/vcs"),
            String::from("checkout"),
            commit_hash.clone(),
            String::from("--"),
            String::from("f1.txt"),
        ])?;
        assert_eq!("file 1 text", get_file_contents("f1.txt")?);
        checkout(&vec![
            String::from("target/debug/vcs"),
            String::from("checkout"),
            commit_hash.clone(),
            String::from("--"),
            String::from("f2.txt"),
        ])?;
        assert_eq!("file 2 text", get_file_contents("f2.txt")?);
        checkout(&vec![
            String::from("target/debug/vcs"),
            String::from("checkout"),
            commit_hash.clone(),
            String::from("--"),
            String::from("f2.txt"),
        ])?;
        assert_eq!("file 2 text", get_file_contents("f2.txt")?);
        Ok(())
    }

    #[test]
    fn test_checkout_commit() -> Result<()> {
        let _test_dir = make_test_dir()?;
        create_dir("test_dir")?;
        set_current_dir("test_dir")?;
        let _ = init(&vec![
            String::from("target/debug/vcs"),
            String::from("init"),
        ]);
        let mut file_one = File::create("f1.txt")?;
        file_one.write_all("file 1 text".as_bytes())?;
        let mut file_two = File::create("f2.txt")?;
        file_two.write_all("file 2 text".as_bytes())?;
        let _ = add(&vec![
            String::from("target/debug/vcs"),
            String::from("add"),
            String::from("f1.txt"),
        ])?;
        let _ = add(&vec![
            String::from("target/debug/vcs"),
            String::from("add"),
            String::from("f2.txt"),
        ])?;
        let (_, commit_hash) = commit(&vec![
            String::from("target/debug/vcs"),
            String::from("commit"),
            String::from("Set up first commit"),
        ])?;
        remove_file("f1.txt")?;
        clear_file_contents("f2.txt")?;
        let mut file_three = File::create("f3.txt")?;
        file_three.write_all("file 3 text".as_bytes())?;
        assert_eq!(
            format!("Switched to commit {}.", commit_hash),
            checkout(&vec![
                String::from("target/debug/vcs"),
                String::from("checkout"),
                commit_hash,
            ])?
        );
        assert!(file_exists("f1.txt"));
        assert_eq!("file 1 text", get_file_contents("f1.txt")?);
        assert!(file_exists("f2.txt"));
        assert_eq!("file 2 text", get_file_contents("f2.txt")?);
        assert!(!file_exists("f3.txt"));
        assert_eq!(
            "Currently in a detached HEAD state. Check out a branch to modify the directory.",
            rm(&vec![
                String::from("target/debug/vcs"),
                String::from("rm"),
                String::from("f1.txt"),
            ])?
        );
        let mut file_three = File::create("f3.txt")?;
        file_three.write_all("file 3 text".as_bytes())?;
        let (output, _) = add(&vec![
            String::from("target/debug/vcs"),
            String::from("add"),
            String::from("f3.txt"),
        ])?;
        assert_eq!(
            "Currently in a detached HEAD state. Check out a branch to modify the directory.",
            output
        );
        Ok(())
    }
}
