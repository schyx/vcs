use std::{collections::HashSet, io::Result};

use chrono::Utc;

use crate::{
    objects::{
        commit::{
            get_commit_parent, get_commit_tree, get_hash_in_commit, get_head_commit, write_commit,
            INITIAL_COMMIT_HASH,
        },
        get_branch_name, get_object_contents,
        tree::{serialize_tree, write_tree},
    },
    operations::{add::add, commit::update_head},
    utils::fs_utils::{directory_exists, file_exists, get_file_contents, write_contents},
};

/// Executes `vcs merge` with `args` as arguments. Returns the string that should be logged to
/// the console.
///
/// The only use case is the following:
///     1. `vcs merge <BRANCH_NAME>`: This merges the <BRANCH_NAME> into the current branch. If a
///        fast forward merge can be performed, adds a commit with commit message and also log
///        `Merge branch <BRANCH_NAME> (BRANCH_COMMIT_ID) into <CURRENT_BRANCH>
///        (CURRENT_COMMIT_ID).` and fast forwards the repository to the state of <BRANCH_NAME>. If
///        a fast-forward merge cannot be performed, then performs a three way merge. Files that do
///        not have merge conflicts will be staged immediately with their changes, while files
///        without merge conflicts will be modified with the following way:
///             <<<<<<< HEAD
///             <CURRENT FILE CONTENTS>
///             =======
///             <FILE CONTENTS IN BRANCH_NAME>
///             >>>>>>> <BRANCH_NAME>
///         If there are files with merge conflicts, log `There are files with merge conflicts. Fix
///         and commit the following:` and log the following for each file:
///             both modified: <FILENAME>
///         Files with no merge conflicts are merged and staged, and if no files have merge
///         conflicts, log and create a new commit with message `Merge branch <BRANCH_NAME>
///         (BRANCH_COMMIT_ID) into <CURRENT_BRANCH> (CURRENT_COMMIT_ID).`
///
///         If the current branch is an ancestor of <BRANCH_NAME> (including if <BRANCH_NAME> is
///         the current branch), log `Already up to date.`.
///
///         If the given branch does not exist, log `Branch <BRANCH_NAME> does not exist.`
///
/// If there are an incorrect number of arguments, log `Incorrect operands.`, and if not in an
/// initialized vcs directory, log `Not in an initialized vcs directory.`.
pub fn merge(args: &Vec<String>) -> Result<String> {
    assert!(args[1] == "merge");
    if !directory_exists(".vcs") {
        return Ok(String::from("Not in an initialized vcs directory."));
    } else if args.len() != 3 {
        return Ok(String::from("Incorrect operands."));
    } else if !file_exists(&format!(".vcs/branches/{}", args[2])) {
        return Ok(format!("Branch {} does not exist.", args[2]));
    } else if !file_exists(&format!(".vcs/branches/{}", get_branch_name()?)) {
        return Ok(String::from(
            "Currently in a detached HEAD state. Check out a branch to modify the directory.",
        ));
    }
    let current_branch = get_branch_name()?;
    let current_hash = get_head_commit()?;
    let original_merging_hash = get_file_contents(&format!(".vcs/branches/{}", args[2]))?;
    if current_hash == original_merging_hash {
        return Ok(String::from("Already up to date."));
    }
    let mut merging_hash = original_merging_hash.clone();
    let mut merging_ancestors: HashSet<String> = HashSet::new();
    while &merging_hash != INITIAL_COMMIT_HASH {
        merging_hash = get_commit_parent(&merging_hash)?.unwrap();
        if current_hash == merging_hash {
            // This should be a fast forward merge
            let message = format!(
                "Merge branch {} ({}) into {} ({}).",
                args[2], original_merging_hash, current_branch, current_hash
            );
            let new_head = write_commit(
                &message,
                &current_hash,
                Utc::now().timestamp(),
                &get_commit_tree(&original_merging_hash)?,
            );
            update_head(new_head)?;
            return Ok(message);
        }
        merging_ancestors.insert(merging_hash.clone());
    }

    let mut changing_hash = current_hash.clone();
    let mut three_way_commit = String::from("");
    while &changing_hash != INITIAL_COMMIT_HASH {
        if changing_hash == original_merging_hash {
            // Branch to merge is ancestor of current branch, should do nothing
            return Ok(String::from("Already up to date."));
        }
        changing_hash = get_commit_parent(&changing_hash)?.unwrap();
        if merging_ancestors.contains(&changing_hash) {
            three_way_commit = changing_hash.clone();
            break;
        }
    }
    if three_way_commit == "" {
        three_way_commit = INITIAL_COMMIT_HASH.to_string();
    }

    let current_tree = serialize_tree(&get_object_contents(&get_commit_tree(&current_hash)?)?);
    let merge_tree = serialize_tree(&get_object_contents(&get_commit_tree(
        &original_merging_hash,
    )?)?);
    let mut collisions: HashSet<String> = HashSet::new();
    let mut base_tree = serialize_tree(&get_object_contents(&get_commit_tree(&three_way_commit)?)?);
    for file in current_tree.keys() {
        if merge_tree.contains_key(file)
            && (base_tree.get(file).unwrap() != current_tree.get(file).unwrap()
                && base_tree.get(file).unwrap() != merge_tree.get(file).unwrap())
        {
            collisions.insert(file.to_string());
        }
    }
    base_tree.extend(current_tree);
    base_tree.extend(merge_tree);
    if collisions.len() == 0 {
        // no conflicts
        let mut tree_lines: Vec<String> = vec![];
        for (fname, fhash) in &base_tree {
            tree_lines.push(format!("{}: {}", fname, fhash));
            let mut contents = get_object_contents(fhash)?;
            contents.drain(0..5);
            write_contents(fname, &contents)?;
        }
        tree_lines.sort();
        let merged_hash = write_tree(&vec![], &tree_lines);
        let message = format!(
            "Merge branch {} ({}) into {} ({}).",
            args[2], original_merging_hash, current_branch, current_hash
        );
        let new_commit_hash = write_commit(
            &message,
            &current_hash,
            Utc::now().timestamp(),
            &merged_hash,
        );
        update_head(new_commit_hash.clone())?;
        return Ok(message);
    }

    println!("{:?}", collisions);
    for (fname, fhash) in &base_tree {
        if collisions.contains(fname) {
            continue;
        }
        println!("non-collisions: {}", fname);
        let mut contents = get_object_contents(fhash)?;
        contents.drain(0..5);
        write_contents(fname, &contents)?;
        let _ = add(&vec![
            String::from("target/debug/vcs"),
            String::from("add"),
            fname.to_string(),
        ]);
    }
    let mut message =
        String::from("There are files with merge conflicts. Fix and commit the following:");
    for fname in collisions {
        let contents = format!(
            "<<<<<<< HEAD\n{}=======\n{}>>>>>>> {}",
            get_contents_in_commit(&current_hash, &fname)?,
            get_contents_in_commit(&original_merging_hash, &fname)?,
            args[2],
        );
        write_contents(&fname, &contents)?;
        message.push_str(&format!("\n\tboth modified: {}", fname));
    }
    return Ok(message);
}

