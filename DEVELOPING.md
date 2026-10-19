# Developer notes (not needed to use the app)

## Build on Windows
Double-click `setup.bat`. It installs Node.js, Rust and the minimal C++ build tools with winget, builds the app, and creates `Remember-Me-Setup.exe` in this folder.

Manual way (tools already installed):
```
npm install
npm run tauri icon app-icon.png
npm run tauri dev       # test
npm run tauri build     # installers in src-tauri/target/release/bundle/
```

## Make a release
1. Raise the version in `src-tauri/tauri.conf.json` and `src-tauri/Cargo.toml` (and `package.json`). If the browser extension changed, also in `browser-extension/manifests/chrome.json` and `firefox.json`, then run `node build.mjs` inside `browser-extension`.
2. Commit and push to `main`.
3. Tag it; GitHub Actions then builds the installers and publishes the Release:
   ```
   git tag v0.1.5
   git push origin v0.1.5
   ```
Every push to `main` also builds; find the files under Actions -> latest run -> Artifacts.

## Browser extension
Source is in `browser-extension/shared`, manifests in `browser-extension/manifests`. `node build.mjs` rebuilds the ready-to-load `chrome/` and `firefox/` folders. Edit the sources, never the generated folders. The Firefox add-on ID (`companion@rememberme.app`) must never change.

## Notes
- The app serves a local-only API on `127.0.0.1` (ports 47821-47830), protected by a token shown in Settings -> Browser extension.
- Linux builds happen on GitHub (Ubuntu 22.04). Windows cannot build Linux packages.
- Linux reminders are desktop notifications (`src-tauri/src/linux_alert.rs`), with sound played by system tools (pw-play, paplay, aplay, ffplay, mpv). The popup window is only the fallback when no notification service exists.
- `ui/` is edited by hand by several people; keep their changes when updating files.
- On Linux `src-tauri/src/main.rs` relaunches the app in a new session when it is started from a terminal (so closing the terminal does not kill it). `RM_FOREGROUND=1` disables this; `RM_DETACHED` marks the relaunched copy. For an AppImage it relaunches `$APPIMAGE` with `APPIMAGE_EXTRACT_AND_RUN=1`.
