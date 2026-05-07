use crate::entry::{Entry, EntryKind};
use crate::mpris;
use crate::pty;
use crate::store;
use crate::view;
use forkos_shared::command::{system_commands, Command};
use forkos_shared::mode::Mode;
use forkos_shared::search;
use forkos_shared::sources;
use iced::keyboard::{self, key};
use iced::widget::{scrollable, text_input};
use iced::{Element, Subscription, Task};
use iced_layershell::to_layer_message;
use std::collections::HashMap;
use std::io::Write;
use std::sync::LazyLock;

pub static BOTTOM_INPUT_ID: LazyLock<text_input::Id> = LazyLock::new(text_input::Id::unique);
pub static PALETTE_INPUT_ID: LazyLock<text_input::Id> = LazyLock::new(text_input::Id::unique);
pub static TERMINAL_INPUT_ID: LazyLock<text_input::Id> = LazyLock::new(text_input::Id::unique);

/// Writer global vers le stdin PTY — accessible depuis n'importe quelle branche update()
pub static PTY_WRITER: LazyLock<std::sync::Mutex<Option<Box<dyn Write + Send>>>> =
    LazyLock::new(|| std::sync::Mutex::new(None));

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
    pub sources_loaded: bool,
    pub current_media: Option<mpris::MediaInfo>,
    pub mpris_player: Option<String>,
    pub media_entry_idx: Option<usize>,
    pub active_windows: HashMap<u64, (String, String)>,
    pub active_window_id: Option<u64>,
    pub bar_open: bool,
    pub screen_height: u32,
    // PTY
    pub pty_lines: Vec<pty::PtyLine>,
    pub pty_input: String,
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
    SourcesLoaded(sources::LoadedSources),
    ContextAction(String),
    NiriEvent(NiriWindowEvent),
    NiriConnectionLost,
    MediaUpdate(Option<mpris::MediaInfo>),
    MediaCommand(mpris::MediaCommand),
    MprisUnavailable,
    BarToggle,
    ScreenGeometry(u32, u32),
    PtyOutput(String),
    PtyInput(String),
    PtyInputChanged(String),
    PtySubmit,
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
            sources_loaded: false,
            current_media: None,
            mpris_player: None,
            media_entry_idx: None,
            active_windows: HashMap::new(),
            active_window_id: None,
            bar_open: true,
            screen_height: 1080,
            pty_lines: Vec::new(),
            pty_input: String::new(),
        };

        let tasks = Task::batch([
            Task::perform(sources::load_all(), Message::SourcesLoaded),
            text_input::focus(TERMINAL_INPUT_ID.clone()),
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
                let size_task = Task::done(Message::SizeChange((0, new_height)));
                let focus_task = if self.bar_open {
                    text_input::focus(TERMINAL_INPUT_ID.clone())
                } else {
                    text_input::focus(BOTTOM_INPUT_ID.clone())
                };
                Task::batch([size_task, focus_task])
            }

            Message::ScreenGeometry(_w, h) => {
                self.screen_height = h;
                Task::none()
            }

            Message::PaletteToggle => {
                if self.palette_open {
                    self.palette_open = false;
                    self.palette_query.clear();
                    text_input::focus(TERMINAL_INPUT_ID.clone())
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
                    return text_input::focus(TERMINAL_INPUT_ID.clone());
                }
                Task::none()
            }

            Message::PaletteCancel => {
                if self.palette_open {
                    self.palette_open = false;
                    self.palette_query.clear();
                    return text_input::focus(TERMINAL_INPUT_ID.clone());
                }
                if self.bar_open {
                    self.bar_open = false;
                    return Task::batch([
                        Task::done(Message::SizeChange((0, 48))),
                        text_input::focus(BOTTOM_INPUT_ID.clone()),
                    ]);
                }
                Task::none()
            }

            Message::PtyOutput(chunk) => {
                let new_lines = pty::parse_ansi(&chunk);
                self.pty_lines.extend(new_lines);
                if self.pty_lines.len() > 5000 {
                    self.pty_lines.drain(0..self.pty_lines.len() - 5000);
                }
                scrollable::snap_to(
                    view::terminal::TERMINAL_SCROLL.clone(),
                    scrollable::RelativeOffset { x: 0.0, y: 1.0 },
                )
            }

            Message::PtyInput(s) => {
                if let Ok(mut w) = PTY_WRITER.lock() {
                    if let Some(writer) = w.as_mut() {
                        let _ = writer.write_all(s.as_bytes());
                    }
                }
                Task::none()
            }

            Message::PtyInputChanged(s) => {
                self.pty_input = s;
                Task::none()
            }

            Message::PtySubmit => {
                let line = self.pty_input.trim().to_string();
                if line.is_empty() {
                    return Task::none();
                }
                self.pty_input.clear();

                let open_task = if !self.bar_open {
                    self.bar_open = true;
                    let h = (self.screen_height as f32 * 0.6) as u32;
                    Task::done(Message::SizeChange((0, h)))
                } else {
                    Task::none()
                };

                let input = format!("{}\n", line);
                let write_result = PTY_WRITER
                    .lock()
                    .ok()
                    .and_then(|mut guard| {
                        guard.as_mut().map(|w| w.write_all(input.as_bytes()).ok())
                    });
                if write_result.is_none() {
                    tracing::warn!("PtySubmit: PTY writer not ready");
                }

                open_task
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
                    Task::none()
                }
                NiriWindowEvent::Closed { id } => {
                    if let Some((app_id, _)) = self.active_windows.remove(&id) {
                        self.add_entry(EntryKind::System {
                            message: format!("fermé : {}", app_id),
                        });
                    }
                    Task::none()
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

                        if track_changed {
                            self.add_entry(kind);
                            self.media_entry_idx = self.entries.len().checked_sub(1);
                        } else if let Some(idx) = self.media_entry_idx {
                            if let Some(e) = self.entries.get_mut(idx) {
                                e.kind = kind;
                            }
                        }

                        self.mpris_player = Some(info.service.clone());
                        self.current_media = Some(info);
                        Task::none()
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

            Message::Noop => Task::none(),

            // Variants générées par #[to_layer_message]
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
            toggle_subscription(),
            pty_subscription(),
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

enum ToggleState {
    Init,
    Socket(tokio::net::UnixListener),
    Signal,
}

/// Essaie d'abord le socket Unix, tombe back sur SIGUSR1 si bind échoue
fn toggle_subscription() -> Subscription<Message> {
    use iced::futures::stream;

    Subscription::run_with_id(
        "toggle-listener",
        stream::unfold(ToggleState::Init, |state| async move {
            match state {
                ToggleState::Init => {
                    let path = "/tmp/narrative.sock";
                    let _ = std::fs::remove_file(path);
                    match tokio::net::UnixListener::bind(path) {
                        Ok(listener) => {
                            Some((Message::Noop, ToggleState::Socket(listener)))
                        }
                        Err(e) => {
                            tracing::warn!("socket bind failed ({}), falling back to SIGUSR1", e);
                            Some((Message::Noop, ToggleState::Signal))
                        }
                    }
                }

                ToggleState::Socket(listener) => {
                    use tokio::io::{AsyncBufReadExt, BufReader};
                    match listener.accept().await {
                        Ok((stream, _)) => {
                            let mut lines = BufReader::new(stream).lines();
                            if let Ok(Some(line)) = lines.next_line().await {
                                if line.trim() == "toggle" {
                                    return Some((Message::BarToggle, ToggleState::Socket(listener)));
                                }
                            }
                            Some((Message::Noop, ToggleState::Socket(listener)))
                        }
                        Err(_) => Some((Message::Noop, ToggleState::Socket(listener))),
                    }
                }

                ToggleState::Signal => {
                    match tokio::signal::unix::signal(
                        tokio::signal::unix::SignalKind::user_defined1(),
                    ) {
                        Ok(mut sig) => {
                            sig.recv().await;
                            Some((Message::BarToggle, ToggleState::Signal))
                        }
                        Err(_) => {
                            tokio::time::sleep(std::time::Duration::from_secs(60)).await;
                            Some((Message::Noop, ToggleState::Signal))
                        }
                    }
                }
            }
        }),
    )
}

/// Spawne le PTY au premier appel, lit le stdout en continu
fn pty_subscription() -> Subscription<Message> {
    use iced::futures::stream;

    Subscription::run_with_id(
        "pty-output",
        stream::unfold(
            None::<tokio::sync::mpsc::UnboundedReceiver<String>>,
            |state| async move {
                let mut rx = match state {
                    Some(rx) => rx,
                    None => {
                        let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<String>();
                        let result = tokio::task::spawn_blocking(move || {
                            crate::pty::spawn_pty(200, 50, tx)
                        })
                        .await;
                        if let Ok(Ok(writer)) = result {
                            if let Ok(mut w) = PTY_WRITER.lock() {
                                *w = Some(writer);
                            }
                        }
                        rx
                    }
                };
                match rx.recv().await {
                    Some(chunk) => Some((Message::PtyOutput(chunk), Some(rx))),
                    None => None,
                }
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
