use crate::{objects::write_object, utils::hash::sha2};

fn commit_string_and_hash(
    message: &str,
    parent: &str,
    time: i32,
    tree_hash: &str,
) -> (String, String) {
    let mut commit_string = String::from("Message\n");
    commit_string.push_str(&message);
    commit_string.push_str("\n");
    commit_string.push_str("Parent\n");
    commit_string.push_str(&parent);
    commit_string.push_str("\n");
    commit_string.push_str("Time\n");
    commit_string.push_str(&time.to_string());
    commit_string.push_str("\n");
    commit_string.push_str("Tree Hash\n");
    commit_string.push_str(&tree_hash);
    (commit_string.clone(), sha2(&commit_string))
}

pub fn write_commit(message: &str, parent: &str, time: i32, tree_hash: &str) -> String {
    let (commit_string, commit_hash) = commit_string_and_hash(message, parent, time, tree_hash);
    let _ = write_object(&commit_hash, &commit_string);
    commit_hash
}

#[cfg(test)]
mod test {
    /* fields all exist */

    use super::*;

    #[test]
    fn test_commit_text() {
        let (commit_text, _) = commit_string_and_hash("message", "parent", 0, "tree_hash");
        assert_eq!(
            "Message\nmessage\nParent\nparent\nTime\n0\nTree Hash\ntree_hash",
            commit_text
        );
    }
}
