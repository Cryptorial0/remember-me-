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

/// Small always-on-top popup at the bottom-left of the primary monitor.
fn open_popup(app: &AppHandle, id: i64, title: String) {
    let handle = app.clone();
    let _ = app.run_on_main_thread(move || {
        let (w, h) = (360.0_f64, 130.0_f64);
        let n = handle
            .webview_windows()
            .keys()
            .filter(|k| k.starts_with("popup-"))
            .count() as f64;
        let label = format!("popup-{}-{}", id, now());
        let url = format!("popup.html?id={}&title={}", id, enc(&title));
        let (x, y) = match handle.primary_monitor() {
            Ok(Some(m)) => {
                let sc = m.scale_factor();
                let p = m.position();
                let sz = m.size();
                (
                    p.x as f64 / sc + sz.width as f64 / sc - w - 4.0,
                    (p.y as f64 + sz.height as f64) / sc - h - 64.0 - n * (h + 6.0),
                )
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
        let _ = b.build();
    });
}

#[tauri::command]
fn add_reminder(db: tauri::State<Db>, title: String, due: i64, repeat: String) -> Result<(), String> {
    db.add(title.trim(), due, &repeat).map_err(|e| e.to_string())
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
fn close_popup(window: tauri::Window) {
    let _ = window.destroy();
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
            list_reminders,
            delete_reminder,
            toggle_done,
            snooze,
            close_popup,
            get_settings,
            set_setting,
            get_custom_sound,
            set_autostart
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

            #[cfg(desktop)]
            {
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

                // Global hotkey: Alt+X (may fail silently on Wayland).
                let _ = app.global_shortcut().register(Shortcut::new(
                    Some(Modifiers::ALT),
                    Code::KeyX,
                ));

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
                        "quit" => app.exit(0),
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

            // Scheduler: checks every 5s, opens bottom-left popups.
            let handle = app.handle().clone();
            std::thread::spawn(move || loop {
                let due = handle.state::<Db>().take_due(now());
                if let Ok(items) = due {
                    if !items.is_empty() {
                        for (id, title) in items {
                            open_popup(&handle, id, title);
                        }
                        let _ = handle.emit("changed", ());
                    }
                }
                std::thread::sleep(Duration::from_secs(5));
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Remember Me?");
}
