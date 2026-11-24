#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod audio;
mod downloads;
mod weather;
mod lyrics;
mod media;
mod notif;
mod settings;
mod shellx;
mod shelf;
mod spotify;
mod stats;
mod sysmon;
mod uia;
mod util;
mod winutil;

use serde::Deserialize;
use settings::tr;
use tauri::image::Image;
use tauri::menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager, WebviewWindow};

#[derive(Deserialize, Clone, Copy, Debug)]
pub struct Rect { pub x: f64, pub y: f64, pub w: f64, pub h: f64 }

/* ---------- komutlar ---------- */
#[tauri::command]
fn media_cmd(cmd: String) {
    let ok = matches!(cmd.as_str(), "toggle" | "next" | "prev" | "shuffle" | "repeat")
        || (cmd.starts_with("seek ") && cmd[5..].parse::<f64>().is_ok())
        || (cmd.starts_with("focus ") && cmd.len() < 200
            && cmd[6..].chars().all(|c| c.is_ascii_alphanumeric() || "._!-{}\\".contains(c)));
    if ok { media::send(&cmd); }
}

#[tauri::command]
fn audio_cmd(cmd: String) {
    let ok = matches!(cmd.as_str(), "up" | "down" | "mute")
        || (cmd.starts_with("set ") && cmd[4..].parse::<f32>().map(|v| (0.0..=1.0).contains(&v)).unwrap_or(false));
    if ok { audio::send(&cmd); }
}

#[tauri::command]
fn set_rect(r: Rect) { winutil::set_rect(r); }

/// Sadece gelistirme (debug) derlemesinde: otomatik test raporu yazar
#[tauri::command]
fn test_report(_s: String) {
    #[cfg(debug_assertions)]
    if util::is_test() { if let Ok(p) = std::env::var("ISLAND_REPORT") { let _ = std::fs::write(p, _s); } }
}

#[tauri::command]
fn kbd_focus(window: WebviewWindow, on: bool) { winutil::kbd_focus(&window, on); }

/// Adaya sag tik menusu
#[tauri::command]
fn show_menu(app: AppHandle, window: WebviewWindow) -> Result<(), String> {
    let e = |x: tauri::Error| x.to_string();
    let menu = Menu::with_items(&app, &[
        &MenuItem::with_id(&app, "settings", tr("settings"), true, None::<&str>).map_err(e)?,
        &CheckMenuItem::with_id(&app, "autostart", tr("autostart"), true, winutil::autostart_enabled(), None::<&str>).map_err(e)?,
        &MenuItem::with_id(&app, "reload", tr("reload"), true, None::<&str>).map_err(e)?,
        &PredefinedMenuItem::separator(&app).map_err(e)?,
        &MenuItem::with_id(&app, "quit", tr("quit"), true, None::<&str>).map_err(e)?,
    ]).map_err(e)?;
    window.popup_menu(&menu).map_err(e)
}

fn on_menu(app: &AppHandle, id: &str) {
    match id {
        "settings" => settings::show_window(app),
        "autostart" => { winutil::set_autostart(!winutil::autostart_enabled()); settings::broadcast(); }
        "reload" => if let Some(w) = app.get_webview_window("main") { let _ = w.eval("location.reload()"); },
        "quit" => app.exit(0),
        other => shelf::on_menu(other),
    }
}

/* ---------- sistem tepsisi ---------- */
fn tray_menu(app: &AppHandle) -> tauri::Result<Menu<tauri::Wry>> {
    Menu::with_items(app, &[
        &MenuItem::with_id(app, "settings", tr("settings"), true, None::<&str>)?,
        &PredefinedMenuItem::separator(app)?,
        &MenuItem::with_id(app, "quit", tr("quit"), true, None::<&str>)?,
    ])
}

/// Dil degisince tepsi menusunu yeniden kur
pub fn rebuild_tray() {
    let app = util::app();
    if let (Some(tray), Ok(menu)) = (app.tray_by_id("island"), tray_menu(app)) { let _ = tray.set_menu(Some(menu)); }
}

fn setup_tray(app: &AppHandle) -> tauri::Result<()> {
    let icon = Image::from_bytes(include_bytes!("../icons/tray.png"))?;
    TrayIconBuilder::with_id("island")
        .icon(icon)
        .tooltip("Dynamic Island")
        .menu(&tray_menu(app)?)
        .show_menu_on_left_click(false)
        .on_tray_icon_event(|tray, ev| {
            // sol tik: Ayarlar; sag tik: menu
            if let TrayIconEvent::Click { button: MouseButton::Left, button_state: MouseButtonState::Up, .. } = ev {
                settings::show_window(tray.app_handle());
            }
        })
        .build(app)?;
    Ok(())
}

fn main() {
    // tek kopya (test modu haric)
    if !util::is_test() && !winutil::single_instance() { return; }

    // WebView2 GPU hizlandirma (ayara gore acik veya yazilimsal yedek)
    if std::env::var_os("WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS").is_none() {
        let args = if settings::gpu_accel_early() {
            "--enable-gpu-rasterization --enable-zero-copy --ignore-gpu-blocklist --enable-accelerated-2d-canvas"
        } else {
            "--disable-gpu --disable-gpu-compositing"
        };
        std::env::set_var("WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS", args);
    }

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            media_cmd, audio_cmd, set_rect, kbd_focus, show_menu, test_report,
            audio::switch_audio_output, sysmon::get_sysmon, stats::get_stats,
            weather::get_weather, downloads::download_open, notif::focus_notif_app,
            settings::get_settings, settings::set_setting, settings::open_settings, shelf::pick_storage_dir,
            shelf::shelf_get, shelf::shelf_add, shelf::shelf_remove, shelf::shelf_clear, shelf::shelf_open,
            shelf::shelf_menu, shelf::shelf_drag, shelf::shelf_copy, shelf::shelf_paste,
            spotify::sp_get, spotify::sp_like, spotify::sp_queue, spotify::sp_skipto, spotify::sp_connect,
            spotify::sp_disconnect, spotify::sp_search, spotify::sp_play, spotify::sp_enqueue,
            spotify::sp_devices, spotify::sp_transfer,
            lyrics::lyrics_get,
        ])
        .on_menu_event(|app, ev| on_menu(app, ev.id().as_ref()))
        .setup(|app| {
            util::set_app(app.handle().clone());
            settings::init();
            let win = app.get_webview_window("main").expect("main window");
            winutil::setup(&win);
            shelf::init();
            spotify::init();
            stats::init();
            media::start();
            notif::start();
            audio::start(!util::is_test());
            sysmon::start();
            weather::start();
            downloads::start();
            if let Err(e) = setup_tray(app.handle()) { eprintln!("tepsi simgesi kurulamadi: {e}"); }
            let _ = win.show();
            // Test modu (sadece gelistirme derlemesinde)
            #[cfg(debug_assertions)]
            if util::is_test() {
                if let Ok(path) = std::env::var("ISLAND_EVAL") {
                    let w = win.clone();
                    std::thread::spawn(move || {
                        std::thread::sleep(std::time::Duration::from_secs(6));
                        if let Ok(js) = std::fs::read_to_string(path) { let _ = w.eval(&js); }
                    });
                }
            }
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("Dynamic Island baslatilamadi")
        .run(|_app, ev| {
            // kapanista RAM'deki raf ogelerini (istenirse) diske kaydet
            if let tauri::RunEvent::Exit = ev { shelf::on_exit(); }
        });
}
