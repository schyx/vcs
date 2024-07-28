use std::io::Error;

use crate::{
    objects::write_object,
    utils::{
        fs_utils::{directory_exists, file_exists, get_file_contents, get_line_in_file},
        hash::sha2,
    },
};

use super::tree::find_file_in_tree;

pub const INITIAL_COMMIT_HASH: &str =
    "0ad65fe4e92832723f5d747f66d8ae6e5ca51b7e73342c9865c8f769143d12cc";

pub fn commit_string_and_hash(
    message: &str,
    parent: &str,
    time: i64,
    tree_hash: &str,
) -> (String, String) {
    let commit_string = format!(
        "Message\n{}\nParent\n{}\nTime\n{}\nTree Hash\n{}",
        message,
        parent,
        time.to_string(),
        tree_hash
    );
    (commit_string.clone(), sha2(&commit_string))
}

pub fn write_commit(message: &str, parent: &str, time: i64, tree_hash: &str) -> String {
    let (commit_string, commit_hash) = commit_string_and_hash(message, parent, time, tree_hash);
    let _ = write_object(&commit_hash, &commit_string);
    commit_hash
}

/// Returns the hash of the current head commit. If unable to get a commit, panics.
pub fn get_head_commit() -> Result<String, Error> {
    assert!(directory_exists(".vcs"));
    let head_branch_or_hash = get_file_contents(".vcs/HEAD")?;
    let branch_name = format!(".vcs/branches/{}", head_branch_or_hash);
    if file_exists(&branch_name) {
        get_file_contents(&branch_name)
    } else {
        panic!("not tested yet");
        // Ok(head_branch_or_hash)
    }
}

/// Returns the hash of the given file, or `DNE` if the file didn't exist in the given commit.
///
/// Panics if the commit doesn't exist
pub fn get_hash_in_commit(commit: &str, filename: &str) -> Result<String, Error> {
    let commit_file = format!(".vcs/objects/{}/{}", &commit[0..2], &commit[2..]);
    let tree_hash = get_line_in_file(&commit_file, 7)?;
    return find_file_in_tree(&tree_hash, filename); // TODO: Test after implementing commit
}

#[cfg(test)]
mod test {
    /* fields all exist */

    use crate::{
        objects::tree::EMPTY_TREE_HASH, operations::init::init, utils::test_dir::make_test_dir,
    };

    use super::*;

    #[test]
    fn test_commit_text() {
        let (commit_text, _) = commit_string_and_hash("message", "parent", 0, "tree_hash");
        assert_eq!(
            "Message\nmessage\nParent\nparent\nTime\n0\nTree Hash\ntree_hash",
            commit_text
        );
    }

    #[test]
    fn test_initial_commit() {
        let _ = make_test_dir();
        let (commit_text, commit_hash) =
            commit_string_and_hash("Initial commit", "No parent", 0, EMPTY_TREE_HASH);
        assert_eq!(
            format!(
                "Message\nInitial commit\nParent\nNo parent\nTime\n0\nTree Hash\n{}",
                EMPTY_TREE_HASH
            ),
            commit_text
        );
        assert_eq!(INITIAL_COMMIT_HASH, commit_hash);
    }

    #[test]
    fn test_get_head() -> Result<(), Error> {
        let _test_dir = make_test_dir()?;
        let _ = init(&vec![
            String::from("target/debug/vcs"),
            String::from("init"),
        ]);
        assert_eq!(INITIAL_COMMIT_HASH, get_head_commit()?);
        Ok(())
    }
}
