pub mod link_metadata;
pub mod ocr;
pub mod types;
pub mod watcher;

#[cfg(target_os = "windows")]
pub mod windows;

pub use watcher::start_clipboard_history;
