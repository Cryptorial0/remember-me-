mod api;
mod api_core;
mod db;

use db::{Db, Reminder};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Emitter, Manager, WebviewUrl, WebviewWindowBuilder, WindowEvent};

fn now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

fn show_main(app: &AppHandle) {
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.show();
        let _ = w.unminimize();
        let _ = w.set_focus();
        let _ = app.emit("quick-add", ());
    }
}

fn ask_quit(app: &AppHandle) {
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.show();
        let _ = w.unminimize();
        let _ = w.set_focus();
        let _ = app.emit("confirm-quit", ());
    }
}

fn enc(s: &str) -> String {
    let mut o = String::new();
    for b in s.bytes() {
        if b.is_ascii_alphanumeric() || b == b'-' || b == b'_' || b == b'.' {
            o.push(b as char);
        } else {
            o.push_str(&format!("%{:02X}", b));
        }
    }
    o
}

/// Right and bottom edge (physical pixels) of the primary monitor's work area,
/// i.e. the screen minus the taskbar wherever it is docked.
#[cfg(windows)]
fn work_area() -> Option<(f64, f64)> {
    #[repr(C)]
    struct Rect {
        left: i32,
        top: i32,
        right: i32,
        bottom: i32,
    }
    #[link(name = "user32")]
    extern "system" {
        fn SystemParametersInfoW(a: u32, b: u32, c: *mut std::ffi::c_void, d: u32) -> i32;
    }
    let mut r = Rect { left: 0, top: 0, right: 0, bottom: 0 };
    // SPI_GETWORKAREA = 0x0030
    let ok = unsafe { SystemParametersInfoW(0x0030, 0, &mut r as *mut Rect as *mut std::ffi::c_void, 0) };
    if ok != 0 && r.right > r.left && r.bottom > r.top {
        Some((r.right as f64, r.bottom as f64))
    } else {
        None
    }
}

#[cfg(not(windows))]
fn work_area() -> Option<(f64, f64)> {
    None
}

/// Small always-on-top popup in the bottom-right corner of the work area.
fn open_popup(app: &AppHandle, id: i64, title: String, link: String) {
    let handle = app.clone();
    let _ = app.run_on_main_thread(move || {
        let (w, h) = (360.0_f64, 130.0_f64);
        let n = handle
            .webview_windows()
            .keys()
            .filter(|k| k.starts_with("popup-"))
            .count() as f64;
        let label = format!("popup-{}-{}", id, now());
        let url = format!("popup.html?id={}&title={}&link={}", id, enc(&title), enc(&link));
        let (x, y) = match handle.primary_monitor() {
            Ok(Some(m)) => {
                let sc = m.scale_factor();
                let p = m.position();
                let sz = m.size();
                let (right, bottom) = work_area().unwrap_or((
                    p.x as f64 + sz.width as f64,
                    p.y as f64 + sz.height as f64 - 64.0 * sc,
                ));
                (right / sc - w - 4.0, bottom / sc - h - 8.0 - n * (h + 6.0))
            }
            _ => (12.0, 500.0),
        };
        #[allow(unused_mut)]
        let mut b = WebviewWindowBuilder::new(&handle, label, WebviewUrl::App(url.into()))
            .title("Remember Me?")
            .inner_size(w, h)
            .position(x, y)
            .decorations(false)
            .transparent(true)
            .shadow(false)
            .always_on_top(true)
            .skip_taskbar(true)
            .resizable(false)
            .focused(false);
        #[cfg(windows)]
        {
            b = b.additional_browser_args(
                "--autoplay-policy=no-user-gesture-required --disable-features=msWebOOUI,msPdfOOUI,msSmartScreenProtection",
            );
        }
        if let Ok(win) = b.build() {
            // Re-assert "always on top" a few times: a fullscreen app that
            // activates right after the popup appears can push it behind.
            let _ = win.set_always_on_top(true);
            std::thread::spawn(move || {
                for _ in 0..4 {
                    std::thread::sleep(Duration::from_millis(500));
                    let _ = win.set_always_on_top(true);
                }
            });
        }
    });
}

