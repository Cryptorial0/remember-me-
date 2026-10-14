# Remember Me? browser companion

A small helper that sends reminders from your browser to the **Remember Me?** desktop app, like the browser helper of a download manager. **The desktop app must be installed and running**; the extension does nothing without it.

## Where to get it
| Browser | Status | How to install |
|---|---|---|
| Firefox | Submitted to Mozilla, **awaiting review** | When approved it installs from the Firefox Add-ons page (link will be added here). Until then: [temporary install](#firefox-until-it-is-approved). |
| Chrome, Brave, Edge | Chrome Web Store listing planned for later | For now: [developer-mode install](#chrome--brave--edge-developer-mode). |

The extension files are attached to every [GitHub Release](../../releases): `remember-me-chrome-extension.zip` and `remember-me-firefox-extension.zip`. (If you downloaded the whole repo, use the `chrome` and `firefox` folders here instead.)

## What it does
1. **Right-click reminder:** select text, a link, or anywhere on a page, right-click, choose **Remind me about this ...**. A small window opens, pre-filled. Pick a time and save.
2. **Read this page later:** click the toolbar button, then **In 1 hour**, **Tonight 8 pm** or **Tomorrow 9 am**. The page's link is saved with the reminder.
3. **Quick add:** click the toolbar button (or press `Alt+Shift+X`), type what you want to remember, pick a date and time, choose a repeat if you like, and save.

### When the reminder goes off
The app shows its popup in the bottom-right corner with your sound. If the reminder has a link, the popup has an **Open** button that opens the page in your default browser. The same **Open link** button is in the app's reminder list. You can also use **Done**, **Snooze** (10 minutes) or **✕**.

## First-time setup (needed once, all browsers)
1. Install and start the Remember Me? desktop app.
2. In the app open **Settings -> Browser extension** and press **Copy token**.
3. Open the extension's setup page (it opens by itself after installing; otherwise see below) and paste the token.
4. Press **Save and test**. You should see "Connected to the app".

Setup page if it does not open: Chrome/Brave: `chrome://extensions` (Brave: `brave://extensions`) -> the extension -> **Details** -> **Extension options**. Firefox: `about:addons` -> the extension -> **Preferences**.

## Chrome / Brave / Edge (developer mode)
Use this until the extension is on the Chrome Web Store.
1. Download `remember-me-chrome-extension.zip` from the Releases page and **extract** it to a folder you will keep, for example `Documents\RememberMe-extension`. Do not delete or move it later: the browser runs the extension straight from that folder.
2. Open `chrome://extensions` (Brave: `brave://extensions`, Edge: `edge://extensions`).
3. Turn on **Developer mode**.
4. Click **Load unpacked** and choose the extracted folder (the one that contains `manifest.json`).
5. Do the first-time setup above.

Good to know:
- The browser may show a "developer mode extensions" warning from time to time. That is expected for this install method.
- It does not update itself. For a new version: download the new zip, replace the files in the same folder, then press the reload button on the extension's card.
- If the toolbar icon is hidden, pin it from the puzzle-piece menu.
- If `Alt+Shift+X` does nothing, set it at `chrome://extensions/shortcuts`.

## Firefox (until it is approved)
The Firefox version is waiting for Mozilla's review. Until then you can load it temporarily:
1. Download and extract `remember-me-firefox-extension.zip`.
2. Open `about:debugging#/runtime/this-firefox`.
3. Click **Load Temporary Add-on...** and choose the `manifest.json` file in the extracted folder.
4. Do the first-time setup above.

A temporary add-on disappears when Firefox restarts, so load it again after a restart. Once Mozilla approves it, install from the Firefox Add-ons page and it stays permanently and updates itself.

If it says it cannot find the app, open `about:addons` -> the extension -> **Permissions** and allow access to `127.0.0.1`.

## Troubleshooting
- **"Remember Me? is not running":** start the desktop app, then try again.
- **"Token problem":** copy the token again from the app (Settings -> Browser extension) and paste it on the setup page.
- **Right-click menu missing:** reload the extension on the extensions page.
- **"This page has no web link to save":** browser pages such as the new-tab page have no link. Use the quick-add form instead.

## Privacy
Everything stays on your computer. The extension talks only to the app on `127.0.0.1`, and the app accepts only requests that carry your secret token. No accounts, no tracking, no remote servers.

## For developers
- Source is in `shared/`; the two manifests are in `manifests/`.
- `node build.mjs` rebuilds the ready-to-load `chrome/` and `firefox/` folders. Edit `shared/` or `manifests/`, never the generated folders.
