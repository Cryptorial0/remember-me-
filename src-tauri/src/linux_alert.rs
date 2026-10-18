//! Linux alerts: a desktop notification (shown by the desktop's own notification service)
//! plus a sound played through the system's audio tools. This avoids the web view's audio,
//! which is silent on some distros, and the popup window, which tiling window managers resize.
use crate::db::{clean_link, Db};
use notify_rust::{Notification, Timeout};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Emitter, Manager};

const RATE: f32 = 44_100.0;

#[derive(Clone, Copy)]
enum Wave {
    Sine,
    Square,
}

struct Tone {
    freq: f32,
    start: f32,
    dur: f32,
    wave: Wave,
    gain: f32,
}

fn now() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs() as i64).unwrap_or(0)
}

/// Same sounds as the Windows/web version (see ui/sound.js).
fn tones(kind: &str, v: f32) -> Vec<Tone> {
    use Wave::*;
    let t = |freq, start, dur, wave, gain| Tone { freq, start, dur, wave, gain };
    match kind {
        "bell" => vec![
            t(523.0, 0.0, 1.6, Sine, 0.25 * v),
            t(1046.0, 0.0, 1.6, Sine, 0.25 * 0.6 * v),
            t(1568.0, 0.0, 1.6, Sine, 0.25 * 0.4 * v),
            t(2093.0, 0.0, 1.6, Sine, 0.25 * 0.25 * v),
        ],
        "beep" => vec![t(1000.0, 0.0, 0.18, Square, 0.15 * v), t(1000.0, 0.35, 0.18, Square, 0.15 * v)],
        "digital" => [800.0, 1000.0, 1200.0, 1000.0, 1200.0, 1500.0]
            .iter()
            .enumerate()
            .map(|(i, f)| t(*f, i as f32 * 0.09, 0.08, Square, 0.12 * v))
            .collect(),
        "soft" => vec![t(520.0, 0.0, 1.0, Sine, 0.3 * v), t(780.0, 0.25, 1.0, Sine, 0.2 * v)],
        _ => vec![
            t(880.0, 0.0, 0.6, Sine, 0.3 * v),
            t(1318.5, 0.18, 0.6, Sine, 0.3 * v),
            t(1760.0, 0.36, 0.6, Sine, 0.3 * v),
        ],
    }
}

/// Renders a sound to a 16-bit mono WAV file in memory.
fn render_wav(kind: &str, volume: f32) -> Vec<u8> {
    let list = tones(kind, volume);
    let total = list.iter().map(|t| t.start + t.dur).fold(0.0_f32, f32::max) + 0.05;
    let n = (total * RATE) as usize;
    let mut buf = vec![0.0_f32; n];
    for t in &list {
        let s0 = (t.start * RATE) as usize;
        let len = (t.dur * RATE) as usize;
        let gain = t.gain.max(0.0001);
        for i in 0..len {
            let idx = s0 + i;
            if idx >= n {
                break;
            }
            let time = i as f32 / RATE;
            // 15 ms attack, then exponential decay to near silence (like the web version)
            let env = if time < 0.015 {
                time / 0.015
            } else {
                (0.0001 / gain).powf((time - 0.015) / (t.dur - 0.015).max(0.001))
            };
            let s = (2.0 * std::f32::consts::PI * t.freq * time).sin();
            let s = match t.wave {
                Wave::Sine => s,
                Wave::Square => {
                    if s >= 0.0 {
                        1.0
                    } else {
                        -1.0
                    }
                }
            };
            buf[idx] += gain * env * s;
        }
    }
    let samples: Vec<i16> = buf.iter().map(|x| (x.clamp(-1.0, 1.0) * 32767.0) as i16).collect();
    let data_len = (samples.len() * 2) as u32;
    let mut out = Vec::with_capacity(44 + samples.len() * 2);
    out.extend_from_slice(b"RIFF");
    out.extend_from_slice(&(36 + data_len).to_le_bytes());
    out.extend_from_slice(b"WAVEfmt ");
    out.extend_from_slice(&16u32.to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes()); // PCM
    out.extend_from_slice(&1u16.to_le_bytes()); // mono
    out.extend_from_slice(&(RATE as u32).to_le_bytes());
    out.extend_from_slice(&(RATE as u32 * 2).to_le_bytes());
    out.extend_from_slice(&2u16.to_le_bytes());
    out.extend_from_slice(&16u16.to_le_bytes());
    out.extend_from_slice(b"data");
    out.extend_from_slice(&data_len.to_le_bytes());
    for s in samples {
        out.extend_from_slice(&s.to_le_bytes());
    }
    out
}