#[tauri::command]
fn add_reminder(
    db: tauri::State<Db>,
    title: String,
    due: i64,
    repeat: String,
    link: Option<String>,
) -> Result<(), String> {
    db.add_with_link(title.trim(), due, &repeat, &link.unwrap_or_default())
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn open_link(url: String) -> Result<(), String> {
    let u = db::clean_link(&url);
    if u.is_empty() {
        return Err("not a valid http(s) link".into());
    }
    #[cfg(windows)]
    let r = std::process::Command::new("explorer.exe").arg(&u).spawn();
    #[cfg(not(windows))]
    let r = std::process::Command::new("xdg-open").arg(&u).spawn();
    r.map(|_| ()).map_err(|e| e.to_string())
}

#[tauri::command]
fn update_reminder(
    db: tauri::State<Db>,
    id: i64,
    title: String,
    due: i64,
    repeat: String,
    link: Option<String>,
) -> Result<(), String> {
    db.update(id, title.trim(), due, &repeat).map_err(|e| e.to_string())?;
    if let Some(l) = link {
        db.set_link(id, &l).map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
fn list_reminders(db: tauri::State<Db>) -> Result<Vec<Reminder>, String> {
    db.list().map_err(|e| e.to_string())
}

#[tauri::command]
fn delete_reminder(db: tauri::State<Db>, id: i64) -> Result<(), String> {
    db.delete(id).map_err(|e| e.to_string())
}

#[tauri::command]
fn toggle_done(app: AppHandle, db: tauri::State<Db>, id: i64) -> Result<(), String> {
    db.toggle_done(id).map_err(|e| e.to_string())?;
    let _ = app.emit("changed", ());
    Ok(())
}

#[tauri::command]
fn snooze(app: AppHandle, db: tauri::State<Db>, id: i64, minutes: i64) -> Result<(), String> {
    db.snooze(id, now() + minutes * 60).map_err(|e| e.to_string())?;
    let _ = app.emit("changed", ());
    Ok(())
}

#[tauri::command]
fn close_popup(window: tauri::Window) {
    let _ = window.destroy();
}

#[tauri::command]
fn get_settings(db: tauri::State<Db>) -> Result<std::collections::HashMap<String, String>, String> {
    let rows = db.settings().map_err(|e| e.to_string())?;
    Ok(rows.into_iter().filter(|(k, _)| k != "custom_sound").collect())
}

#[tauri::command]
fn set_setting(db: tauri::State<Db>, key: String, value: String) -> Result<(), String> {
    const OK: [&str; 5] = ["sound", "volume", "repeats", "custom_sound", "custom_name"];
    if !OK.contains(&key.as_str()) || value.len() > 3_000_000 {
        return Err("rejected".into());
    }
    db.set_setting(&key, &value).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_custom_sound(db: tauri::State<Db>) -> Result<Option<String>, String> {
    db.get_setting("custom_sound").map_err(|e| e.to_string())
}

#[tauri::command]
fn set_autostart(app: AppHandle, db: tauri::State<Db>, enabled: bool) -> Result<(), String> {
    use tauri_plugin_autostart::ManagerExt;
    db.set_setting("autostart", if enabled { "1" } else { "0" })
        .map_err(|e| e.to_string())?;
    let a = app.autolaunch();
    let r = if enabled { a.enable() } else { a.disable() };
    r.map_err(|e| e.to_string())
}

#[tauri::command]
fn set_hotkey(app: AppHandle, db: tauri::State<Db>, hotkey: String) -> Result<(), String> {
    use std::str::FromStr;
    use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};

    let lower = hotkey.to_lowercase();
    if !(lower.contains("ctrl") || lower.contains("alt")) {
        return Err("Include Ctrl or Alt.".into());
    }
    let new = Shortcut::from_str(&hotkey).map_err(|e| e.to_string())?;
    let gs = app.global_shortcut();
    let old = db
        .get_setting("hotkey")
        .ok()
        .flatten()
        .unwrap_or_else(|| "Alt+X".into());
    let old_sc = Shortcut::from_str(&old).ok();
    if let Some(o) = old_sc.clone() {
        let _ = gs.unregister(o);
    }
    if let Err(e) = gs.register(new) {
        // Keep the old hotkey working if the new one is taken.
        if let Some(o) = old_sc {
            let _ = gs.register(o);
        }
        return Err(format!("{} (probably used by another app)", e));
    }
    db.set_setting("hotkey", &hotkey).map_err(|e| e.to_string())
}

#[tauri::command]
fn quit_app(app: AppHandle) {
    app.exit(0);
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mut builder = tauri::Builder::default();

    #[cfg(desktop)]
    {
        use tauri_plugin_autostart::MacosLauncher;
        use tauri_plugin_global_shortcut::ShortcutState;
        builder = builder
            .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
                show_main(app)
            }))
            .plugin(tauri_plugin_autostart::init(
                MacosLauncher::LaunchAgent,
                Some(vec!["--hidden"]),
            ))
            .plugin(
                tauri_plugin_global_shortcut::Builder::new()
                    .with_handler(|app, _shortcut, event| {
                        if event.state() == ShortcutState::Pressed {
                            show_main(app);
                        }
                    })
                    .build(),
            );
    }

    builder
        .invoke_handler(tauri::generate_handler![
            add_reminder,
            open_link,
            update_reminder,
            list_reminders,
            delete_reminder,
            toggle_done,
            snooze,
            close_popup,
            get_settings,
            set_setting,
            get_custom_sound,
            set_autostart,
            set_hotkey,
            quit_app
        ])
        .on_window_event(|window, event| {
            // Close button hides to tray instead of quitting.
            if window.label() == "main" {
                if let WindowEvent::CloseRequested { api, .. } = event {
                    api.prevent_close();
                    let _ = window.hide();
                }
            }
        })
        .setup(|app| {
            let dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&dir)?;
            app.manage(Db::open(&dir.join("reminders.db"))?);
            api::start(app.handle().clone());

            #[cfg(desktop)]
            {
                use std::str::FromStr;
                use tauri::{
                    menu::{Menu, MenuItem},
                    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
                };
                use tauri_plugin_autostart::ManagerExt;
                use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut};

                // Start on login (re-registered every launch so the path stays current).
                let want = app
                    .state::<Db>()
                    .get_setting("autostart")
                    .ok()
                    .flatten()
                    .map(|v| v != "0")
                    .unwrap_or(true);
                let auto = app.autolaunch();
                if want {
                    let _ = auto.enable();
                } else {
                    let _ = auto.disable();
                }

                // Global hotkey from settings (default Alt+X). May fail silently on Wayland.
                let hk = app
                    .state::<Db>()
                    .get_setting("hotkey")
                    .ok()
                    .flatten()
                    .unwrap_or_else(|| "Alt+X".into());
                let sc = Shortcut::from_str(&hk)
                    .unwrap_or_else(|_| Shortcut::new(Some(Modifiers::ALT), Code::KeyX));
                let _ = app.global_shortcut().register(sc);

                // Tray.
                let show = MenuItem::with_id(app, "show", "Open Remember Me?", true, None::<&str>)?;
                let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
                let menu = Menu::with_items(app, &[&show, &quit])?;
                TrayIconBuilder::new()
                    .icon(app.default_window_icon().unwrap().clone())
                    .tooltip("Remember Me?")
                    .menu(&menu)
                    .on_menu_event(|app, ev| match ev.id.as_ref() {
                        "show" => show_main(app),
                        "quit" => ask_quit(app),
                        _ => {}
                    })
                    .on_tray_icon_event(|tray, ev| {
                        if let TrayIconEvent::Click {
                            button: MouseButton::Left,
                            button_state: MouseButtonState::Up,
                            ..
                        } = ev
                        {
                            show_main(tray.app_handle());
                        }
                    })
                    .build(app)?;

                // Launched by autostart with --hidden: stay in tray.
                if !std::env::args().any(|a| a == "--hidden") {
                    show_main(app.handle());
                }
            }
            #[cfg(not(desktop))]
            show_main(app.handle());

            // Scheduler: checks every 5s and opens bottom-right popups.
            // It also re-asserts "always on top" for open popups, because Windows
            // can push them behind when another app goes fullscreen.
            let handle = app.handle().clone();
            std::thread::spawn(move || loop {
                let due = handle.state::<Db>().take_due(now());
                if let Ok(items) = due {
                    if !items.is_empty() {
                        for (id, title) in items {
                            let link = handle.state::<Db>().get_link(id).unwrap_or_default();
                            open_popup(&handle, id, title, link);
                        }
                        let _ = handle.emit("changed", ());
                    }
                }
                for (label, w) in handle.webview_windows() {
                    if label.starts_with("popup-") {
                        let _ = w.set_always_on_top(true);
                    }
                }
                std::thread::sleep(Duration::from_secs(5));
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Remember Me?");
}
