use sha2::{Digest, Sha256};
use std::fmt::Write;

pub fn hash_bytes(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let digest = hasher.finalize();
    let mut output = String::with_capacity(digest.len() * 2);
    for byte in digest {
        let _ = write!(&mut output, "{:02x}", byte);
    }
    output
}

pub fn hash_string(s: &str) -> String {
    hash_bytes(s.as_bytes())
}