/// Sound files live here; nothing but our own generated/uploaded sounds is ever written.
fn sound_dir() -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("remember-me-{}", std::process::id()));
    let _ = std::fs::create_dir_all(&dir);
    dir
}

fn custom_file(db: &Db) -> Option<std::path::PathBuf> {
    let url = db.get_setting("custom_sound").ok().flatten()?;
    let rest = url.strip_prefix("data:")?;
    let (meta, data) = rest.split_once(',')?;
    let mime = meta.split(';').next().unwrap_or("");
    let ext = match mime {
        "audio/mpeg" | "audio/mp3" => "mp3",
        "audio/wav" | "audio/x-wav" | "audio/wave" => "wav",
        "audio/ogg" | "application/ogg" => "ogg",
        "audio/flac" | "audio/x-flac" => "flac",
        "audio/mp4" | "audio/x-m4a" | "audio/aac" => "m4a",
        "audio/webm" => "webm",
        _ => "audio",
    };
    let bytes = decode_base64(data)?;
    let path = sound_dir().join(format!("custom.{ext}"));
    std::fs::write(&path, bytes).ok()?;
    Some(path)
}

fn decode_base64(s: &str) -> Option<Vec<u8>> {
    let mut out = Vec::with_capacity(s.len() * 3 / 4);
    let (mut acc, mut bits) = (0u32, 0u32);
    for c in s.bytes() {
        let v = match c {
            b'A'..=b'Z' => c - b'A',
            b'a'..=b'z' => c - b'a' + 26,
            b'0'..=b'9' => c - b'0' + 52,
            b'+' => 62,
            b'/' => 63,
            b'=' | b'\n' | b'\r' | b' ' => continue,
            _ => return None,
        } as u32;
        acc = (acc << 6) | v;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push(((acc >> bits) & 0xFF) as u8);
        }
    }
    Some(out)
}

/// Tries common players in turn. `volume` is passed to the player only for uploaded files
/// (built-in sounds already have the volume baked in). Returns true if something played.
fn play_blocking(file: &str, volume: Option<f32>, stop: &AtomicBool) -> bool {
    let v = volume.map(|x| x.clamp(0.0, 1.0));
    let mut list: Vec<(&str, Vec<String>)> = Vec::new();
    list.push(("pw-play", v.map(|x| vec![format!("--volume={x}")]).unwrap_or_default()));
    list.push(("paplay", v.map(|x| vec![format!("--volume={}", (x * 65536.0) as u32)]).unwrap_or_default()));
    list.push((
        "ffplay",
        [vec!["-nodisp", "-autoexit", "-loglevel", "quiet"].into_iter().map(String::from).collect::<Vec<_>>(),
            v.map(|x| vec!["-volume".to_string(), ((x * 100.0) as u32).to_string()]).unwrap_or_default()]
            .concat(),
    ));
    list.push((
        "mpv",
        [vec!["--no-video".to_string(), "--really-quiet".to_string()],
            v.map(|x| vec![format!("--volume={}", (x * 100.0) as u32)]).unwrap_or_default()]
            .concat(),
    ));
    list.push(("aplay", vec!["-q".to_string()]));
    list.push(("play", vec!["-q".to_string()]));

    for (prog, mut args) in list {
        args.push(file.to_string());
        let spawned = Command::new(prog)
            .args(&args)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn();
        let Ok(mut child) = spawned else { continue };
        loop {
            if stop.load(Ordering::Relaxed) {
                let _ = child.kill();
                let _ = child.wait();
                return true;
            }
            match child.try_wait() {
                Ok(Some(status)) if status.success() => return true,
                Ok(Some(_)) | Err(_) => break, // this player could not play it: try the next one
                Ok(None) => std::thread::sleep(Duration::from_millis(50)),
            }
        }
    }
    false
}

