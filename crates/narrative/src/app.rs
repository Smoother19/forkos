use crate::entry::{ContextAction, Entry, EntryKind, NotifItem};
use crate::store;
use crate::view;
use forkos_shared::command::{system_commands, Command};
use forkos_shared::mode::Mode;
use forkos_shared::search;
use forkos_shared::sources;
use iced::keyboard::{self, key};
use iced::widget::text_input;
use iced::{Element, Subscription, Task};
use std::sync::LazyLock;

pub static BOTTOM_INPUT_ID: LazyLock<text_input::Id> = LazyLock::new(text_input::Id::unique);
pub static PALETTE_INPUT_ID: LazyLock<text_input::Id> = LazyLock::new(text_input::Id::unique);

pub struct Narrative {
    pub entries: Vec<Entry>,
    pub session_id: String,
    pub commands: Vec<Command>,
    pub palette_open: bool,
    pub palette_query: String,
    pub palette_filtered: Vec<Command>,
    pub palette_selected: usize,
    pub bottom_query: String,
    pub sources_loaded: bool,
}

#[derive(Debug, Clone)]
pub enum Message {
    PaletteToggle,
    PaletteQueryChanged(String),
    PaletteSelectNext,
    PaletteSelectPrevious,
    PaletteExecute,
    PaletteCancel,
    BottomInputChanged(String),
    BottomInputSubmit,
    SourcesLoaded(sources::LoadedSources),
    ContextAction(String),
    MediaPlayPause,
    MediaNext,
    MediaPrev,
}

