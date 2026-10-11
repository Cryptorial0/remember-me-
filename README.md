# Remember Me?  (desktop core)

## Easiest (Windows)
Double-click setup.bat (run once)
It installs Node.js, Rust, VS Build Tools (C++) and WebView2 with winget, builds
the app, and creates Remember-Me-Setup.exe in this folder. Run that exe to install.
Accept the admin (UAC) prompts. First run takes a long time (several GB).

## Manual (Windows)
Prerequisites: Rust (rustup.rs), Node.js LTS, VS Build Tools "Desktop development with C++", WebView2.

    npm install
    npm run tauri icon app-icon.png     # once
    npm run tauri dev                   # test
    npm run tauri build                 # installer in src-tauri/target/release/bundle/

## Use
- Alt+X: open quick-add
- Settings panel: sound, volume, repeats, own sound file, start-with-computer
- Close button hides to tray. Tray menu -> Quit to exit.
- Starts on login, hidden in tray.
- DB: reminders.db in the app data folder (com.rememberme.app).
