mod db;

use db::{Db, Reminder};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Emitter, Manager, WindowEvent};
use tauri_plugin_notification::NotificationExt;

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
fn toggle_done(db: tauri::State<Db>, id: i64) -> Result<(), String> {
    db.toggle_done(id).map_err(|e| e.to_string())
}

#[tauri::command]
fn snooze(db: tauri::State<Db>, id: i64, minutes: i64) -> Result<(), String> {
    db.snooze(id, now() + minutes * 60).map_err(|e| e.to_string())
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
        .plugin(tauri_plugin_notification::init())
        .invoke_handler(tauri::generate_handler![
            add_reminder,
            list_reminders,
            delete_reminder,
            toggle_done,
            snooze
        ])
        .on_window_event(|window, event| {
            // Close button hides to tray instead of quitting.
            if let WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
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

                // Start on login.
                let auto = app.autolaunch();
                if !auto.is_enabled().unwrap_or(false) {
                    let _ = auto.enable();
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

            // Scheduler: checks every 5s, fires native notifications.
            let handle = app.handle().clone();
            std::thread::spawn(move || loop {
                let due = handle.state::<Db>().take_due(now());
                if let Ok(items) = due {
                    if !items.is_empty() {
                        for (_, title) in items {
                            let _ = handle
                                .notification()
                                .builder()
                                .title("Remember Me?")
                                .body(title)
                                .show();
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