impl Narrative {
    pub fn new() -> (Self, Task<Message>) {
        let session_id = uuid::Uuid::new_v4().to_string();

        let mut entries = store::load_current_session(&session_id);
        if entries.is_empty() {
            if let Some(e) = store::append(
                EntryKind::System { message: "bonjour. début de session.".to_string() },
                &session_id,
            ) {
                entries.push(e);
            }
        }

        // Entrées de démo (en mémoire uniquement)
        entries.extend(demo_entries());

        let commands = system_commands();
        let palette_filtered = search::filter_and_sort(&commands, Mode::Universal, "");

        let state = Self {
            entries,
            session_id,
            commands,
            palette_open: false,
            palette_query: String::new(),
            palette_filtered,
            palette_selected: 0,
            bottom_query: String::new(),
            sources_loaded: false,
        };

        let tasks = Task::batch([
            Task::perform(sources::load_all(), Message::SourcesLoaded),
            text_input::focus(BOTTOM_INPUT_ID.clone()),
        ]);

        (state, tasks)
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::PaletteToggle => {
                if self.palette_open {
                    self.palette_open = false;
                    self.palette_query.clear();
                    text_input::focus(BOTTOM_INPUT_ID.clone())
                } else {
                    self.palette_open = true;
                    self.palette_query.clear();
                    self.palette_selected = 0;
                    self.recompute_palette();
                    text_input::focus(PALETTE_INPUT_ID.clone())
                }
            }

            Message::PaletteQueryChanged(q) => {
                self.palette_query = q;
                self.palette_selected = 0;
                self.recompute_palette();
                Task::none()
            }

            Message::PaletteSelectNext => {
                if self.palette_open {
                    let count = self.palette_filtered.len();
                    if count > 0 {
                        self.palette_selected = (self.palette_selected + 1).min(count - 1);
                    }
                }
                Task::none()
            }

            Message::PaletteSelectPrevious => {
                if self.palette_open && self.palette_selected > 0 {
                    self.palette_selected -= 1;
                }
                Task::none()
            }

            Message::PaletteExecute => {
                if self.palette_open {
                    if let Some(cmd) = self.palette_filtered.get(self.palette_selected).cloned() {
                        // Enregistre la recherche palette
                        let palette_kind = EntryKind::PaletteSearch {
                            query: self.palette_query.clone(),
                            result_chosen: Some(cmd.name.clone()),
                        };
                        if let Some(e) = store::append(palette_kind, &self.session_id) {
                            self.entries.push(e);
                        }

                        // Enregistre le lancement
                        let launch_kind = EntryKind::AppLaunched {
                            name: cmd.name.clone(),
                            detail: cmd.description.clone(),
                            icon: cmd.icon.clone(),
                            duration: None,
                        };
                        if let Some(e) = store::append(launch_kind, &self.session_id) {
                            self.entries.push(e);
                        }

                        if let Some(exec) = &cmd.exec {
                            spawn_exec(exec);
                        }
                    }
                    self.palette_open = false;
                    self.palette_query.clear();
                    return text_input::focus(BOTTOM_INPUT_ID.clone());
                }
                Task::none()
            }

            Message::PaletteCancel => {
                if self.palette_open {
                    self.palette_open = false;
                    self.palette_query.clear();
                    return text_input::focus(BOTTOM_INPUT_ID.clone());
                }
                Task::none()
            }

            Message::BottomInputChanged(q) => {
                self.bottom_query = q;
                Task::none()
            }

            Message::BottomInputSubmit => {
                let query = self.bottom_query.trim().to_string();
                if !query.is_empty() {
                    let kind = EntryKind::System { message: query };
                    if let Some(e) = store::append(kind, &self.session_id) {
                        self.entries.push(e);
                    }
                    self.bottom_query.clear();
                }
                Task::none()
            }

            Message::SourcesLoaded(loaded) => {
                let mut new_commands = system_commands();
                new_commands.extend(loaded.commands);
                self.commands = new_commands;
                self.sources_loaded = true;
                self.recompute_palette();
                Task::none()
            }

            Message::ContextAction(cmd) => {
                spawn_exec(&cmd);
                Task::none()
            }

            Message::MediaPlayPause | Message::MediaNext | Message::MediaPrev => {
                // MPRIS D-Bus — TODO: phase 7
                Task::none()
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        view::render(self)
    }

    pub fn subscription(&self) -> Subscription<Message> {
        keyboard::on_key_press(|key, modifiers| {
            let ctrl_k = modifiers.control()
                && matches!(&key, keyboard::Key::Character(c) if c.as_str() == "k");
            if ctrl_k {
                return Some(Message::PaletteToggle);
            }
            match key {
                keyboard::Key::Named(key::Named::ArrowDown) => Some(Message::PaletteSelectNext),
                keyboard::Key::Named(key::Named::ArrowUp) => Some(Message::PaletteSelectPrevious),
                keyboard::Key::Named(key::Named::Escape) => Some(Message::PaletteCancel),
                _ => None,
            }
        })
    }

    fn recompute_palette(&mut self) {
        let (mode, eq) = Mode::detect(&self.palette_query);
        let mode = match mode {
            Mode::Commands => Mode::Commands,
            _ => Mode::Universal,
        };
        self.palette_filtered = search::filter_and_sort(&self.commands, mode, eq);
        let max = self.palette_filtered.len().saturating_sub(1);
        self.palette_selected = self.palette_selected.min(max);
    }
}

fn spawn_exec(exec: &str) {
    let parts: Vec<&str> = exec.split_whitespace().collect();
    if let Some(prog) = parts.first() {
        let _ = std::process::Command::new(prog).args(&parts[1..]).spawn();
    }
}

fn demo_entries() -> Vec<Entry> {
    use chrono::{Duration, Local};

    let now = Local::now();

    vec![
        Entry {
            id: 1000,
            timestamp: now - Duration::minutes(45),
            kind: EntryKind::AppLaunched {
                name: "firefox".to_string(),
                detail: "navigateur · flatpak".to_string(),
                icon: "🌐".to_string(),
                duration: Some(720),
            },
        },
        Entry {
            id: 1001,
            timestamp: now - Duration::minutes(30),
            kind: EntryKind::Media {
                title: "Discovery".to_string(),
                artist: "Daft Punk".to_string(),
                progress_secs: 134,
                duration_secs: 360,
                playing: true,
            },
        },
        Entry {
            id: 1002,
            timestamp: now - Duration::minutes(20),
            kind: EntryKind::Notifications {
                source: "mails".to_string(),
                count: 3,
                items: vec![
                    NotifItem {
                        sender: "marc.l@labo".to_string(),
                        preview: "retour sur le proto immuable, j'ai testé ta branche...".to_string(),
                        actions: vec![
                            ContextAction {
                                label: "lire".to_string(),
                                command: "xdg-open mailto:marc.l@labo".to_string(),
                            },
                            ContextAction {
                                label: "répondre".to_string(),
                                command: "xdg-open mailto:marc.l@labo".to_string(),
                            },
                        ],
                    },
                    NotifItem {
                        sender: "github notifications".to_string(),
                        preview: "3 PR mergées sur forkos/core".to_string(),
                        actions: vec![ContextAction {
                            label: "voir".to_string(),
                            command: "xdg-open https://github.com/notifications".to_string(),
                        }],
                    },
                ],
            },
        },
        Entry {
            id: 1003,
            timestamp: now - Duration::minutes(10),
            kind: EntryKind::FileEdit {
                path: "~/project/forkos/crates/narrative/src/app.rs".to_string(),
                lines: 142,
                preview: "pub struct Narrative {\n    pub entries: Vec<Entry>,\n    pub session_id: String,".to_string(),
                modified_ago: "il y a 10 min".to_string(),
            },
        },
        Entry {
            id: 1004,
            timestamp: now - Duration::minutes(5),
            kind: EntryKind::Shell {
                command: "cargo build --release -p narrative".to_string(),
                output_preview: "   Compiling narrative v0.1.0\n   Finished release [optimized]".to_string(),
                exit_code: 0,
            },
        },
    ]
}
