use windows_sys::Win32::Foundation::{CloseHandle, HWND};
use windows_sys::Win32::System::Threading::{
    OpenProcess, QueryFullProcessImageNameW, PROCESS_QUERY_LIMITED_INFORMATION,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    GetForegroundWindow, GetWindowTextLengthW, GetWindowTextW, GetWindowThreadProcessId,
};

pub fn active_window_source() -> (Option<String>, Option<String>) {
    let hwnd = unsafe { GetForegroundWindow() };
    if hwnd.is_null() {
        return (None, None);
    }

    let title = active_window_title(hwnd);
    let exe_path = active_window_exe_path(hwnd);
    (title, exe_path)
}

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

fn active_window_exe_path(hwnd: HWND) -> Option<String> {
    let mut pid: u32 = 0;
    unsafe {
        GetWindowThreadProcessId(hwnd, &mut pid);
    }
    if pid == 0 {
        return None;
    }

    let handle = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid) };
    if handle.is_null() {
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

pub fn summarize_file_paths(paths: &[String]) -> String {
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
