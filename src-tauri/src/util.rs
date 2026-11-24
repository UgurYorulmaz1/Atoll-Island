// Shared helpers: app handle, event emitting, data folder, shell commands
use std::path::PathBuf;
use std::sync::OnceLock;
use tauri::{AppHandle, Emitter, Manager};

static APP: OnceLock<AppHandle> = OnceLock::new();

pub fn set_app(app: AppHandle) { let _ = APP.set(app); }
pub fn app() -> &'static AppHandle { APP.get().expect("app not set") }

pub fn emit<S: serde::Serialize + Clone>(event: &str, payload: S) {
    if let Some(a) = APP.get() { let _ = a.emit(event, payload); }
}

pub fn is_test() -> bool { std::env::var("ISLAND_TEST").map(|v| v == "1").unwrap_or(false) }

/// %APPDATA%\com.island.dynamic (...-test in test mode)
pub fn data_dir() -> PathBuf {
    let base = app().path().app_data_dir().unwrap_or_else(|_| std::env::temp_dir().join("island"));
    let dir = if is_test() { PathBuf::from(format!("{}-test", base.display())) } else { base };
    let _ = std::fs::create_dir_all(&dir);
    dir
}

pub fn b64(data: &[u8]) -> String {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD.encode(data)
}

pub fn wide(s: &str) -> windows::core::HSTRING { windows::core::HSTRING::from(s) }

/// Open a file/URL with its default app
pub fn shell_open(target: &str) {
    use windows::core::w;
    use windows::Win32::UI::Shell::ShellExecuteW;
    use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;
    unsafe { let _ = ShellExecuteW(None, w!("open"), &wide(target), None, None, SW_SHOWNORMAL); }
}

/// Show the file selected in Explorer
pub fn show_in_folder(path: &str) {
    let _ = std::process::Command::new("explorer.exe").arg(format!("/select,{}", path)).spawn();
}

/// Windows file time (100 ns since 1601) -> unix ms
pub fn filetime_to_unix_ms(ut: i64) -> i64 { (ut - 116_444_736_000_000_000) / 10_000 }
