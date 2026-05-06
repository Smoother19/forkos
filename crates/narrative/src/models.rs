use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Une entrée du fil narratif. Représente n'importe quel événement
/// inscriptible : note libre, tâche, lancement d'app, écoute musicale, etc.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entry {
    pub id: Uuid,
    pub timestamp: DateTime<Local>,
    pub kind: Kind,
    pub payload: Payload,
}

impl Entry {
    pub fn new(kind: Kind, payload: Payload) -> Self {
        Self {
            id: Uuid::new_v4(),
            timestamp: Local::now(),
            kind,
            payload,
        }
    }

    pub fn note(text: impl Into<String>) -> Self {
        let text = text.into();
        Self::new(Kind::Note, Payload::Note { text })
    }
}

/// Catégorie d'entrée. Utilisée pour le rendu visuel et le filtrage.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Kind {
    Note,
    Task,
    App,
    Web,
    Music,
    Git,
    Search,
    System,
}

impl Kind {
    pub fn label(self) -> &'static str {
        match self {
            Kind::Note => "note",
            Kind::Task => "tâche",
            Kind::App => "app",
            Kind::Web => "web",
            Kind::Music => "musique",
            Kind::Git => "git",
            Kind::Search => "recherche",
            Kind::System => "système",
        }
    }

    pub fn icon(self) -> &'static str {
        match self {
            Kind::Note => "✎",
            Kind::Task => "◇",
            Kind::App => "▣",
            Kind::Web => "🌐",
            Kind::Music => "♪",
            Kind::Git => "⌥",
            Kind::Search => "⌕",
            Kind::System => "⚙",
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Kind::Note => "note",
            Kind::Task => "task",
            Kind::App => "app",
            Kind::Web => "web",
            Kind::Music => "music",
            Kind::Git => "git",
            Kind::Search => "search",
            Kind::System => "system",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "note" => Some(Kind::Note),
            "task" => Some(Kind::Task),
            "app" => Some(Kind::App),
            "web" => Some(Kind::Web),
            "music" => Some(Kind::Music),
            "git" => Some(Kind::Git),
            "search" => Some(Kind::Search),
            "system" => Some(Kind::System),
            _ => None,
        }
    }
}

/// Contenu typé d'une entrée. Sérialisé en JSON dans la colonne `payload`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Payload {
    Note {
        text: String,
    },
    Task {
        text: String,
        done: bool,
    },
    App {
        name: String,
        exec: String,
    },
    Web {
        url: String,
        title: Option<String>,
    },
    Music {
        track: String,
        artist: Option<String>,
    },
    Git {
        repo: String,
        message: String,
        sha: Option<String>,
    },
    Search {
        query: String,
        engine: String,
    },
    System {
        message: String,
    },
}

#[allow(dead_code)] // utilitaires pour futurs filtres/aperçus
impl Payload {
    /// Texte court pour l'affichage condensé (preview).
    pub fn primary_text(&self) -> &str {
        match self {
            Payload::Note { text } => text,
            Payload::Task { text, .. } => text,
            Payload::App { name, .. } => name,
            Payload::Web { url, title } => title.as_deref().unwrap_or(url),
            Payload::Music { track, .. } => track,
            Payload::Git { message, .. } => message,
            Payload::Search { query, .. } => query,
            Payload::System { message } => message,
        }
    }

    /// Texte secondaire (sous-titre, contexte).
    pub fn secondary_text(&self) -> Option<String> {
        match self {
            Payload::Note { .. } => None,
            Payload::Task { done, .. } => Some(if *done { "fait".into() } else { "à faire".into() }),
            Payload::App { exec, .. } => Some(exec.clone()),
            Payload::Web { url, title } => {
                if title.is_some() {
                    Some(url.clone())
                } else {
                    None
                }
            }
            Payload::Music { artist, .. } => artist.clone(),
            Payload::Git { repo, sha, .. } => {
                let short_sha = sha.as_ref().map(|s| &s[..s.len().min(7)]);
                Some(match short_sha {
                    Some(s) => format!("{} · {}", repo, s),
                    None => repo.clone(),
                })
            }
            Payload::Search { engine, .. } => Some(format!("via {}", engine)),
            Payload::System { .. } => None,
        }
    }
}