fn setting(db: &Db, key: &str) -> Option<String> {
    db.get_setting(key).ok().flatten()
}

/// Starts the reminder sound in the background. `plays` overrides the saved repeat count.
/// Returns a flag: set it to true to stop the sound.
pub fn start_sound(db: &Db, plays: Option<u32>) -> Arc<AtomicBool> {
    let stop = Arc::new(AtomicBool::new(false));
    let kind = setting(db, "sound").unwrap_or_else(|| "chime".into());
    if kind == "none" {
        return stop;
    }
    let volume = setting(db, "volume").and_then(|v| v.parse::<f32>().ok()).unwrap_or(0.6).clamp(0.0, 1.0);
    let repeats = setting(db, "repeats").and_then(|v| v.parse::<u32>().ok()).unwrap_or(0).min(10);
    let total = plays.unwrap_or(if repeats == 0 { 30 } else { repeats }); // 0 = until dismissed (about 3 min)

    let (file, player_volume) = match (kind.as_str() == "custom").then(|| custom_file(db)).flatten() {
        Some(p) => (p, Some(volume)),
        None => {
            let name = if kind == "custom" { "chime".to_string() } else { kind.clone() };
            let path = sound_dir().join(format!("{name}-{}.wav", (volume * 100.0) as u32));
            if std::fs::write(&path, render_wav(&name, volume)).is_err() {
                return stop;
            }
            (path, None)
        }
    };

    let flag = stop.clone();
    std::thread::spawn(move || {
        let file = file.to_string_lossy().to_string();
        for i in 0..total {
            if flag.load(Ordering::Relaxed) {
                return;
            }
            let started = Instant::now();
            if !play_blocking(&file, player_volume, &flag) {
                return; // no audio player found; nothing more to try
            }
            if i + 1 < total {
                while started.elapsed() < Duration::from_secs(6) {
                    if flag.load(Ordering::Relaxed) {
                        return;
                    }
                    std::thread::sleep(Duration::from_millis(100));
                }
            }
        }
    });
    stop
}

static TEST: Mutex<Option<Arc<AtomicBool>>> = Mutex::new(None);

/// Settings page: "Test sound" (plays once) and "Stop".
pub fn test_sound(db: &Db) {
    stop_test();
    let flag = start_sound(db, Some(1));
    *TEST.lock().unwrap() = Some(flag);
}

pub fn stop_test() {
    if let Some(f) = TEST.lock().unwrap().take() {
        f.store(true, Ordering::Relaxed);
    }
}

fn show_window(app: &AppHandle) {
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.show();
        let _ = w.unminimize();
        let _ = w.set_focus();
    }
}

fn plain(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}

/// Shows the reminder as a desktop notification and plays the sound until the notification
/// is dismissed or one of its buttons is used. Returns false if no notification service is
/// available (the caller then falls back to the popup window).
pub fn alert(app: &AppHandle, id: i64, title: String, link: String) -> bool {
    let link = clean_link(&link);
    let mut n = Notification::new();
    n.appname("Remember Me?")
        .summary("Remember Me?")
        .body(&plain(&title))
        .icon("remember-me")
        .timeout(Timeout::Never)
        .action("default", "Open")
        .action("done", "Done")
        .action("snooze", "Snooze 10m");
    if !link.is_empty() {
        n.action("open", "Open link");
    }
    let Ok(handle) = n.show() else { return false };

    let stop = start_sound(&app.state::<Db>(), None);
    let app = app.clone();
    std::thread::spawn(move || {
        handle.wait_for_action(|action| match action {
            "done" => {
                let _ = app.state::<Db>().toggle_done(id);
                let _ = app.emit("changed", ());
            }
            "snooze" => {
                let _ = app.state::<Db>().snooze(id, now() + 600);
                let _ = app.emit("changed", ());
            }
            "open" => {
                let _ = Command::new("xdg-open").arg(&link).spawn();
            }
            "default" => show_window(&app),
            _ => {}
        });
        stop.store(true, Ordering::Relaxed);
    });
    true
}
