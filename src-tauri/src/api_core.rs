//! Request handling for the local browser API (no networking here, so it is easy to test).
use crate::db::Db;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct Reply {
    pub status: u16,
    pub body: String,
    /// true when data changed and the UI should refresh
    pub changed: bool,
}

fn reply(status: u16, body: &str) -> Reply {
    Reply { status, body: body.to_string(), changed: false }
}

fn ok_changed() -> Reply {
    Reply { status: 200, body: r#"{"ok":true}"#.into(), changed: true }
}

fn db_err() -> Reply {
    reply(500, r#"{"error":"db"}"#)
}

fn now() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs() as i64).unwrap_or(0)
}

/// The only settings the browser may read or change (never the token, hotkey or autostart).
const SAFE_SETTINGS: [&str; 3] = ["sound", "volume", "repeats"];
const SOUNDS: [&str; 7] = ["none", "chime", "bell", "beep", "digital", "soft", "custom"];

fn parse_reminder(body: &str) -> Result<(String, i64, String, Option<String>), Reply> {
    let v: serde_json::Value =
        serde_json::from_str(body).map_err(|_| reply(400, r#"{"error":"bad json"}"#))?;
    let title = v["title"].as_str().unwrap_or("").trim().to_string();
    let due = v["due"].as_i64().unwrap_or(0);
    let repeat = v["repeat"].as_str().unwrap_or("none").to_string();
    if title.is_empty() || title.chars().count() > 500 || due <= 0 {
        return Err(reply(400, r#"{"error":"title and due are required"}"#));
    }
    let link = v["link"].as_str().map(|s| s.to_string());
    Ok((title, due, repeat, link))
}

fn valid_setting(key: &str, value: &str) -> bool {
    match key {
        "sound" => SOUNDS.contains(&value),
        "volume" => value.parse::<f64>().map(|v| (0.0..=1.0).contains(&v)).unwrap_or(false),
        "repeats" => value.parse::<u8>().map(|v| v <= 10).unwrap_or(false),
        _ => false,
    }
}

/// `host_ok`: Host header was 127.0.0.1:<port> or localhost:<port> (blocks DNS-rebinding).
pub fn route(
    db: &Db,
    method: &str,
    path: &str,
    host_ok: bool,
    token: &str,
    given_token: &str,
    body: &str,
) -> Reply {
    if !host_ok {
        return reply(403, r#"{"error":"bad host"}"#);
    }
    if method == "OPTIONS" {
        return reply(204, "");
    }
    if method == "GET" && path == "/ping" {
        return reply(200, r#"{"app":"remember-me","ok":true}"#);
    }
    if given_token != token {
        return reply(401, r#"{"error":"bad token"}"#);
    }

    let segs: Vec<&str> = path.trim_matches('/').split('/').collect();
    let id_of = |s: &str| s.parse::<i64>().ok();

    match (method, segs.as_slice()) {
        ("GET", ["reminders"]) => match db.list() {
            Ok(all) => {
                let open: Vec<_> = all.into_iter().filter(|r| !r.done).collect();
                reply(200, &serde_json::to_string(&open).unwrap_or_else(|_| "[]".into()))
            }
            Err(_) => db_err(),
        },
        ("POST", ["reminders"]) => match parse_reminder(body) {
            Ok((t, d, r, l)) => match db.add_with_link(&t, d, &r, &l.unwrap_or_default()) {
                Ok(_) => ok_changed(),
                Err(_) => db_err(),
            },
            Err(e) => e,
        },
        ("PUT", ["reminders", id]) => {
            let Some(id) = id_of(id) else { return reply(400, r#"{"error":"bad id"}"#) };
            match parse_reminder(body) {
                Ok((t, d, r, l)) => match db.update(id, &t, d, &r) {
                    Ok(_) => {
                        if let Some(l) = l {
                            let _ = db.set_link(id, &l);
                        }
                        ok_changed()
                    }
                    Err(_) => db_err(),
                },
                Err(e) => e,
            }
        }
        ("DELETE", ["reminders", id]) => {
            let Some(id) = id_of(id) else { return reply(400, r#"{"error":"bad id"}"#) };
            match db.delete(id) {
                Ok(_) => ok_changed(),
                Err(_) => db_err(),
            }
        }
        ("POST", ["reminders", id, "done"]) => {
            let Some(id) = id_of(id) else { return reply(400, r#"{"error":"bad id"}"#) };
            match db.toggle_done(id) {
                Ok(_) => ok_changed(),
                Err(_) => db_err(),
            }
        }
        ("POST", ["reminders", id, "snooze"]) => {
            let Some(id) = id_of(id) else { return reply(400, r#"{"error":"bad id"}"#) };
            let minutes = serde_json::from_str::<serde_json::Value>(body)
                .ok()
                .and_then(|v| v["minutes"].as_i64())
                .unwrap_or(10);
            if !(1..=1440).contains(&minutes) {
                return reply(400, r#"{"error":"minutes must be 1-1440"}"#);
            }
            match db.snooze(id, now() + minutes * 60) {
                Ok(_) => ok_changed(),
                Err(_) => db_err(),
            }
        }
        ("GET", ["settings"]) => match db.settings() {
            Ok(rows) => {
                let map: std::collections::BTreeMap<String, String> =
                    rows.into_iter().filter(|(k, _)| SAFE_SETTINGS.contains(&k.as_str())).collect();
                reply(200, &serde_json::to_string(&map).unwrap_or_else(|_| "{}".into()))
            }
            Err(_) => db_err(),
        },
        ("PUT", ["settings"]) => {
            let v: serde_json::Value = match serde_json::from_str(body) {
                Ok(v) => v,
                Err(_) => return reply(400, r#"{"error":"bad json"}"#),
            };
            let key = v["key"].as_str().unwrap_or("");
            let value = v["value"].as_str().unwrap_or("");
            if !SAFE_SETTINGS.contains(&key) || !valid_setting(key, value) {
                return reply(400, r#"{"error":"setting not allowed or value invalid"}"#);
            }
            match db.set_setting(key, value) {
                Ok(_) => ok_changed(),
                Err(_) => db_err(),
            }
        }
        _ => reply(404, r#"{"error":"not found"}"#),
    }
}
