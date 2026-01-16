mod app;
mod clipboard;
mod hotkeys;
mod migration;
mod storage;
mod ui;

#[cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
fn main() {
    app::run();
}
