use std::path::{Path, PathBuf};

pub fn default_db_path() -> anyhow::Result<PathBuf> {
    let exe_path = std::env::current_exe()?;
    Ok(exe_path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("clipboard_history.db"))
}

pub fn images_dir() -> anyhow::Result<PathBuf> {
    let exe_path = std::env::current_exe()?;
    Ok(exe_path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("clipboard_images"))
}

pub fn image_path_for_hash(hash: &str) -> anyhow::Result<PathBuf> {
    Ok(images_dir()?.join(format!("image_{hash}.bmp")))
}
