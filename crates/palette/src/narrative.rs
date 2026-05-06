/// Insertion directe dans le fil narratif (sessions.db) depuis la palette.
///
/// On réplique uniquement le sous-ensemble minimal du schéma de `narrative::storage`
/// pour éviter une dépendance circulaire entre crates. Le schéma doit rester
/// synchronisé avec `crates/narrative/src/storage.rs`.
use anyhow::Result;

/// Chemin de la DB narrative (même que `narrative::storage::open_default`).
fn db_path() -> std::path::PathBuf {
    let base = std::env::var("XDG_DATA_HOME")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| {
            let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
            std::path::PathBuf::from(home).join(".local").join("share")
        });
    base.join("forkos").join("sessions.db")
}

/// Insère une entrée dans le fil narratif à partir du texte brut du composer.
///
/// Syntaxe reconnue (identique à `narrative::app::parse_composer`) :
///   `[ ] texte`  → Task { done: false }
///   `[x] texte`  → Task { done: true }
///   `[X] texte`  → Task { done: true }
///   *(autre)*    → Note { text }
pub fn insert_from_text(raw: &str) -> Result<()> {
    let (kind, payload) = parse(raw);

    let path = db_path();
    // Crée le répertoire parent si nécessaire (première utilisation).
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let conn = rusqlite::Connection::open(&path)?;

    // Crée la table si elle n'existe pas encore (cas où narrative n'a jamais tourné).
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS entries (
            id        TEXT PRIMARY KEY,
            timestamp INTEGER NOT NULL,
            kind      TEXT NOT NULL,
            payload   TEXT NOT NULL
        );",
    )?;

    let id = uuid::Uuid::new_v4().to_string();
    // narrative/storage.rs stocke en secondes Unix (timestamp_opt(ts, 0))
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;

    conn.execute(
        "INSERT INTO entries (id, timestamp, kind, payload) VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![id, ts, kind, payload],
    )?;

    tracing::info!("narrative: entrée insérée (kind={}, id={})", kind, id);
    Ok(())
}

/// Retourne `(kind_str, payload_json)` depuis le texte brut.
fn parse(raw: &str) -> (&'static str, String) {
    if let Some(rest) = raw.strip_prefix("[ ]") {
        let text = rest.trim();
        let payload = serde_json::json!({ "type": "task", "text": text, "done": false }).to_string();
        return ("task", payload);
    }
    if let Some(rest) = raw.strip_prefix("[x]").or_else(|| raw.strip_prefix("[X]")) {
        let text = rest.trim();
        let payload = serde_json::json!({ "type": "task", "text": text, "done": true }).to_string();
        return ("task", payload);
    }
    // Note libre
    let payload = serde_json::json!({ "type": "note", "text": raw }).to_string();
    ("note", payload)
}
