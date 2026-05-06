use crate::entry::{Entry, EntryKind};
use chrono::{DateTime, Local};
use rusqlite::{params, Connection, Result};

fn db_path() -> String {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    format!("{}/.local/share/forkos/sessions.db", home)
}

fn open() -> Result<Connection> {
    let path = db_path();
    if let Some(parent) = std::path::Path::new(&path).parent() {
        std::fs::create_dir_all(parent).ok();
    }
    let conn = Connection::open(&path)?;
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS entries (
            id        INTEGER PRIMARY KEY AUTOINCREMENT,
            timestamp TEXT    NOT NULL,
            kind_json TEXT    NOT NULL,
            session_id TEXT   NOT NULL
        );",
    )?;
    Ok(conn)
}

pub fn load_current_session(session_id: &str) -> Vec<Entry> {
    let conn = match open() {
        Ok(c) => c,
        Err(_) => return vec![],
    };
    let mut stmt = match conn.prepare(
        "SELECT id, timestamp, kind_json FROM entries WHERE session_id = ?1 ORDER BY id ASC",
    ) {
        Ok(s) => s,
        Err(_) => return vec![],
    };

    let rows = match stmt.query_map(params![session_id], |row| {
        Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?, row.get::<_, String>(2)?))
    }) {
        Ok(r) => r,
        Err(_) => return vec![],
    };

    rows.flatten()
    .filter_map(|(id, ts_str, kind_json)| {
        let timestamp = DateTime::parse_from_rfc3339(&ts_str)
            .map(|dt| dt.with_timezone(&Local))
            .ok()?;
        let kind = serde_json::from_str(&kind_json).ok()?;
        Some(Entry { id: id as u64, timestamp, kind })
    })
    .collect()
}

pub fn append(kind: EntryKind, session_id: &str) -> Option<Entry> {
    let conn = open().ok()?;
    let timestamp = Local::now();
    let kind_json = serde_json::to_string(&kind).ok()?;
    conn.execute(
        "INSERT INTO entries (timestamp, kind_json, session_id) VALUES (?1, ?2, ?3)",
        params![timestamp.to_rfc3339(), kind_json, session_id],
    )
    .ok()?;
    let id = conn.last_insert_rowid() as u64;
    Some(Entry { id, timestamp, kind })
}

pub fn load_recent_sessions(limit: usize) -> Vec<(String, Vec<Entry>)> {
    let conn = match open() {
        Ok(c) => c,
        Err(_) => return vec![],
    };
    let mut stmt = match conn.prepare(
        "SELECT DISTINCT session_id FROM entries GROUP BY session_id ORDER BY MIN(id) DESC LIMIT ?1",
    ) {
        Ok(s) => s,
        Err(_) => return vec![],
    };

    let rows = match stmt.query_map(params![limit as i64], |row| row.get::<_, String>(0)) {
        Ok(r) => r,
        Err(_) => return vec![],
    };

    let session_ids: Vec<String> = rows.flatten().collect();

    session_ids.into_iter().map(|sid| {
        let entries = load_current_session(&sid);
        (sid, entries)
    }).collect()
}
