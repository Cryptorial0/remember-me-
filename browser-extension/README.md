# Remember Me? browser companion

Needs the **Remember Me?** desktop app installed and running. The extension only sends reminders to it (like a download manager's browser helper).

## What it does
1. **Right-click** a page, selected text, or a link -> "Remind me about this ...". A small window opens, pre-filled.
2. **Read this page later:** click the toolbar button, then "In 1 hour", "Tonight 8 pm" or "Tomorrow 9 am". The page link is saved, and the reminder popup has an **Open** button.
3. **Quick add:** toolbar button (or `Alt+Shift+X`): type, pick a time, Save.

## Which folder?
| Browser | Folder |
|---|---|
| Chrome, Brave, Edge | `chrome/` |
| Firefox | `firefox/` |

(Both are generated from `shared/`. If you change code, edit `shared/` or `manifests/` and run `node build.mjs`.)

## Install in Chrome / Brave / Edge
1. Open `chrome://extensions` (Brave: `brave://extensions`).
2. Turn on **Developer mode**.
3. Click **Load unpacked** and choose the `chrome` folder.
4. The setup page opens. In the app go to **Settings -> Browser extension -> Copy token**, paste it, press **Save and test**.

If the hotkey does nothing, set it at `chrome://extensions/shortcuts`.

## Install in Firefox
- **Quick try (lasts until Firefox restarts):** open `about:debugging#/runtime/this-firefox`, click **Load Temporary Add-on**, choose `firefox/manifest.json`.
- **Permanent:** Firefox only keeps signed add-ons. Upload `remember-me-firefox-extension.zip` (from the Releases page) at https://addons.mozilla.org/developers/ as a **self-distributed (unlisted)** add-on; it is free, and you get a signed `.xpi` to install. Firefox Developer Edition and Nightly can load unsigned add-ons.
- Then paste the token on the setup page, same as above.
- If it says it cannot find the app, open `about:addons` -> the extension -> **Permissions** and allow access to `127.0.0.1`.

## Privacy
Everything stays on your computer. The extension talks only to `127.0.0.1`, and the app accepts only requests that carry your secret token.
