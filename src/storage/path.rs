use std::path::{Path, PathBuf};

fn local_data_dir() -> anyhow::Result<PathBuf> {
    let base = std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    Ok(base.join("gpui-clipboard-manager"))
}

pub fn default_db_path() -> anyhow::Result<PathBuf> {
    Ok(local_data_dir()?.join("clipboard_history.db"))
}

pub fn images_dir() -> anyhow::Result<PathBuf> {
    Ok(local_data_dir()?.join("clipboard_images"))
}

pub fn image_path_for_hash(hash: &str) -> anyhow::Result<PathBuf> {
    Ok(images_dir()?.join(format!("image_{hash}.bmp")))
}
