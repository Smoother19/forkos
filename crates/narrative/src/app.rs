use crate::entry::{Entry, EntryKind};
use crate::mpris;
use crate::store;
use crate::view;
use forkos_shared::command::{system_commands, Command};
use forkos_shared::mode::Mode;
use forkos_shared::search;
use forkos_shared::shell;
use forkos_shared::sources;
use iced::keyboard::{self, key};
use iced::widget::{scrollable, text_input};
use iced::{Element, Subscription, Task};
use iced_layershell::to_layer_message;
use std::collections::HashMap;
use std::sync::LazyLock;

pub static BOTTOM_INPUT_ID: LazyLock<text_input::Id> = LazyLock::new(text_input::Id::unique);
pub static PALETTE_INPUT_ID: LazyLock<text_input::Id> = LazyLock::new(text_input::Id::unique);

#[derive(Debug, Clone)]
pub enum NiriWindowEvent {
    Opened { id: u64, title: String, app_id: String },
    Closed { id: u64 },
    Focused { id: Option<u64> },
}

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
    pub current_media: Option<mpris::MediaInfo>,
    pub mpris_player: Option<String>,
    pub media_entry_idx: Option<usize>,
    pub scroll_at_bottom: bool,
    pub active_windows: HashMap<u64, (String, String)>,
    pub active_window_id: Option<u64>,
    pub bar_open: bool,
    pub screen_height: u32,
}

