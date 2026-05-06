use crate::models::{Entry, Kind, Payload};
use crate::storage::Store;
use crate::view;
use anyhow::Result;
use iced::keyboard::{self, key};
use iced::{Element, Subscription, Task};
use std::sync::Arc;
use std::sync::Mutex;
use uuid::Uuid;

pub struct Narrative {
    pub entries: Vec<Entry>,
    pub composer: String,
    pub store: Arc<Mutex<Store>>,
    pub error: Option<String>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)] // Reload/ErrorOccurred/ClearError gardés pour futurs hooks externes
pub enum Message {
    ComposerChanged(String),
    /// Soumet le composer : si non vide, crée une Note et l'ajoute au fil.
    SubmitComposer,
    /// Bascule l'état done/pending d'une Task.
    ToggleTask(Uuid),
    /// Click sur un bloc App → relance.
    LaunchApp(String),
    /// Click sur un bloc Web → ouvre l'URL.
    OpenUrl(String),
    /// Quitte (Escape).
    Quit,
    /// Recharge le fil depuis le store.
    Reload,
    /// Erreur transitoire affichée en bas.
    ErrorOccurred(String),
    /// Efface l'erreur.
    ClearError,
}

impl Narrative {
    pub fn new() -> (Self, Task<Message>) {
        let store = match Store::open_default() {
            Ok(s) => Arc::new(Mutex::new(s)),
            Err(e) => {
                tracing::error!("ouverture store: {}", e);
                // On crée une instance dégradée — l'app reste utilisable
                // mais sans persistance.
                eprintln!("[narrative] ATTENTION : impossible d'ouvrir la base ({}) — mode mémoire seule", e);
                std::process::exit(1);
            }
        };

        // Premier chargement
        let entries = load_or_seed(&store);

        let state = Self {
            entries,
            composer: String::new(),
            store,
            error: None,
        };
        (state, Task::none())
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ComposerChanged(value) => {
                self.composer = value;
                Task::none()
            }

            Message::SubmitComposer => {
                let text = self.composer.trim().to_string();
                if text.is_empty() {
                    return Task::none();
                }
                let entry = parse_composer(&text);
                let insert_result = self.store.lock().unwrap().insert(&entry);
                if let Err(e) = insert_result {
                    return self.report_error(format!("insert: {}", e));
                }
                self.entries.push(entry);
                self.composer.clear();
                Task::none()
            }

            Message::ToggleTask(id) => {
                let toggle_result = self.store.lock().unwrap().toggle_task(id);
                if let Err(e) = toggle_result {
                    return self.report_error(format!("toggle_task: {}", e));
                }
                // Mise à jour optimiste in-place
                if let Some(entry) = self.entries.iter_mut().find(|e| e.id == id) {
                    if let Payload::Task { done, .. } = &mut entry.payload {
                        *done = !*done;
                    }
                }
                Task::none()
            }

            Message::LaunchApp(exec) => {
                tracing::info!("launch app: {}", exec);
                spawn_exec(&exec);
                let entry = Entry::new(
                    Kind::System,
                    Payload::System {
                        message: format!("→ relance « {} »", first_word(&exec)),
                    },
                );
                let _ = self.store.lock().unwrap().insert(&entry);
                self.entries.push(entry);
                Task::none()
            }

            Message::OpenUrl(url) => {
                tracing::info!("open url: {}", url);
                open_url(&url);
                Task::none()
            }

            Message::Reload => {
                let recent_result = self.store.lock().unwrap().recent(500);
                match recent_result {
                    Ok(entries) => {
                        self.entries = entries;
                        Task::none()
                    }
                    Err(e) => self.report_error(format!("reload: {}", e)),
                }
            }

            Message::Quit => std::process::exit(0),

            Message::ErrorOccurred(msg) => {
                self.error = Some(msg);
                Task::none()
            }

            Message::ClearError => {
                self.error = None;
                Task::none()
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        view::render(self)
    }

    pub fn subscription(&self) -> Subscription<Message> {
        keyboard::on_key_press(|key, _modifiers| match key {
            keyboard::Key::Named(key::Named::Escape) => Some(Message::Quit),
            _ => None,
        })
    }

    fn report_error(&mut self, msg: String) -> Task<Message> {
        tracing::error!("{}", msg);
        self.error = Some(msg);
        Task::none()
    }
}

/// Charge les entrées existantes ou injecte un seed si la base est vide.
fn load_or_seed(store: &Arc<Mutex<Store>>) -> Vec<Entry> {
    let s = store.lock().unwrap();
    match s.count() {
        Ok(0) => {
            // Première ouverture — un seed riche pour montrer tous les types de blocs
            let welcome = vec![
                Entry::new(
                    Kind::System,
                    Payload::System {
                        message: "bienvenue dans le fil narratif de forkOS".into(),
                    },
                ),
                Entry::note(
                    "le bureau est un fil de session continu. \
                     écris une note, lance une app, et tout s'inscrit ici.",
                ),
                Entry::new(
                    Kind::Task,
                    Payload::Task {
                        text: "essayer ⌘K pour ouvrir la palette".into(),
                        done: false,
                    },
                ),
                Entry::new(
                    Kind::Task,
                    Payload::Task {
                        text: "lire le manifeste du projet".into(),
                        done: true,
                    },
                ),
                Entry::new(
                    Kind::App,
                    Payload::App {
                        name: "Helix".into(),
                        exec: "helix".into(),
                    },
                ),
                Entry::new(
                    Kind::Web,
                    Payload::Web {
                        url: "https://rosepinetheme.com/palette/".into(),
                        title: Some("Rose Pine Dawn — référence du thème".into()),
                    },
                ),
                Entry::new(
                    Kind::Music,
                    Payload::Music {
                        track: "Hyperion".into(),
                        artist: Some("Rone".into()),
                    },
                ),
                Entry::new(
                    Kind::Git,
                    Payload::Git {
                        repo: "forkos".into(),
                        message: "palette: fin phase 1 — @contacts, #tags, icônes".into(),
                        sha: Some("0b5c650abc1234567890".into()),
                    },
                ),
                Entry::new(
                    Kind::Search,
                    Payload::Search {
                        query: "iced layer-shell wayland".into(),
                        engine: "DuckDuckGo".into(),
                    },
                ),
                Entry::note(
                    "phase 2 démarrée. modèle SQLite, blocs typés, composer en haut. \
                     prochain gros morceau : intégration ⌘K + niri.",
                ),
            ];
            for e in &welcome {
                let _ = s.insert(e);
            }
            welcome
        }
        Ok(_) => s.recent(500).unwrap_or_default(),
        Err(e) => {
            tracing::error!("count: {}", e);
            Vec::new()
        }
    }
}

fn spawn_exec(exec: &str) {
    let parts: Vec<&str> = exec.split_whitespace().collect();
    if parts.is_empty() {
        return;
    }
    if let Err(e) = std::process::Command::new(parts[0])
        .args(&parts[1..])
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .spawn()
    {
        tracing::error!("spawn {}: {}", parts[0], e);
        eprintln!("[narrative] échec spawn `{}`: {}", parts[0], e);
    }
}

fn open_url(url: &str) {
    let candidates: &[(&str, &[&str])] = if is_wsl() {
        &[
            ("wslview", &[]),
            ("explorer.exe", &[]),
            ("xdg-open", &[]),
        ]
    } else {
        &[
            ("xdg-open", &[]),
            ("wslview", &[]),
            ("explorer.exe", &[]),
        ]
    };

    for (program, _) in candidates {
        if std::process::Command::new(program)
            .arg(url)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .is_ok()
        {
            return;
        }
    }
    eprintln!("[narrative] aucun outil pour ouvrir {}", url);
}

fn is_wsl() -> bool {
    std::env::var_os("WSL_DISTRO_NAME").is_some()
        || std::env::var_os("WSL_INTEROP").is_some()
        || std::path::Path::new("/proc/sys/fs/binfmt_misc/WSLInterop").exists()
}

fn first_word(s: &str) -> &str {
    s.split_whitespace().next().unwrap_or(s)
}

/// Interprète le texte du composer pour créer l'entrée appropriée.
///
/// Syntaxe reconnue :
///   `[ ] texte`  → Task { done: false }
///   `[x] texte`  → Task { done: true }
///   `[X] texte`  → Task { done: true }
///   *(autre)*    → Note { text }
fn parse_composer(raw: &str) -> Entry {
    // Tâche non faite : "[ ] …"
    if let Some(rest) = raw.strip_prefix("[ ]") {
        let text = rest.trim().to_string();
        return Entry::new(Kind::Task, Payload::Task { text, done: false });
    }
    // Tâche faite : "[x] …" ou "[X] …"
    if let Some(rest) = raw.strip_prefix("[x]").or_else(|| raw.strip_prefix("[X]")) {
        let text = rest.trim().to_string();
        return Entry::new(Kind::Task, Payload::Task { text, done: true });
    }
    // Fallback : note libre
    Entry::note(raw)
}

/// Trait pour résultat → Message ergonomique.
#[allow(dead_code)]
pub trait IntoMessage<T> {
    fn into_message(self) -> Result<T>;
}
