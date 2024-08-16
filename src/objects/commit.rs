use std::{
    fs::File,
    io::{BufRead, BufReader, Error},
};

use crate::{
    objects::write_object,
    utils::{
        fs_utils::{directory_exists, file_exists, get_file_contents, get_line_in_object},
        hash::sha2,
    },
};

use super::tree::find_file_in_tree;

pub const INITIAL_COMMIT_HASH: &str =
    "4dc93cdee44eeb4d71d3c1ff17bd16a715213cc4d8f27ac9d2ed77fadc3ffa63";

pub fn commit_string_and_hash(
    message: &str,
    parent: &str,
    time: i64,
    tree_hash: &str,
) -> (String, String) {
    let commit_string = format!(
        "Parent\n{}\nTime\n{}\nTree Hash\n{}\nMessage\n{}",
        parent,
        time.to_string(),
        tree_hash,
        message
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
    let tree_hash = get_commit_tree(&commit)?;
    return find_file_in_tree(&tree_hash, filename);
}

/// Given a commit hash, returns the attached commit message
pub fn get_commit_message(commit: &str) -> Result<String, Error> {
    let filename = format!(".vcs/objects/{}/{}", &commit[0..2], &commit[2..]);
    let file = File::open(filename)?;
    let reader = BufReader::new(file);
    // Skip 7 because that's the number of lines before message starts
    let lines: Vec<String> = reader.lines().skip(7).filter_map(Result::ok).collect();
    Ok(lines.join("\n"))
}

/// Given a commit hash, returns the hash of the tree it points to
pub fn get_commit_tree(commit: &str) -> Result<String, Error> {
    get_line_in_object(commit, 5)
}

/// Given a commit hash, returns the parent hash of the commit if it exists
pub fn get_commit_parent(commit: &str) -> Result<Option<String>, Error> {
    let line = get_line_in_object(commit, 1)?;
    if line == "No parent" {
        return Ok(None);
    }
    return Ok(Some(line));
}

/// Given a commit hash, returns the time of the commit if it exists
pub fn get_commit_time(commit: &str) -> Result<i64, Error> {
    let line = get_line_in_object(commit, 3)?;
    match line.parse::<i64>() {
        Ok(value) => Ok(value),
        Err(e) => Err(Error::new(std::io::ErrorKind::InvalidData, e)),
    }
}

#[cfg(test)]
mod test {
    /* fields all exist */

    use std::{fs::File, io::Write};

    use crate::{
        objects::tree::EMPTY_TREE_HASH,
        operations::{add::add, commit::commit, init::init},
        utils::test_dir::make_test_dir,
    };

    use super::*;

    #[test]
    fn test_commit_text() {
        let (commit_text, _) = commit_string_and_hash("message", "parent", 0, "tree_hash");
        assert_eq!(
            "Parent\nparent\nTime\n0\nTree Hash\ntree_hash\nMessage\nmessage",
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
                "Parent\nNo parent\nTime\n0\nTree Hash\n{}\nMessage\nInitial commit",
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

    #[test]
    fn test_file_dne_in_prev_commit() -> Result<(), Error> {
        let _test_dir = make_test_dir()?;
        let _ = init(&vec![
            String::from("target/debug/vcs"),
            String::from("init"),
        ]);
        assert_eq!("DNE", get_hash_in_commit(INITIAL_COMMIT_HASH, "file.py")?);
        Ok(())
    }

    #[test]
    fn test_file_exists_in_prev_commit() -> Result<(), Error> {
        let _test_dir = make_test_dir()?;
        let _ = init(&vec![
            String::from("target/debug/vcs"),
            String::from("init"),
        ]);
        let mut file = File::create("test.txt")?;
        let _ = file.write("test prev commit hash thing".as_bytes());
        let (_, file_hash) = add(&vec![
            String::from("target/debug/vcs"),
            String::from("add"),
            String::from("test.txt"),
        ])?;
        let (_, commit_hash) = commit(&vec![
            String::from("target/debug/vcs"),
            String::from("commit"),
            String::from("message heheheha"),
        ])?;
        assert_eq!(file_hash, get_hash_in_commit(&commit_hash, "test.txt")?);
        Ok(())
    }

    #[test]
    fn test_getters_on_commit() -> Result<(), Error> {
        let _test_dir = make_test_dir()?;
        let _ = init(&vec![
            String::from("target/debug/vcs"),
            String::from("init"),
        ]);
        assert_eq!(None, get_commit_parent(INITIAL_COMMIT_HASH)?);
        assert_eq!(0, get_commit_time(INITIAL_COMMIT_HASH)?);
        assert_eq!("Initial commit", get_commit_message(INITIAL_COMMIT_HASH)?);
        assert_eq!(EMPTY_TREE_HASH, get_commit_tree(INITIAL_COMMIT_HASH)?);
        let mut file = File::create("test.txt")?;
        let _ = file.write("test prev commit hash thing".as_bytes());
        let (_, _) = add(&vec![
            String::from("target/debug/vcs"),
            String::from("add"),
            String::from("test.txt"),
        ])?;
        let (_, commit_hash) = commit(&vec![
            String::from("target/debug/vcs"),
            String::from("commit"),
            String::from("message heheheha"),
        ])?;
        assert_eq!(
            INITIAL_COMMIT_HASH,
            get_commit_parent(&commit_hash)?.unwrap()
        );
        Ok(())
    }
}