#[to_layer_message]
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
    NiriEvent(NiriWindowEvent),
    NiriConnectionLost,
    MediaUpdate(Option<mpris::MediaInfo>),
    MediaCommand(mpris::MediaCommand),
    MprisUnavailable,
    FeedScrolled(scrollable::Viewport),
    BarToggle,
    ScreenGeometry(u32, u32),
    ShellExecuted(shell::ShellEntry),
    Noop,
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
            current_media: None,
            mpris_player: None,
            media_entry_idx: None,
            scroll_at_bottom: true,
            active_windows: HashMap::new(),
            active_window_id: None,
            bar_open: true,
            screen_height: 1080,
        };

        let tasks = Task::batch([
            Task::perform(sources::load_all(), Message::SourcesLoaded),
            text_input::focus(BOTTOM_INPUT_ID.clone()),
        ]);

        (state, tasks)
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::BarToggle => {
                self.bar_open = !self.bar_open;
                let new_height = if self.bar_open {
                    (self.screen_height as f32 * 0.6) as u32
                } else {
                    48
                };
                Task::done(Message::SizeChange((0, new_height)))
            }

            Message::ScreenGeometry(_w, h) => {
                self.screen_height = h;
                Task::none()
            }

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
                        self.add_entry(EntryKind::PaletteSearch {
                            query: self.palette_query.clone(),
                            result_chosen: Some(cmd.name.clone()),
                        });
                        self.add_entry(EntryKind::AppLaunched {
                            name: cmd.name.clone(),
                            detail: cmd.description.clone(),
                            icon: cmd.icon.clone(),
                            duration: None,
                        });
                        if let Some(exec) = &cmd.exec {
                            spawn_exec(exec);
                        }
                    }
                    self.palette_open = false;
                    self.palette_query.clear();
                    let snap = self.snap_if_at_bottom();
                    return Task::batch([text_input::focus(BOTTOM_INPUT_ID.clone()), snap]);
                }
                Task::none()
            }

            Message::PaletteCancel => {
                if self.palette_open {
                    self.palette_open = false;
                    self.palette_query.clear();
                    return text_input::focus(BOTTOM_INPUT_ID.clone());
                }
                if self.bar_open {
                    self.bar_open = false;
                    return Task::done(Message::SizeChange((0, 48)));
                }
                Task::none()
            }

            Message::BottomInputChanged(q) => {
                self.bottom_query = q;
                Task::none()
            }

            Message::BottomInputSubmit => {
                let raw = self.bottom_query.trim().to_string();
                if raw.is_empty() {
                    return Task::none();
                }
                self.bottom_query.clear();

                // Préfixe > → palette
                if let Some(rest) = raw.strip_prefix('>') {
                    let q = rest.trim().to_string();
                    self.palette_open = true;
                    self.palette_query = q;
                    self.palette_selected = 0;
                    self.recompute_palette();
                    return text_input::focus(PALETTE_INPUT_ID.clone());
                }

                // Préfixe $ → shell explicite ; pas de préfixe → shell implicite
                let cmd = raw
                    .strip_prefix('$')
                    .map(|s| s.trim().to_string())
                    .unwrap_or(raw.clone());

                if cmd.is_empty() {
                    return Task::none();
                }

                self.add_entry(EntryKind::Shell {
                    command: cmd.clone(),
                    output_preview: "…".to_string(),
                    exit_code: -1,
                });
                let snap = self.snap_if_at_bottom();
                Task::batch([
                    Task::perform(shell::execute(cmd), Message::ShellExecuted),
                    snap,
                ])
            }

            Message::ShellExecuted(entry) => {
                if let Some(last) = self.entries.iter_mut().rev().find(|e| {
                    matches!(&e.kind, EntryKind::Shell { exit_code: -1, .. })
                }) {
                    last.kind = EntryKind::Shell {
                        command: entry.command.clone(),
                        output_preview: truncate_output(&entry.output, 12),
                        exit_code: if entry.success { 0 } else { 1 },
                    };
                }
                self.snap_if_at_bottom()
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
                if !cmd.is_empty() {
                    spawn_exec(&cmd);
                }
                Task::none()
            }

            Message::NiriEvent(ev) => match ev {
                NiriWindowEvent::Opened { id, title, app_id } => {
                    self.active_windows.insert(id, (app_id.clone(), title.clone()));
                    self.add_entry(EntryKind::AppLaunched {
                        name: app_id,
                        detail: title,
                        icon: String::new(),
                        duration: None,
                    });
                    self.snap_if_at_bottom()
                }
                NiriWindowEvent::Closed { id } => {
                    if let Some((app_id, _)) = self.active_windows.remove(&id) {
                        self.add_entry(EntryKind::System {
                            message: format!("fermé : {}", app_id),
                        });
                    }
                    self.snap_if_at_bottom()
                }
                NiriWindowEvent::Focused { id } => {
                    self.active_window_id = id;
                    Task::none()
                }
            },

            Message::NiriConnectionLost => Task::none(),

            Message::MediaUpdate(info) => {
                match info {
                    Some(info) => {
                        let track_changed = self
                            .current_media
                            .as_ref()
                            .map(|m| m.title != info.title || m.artist != info.artist)
                            .unwrap_or(true);

                        let kind = EntryKind::Media {
                            title: info.title.clone(),
                            artist: info.artist.clone(),
                            progress_secs: info.progress_secs,
                            duration_secs: info.duration_secs,
                            playing: info.playing,
                        };

                        let snap = if track_changed {
                            self.add_entry(kind);
                            self.media_entry_idx = self.entries.len().checked_sub(1);
                            self.snap_if_at_bottom()
                        } else {
                            if let Some(idx) = self.media_entry_idx {
                                if let Some(e) = self.entries.get_mut(idx) {
                                    e.kind = kind;
                                }
                            }
                            Task::none()
                        };

                        self.mpris_player = Some(info.service.clone());
                        self.current_media = Some(info);
                        snap
                    }
                    None => {
                        self.mpris_player = None;
                        Task::none()
                    }
                }
            }

            Message::MediaCommand(cmd) => {
                if let Some(service) = self.mpris_player.clone() {
                    Task::perform(
                        async move {
                            if let Ok(conn) = zbus::Connection::session().await {
                                mpris::send_command(&conn, &service, cmd).await;
                            }
                        },
                        |_| Message::Noop,
                    )
                } else {
                    Task::none()
                }
            }

            Message::MprisUnavailable => Task::none(),

            Message::FeedScrolled(viewport) => {
                self.scroll_at_bottom = viewport.relative_offset().y >= 0.99;
                Task::none()
            }

            Message::Noop => Task::none(),

            // Variants ajoutées par #[to_layer_message] — interceptées par le runtime
            _ => Task::none(),
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        view::render(self)
    }

    pub fn subscription(&self) -> Subscription<Message> {
        Subscription::batch([
            keyboard_subscription(),
            niri_subscription(),
            mpris_subscription(),
            signal_subscription(),
        ])
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

    fn add_entry(&mut self, kind: EntryKind) {
        if let Some(e) = store::append(kind, &self.session_id) {
            self.entries.push(e);
        }
    }

    fn snap_if_at_bottom(&self) -> Task<Message> {
        if self.scroll_at_bottom {
            scrollable::snap_to(
                view::FEED_SCROLL_ID.clone(),
                scrollable::RelativeOffset { x: 0.0, y: 1.0 },
            )
        } else {
            Task::none()
        }
    }
}

fn truncate_output(out: &str, max_lines: usize) -> String {
    let lines: Vec<&str> = out.lines().collect();
    if lines.len() <= max_lines {
        out.to_string()
    } else {
        let kept: Vec<&str> = lines.iter().take(max_lines).copied().collect();
        format!("{}\n… ({} lignes en plus)", kept.join("\n"), lines.len() - max_lines)
    }
}

fn keyboard_subscription() -> Subscription<Message> {
    keyboard::on_key_press(|key, modifiers| {
        let ctrl_k = modifiers.control()
            && matches!(&key, keyboard::Key::Character(c) if c.as_str() == "k");
        if ctrl_k {
            return Some(Message::PaletteToggle);
        }
        match key {
            keyboard::Key::Named(key::Named::F20) => Some(Message::BarToggle),
            keyboard::Key::Named(key::Named::ArrowDown) => Some(Message::PaletteSelectNext),
            keyboard::Key::Named(key::Named::ArrowUp) => Some(Message::PaletteSelectPrevious),
            keyboard::Key::Named(key::Named::Escape) => Some(Message::PaletteCancel),
            _ => None,
        }
    })
}

