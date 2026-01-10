mod app;
mod clipboard;
mod hotkeys;
mod migration;
mod storage;
mod ui;

fn main() {
    load_env_from_exe_dir();
    app::run();
}

fn load_env_from_exe_dir() {
    let exe_path = match std::env::current_exe() {
        Ok(path) => path,
        Err(err) => {
            eprintln!("Failed to resolve executable path: {err}");
            return;
        }
    };
    let env_path = exe_path
        .parent()
        .unwrap_or_else(|| std::path::Path::new("."))
        .join(".env");
    if !env_path.exists() {
        return;
    }
    if let Err(err) = dotenvy::from_path(&env_path) {
        eprintln!("Failed to load .env file: {err}");
    }
}
