#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod app;
mod clipboard;
mod hotkeys;
mod migration;
mod storage;
mod ui;
mod utils;

fn main() {
    app::run();
}
