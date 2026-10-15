# Remember Me?

A small, private reminder app. Type what you want to be reminded about, pick a date and time, and a popup with a sound appears when it is due. Everything stays on your computer: no account, no cloud, no internet needed.

**Status:** Windows app ready. Linux builds come from the same code. Browser extension (Chrome, Brave, Firefox) and Android are next.

## Features
- Reminders with date/time and repeat (none, daily, weekly, monthly)
- Edit, Done, Snooze 10m / 1h, Delete
- Popup in the bottom-right corner with sound and a small animation; stays on top
- Sound settings: Silent, Chime, Bell, Beep, Digital, Soft, or upload your own file; volume; repeat; test and stop buttons
- Global hotkey (default **Alt+X**) to open the app; change it in Settings
- Runs in the system tray, starts with your computer (can be turned off), asks before quitting
- Local SQLite database
- Searchable as **RM**: the installer adds a second Start-menu entry named "RM", so typing `rm` finds it

## Get the installer (no build tools needed)
GitHub builds it for you.

**Easiest: a Release with download links**
1. Push this repo to GitHub.
2. In a terminal inside the project, run:
   ```
   git tag v0.1.3
   git push origin v0.1.3
   ```
3. Wait 10-15 minutes, then open the repo's **Releases** page (right side of the repo home page).
4. Download `Remember Me_0.1.3_x64-setup.exe` (Windows) or the `.deb` / `.rpm` / `.AppImage` (Linux) and run it.

For a new version, change the version in `src-tauri/tauri.conf.json`, then tag again (`v0.1.4`, ...).

**Without a tag:** every push to `main` also builds. Open **Actions** -> latest run -> **Artifacts**.

## Browser companion (optional)
A small helper for Firefox, Chrome, Brave and Edge that sends reminders from your browser to the app. **The desktop app must be installed and running.**

- Right-click selected text, a link or a page: **Remind me about this**
- **Read this page later** in one click (in 1 hour, tonight, tomorrow); the link is saved
- Quick add from the toolbar button or `Alt+Shift+X`
- When the reminder goes off, the popup has an **Open** button that opens the saved page

Status: Firefox is submitted and awaiting Mozilla's review. A Chrome Web Store listing will come later; until then Chrome, Brave and Edge users install it in developer mode. Step-by-step instructions, setup and troubleshooting are in [`browser-extension/README.md`](browser-extension/README.md). The extension files are attached to every GitHub Release.

## Build it yourself (Windows)
Double-click **`setup.bat`**. It installs Node.js, Rust and the C++ build tools with winget, builds the app, and creates `Remember-Me-Setup.exe` in this folder. The first run takes a while.

Manual way (tools already installed):
```
npm install
npm run tauri icon app-icon.png
npm run tauri dev       # test
npm run tauri build     # installer in src-tauri/target/release/bundle/
```

## Using it
- Closing the window hides it to the tray. Reminders keep working.
- Tray icon: click to open, right-click for Quit. Fully quitting stops reminders until you start it again.
- Where your data is: `%APPDATA%\com.rememberme.app\reminders.db` (Windows), `~/.local/share/com.rememberme.app/` (Linux).

## Linux
Download the file for your distro from the Releases page (64-bit Intel/AMD only). Open a terminal in the folder where you saved it. The file name starts with `Remember` (it may contain a space or a dot, so use the `*` as shown).

**Ubuntu 22.04+, Debian 12+, Linux Mint, Pop!_OS (.deb)**
```
sudo apt install ./Remember*.deb
```
Remove it: `sudo apt remove remember-me`

**Fedora, openSUSE (.rpm)**
```
sudo dnf install ./Remember*.rpm
```
Remove it: `sudo dnf remove remember-me`

**Arch, Manjaro, Omarchy and everything else (.AppImage)**
```
chmod +x ./Remember*.AppImage
./Remember*.AppImage
```
- Needs the `fuse2` package on Arch (`sudo pacman -S fuse2`). If FUSE is not available, run `./Remember*.AppImage --appimage-extract-and-run`.
- Remove it by deleting the file.

After a `.deb`/`.rpm` install, start it from your app menu or with the command `remember-me`. If you turned on "Start with the computer", also delete `~/.config/autostart/` entries for Remember Me when you remove it.

Status: the `.deb` was checked by installing and removing it on Ubuntu. The `.rpm` and `.AppImage` commands are not tested yet.

Search for it in your launcher with `rm`, `remember` or `reminder`.

Good to know:
- **Tray icon:** GNOME hides tray icons unless you install the *AppIndicator* extension (`gnome-shell-extension-appindicator`; Ubuntu includes it). Without it, closing the window hides the app: start it again from your app menu to bring the window back. Reminders keep working.
- **Hotkey on Wayland:** `Alt+X` may not work. Add a custom keyboard shortcut in your system settings that runs `remember-me` (starting it again just shows the window). On Hyprland/Omarchy, add a line such as `bind = SUPER ALT, R, exec, remember-me` to your Hyprland bindings.
- **Sound** needs the common GStreamer plugins (`gstreamer1.0-plugins-good`), which most desktops already have.
- The app starts in X11 (XWayland) mode when available, so the reminder popup can be placed in the corner and stay on top.
- Tested on a virtual X11 desktop with a normal window manager and with a tiling one (i3). **Not yet tested** on real GNOME, KDE or Hyprland.

## Known limits
- The popup can't draw over games in true exclusive fullscreen. The sound still plays.
- Windows can't have `?` in file names, so installer and shortcut are named "Remember Me".

## Roadmap
1. Chrome Web Store listing and Firefox approval
2. Linux testing and packaging
3. Android
