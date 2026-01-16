pub mod link_metadata;
pub mod ocr;
pub mod watcher;

pub use watcher::start_clipboard_history;

#[cfg(target_os = "windows")]
pub use watcher::ClipboardEntry;
