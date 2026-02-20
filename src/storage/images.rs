use std::fs;
use std::path::PathBuf;

use crate::storage::path::{image_path_for_hash, images_dir};

pub fn save_image_bytes(hash: &str, bytes: &[u8]) -> anyhow::Result<PathBuf> {
    let dir = images_dir()?;
    fs::create_dir_all(&dir)?;
    let path = image_path_for_hash(hash)?;
    if !path.exists() {
        fs::write(&path, bytes)?;
    }
    Ok(path)
}