fn get_contents_in_commit(commit: &str, fname: &str) -> Result<String> {
    let fhash = get_hash_in_commit(commit, fname)?;
    let mut contents = get_object_contents(&fhash)?;
    contents.drain(0..5); // gets rid of blob
    return Ok(contents);
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::{
        env::set_current_dir,
        fs::{create_dir, File},
        io::Write,
    };

    use crate::{
        objects::{
            commit::{get_commit_message, get_commit_parent, get_commit_tree, get_head_commit},
            get_branch_name,
        },
        operations::{add::add, branch::branch, checkout::checkout, commit::commit, init::init},
        utils::{
            fs_utils::{file_exists, get_file_contents},
            hash::sha2,
            test_dir::make_test_dir,
        },
    };

    // Partitions for merge
    //      Failure cases: Not in vcs dir, incorrect operands, branch doesn't exist, no failures
    // Further partition for no failures: Fast-forward merge, not fast-forward and no merge
    //      conflict, merge conflict, <BRANCH_NAME> is the current branch, <BRANCH_NAME> is an
    //      ancestor of the current branch.

    #[test]
    fn not_in_vcs_dir() -> Result<()> {
        let _test_dir = make_test_dir()?;
        let test_args: Vec<String> = vec![String::from("target/debug/vcs"), String::from("merge")];
        assert_eq!("Not in an initialized vcs directory.", merge(&test_args)?);
        Ok(())
    }

    #[test]
    fn incorrect_operands() -> Result<()> {
        let _test_dir = make_test_dir()?;
        let _ = init(&vec![
            String::from("target/debug/vcs"),
            String::from("init"),
        ]);
        let test_args: Vec<String> = vec![String::from("target/debug/vcs"), String::from("merge")];
        assert_eq!("Incorrect operands.", merge(&test_args)?);
        Ok(())
    }

    #[test]
    fn test_branch_dne() -> Result<()> {
        let _test_dir = make_test_dir()?;
        create_dir("test_dir")?;
        set_current_dir("test_dir")?;
        let _ = init(&vec![
            String::from("target/debug/vcs"),
            String::from("init"),
        ]);
        let _ = File::create("file1")?;
        let _ = add(&vec![
            String::from("target/debug/vcs"),
            String::from("add"),
            String::from("file1"),
        ])?;
        let _ = commit(&vec![
            String::from("target/debug/vcs"),
            String::from("commit"),
            String::from("Add file1"),
        ])?;
        assert_eq!(
            "Branch dnebranch does not exist.",
            merge(&vec![
                String::from("target/debug/vcs"),
                String::from("merge"),
                String::from("dnebranch"),
            ])?
        );
        Ok(())
    }

    #[test]
    fn test_fast_forward_merge() -> Result<()> {
        let _test_dir = make_test_dir()?;
        create_dir("test_dir")?;
        set_current_dir("test_dir")?;
        let _ = init(&vec![
            String::from("target/debug/vcs"),
            String::from("init"),
        ]);
        let mut file_one = File::create("file1")?;
        let _ = add(&vec![
            String::from("target/debug/vcs"),
            String::from("add"),
            String::from("file1"),
        ])?;
        let (_, common_parent_hash) = commit(&vec![
            String::from("target/debug/vcs"),
            String::from("commit"),
            String::from("Add file1"),
        ])?;
        let _ = branch(&vec![
            String::from("target/debug/vcs"),
            String::from("branch"),
            String::from("test"),
        ])?;
        let _ = checkout(&vec![
            String::from("target/debug/vcs"),
            String::from("checkout"),
            String::from("test"),
        ])?;
        file_one.write_all("add to file 1".as_bytes())?;
        let _ = add(&vec![
            String::from("target/debug/vcs"),
            String::from("add"),
            String::from("file1"),
        ])?;
        let _ = commit(&vec![
            String::from("target/debug/vcs"),
            String::from("commit"),
            String::from("Write file1"),
        ])?;
        let mut file_two = File::create("file2")?;
        file_two.write_all("Text in file 2".as_bytes())?;
        let _ = add(&vec![
            String::from("target/debug/vcs"),
            String::from("add"),
            String::from("file2"),
        ])?;
        let (_, last_test_commit_hash) = commit(&vec![
            String::from("target/debug/vcs"),
            String::from("commit"),
            String::from("Add and write file2"),
        ])?;
        let _ = checkout(&vec![
            String::from("target/debug/vcs"),
            String::from("checkout"),
            String::from("main"),
        ])?;
        assert!(!file_exists("file2"));
        assert_eq!("", get_file_contents("file1")?);
        assert_eq!(
            format!(
                "Merge branch test ({}) into main ({}).",
                last_test_commit_hash, common_parent_hash
            ),
            merge(&vec![
                String::from("target/debug/vcs"),
                String::from("merge"),
                String::from("test"),
            ])?
        );
        let current_commit_hash = get_head_commit()?;
        assert_eq!(
            format!(
                "Merge branch test ({}) into main ({}).",
                last_test_commit_hash, common_parent_hash
            ),
            get_commit_message(&current_commit_hash)?
        );
        assert_eq!(
            common_parent_hash,
            get_commit_parent(&current_commit_hash)?.unwrap()
        );
        assert_eq!(
            get_commit_tree(&last_test_commit_hash)?,
            get_commit_tree(&current_commit_hash)?
        );
        assert_eq!("", get_file_contents(".vcs/index")?);
        Ok(())
    }

    #[test]
    fn test_no_ff_no_merge_conflict() -> Result<()> {
        let _test_dir = make_test_dir()?;
        create_dir("test_dir")?;
        set_current_dir("test_dir")?;
        let _ = init(&vec![
            String::from("target/debug/vcs"),
            String::from("init"),
        ]);
        let mut file_one = File::create("file1")?;
        let _ = add(&vec![
            String::from("target/debug/vcs"),
            String::from("add"),
            String::from("file1"),
        ])?;
        let _ = commit(&vec![
            String::from("target/debug/vcs"),
            String::from("commit"),
            String::from("Add file1"),
        ])?;
        let _ = branch(&vec![
            String::from("target/debug/vcs"),
            String::from("branch"),
            String::from("test"),
        ])?;
        let _ = checkout(&vec![
            String::from("target/debug/vcs"),
            String::from("checkout"),
            String::from("test"),
        ])?;
        file_one.write_all("add to file 1".as_bytes())?;
        let (_, f1_hash) = add(&vec![
            String::from("target/debug/vcs"),
            String::from("add"),
            String::from("file1"),
        ])?;
        let _ = commit(&vec![
            String::from("target/debug/vcs"),
            String::from("commit"),
            String::from("Write file1"),
        ])?;
        let mut file_two = File::create("file2")?;
        file_two.write_all("Text in file 2".as_bytes())?;
        let (_, f2_hash) = add(&vec![
            String::from("target/debug/vcs"),
            String::from("add"),
            String::from("file2"),
        ])?;
        let (_, test_commit_hash) = commit(&vec![
            String::from("target/debug/vcs"),
            String::from("commit"),
            String::from("Add and write file2"),
        ])?;
        let _ = checkout(&vec![
            String::from("target/debug/vcs"),
            String::from("checkout"),
            String::from("main"),
        ])?;
        let mut file_three = File::create("file3")?;
        file_three.write_all("f3txt".as_bytes())?;
        let (_, f3_hash) = add(&vec![
            String::from("target/debug/vcs"),
            String::from("add"),
            String::from("file3"),
        ])?;
        let (_, main_commit_hash) = commit(&vec![
            String::from("target/debug/vcs"),
            String::from("commit"),
            String::from("Write and add file3"),
        ])?;
        assert_eq!(
            format!(
                "Merge branch test ({}) into main ({}).",
                test_commit_hash, main_commit_hash
            ),
            merge(&vec![
                String::from("target/debug/vcs"),
                String::from("merge"),
                String::from("test"),
            ])?
        );
        assert_eq!("", get_file_contents(".vcs/index")?);
        let head_commit = get_head_commit()?;
        assert_eq!(
            sha2(&format!(
                "Trees\nBlobs\nfile1: {}\nfile2: {}\nfile3: {}",
                f1_hash, f2_hash, f3_hash
            )),
            get_commit_tree(&head_commit)?
        );
        assert_eq!("main", get_branch_name()?);
        assert_eq!("f3txt", get_file_contents("file3")?);
        assert_eq!("Text in file 2", get_file_contents("file2")?);
        assert_eq!("add to file 1", get_file_contents("file1")?);
        Ok(())
    }

    #[test]
    fn test_merge_conflict() -> Result<()> {
        let _test_dir = make_test_dir()?;
        create_dir("test_dir")?;
        set_current_dir("test_dir")?;
        let _ = init(&vec![
            String::from("target/debug/vcs"),
            String::from("init"),
        ]);
        let mut file_one = File::create("file1")?;
        let _ = add(&vec![
            String::from("target/debug/vcs"),
            String::from("add"),
            String::from("file1"),
        ])?;
        let _ = commit(&vec![
            String::from("target/debug/vcs"),
            String::from("commit"),
            String::from("Add file1"),
        ])?;
        let _ = branch(&vec![
            String::from("target/debug/vcs"),
            String::from("branch"),
            String::from("checkmerge"),
        ])?;
        let _ = branch(&vec![
            String::from("target/debug/vcs"),
            String::from("branch"),
            String::from("conflictbranch"),
        ])?;
        let _ = checkout(&vec![
            String::from("target/debug/vcs"),
            String::from("checkout"),
            String::from("checkmerge"),
        ])?;
        file_one.write_all("Stuff in checkmerge".as_bytes())?;
        let _ = add(&vec![
            String::from("target/debug/vcs"),
            String::from("add"),
            String::from("file1"),
        ])?;
        let mut file_two = File::create("file2")?;
        file_two.write_all("file2 text".as_bytes())?;
        let (_, f2_hash) = add(&vec![
            String::from("target/debug/vcs"),
            String::from("add"),
            String::from("file2"),
        ])?;
        let _ = commit(&vec![
            String::from("target/debug/vcs"),
            String::from("commit"),
            String::from("checkmerge commit!"),
        ])?;
        let _ = checkout(&vec![
            String::from("target/debug/vcs"),
            String::from("checkout"),
            String::from("conflictbranch"),
        ])?;
        assert_eq!("", get_file_contents("file1")?);
        assert!(!file_exists("file2"));
        let mut file_one = File::create("file1")?;
        file_one.write_all("Stuff in conflictbranch".as_bytes())?;
        let _ = add(&vec![
            String::from("target/debug/vcs"),
            String::from("add"),
            String::from("file1"),
        ])?;
        let mut file_three = File::create("file3")?;
        file_three.write_all("file3 text".as_bytes())?;
        let _ = add(&vec![
            String::from("target/debug/vcs"),
            String::from("add"),
            String::from("file3"),
        ])?;
        let _ = commit(&vec![
            String::from("target/debug/vcs"),
            String::from("commit"),
            String::from("conflictbranch commit!"),
        ])?;
        assert_eq!(
            "There are files with merge conflicts. Fix and commit the following:\n\tboth modified: file1",
            merge(&vec![
                String::from("target/debug/vcs"),
                String::from("merge"),
                String::from("checkmerge"),
            ])?
        );
        assert_eq!(
            format!("blob {} file2", f2_hash),
            get_file_contents(".vcs/index")?
        );
        assert_eq!("file2 text", get_file_contents("file2")?);
        assert_eq!("file3 text", get_file_contents("file3")?);
        assert_eq!(
            "<<<<<<< HEAD\nStuff in conflictbranch=======\nStuff in checkmerge>>>>>>> checkmerge",
            get_file_contents("file1")?
        );
        Ok(())
    }

    #[test]
    fn test_merge_from_self() -> Result<()> {
        let _test_dir = make_test_dir()?;
        create_dir("test_dir")?;
        set_current_dir("test_dir")?;
        let _ = init(&vec![
            String::from("target/debug/vcs"),
            String::from("init"),
        ]);
        let _ = File::create("file1")?;
        let _ = add(&vec![
            String::from("target/debug/vcs"),
            String::from("add"),
            String::from("file1"),
        ])?;
        let _ = commit(&vec![
            String::from("target/debug/vcs"),
            String::from("commit"),
            String::from("Add file1"),
        ])?;
        assert_eq!(
            "Already up to date.",
            merge(&vec![
                String::from("target/debug/vcs"),
                String::from("merge"),
                String::from("main"),
            ])?
        );
        Ok(())
    }

    #[test]
    fn test_merge_from_ancestor() -> Result<()> {
        let _test_dir = make_test_dir()?;
        create_dir("test_dir")?;
        set_current_dir("test_dir")?;
        let _ = init(&vec![
            String::from("target/debug/vcs"),
            String::from("init"),
        ]);
        let mut file_one = File::create("file1")?;
        let _ = add(&vec![
            String::from("target/debug/vcs"),
            String::from("add"),
            String::from("file1"),
        ])?;
        let _ = commit(&vec![
            String::from("target/debug/vcs"),
            String::from("commit"),
            String::from("Add file1"),
        ])?;
        let _ = branch(&vec![
            String::from("target/debug/vcs"),
            String::from("branch"),
            String::from("test"),
        ])?;
        let _ = checkout(&vec![
            String::from("target/debug/vcs"),
            String::from("checkout"),
            String::from("test"),
        ])?;
        file_one.write_all("add to file 1".as_bytes())?;
        let _ = add(&vec![
            String::from("target/debug/vcs"),
            String::from("add"),
            String::from("file1"),
        ])?;
        let _ = commit(&vec![
            String::from("target/debug/vcs"),
            String::from("commit"),
            String::from("Write file1"),
        ])?;
        let mut file_two = File::create("file2")?;
        file_two.write_all("Text in file 2".as_bytes())?;
        let _ = add(&vec![
            String::from("target/debug/vcs"),
            String::from("add"),
            String::from("file2"),
        ])?;
        let _ = commit(&vec![
            String::from("target/debug/vcs"),
            String::from("commit"),
            String::from("Add and write file2"),
        ])?;
        assert_eq!(
            "Already up to date.",
            merge(&vec![
                String::from("target/debug/vcs"),
                String::from("merge"),
                String::from("main"),
            ])?
        );
        assert!(file_exists("file2"));
        assert_eq!("add to file 1", get_file_contents("file1")?);
        Ok(())
    }
}
