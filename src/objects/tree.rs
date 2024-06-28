use std::collections::HashMap;

use crate::{objects::write_object, utils::hash::sha2};

/// Given the subtrees and subblobs, outputs the text and hash of the tree object, respectively
fn get_tree_text_and_hash(
    subtrees: &HashMap<String, String>,
    subblobs: &HashMap<String, String>,
) -> (String, String) {
    let mut output = String::from("Trees\n");
    for (tree, hash) in subtrees {
        output.push_str(tree);
        output.push_str(": ");
        output.push_str(hash);
        output.push_str("\n");
    }

    output.push_str("Blobs");
    for (blob, hash) in subblobs {
        output.push_str("\n");
        output.push_str(blob);
        output.push_str(": ");
        output.push_str(hash);
    }
    (output.clone(), sha2(&output))
}

/// Returns the hash of a tree with subtrees and subblobs. Also creates the tree object
pub fn write_tree(
    subtrees: &HashMap<String, String>,
    subblobs: &HashMap<String, String>,
) -> String {
    let (tree_text, tree_hash) = get_tree_text_and_hash(subtrees, subblobs);
    let _ = write_object(&tree_hash, &tree_text);
    tree_hash
}

#[cfg(test)]
mod tests {
    /*
     * Testing partition for get_tree_text
     *      subtrees: empty, nonempty
     *      subblobs: empty, nonempty
     */

    use super::*;

    #[test]
    fn all_empty() {
        let subtrees: HashMap<String, String> = HashMap::new();
        let subblobs: HashMap<String, String> = HashMap::new();
        let (tree_text, _) = get_tree_text_and_hash(&subtrees, &subblobs);
        assert_eq!("Trees\nBlobs", tree_text);
    }

    #[test]
    fn both_populated() {
        let subtrees: HashMap<String, String> = [("hello".to_string(), "world".to_string())]
            .iter()
            .cloned()
            .collect();
        let subblobs: HashMap<String, String> = [
            ("I".to_string(), "love".to_string()),
            ("rust".to_string(), "".to_string()),
        ]
        .iter()
        .cloned()
        .collect();
        let (tree_text, _) = get_tree_text_and_hash(&subtrees, &subblobs);
        assert!(
            "Trees\nhello: world\nBlobs\nI: love\nrust: ".to_string() == tree_text
                || "Trees\nhello: world\nBlobs\nrust: \nI: love".to_string() == tree_text
        ); // order isn't fixed by HashMaps, so we check if either is correct
    }
}
