extern crate hex;
extern crate sha2;

use hex::encode;
use sha2::{Digest, Sha256};

pub fn sha2(string: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(string.as_bytes());
    let result = hasher.finalize();
    let byte_arr: [u8; 32] = result.into();
    encode(&byte_arr)
}
