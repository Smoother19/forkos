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
    pub bar_animating: bool,
    pub bar_current_height: u32,
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
    BarAnimationTick,
    ScreenGeometry(u32, u32),
    PtyOutput(String),
    PtyInput(String),
    PtyInputChanged(String),
    PtySubmit,
    PtyRawInput(String),
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

        // Vide le fichier de commande au démarrage pour repartir propre
        let _ = std::fs::write("/tmp/narrative-cmd", "");

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
            bar_animating: false,
            bar_current_height: 648, // 60% de 1080
            screen_height: 1080,
            pty_lines: Vec::new(),
            pty_input: String::new(),
        };

        let foot_warning = {
            let ok = std::process::Command::new("which")
                .arg("foot")
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false);
            if !ok {
                Task::done(Message::PtyOutput(
                    "\x1b[33m[forkOS] foot non trouvé — nano/vim/htop seront ouverts avec xterm\x1b[0m\n"
                        .to_string(),
                ))
            } else {
                Task::none()
            }
        };

        let tasks = Task::batch([
            Task::perform(sources::load_all(), Message::SourcesLoaded),
            text_input::focus(BOTTOM_INPUT_ID.clone()),
            foot_warning,
        ]);

        (state, tasks)
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::BarToggle => {
                self.bar_open = !self.bar_open;
                self.bar_animating = true;
                text_input::focus(BOTTOM_INPUT_ID.clone())
            }

            Message::BarAnimationTick => {
                let target = if self.bar_open {
                    (self.screen_height as f32 * 0.6) as u32
                } else {
                    48
                };

                let diff = target as i32 - self.bar_current_height as i32;

                if diff.abs() <= 2 {
                    self.bar_current_height = target;
                    self.bar_animating = false;
                    return Task::done(Message::SizeChange((0, target)));
                }

                // Ease-out : avance de 20% de la distance restante
                let step = (diff as f32 * 0.20).round() as i32;
                let step = if step == 0 { diff.signum() } else { step };
                self.bar_current_height = (self.bar_current_height as i32 + step).max(0) as u32;

                Task::done(Message::SizeChange((0, self.bar_current_height)))
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
                if !self.palette_open {
                    return Task::none();
                }
                let cmd = match self.palette_filtered.get(self.palette_selected).cloned() {
                    Some(c) => c,
                    None => {
                        self.palette_open = false;
                        self.palette_query.clear();
                        return text_input::focus(BOTTOM_INPUT_ID.clone());
                    }
                };

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
                    if is_tui_app(exec) {
                        let terminal_cmd = format!(
                            "(foot sh -c {} 2>/dev/null || xterm -e sh -c {}) &",
                            shell_quote(exec),
                            shell_quote(exec)
                        );
                        let _ = std::process::Command::new("sh")
                            .arg("-c")
                            .arg(&terminal_cmd)
                            .spawn();
                    } else if is_gui_app(exec) {
                        let to_send = build_gui_command(exec);
                        if let Ok(mut guard) = PTY_WRITER.lock() {
                            if let Some(w) = guard.as_mut() {
                                let _ = w.write_all(to_send.as_bytes());
                            }
                        }
                    } else {
                        let to_send = format!("{}\n", exec);
                        if let Ok(mut guard) = PTY_WRITER.lock() {
                            if let Some(w) = guard.as_mut() {
                                let _ = w.write_all(to_send.as_bytes());
                            }
                        }
                    }
                }

                self.palette_open = false;
                self.palette_query.clear();

                let open_task = if !self.bar_open {
                    self.bar_open = true;
                    self.bar_animating = true;
                    Task::none()
                } else {
                    Task::none()
                };

                Task::batch([open_task, text_input::focus(BOTTOM_INPUT_ID.clone())])
            }

            Message::PaletteCancel => {
                if self.palette_open {
                    self.palette_open = false;
                    self.palette_query.clear();
                    self.palette_selected = 0;
                } else if self.bar_open {
                    self.bar_open = false;
                    return Task::batch([
                        Task::done(Message::SizeChange((0, 48))),
                        text_input::focus(BOTTOM_INPUT_ID.clone()),
                    ]);
                }
                text_input::focus(BOTTOM_INPUT_ID.clone())
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

            Message::PtyInput(s) | Message::PtyRawInput(s) => {
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
                self.pty_input.clear();

                let open_task = if !self.bar_open {
                    self.bar_open = true;
                    self.bar_animating = true;
                    Task::none()
                } else {
                    Task::none()
                };

                if line.is_empty() {
                    // Entrée vide → prompt frais
                    if let Ok(mut guard) = PTY_WRITER.lock() {
                        if let Some(w) = guard.as_mut() {
                            let _ = w.write_all(b"\n");
                        }
                    }
                } else if is_tui_app(&line) {
                    // TUI → terminal externe (foot ou xterm)
                    let terminal_cmd = format!(
                        "(foot sh -c {} 2>/dev/null || xterm -e sh -c {}) &",
                        shell_quote(&line),
                        shell_quote(&line)
                    );
                    let _ = std::process::Command::new("sh")
                        .arg("-c")
                        .arg(&terminal_cmd)
                        .spawn();
                    let app_name = line.split_whitespace().next().unwrap_or("").to_string();
                    let feedback = format!("echo '→ {} ouvert dans un terminal externe'\n", app_name);
                    if let Ok(mut guard) = PTY_WRITER.lock() {
                        if let Some(w) = guard.as_mut() {
                            let _ = w.write_all(feedback.as_bytes());
                        }
                    }
                } else if is_gui_app(&line) {
                    // GUI → background détaché + focus auto après 800ms
                    let to_send = build_gui_command(&line);
                    if let Ok(mut guard) = PTY_WRITER.lock() {
                        if let Some(w) = guard.as_mut() {
                            let _ = w.write_all(to_send.as_bytes());
                        }
                    }
                } else {
                    // Commande normale → PTY direct
                    let to_send = format!("{}\n", line);
                    if let Ok(mut guard) = PTY_WRITER.lock() {
                        if let Some(w) = guard.as_mut() {
                            let _ = w.write_all(to_send.as_bytes());
                        }
                    }
                }

                Task::batch([
                    open_task,
                    text_input::focus(BOTTOM_INPUT_ID.clone()),
                ])
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
                    let parts: Vec<&str> = cmd.split_whitespace().collect();
                    if let Some(prog) = parts.first() {
                        let _ = std::process::Command::new(prog).args(&parts[1..]).spawn();
                    }
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
            bar_animation_subscription(self.bar_animating),
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

fn bar_animation_subscription(animating: bool) -> Subscription<Message> {
    if !animating {
        return Subscription::none();
    }
    iced::time::every(std::time::Duration::from_millis(16))
        .map(|_| Message::BarAnimationTick)
}

fn build_gui_command(line: &str) -> String {
    let clean = line.trim_end_matches('&').trim();
    let binary = clean
        .split_whitespace()
        .next()
        .unwrap_or("")
        .rsplit('/')
        .next()
        .unwrap_or("");
    // Lance en background puis tente un focus après 800ms
    format!(
        "({} &>/dev/null &) && (sleep 0.8 && niri msg action focus-window --app-id {} 2>/dev/null) &\n",
        clean, binary
    )
}

fn keyboard_subscription() -> Subscription<Message> {
    keyboard::on_key_press(|key, modifiers| {
        // Raccourcis Ctrl — les seuls capturés globalement sans risque
        if modifiers.control() {
            if let keyboard::Key::Character(c) = &key {
                return match c.as_str() {
                    "k" => Some(Message::PaletteToggle),
                    "c" => Some(Message::PtyRawInput("\x03".to_string())), // SIGINT
                    "d" => Some(Message::PtyRawInput("\x04".to_string())), // EOF
                    "l" => Some(Message::PtyRawInput("\x0c".to_string())), // clear
                    "a" => Some(Message::PtyRawInput("\x01".to_string())), // début ligne
                    "e" => Some(Message::PtyRawInput("\x05".to_string())), // fin ligne
                    "u" => Some(Message::PtyRawInput("\x15".to_string())), // efface ligne
                    _ => None,
                };
            }
        }

        // Touches sans modificateur — capturées uniquement pour le PTY
        // PAS de Esc, PAS de Enter : gérés par les widgets locaux
        if modifiers == keyboard::Modifiers::empty() {
            return match &key {
                keyboard::Key::Named(key::Named::F20) => Some(Message::BarToggle),
                keyboard::Key::Named(key::Named::Tab) => {
                    Some(Message::PtyRawInput("\t".to_string()))
                }
                keyboard::Key::Named(key::Named::ArrowUp) => {
                    Some(Message::PtyRawInput("\x1b[A".to_string()))
                }
                keyboard::Key::Named(key::Named::ArrowDown) => {
                    Some(Message::PtyRawInput("\x1b[B".to_string()))
                }
                _ => None,
            };
        }

        None
    })
}

/// Polling /tmp/narrative-cmd toutes les 50ms — F20 écrit dans ce fichier
fn toggle_subscription() -> Subscription<Message> {
    use iced::futures::stream;

    Subscription::run_with_id(
        "toggle-file-watch",
        stream::unfold((0u64, false), |(last_size, initialized)| async move {
            if !initialized {
                let _ = std::fs::write("/tmp/narrative-cmd", "");
                tracing::info!("toggle watcher prêt : /tmp/narrative-cmd");
                return Some((Message::Noop, (0u64, true)));
            }

            tokio::time::sleep(std::time::Duration::from_millis(50)).await;

            let path = "/tmp/narrative-cmd";
            let current_size = std::fs::metadata(path).map(|m| m.len()).unwrap_or(0);

            if current_size > last_size {
                let _ = std::fs::write(path, "");
                tracing::debug!("toggle déclenché via fichier");
                return Some((Message::BarToggle, (0u64, true)));
            }

            Some((Message::Noop, (current_size, true)))
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
                    None => {
                        // PTY mort — redémarre dans 1s
                        tracing::warn!("PTY died, restarting...");
                        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                        Some((Message::Noop, None))
                    }
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

fn is_gui_app(cmd: &str) -> bool {
    let binary = cmd
        .split_whitespace()
        .next()
        .unwrap_or("")
        .rsplit('/')
        .next()
        .unwrap_or("");
    const GUI_APPS: &[&str] = &[
        "firefox", "chromium", "chrome", "brave",
        "nautilus", "thunar", "nemo", "dolphin",
        "code", "codium", "subl",
        "gimp", "inkscape", "krita", "blender",
        "vlc", "mpv", "totem",
        "libreoffice", "soffice",
        "slack", "discord", "telegram-desktop",
        "kitty", "alacritty", "foot",
        "gedit", "mousepad", "kate",
        "gnome-terminal", "konsole", "pcmanfm",
    ];
    GUI_APPS.contains(&binary)
}

fn is_tui_app(cmd: &str) -> bool {
    let binary = cmd
        .split_whitespace()
        .next()
        .unwrap_or("")
        .rsplit('/')
        .next()
        .unwrap_or("");
    const TUI_APPS: &[&str] = &[
        "nano", "vim", "nvim", "vi", "emacs",
        "htop", "btop", "top",
        "less", "more", "man",
        "ranger", "lf", "nnn", "mc", "ncdu",
        "fzf", "ssh", "mosh",
        "tmux", "screen",
        "python", "python3", "ipython",
        "node", "irb", "ghci",
        "bash", "zsh", "fish", "sh",
    ];
    TUI_APPS.contains(&binary)
}

fn shell_quote(cmd: &str) -> String {
    format!("'{}'", cmd.replace('\'', "'\\''"))
}
