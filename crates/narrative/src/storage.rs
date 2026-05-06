use crate::models::{Entry, Kind, Payload};
use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Local, TimeZone};
use rusqlite::{params, Connection};
use std::path::PathBuf;
use uuid::Uuid;

/// Wrapper autour d'une connexion SQLite contenant le fil narratif.
/// Une seule table `entries` avec id, timestamp Unix, kind texte, et
/// payload JSON.
pub struct Store {
    conn: Connection,
}

impl Store {
    /// Ouvre (ou crée) la base à `~/.local/share/forkos/sessions.db`.
    pub fn open_default() -> Result<Self> {
        let path = default_db_path()?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).with_context(|| {
                format!("création du dossier {}", parent.display())
            })?;
        }
        Self::open(&path)
    }

    pub fn open(path: &std::path::Path) -> Result<Self> {
        let conn = Connection::open(path)
            .with_context(|| format!("ouverture SQLite à {}", path.display()))?;
        let store = Self { conn };
        store.init_schema()?;
        Ok(store)
    }

    fn init_schema(&self) -> Result<()> {
        self.conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS entries (
                id          TEXT PRIMARY KEY,
                timestamp   INTEGER NOT NULL,
                kind        TEXT NOT NULL,
                payload     TEXT NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_entries_timestamp
                ON entries(timestamp DESC);
            CREATE INDEX IF NOT EXISTS idx_entries_kind
                ON entries(kind);",
        )?;
        Ok(())
    }

    /// Ajoute une entrée au fil. Ne renvoie rien : à utiliser pour les
    /// flux d'événements externes (hooks git, MPRIS, etc.).
    pub fn insert(&self, entry: &Entry) -> Result<()> {
        let payload_json = serde_json::to_string(&entry.payload)
            .context("sérialisation du payload")?;
        self.conn.execute(
            "INSERT INTO entries (id, timestamp, kind, payload)
             VALUES (?1, ?2, ?3, ?4)",
            params![
                entry.id.to_string(),
                entry.timestamp.timestamp(),
                entry.kind.as_str(),
                payload_json,
            ],
        )?;
        Ok(())
    }

    /// Charge les `limit` dernières entrées, triées du plus ancien
    /// au plus récent (ordre d'affichage du fil).
    pub fn recent(&self, limit: usize) -> Result<Vec<Entry>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, timestamp, kind, payload
             FROM entries
             ORDER BY timestamp DESC
             LIMIT ?1",
        )?;
        let rows = stmt.query_map(params![limit as i64], row_to_entry)?;

        let mut entries: Vec<Entry> = rows.filter_map(Result::ok).collect();
        entries.reverse();
        Ok(entries)
    }

    /// Retourne le nombre total d'entrées.
    pub fn count(&self) -> Result<usize> {
        let count: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM entries", [], |r| r.get(0))?;
        Ok(count as usize)
    }

    /// Bascule l'état d'une tâche (pour les blocs Task cliquables).
    pub fn toggle_task(&self, id: Uuid) -> Result<()> {
        let payload_json: String = self
            .conn
            .query_row(
                "SELECT payload FROM entries WHERE id = ?1",
                params![id.to_string()],
                |r| r.get(0),
            )
            .context("entrée introuvable")?;

        let mut payload: Payload = serde_json::from_str(&payload_json)?;
        if let Payload::Task { done, .. } = &mut payload {
            *done = !*done;
        } else {
            return Err(anyhow!("entry {} n'est pas une tâche", id));
        }

        let new_json = serde_json::to_string(&payload)?;
        self.conn.execute(
            "UPDATE entries SET payload = ?1 WHERE id = ?2",
            params![new_json, id.to_string()],
        )?;
        Ok(())
    }
}

fn row_to_entry(row: &rusqlite::Row<'_>) -> rusqlite::Result<Entry> {
    let id_str: String = row.get(0)?;
    let timestamp: i64 = row.get(1)?;
    let kind_str: String = row.get(2)?;
    let payload_json: String = row.get(3)?;

    let id = Uuid::parse_str(&id_str)
        .map_err(|e| rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e)))?;
    let timestamp: DateTime<Local> = Local
        .timestamp_opt(timestamp, 0)
        .single()
        .ok_or_else(|| rusqlite::Error::FromSqlConversionFailure(
            1,
            rusqlite::types::Type::Integer,
            "timestamp invalide".into(),
        ))?;
    let kind = Kind::from_str(&kind_str).ok_or_else(|| {
        rusqlite::Error::FromSqlConversionFailure(
            2,
            rusqlite::types::Type::Text,
            format!("kind inconnu: {}", kind_str).into(),
        )
    })?;
    let payload: Payload = serde_json::from_str(&payload_json).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(3, rusqlite::types::Type::Text, Box::new(e))
    })?;

    Ok(Entry { id, timestamp, kind, payload })
}

fn default_db_path() -> Result<PathBuf> {
    let home = std::env::var("HOME").context("HOME non défini")?;
    Ok(PathBuf::from(home).join(".local/share/forkos/sessions.db"))
}
