use rusqlite::{params, Connection, OptionalExtension};
use serde::Serialize;
use std::path::Path;
use std::sync::Mutex;

pub struct Db(pub Mutex<Connection>);

#[derive(Serialize)]
pub struct Reminder {
    pub id: i64,
    pub title: String,
    pub due: i64, // unix seconds, UTC
    pub repeat: String, // none | daily | weekly
    pub done: bool,
    pub fired: bool,
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
        Ok(Db(Mutex::new(conn)))
    }

    pub fn add(&self, title: &str, due: i64, repeat: &str) -> rusqlite::Result<()> {
        let rep = match repeat {
            "daily" | "weekly" => repeat,
            _ => "none",
        };
        let c = self.0.lock().unwrap();
        c.execute(
            "INSERT INTO reminders (title, due, repeat) VALUES (?1, ?2, ?3)",
            params![title, due, rep],
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
        let items: Vec<(i64, String, i64, String)> = {
            let mut s = c.prepare(
                "SELECT id, title, due, repeat FROM reminders
                 WHERE done = 0 AND fired = 0 AND due <= ?1",
            )?;
            let rows = s.query_map([now], |r| {
                Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?))
            })?;
            rows.collect::<Result<_, _>>()?
        };
        for (id, _, due, rep) in &items {
            let step: i64 = match rep.as_str() {
                "daily" => 86_400,
                "weekly" => 604_800,
                _ => 0,
            };
            if step > 0 {
                let mut next = *due;
                while next <= now {
                    next += step;
                }
                c.execute("UPDATE reminders SET due = ?1 WHERE id = ?2", params![next, id])?;
            } else {
                c.execute("UPDATE reminders SET fired = 1 WHERE id = ?1", params![id])?;
            }
        }
        Ok(items.into_iter().map(|(i, t, _, _)| (i, t)).collect())
    }
}
