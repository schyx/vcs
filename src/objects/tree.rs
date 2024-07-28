use std::{collections::HashMap, io::Error};

use crate::{objects::write_object, utils::hash::sha2};

use super::get_object_contents;

pub const EMPTY_TREE_HASH: &str =
    "c26c7c45d0bbe8f237fa087485e47bffd26e0a93e1cb14caf8711169014262fe";

/// Given the subtrees and subblobs, outputs the text and hash of the tree object, respectively
fn get_tree_text_and_hash(subtrees: &Vec<String>, subblobs: &Vec<String>) -> (String, String) {
    let mut output = String::from("Trees\n");
    for line in subtrees {
        output.push_str(line);
        output.push_str("\n");
    }

    output.push_str("Blobs");
    for line in subblobs {
        output.push_str("\n");
        output.push_str(line);
    }
    (output.clone(), sha2(&output))
}

fn serialize_tree(tree_contents: &str) -> HashMap<&str, &str> {
    let mut tree = HashMap::new();
    for line in tree_contents.split('\n') {
        if line == "Trees" || line == "Blobs" {
            continue;
        }
        let split_line: Vec<&str> = line.split(": ").collect();
        let (object_name, object_hash) = (split_line[0], split_line[1]);
        tree.insert(object_name, object_hash);
    }
    tree
}

/// Returns the hash of a tree with subtrees and subblobs. Also creates the tree object
pub fn write_tree(subtrees: &Vec<String>, subblobs: &Vec<String>) -> String {
    let (tree_text, tree_hash) = get_tree_text_and_hash(subtrees, subblobs);
    let _ = write_object(&tree_hash, &tree_text);
    tree_hash
}

/// Returns the hash of `filename` in the tree given by `tree_hash`, or `DNE` if hash doesn't exist
///
/// Throws an error if `tree_hash` is not a valid tree
pub fn find_file_in_tree(tree_hash: &str, filename: &str) -> Result<String, Error> {
    let tree_contents = get_object_contents(tree_hash)?;
    let serialized_tree = serialize_tree(&tree_contents);
    if filename.contains('/') {
        let mut parts = filename.splitn(2, '/');
        let first_part = parts.next();
        let second_part = parts.next();
        match (first_part, second_part) {
            (Some(parent), Some(subpath)) => {
                if let Some(hash) = serialized_tree.get(parent) {
                    find_file_in_tree(hash, subpath)
                } else {
                    Ok(String::from("DNE"))
                }
            }
            _ => panic!("Expected strings for both parts!"),
        }
    } else {
        if let Some(hash) = serialized_tree.get(filename) {
            Ok((*hash).to_string())
        } else {
            Ok(String::from("DNE"))
        }
    }
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
        let subtrees: Vec<String> = vec![];
        let subblobs: Vec<String> = vec![];
        let (tree_text, tree_hash) = get_tree_text_and_hash(&subtrees, &subblobs);
        assert_eq!("Trees\nBlobs", tree_text);
        assert_eq!(EMPTY_TREE_HASH, tree_hash);
    }

    #[test]
    fn both_populated() {
        let subtrees: Vec<String> = [(String::from("hello: world"))].iter().cloned().collect();
        let subblobs: Vec<String> = [(String::from("I: love")), (String::from("rust: "))]
            .iter()
            .cloned()
            .collect();
        let (tree_text, _) = get_tree_text_and_hash(&subtrees, &subblobs);
        assert!(
            String::from("Trees\nhello: world\nBlobs\nI: love\nrust: ") == tree_text
                || String::from("Trees\nhello: world\nBlobs\nrust: \nI: love") == tree_text
        ); // order isn't fixed by HashMaps, so we check if either is correct
    }
}
