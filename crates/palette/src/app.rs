use crate::command::{system_commands, Command};
use crate::grep::GrepMatch;
use crate::mode::Mode;
use crate::search;
use crate::shell::ShellEntry;
use crate::sources;
use crate::view;
use iced::keyboard::{self, key};
use iced::{Element, Subscription, Task};

pub struct Palette {
    pub query: String,
    pub commands: Vec<Command>,
    pub selected: usize,
    pub is_loading: bool,
    // Shell
    pub shell_history: Vec<ShellEntry>,
    pub shell_history_nav: Option<usize>,
    // PATH pour autocomplétion shell
    pub path_commands: Vec<String>,
    // Grep
    pub grep_results: Vec<GrepMatch>,
    pub grep_loading: bool,
}

#[derive(Debug, Clone)]
pub enum Message {
    QueryChanged(String),
    SelectNext,
    SelectPrevious,
    Execute,
    Quit,
    // Sources chargées au démarrage
    SourcesLoaded(sources::LoadedSources),
    // Résultats async
    ShellExecuted(ShellEntry),
    GrepCompleted(Vec<GrepMatch>),
}

impl Default for Palette {
    fn default() -> Self {
        Self {
            query: String::new(),
            selected: 0,
            commands: system_commands(),
            is_loading: true,
            shell_history: Vec::new(),
            shell_history_nav: None,
            path_commands: Vec::new(),
            grep_results: Vec::new(),
            grep_loading: false,
        }
    }
}

impl Palette {
    pub fn new() -> (Self, Task<Message>) {
        let load = Task::perform(sources::load_all(), Message::SourcesLoaded);
        (Self::default(), load)
    }

    pub fn mode(&self) -> Mode {
        Mode::detect(&self.query).0
    }

    pub fn effective_query(&self) -> &str {
        Mode::detect(&self.query).1
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::QueryChanged(value) => {
                let old_mode = self.mode();
                self.query = value;
                self.selected = 0;
                self.shell_history_nav = None;

                let new_mode = self.mode();
                let eq = self.effective_query().to_string();

                if new_mode == Mode::FileContent && !eq.is_empty() {
                    self.grep_loading = true;
                    let search_dir = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
                    return Task::perform(
                        crate::grep::search(eq, search_dir),
                        Message::GrepCompleted,
                    );
                }

                if old_mode != new_mode {
                    self.grep_results.clear();
                    self.grep_loading = false;
                }

                Task::none()
            }

            Message::SelectNext => {
                match self.mode() {
                    Mode::Shell => self.shell_history_up(),
                    _ => {
                        let count = self.visible_count();
                        if count > 0 {
                            self.selected = (self.selected + 1).min(count - 1);
                        }
                    }
                }
                Task::none()
            }

            Message::SelectPrevious => {
                match self.mode() {
                    Mode::Shell => self.shell_history_down(),
                    _ => {
                        if self.selected > 0 {
                            self.selected -= 1;
                        }
                    }
                }
                Task::none()
            }

            Message::Execute => match self.mode() {
                Mode::Shell => {
                    let cmd = self.effective_query().to_string();
                    if cmd.is_empty() {
                        return Task::none();
                    }
                    self.query = "$ ".to_string();
                    self.shell_history_nav = None;
                    Task::perform(crate::shell::execute(cmd), Message::ShellExecuted)
                }

                Mode::Web => {
                    let q = self.effective_query().to_string();
                    if !q.is_empty() {
                        let url = format!("https://duckduckgo.com/?q={}", q.replace(' ', "+"));
                        spawn_detached("xdg-open", &[&url]);
                    }
                    std::process::exit(0);
                }

                Mode::Calculator => {
                    if let Some(result) = crate::calculator::evaluate(self.effective_query()) {
                        // Copie dans le presse-papiers
                        let script = format!(
                            "echo -n '{}' | xclip -selection clipboard 2>/dev/null || \
                             echo -n '{}' | xsel --clipboard 2>/dev/null",
                            result, result
                        );
                        let _ = std::process::Command::new("sh").arg("-c").arg(script).spawn();
                    }
                    std::process::exit(0);
                }

                _ => {
                    // Mode Universal ou Commands — exécute la commande sélectionnée
                    if let Some(exec_cmd) = self
                        .selected_command()
                        .and_then(|c| c.exec.clone())
                    {
                        spawn_exec(&exec_cmd);
                    }
                    std::process::exit(0);
                }
            },

            Message::Quit => std::process::exit(0),

            Message::SourcesLoaded(sources) => {
                // On garde les commandes système hardcodées, on ajoute les sources réelles
                let mut new_commands: Vec<Command> = system_commands();
                new_commands.extend(sources.commands);
                self.commands = new_commands;
                self.path_commands = sources.path_commands;
                self.is_loading = false;
                Task::none()
            }

            Message::ShellExecuted(entry) => {
                self.shell_history.push(entry);
                Task::none()
            }

            Message::GrepCompleted(results) => {
                self.grep_results = results;
                self.grep_loading = false;
                Task::none()
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        view::render(self)
    }

    pub fn subscription(&self) -> Subscription<Message> {
        keyboard::on_key_press(|key, _modifiers| match key {
            keyboard::Key::Named(key::Named::ArrowDown) => Some(Message::SelectNext),
            keyboard::Key::Named(key::Named::ArrowUp) => Some(Message::SelectPrevious),
            keyboard::Key::Named(key::Named::Enter) => Some(Message::Execute),
            keyboard::Key::Named(key::Named::Escape) => Some(Message::Quit),
            _ => None,
        })
    }

    pub fn visible_commands(&self) -> Vec<&Command> {
        search::filter_and_sort(&self.commands, self.mode(), self.effective_query())
    }

    pub fn visible_count(&self) -> usize {
        self.visible_commands().len()
    }

    pub fn selected_command(&self) -> Option<&Command> {
        self.visible_commands().get(self.selected).copied()
    }

    fn shell_history_up(&mut self) {
        if self.shell_history.is_empty() {
            return;
        }
        let max = self.shell_history.len() - 1;
        let idx = self.shell_history_nav.map(|i| i.saturating_sub(1)).unwrap_or(max);
        self.shell_history_nav = Some(idx);
        self.query = format!("$ {}", self.shell_history[idx].command);
    }

    fn shell_history_down(&mut self) {
        if let Some(i) = self.shell_history_nav {
            if i + 1 >= self.shell_history.len() {
                self.shell_history_nav = None;
                self.query = "$ ".to_string();
            } else {
                self.shell_history_nav = Some(i + 1);
                self.query = format!("$ {}", self.shell_history[i + 1].command);
            }
        }
    }
}

/// Lance un processus détaché (non-bloquant, hérite pas du terminal).
fn spawn_detached(program: &str, args: &[&str]) {
    let _ = std::process::Command::new(program).args(args).spawn();
}

/// Exécute une commande Exec= depuis un fichier .desktop ou similaire.
fn spawn_exec(exec: &str) {
    let parts: Vec<&str> = exec.split_whitespace().collect();
    if parts.is_empty() {
        return;
    }
    let _ = std::process::Command::new(parts[0]).args(&parts[1..]).spawn();
}
