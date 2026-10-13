use chrono::{Days, Local, Months, TimeZone};
use rusqlite::{params, Connection, OptionalExtension};
use serde::Serialize;
use std::path::Path;
use std::sync::Mutex;

pub struct Db(pub Mutex<Connection>);

#[derive(Serialize)]
pub struct Reminder {
    pub id: i64,
    pub title: String,
    pub due: i64,       // unix seconds, UTC
    pub repeat: String, // none | daily | weekly | monthly
    pub done: bool,
    pub fired: bool,
}

fn clean_repeat(r: &str) -> &str {
    match r {
        "daily" | "weekly" | "monthly" => r,
        _ => "none",
    }
}

/// Next occurrence after `now`, counted from the original `anchor` in LOCAL time,
/// so daylight saving and month lengths never drift (Jan 31 -> Feb 28 -> Mar 31).
fn next_due(anchor: i64, rep: &str, now: i64) -> Option<i64> {
    let base = Local.timestamp_opt(anchor, 0).single()?;
    for k in 1..=20000u32 {
        let t = match rep {
            "daily" => base.checked_add_days(Days::new(k as u64))?,
            "weekly" => base.checked_add_days(Days::new(7 * k as u64))?,
            "monthly" => base.checked_add_months(Months::new(k))?,
            _ => return None,
        };
        let ts = t.timestamp();
        if ts > now {
            return Some(ts);
        }
    }
    None
}

impl Db {
    pub fn open(path: &Path) -> rusqlite::Result<Self> {
        let conn = Connection::open(path)?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS reminders (
                id      INTEGER PRIMARY KEY AUTOINCREMENT,
                title   TEXT    NOT NULL,
                due     INTEGER NOT NULL,
                repeat  TEXT    NOT NULL DEFAULT 'none',
                done    INTEGER NOT NULL DEFAULT 0,
                fired   INTEGER NOT NULL DEFAULT 0
            );
            CREATE TABLE IF NOT EXISTS settings (key TEXT PRIMARY KEY, value TEXT NOT NULL);",
        )?;
        // Older databases lack the anchor column; ignore the error if it already exists.
        let _ = conn.execute("ALTER TABLE reminders ADD COLUMN anchor INTEGER", []);
        Ok(Db(Mutex::new(conn)))
    }

    pub fn add(&self, title: &str, due: i64, repeat: &str) -> rusqlite::Result<()> {
        let c = self.0.lock().unwrap();
        c.execute(
            "INSERT INTO reminders (title, due, repeat, anchor) VALUES (?1, ?2, ?3, ?2)",
            params![title, due, clean_repeat(repeat)],
        )?;
        Ok(())
    }

    pub fn update(&self, id: i64, title: &str, due: i64, repeat: &str) -> rusqlite::Result<()> {
        let c = self.0.lock().unwrap();
        c.execute(
            "UPDATE reminders SET title = ?1, due = ?2, repeat = ?3, anchor = ?2, fired = 0, done = 0
             WHERE id = ?4",
            params![title, due, clean_repeat(repeat), id],
        )?;
        Ok(())
    }

    pub fn list(&self) -> rusqlite::Result<Vec<Reminder>> {
        let c = self.0.lock().unwrap();
        let mut s = c.prepare(
            "SELECT id, title, due, repeat, done, fired FROM reminders ORDER BY done, due",
        )?;
        let rows = s.query_map([], |r| {
            Ok(Reminder {
                id: r.get(0)?,
                title: r.get(1)?,
                due: r.get(2)?,
                repeat: r.get(3)?,
                done: r.get::<_, i64>(4)? != 0,
                fired: r.get::<_, i64>(5)? != 0,
            })
        })?;
        rows.collect()
    }

    pub fn delete(&self, id: i64) -> rusqlite::Result<()> {
        let c = self.0.lock().unwrap();
        c.execute("DELETE FROM reminders WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn toggle_done(&self, id: i64) -> rusqlite::Result<()> {
        let c = self.0.lock().unwrap();
        c.execute("UPDATE reminders SET done = 1 - done WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn snooze(&self, id: i64, new_due: i64) -> rusqlite::Result<()> {
        let c = self.0.lock().unwrap();
        c.execute(
            "UPDATE reminders SET due = ?1, fired = 0, done = 0 WHERE id = ?2",
            params![new_due, id],
        )?;
        Ok(())
    }

    pub fn settings(&self) -> rusqlite::Result<Vec<(String, String)>> {
        let c = self.0.lock().unwrap();
        let mut s = c.prepare("SELECT key, value FROM settings")?;
        let rows = s.query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?;
        rows.collect()
    }

    pub fn get_setting(&self, key: &str) -> rusqlite::Result<Option<String>> {
        let c = self.0.lock().unwrap();
        c.query_row("SELECT value FROM settings WHERE key = ?1", params![key], |r| r.get(0))
            .optional()
    }

    pub fn set_setting(&self, key: &str, value: &str) -> rusqlite::Result<()> {
        let c = self.0.lock().unwrap();
        c.execute(
            "INSERT INTO settings (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![key, value],
        )?;
        Ok(())
    }

    /// Returns reminders that are due now and marks them handled
    /// (repeating ones are moved to their next occurrence).
    pub fn take_due(&self, now: i64) -> rusqlite::Result<Vec<(i64, String)>> {
        let c = self.0.lock().unwrap();
        let items: Vec<(i64, String, String, i64)> = {
            let mut s = c.prepare(
                "SELECT id, title, repeat, COALESCE(anchor, due) FROM reminders
                 WHERE done = 0 AND fired = 0 AND due <= ?1",
            )?;
            let rows = s.query_map([now], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)))?;
            rows.collect::<Result<_, _>>()?
        };
        for (id, _, rep, anchor) in &items {
            let next = if rep == "none" { None } else { next_due(*anchor, rep, now) };
            match next {
                Some(n) => {
                    c.execute("UPDATE reminders SET due = ?1 WHERE id = ?2", params![n, id])?;
                }
                None => {
                    c.execute("UPDATE reminders SET fired = 1 WHERE id = ?1", params![id])?;
                }
            }
        }
        Ok(items.into_iter().map(|(i, t, _, _)| (i, t)).collect())
    }
}
