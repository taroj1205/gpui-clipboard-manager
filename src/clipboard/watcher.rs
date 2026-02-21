use gpui::App;
use std::time::Duration;

#[cfg(target_os = "windows")]
use crate::clipboard::link_metadata::{fetch_link_metadata, parse_link_url, LinkMetadata};
#[cfg(target_os = "windows")]
use crate::clipboard::ocr::extract_text_from_image;
#[cfg(target_os = "windows")]
use crate::clipboard::types::{ClipboardEntry, ClipboardEntryInput};
#[cfg(target_os = "windows")]
use crate::clipboard::windows::{active_window_source, summarize_file_paths};
#[cfg(target_os = "windows")]
use crate::storage::history::{
    insert_clipboard_entry, load_last_hash, open_db,
    ClipboardEntryInput as StorageClipboardEntryInput,
};
#[cfg(target_os = "windows")]
use crate::storage::images::save_image_bytes;
#[cfg(target_os = "windows")]
use crate::storage::path::default_db_path;
#[cfg(target_os = "windows")]
use crate::utils::hash_bytes;
#[cfg(target_os = "windows")]
use clipboard_win::{formats, Clipboard, Format, Getter};
use std::sync::mpsc::Sender;
use std::sync::{Mutex, OnceLock};

static IGNORE_HASH: OnceLock<Mutex<Option<String>>> = OnceLock::new();

pub fn ignore_next_hash(hash: String) {
    let lock = IGNORE_HASH.get_or_init(|| Mutex::new(None));
    if let Ok(mut guard) = lock.lock() {
        *guard = Some(hash);
    }
}

pub fn start_clipboard_history(cx: &mut App, update_tx: Sender<()>) {
    #[cfg(not(target_os = "windows"))]
    let _ = update_tx;

    #[cfg(target_os = "windows")]
    {
        if let Err(err) = start_windows_clipboard_history(cx, update_tx) {
            eprintln!("Failed to start clipboard history: {err}");
        }
    }
}

#[cfg(target_os = "windows")]
fn start_windows_clipboard_history(cx: &mut App, update_tx: Sender<()>) -> anyhow::Result<()> {
    let db_path = default_db_path()?;

    cx.spawn(async move |cx| {
        let db = match open_db(&db_path).await {
            Ok(db) => db,
            Err(err) => {
                eprintln!("Failed to open clipboard database: {err}");
                return;
            }
        };
        let mut last_hash = match load_last_hash(&db).await {
            Ok(hash) => hash,
            Err(err) => {
                eprintln!("Failed to load clipboard history: {err}");
                None
            }
        };

        loop {
            match read_clipboard_entry().await {
                Ok(Some(entry)) => {
                    if last_hash.as_deref() != Some(entry.content_hash.as_str()) {
                        let ignore_hash = IGNORE_HASH
                            .get_or_init(|| Mutex::new(None))
                            .lock()
                            .ok()
                            .and_then(|mut guard| guard.take());

                        if ignore_hash.as_deref() == Some(entry.content_hash.as_str()) {
                            last_hash = Some(entry.content_hash);
                        } else if let Err(err) = insert_clipboard_entry(
                            &db,
                            StorageClipboardEntryInput {
                                content_type: &entry.content_type,
                                content_hash: &entry.content_hash,
                                content: &entry.content,
                                text_content: entry.text_content.as_deref(),
                                ocr_text: entry.ocr_text.as_deref(),
                                image_path: entry.image_path.as_deref(),
                                file_paths: entry.file_paths.as_deref(),
                                link_url: entry.link_url.as_deref(),
                                link_title: entry.link_title.as_deref(),
                                link_description: entry.link_description.as_deref(),
                                link_site_name: entry.link_site_name.as_deref(),
                                source_app_title: entry.source_app_title.as_deref(),
                                source_exe_path: entry.source_exe_path.as_deref(),
                            },
                        )
                        .await
                        {
                            eprintln!("Failed to write clipboard entry: {err}");
                        } else {
                            last_hash = Some(entry.content_hash);
                            let _ = update_tx.send(());
                        }
                    }
                }
                Ok(None) => {}
                Err(err) => {
                    eprintln!("Failed to read clipboard: {err}");
                }
            }

            cx.background_executor()
                .timer(Duration::from_millis(400))
                .await;
        }
    })
    .detach();

    Ok(())
}