fn signal_subscription() -> Subscription<Message> {
    use iced::futures::stream;

    Subscription::run_with_id(
        "sigusr1-toggle",
        stream::unfold(
            None::<tokio::signal::unix::Signal>,
            |state| async move {
                let mut sig = match state {
                    Some(s) => s,
                    None => {
                        match tokio::signal::unix::signal(
                            tokio::signal::unix::SignalKind::user_defined1(),
                        ) {
                            Ok(s) => s,
                            Err(_) => {
                                std::future::pending::<()>().await;
                                unreachable!()
                            }
                        }
                    }
                };
                sig.recv().await;
                Some((Message::BarToggle, Some(sig)))
            },
        ),
    )
}

enum NiriState {
    Disconnected,
    Connected(
        tokio::process::Child,
        tokio::io::Lines<tokio::io::BufReader<tokio::process::ChildStdout>>,
    ),
}

fn niri_subscription() -> Subscription<Message> {
    use iced::futures::stream;
    use tokio::io::{AsyncBufReadExt, BufReader};

    Subscription::run_with_id(
        "niri-ipc",
        stream::unfold(NiriState::Disconnected, |state| async move {
            match state {
                NiriState::Disconnected => {
                    match tokio::process::Command::new("niri")
                        .args(["msg", "-j", "event-stream"])
                        .stdout(std::process::Stdio::piped())
                        .stderr(std::process::Stdio::null())
                        .spawn()
                    {
                        Ok(mut child) => match child.stdout.take() {
                            Some(stdout) => {
                                let lines = BufReader::new(stdout).lines();
                                Some((Message::Noop, NiriState::Connected(child, lines)))
                            }
                            None => {
                                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                                Some((Message::Noop, NiriState::Disconnected))
                            }
                        },
                        Err(_) => {
                            tokio::time::sleep(std::time::Duration::from_secs(10)).await;
                            Some((Message::Noop, NiriState::Disconnected))
                        }
                    }
                }
                NiriState::Connected(child, mut lines) => match lines.next_line().await {
                    Ok(Some(line)) => {
                        let msg = parse_niri_event(&line)
                            .map(Message::NiriEvent)
                            .unwrap_or(Message::Noop);
                        Some((msg, NiriState::Connected(child, lines)))
                    }
                    _ => {
                        drop(child);
                        tokio::time::sleep(std::time::Duration::from_secs(3)).await;
                        Some((Message::NiriConnectionLost, NiriState::Disconnected))
                    }
                },
            }
        }),
    )
}

enum MprisState {
    Disconnected,
    Connected(zbus::Connection),
}

fn mpris_subscription() -> Subscription<Message> {
    use iced::futures::stream;

    Subscription::run_with_id(
        "mpris-poll",
        stream::unfold(MprisState::Disconnected, |state| async move {
            match state {
                MprisState::Disconnected => {
                    match zbus::Connection::session().await {
                        Ok(conn) => {
                            let info = crate::mpris::poll_media(&conn).await;
                            Some((Message::MediaUpdate(info), MprisState::Connected(conn)))
                        }
                        Err(_) => {
                            tokio::time::sleep(std::time::Duration::from_secs(30)).await;
                            Some((Message::MprisUnavailable, MprisState::Disconnected))
                        }
                    }
                }
                MprisState::Connected(conn) => {
                    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                    let info = crate::mpris::poll_media(&conn).await;
                    Some((Message::MediaUpdate(info), MprisState::Connected(conn)))
                }
            }
        }),
    )
}

fn parse_niri_event(line: &str) -> Option<NiriWindowEvent> {
    let v: serde_json::Value = serde_json::from_str(line).ok()?;

    if let Some(data) = v.get("WindowOpenedOrChanged").or_else(|| v.get("WindowOpened")) {
        let win = data.get("window")?;
        let id = win.get("id")?.as_u64()?;
        let title = win.get("title").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let app_id = win.get("app_id").and_then(|v| v.as_str()).unwrap_or("").to_string();
        return Some(NiriWindowEvent::Opened { id, title, app_id });
    }

    if let Some(data) = v.get("WindowClosed") {
        let id = data
            .get("id")
            .or_else(|| data.get("window_id"))
            .and_then(|v| v.as_u64())?;
        return Some(NiriWindowEvent::Closed { id });
    }

    if let Some(data) = v.get("WindowFocusChanged").or_else(|| v.get("WindowFocused")) {
        let id = data
            .get("id")
            .or_else(|| data.get("window_id"))
            .and_then(|v| v.as_u64());
        return Some(NiriWindowEvent::Focused { id });
    }

    None
}

fn spawn_exec(exec: &str) {
    let parts: Vec<&str> = exec.split_whitespace().collect();
    if let Some(prog) = parts.first() {
        let _ = std::process::Command::new(prog).args(&parts[1..]).spawn();
    }
}

// Unused but kept for completeness — narrative n'a pas de demo data
#[allow(dead_code)]
fn _demo_entries_removed() {}
