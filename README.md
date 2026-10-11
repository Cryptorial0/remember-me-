# Remember Me?  (Phase 1: desktop core)

## Windows prerequisites
1. Rust: https://rustup.rs  (then restart terminal)
2. Node.js LTS: https://nodejs.org
3. Visual Studio Build Tools -> "Desktop development with C++" workload
4. WebView2 runtime (preinstalled on Windows 10/11 updated)

## Run
    npm install
    npm run tauri icon app-icon.png     # generates src-tauri/icons (once)
    npm run tauri dev                   # test
    npm run tauri build                 # installer in src-tauri/target/release/bundle/

## Use
- Alt+X: open quick-add
- Close button hides to tray. Tray menu -> Quit to exit.
- Starts on login, hidden in tray.
- DB: reminders.db in the app data folder (com.rememberme.app).
