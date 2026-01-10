use gpui::App;
use std::time::Duration;

#[cfg(target_os = "windows")]
use clipboard_win::{Clipboard, Format, Getter, formats};
#[cfg(target_os = "windows")]
use sha2::{Digest, Sha256};
#[cfg(target_os = "windows")]
use windows_sys::Win32::Foundation::{CloseHandle, HWND};
#[cfg(target_os = "windows")]
use windows_sys::Win32::System::Threading::{
    OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION, QueryFullProcessImageNameW,
};
#[cfg(target_os = "windows")]
use windows_sys::Win32::UI::WindowsAndMessaging::{
    GetForegroundWindow, GetWindowTextLengthW, GetWindowTextW, GetWindowThreadProcessId,
};

#[cfg(target_os = "windows")]
use crate::storage::history::{insert_clipboard_entry, load_last_hash, open_db};
#[cfg(target_os = "windows")]
use crate::storage::images::save_image_bytes;
#[cfg(target_os = "windows")]
use crate::storage::path::default_db_path;

pub fn start_clipboard_history(cx: &mut App) {
    #[cfg(target_os = "windows")]
    {
        if let Err(err) = start_windows_clipboard_history(cx) {
            eprintln!("Failed to start clipboard history: {err}");
        }
    }
}

#[cfg(target_os = "windows")]
fn start_windows_clipboard_history(cx: &mut App) -> anyhow::Result<()> {
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
            match read_clipboard_entry() {
                Ok(Some(entry)) => {
                    if last_hash.as_deref() != Some(entry.content_hash.as_str()) {
                        if let Err(err) = insert_clipboard_entry(
                            &db,
                            &entry.content_type,
                            &entry.content_hash,
                            &entry.content,
                            entry.text_content.as_deref(),
                            entry.image_path.as_deref(),
                            entry.file_paths.as_deref(),
                            entry.source_app_title.as_deref(),
                            entry.source_exe_path.as_deref(),
                        )
                        .await
                        {
                            eprintln!("Failed to write clipboard entry: {err}");
                        } else {
                            last_hash = Some(entry.content_hash);
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
struct ClipboardEntry {
    content_type: String,
    content_hash: String,
    content: String,
    text_content: Option<String>,
    image_path: Option<String>,
    file_paths: Option<String>,
    source_app_title: Option<String>,
    source_exe_path: Option<String>,
}

#[cfg(target_os = "windows")]
fn read_clipboard_entry() -> anyhow::Result<Option<ClipboardEntry>> {
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
            return Ok(Some(build_entry(
                "files",
                content_hash,
                content,
                None,
                None,
                Some(file_paths),
            )));
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
            return Ok(Some(build_entry(
                "image",
                content_hash,
                "Image".to_string(),
                None,
                Some(image_path.to_string_lossy().to_string()),
                None,
            )));
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
            return Ok(Some(build_entry(
                "text",
                content_hash,
                trimmed.to_string(),
                Some(trimmed.to_string()),
                None,
                None,
            )));
        }
    }

    Ok(None)
}

#[cfg(target_os = "windows")]
fn build_entry(
    content_type: &str,
    content_hash: String,
    content: String,
    text_content: Option<String>,
    image_path: Option<String>,
    file_paths: Option<String>,
) -> ClipboardEntry {
    let (source_app_title, source_exe_path) = active_window_source();
    ClipboardEntry {
        content_type: content_type.to_string(),
        content_hash,
        content,
        text_content,
        image_path,
        file_paths,
        source_app_title,
        source_exe_path,
    }
}

#[cfg(target_os = "windows")]
fn active_window_source() -> (Option<String>, Option<String>) {
    let hwnd = unsafe { GetForegroundWindow() };
    if hwnd == std::ptr::null_mut() {
        return (None, None);
    }

    let title = active_window_title(hwnd);
    let exe_path = active_window_exe_path(hwnd);
    (title, exe_path)
}

#[cfg(target_os = "windows")]
fn active_window_title(hwnd: HWND) -> Option<String> {
    let len = unsafe { GetWindowTextLengthW(hwnd) };
    if len <= 0 {
        return None;
    }

    let mut buffer = vec![0u16; (len + 1) as usize];
    let copied = unsafe { GetWindowTextW(hwnd, buffer.as_mut_ptr(), buffer.len() as i32) };
    if copied <= 0 {
        return None;
    }

    buffer.truncate(copied as usize);
    let title = String::from_utf16_lossy(&buffer);
    let trimmed = title.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

#[cfg(target_os = "windows")]
fn active_window_exe_path(hwnd: HWND) -> Option<String> {
    let mut pid: u32 = 0;
    unsafe {
        GetWindowThreadProcessId(hwnd, &mut pid);
    }
    if pid == 0 {
        return None;
    }

    let handle = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid) };
    if handle == std::ptr::null_mut() {
        return None;
    }

    let mut buffer = vec![0u16; 1024];
    let mut len = buffer.len() as u32;
    let result = unsafe { QueryFullProcessImageNameW(handle, 0, buffer.as_mut_ptr(), &mut len) };
    unsafe {
        CloseHandle(handle);
    }
    if result == 0 || len == 0 {
        return None;
    }

    buffer.truncate(len as usize);
    let path = String::from_utf16_lossy(&buffer);
    let trimmed = path.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

#[cfg(target_os = "windows")]
fn summarize_file_paths(paths: &[String]) -> String {
    let mut names: Vec<String> = Vec::with_capacity(paths.len());
    for path in paths {
        let name = std::path::Path::new(path)
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or(path);
        names.push(name.to_string());
    }

    let summary = names.join(", ");
    const MAX_LEN: usize = 500;
    if summary.len() <= MAX_LEN {
        summary
    } else {
        let mut truncated = summary[..MAX_LEN].to_string();
        truncated.push_str("...");
        truncated
    }
}

#[cfg(target_os = "windows")]
fn hash_bytes(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let digest = hasher.finalize();
    let mut output = String::with_capacity(digest.len() * 2);
    for byte in digest {
        use std::fmt::Write;
        let _ = write!(&mut output, "{:02x}", byte);
    }
    output
}
