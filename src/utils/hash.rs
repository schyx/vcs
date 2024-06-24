extern crate sha2;

use sha2::{Digest, Sha256};

pub fn sha2(string: String) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(string.as_bytes());
    let result = hasher.finalize();
    result.into()
}