#[cfg(target_os = "windows")]
async fn read_clipboard_entry() -> anyhow::Result<Option<ClipboardEntry>> {
    let _clip = Clipboard::new_attempts(10)
        .map_err(|err| anyhow::anyhow!("Clipboard open failed: {err}"))?;

    if formats::FileList.is_format_avail() {
        let mut files: Vec<String> = Vec::new();
        let _ = formats::FileList
            .read_clipboard(&mut files)
            .map_err(|err| anyhow::anyhow!("Clipboard read failed: {err}"))?;
        if !files.is_empty() {
            let file_paths = serde_json::to_string(&files)?;
            let content_hash = hash_bytes(file_paths.as_bytes());
            let content = summarize_file_paths(&files);
            return Ok(Some(build_entry(ClipboardEntryInput {
                content_type: "files".to_string(),
                content_hash,
                content,
                text_content: None,
                ocr_text: None,
                image_path: None,
                file_paths: Some(file_paths),
                link_metadata: None,
            })));
        }
    }

    if formats::Bitmap.is_format_avail() {
        let mut bytes: Vec<u8> = Vec::new();
        let _ = formats::Bitmap
            .read_clipboard(&mut bytes)
            .map_err(|err| anyhow::anyhow!("Clipboard read failed: {err}"))?;
        if !bytes.is_empty() {
            let content_hash = hash_bytes(&bytes);
            let image_path = save_image_bytes(&content_hash, &bytes)?;
            let ocr_text = match extract_text_from_image(&bytes).await {
                Ok(text) => text,
                Err(err) => {
                    eprintln!("Failed to OCR image: {err}");
                    None
                }
            };
            return Ok(Some(build_entry(ClipboardEntryInput {
                content_type: "image".to_string(),
                content_hash,
                content: "Image".to_string(),
                text_content: None,
                ocr_text,
                image_path: Some(image_path.to_string_lossy().to_string()),
                file_paths: None,
                link_metadata: None,
            })));
        }
    }

    if formats::Unicode.is_format_avail() {
        let mut text = String::new();
        let _ = formats::Unicode
            .read_clipboard(&mut text)
            .map_err(|err| anyhow::anyhow!("Clipboard read failed: {err}"))?;
        let trimmed = text.trim();
        if !trimmed.is_empty() {
            let content_hash = hash_bytes(trimmed.as_bytes());
            if let Some(url) = parse_link_url(trimmed) {
                let mut link_metadata = match fetch_link_metadata(&url).await {
                    Ok(metadata) => metadata,
                    Err(err) => {
                        eprintln!("Failed to fetch link metadata: {err}");
                        None
                    }
                };
                if link_metadata.is_none() {
                    link_metadata = Some(LinkMetadata {
                        url: url.to_string(),
                        title: None,
                        description: None,
                        site_name: None,
                    });
                }
                return Ok(Some(build_entry(ClipboardEntryInput {
                    content_type: "link".to_string(),
                    content_hash,
                    content: trimmed.to_string(),
                    text_content: Some(trimmed.to_string()),
                    ocr_text: None,
                    image_path: None,
                    file_paths: None,
                    link_metadata,
                })));
            }
            return Ok(Some(build_entry(ClipboardEntryInput {
                content_type: "text".to_string(),
                content_hash,
                content: trimmed.to_string(),
                text_content: Some(trimmed.to_string()),
                ocr_text: None,
                image_path: None,
                file_paths: None,
                link_metadata: None,
            })));
        }
    }

    Ok(None)
}

#[cfg(target_os = "windows")]
fn build_entry(input: ClipboardEntryInput) -> ClipboardEntry {
    let (source_app_title, source_exe_path) = active_window_source();
    let mut entry = ClipboardEntry::from(input);
    entry.source_app_title = source_app_title;
    entry.source_exe_path = source_exe_path;
    entry
}
