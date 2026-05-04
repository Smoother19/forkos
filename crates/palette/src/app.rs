use crate::command::{default_commands, Command};
use crate::search;
use crate::view;
use iced::keyboard::{self, key};
use iced::{Element, Subscription, Task};

pub struct Palette {
    pub query: String,
    pub commands: Vec<Command>,
    pub selected: usize,
}

#[derive(Debug, Clone)]
pub enum Message {
    QueryChanged(String),
    SelectNext,
    SelectPrevious,
    Execute,
    Quit,
}

impl Default for Palette {
    fn default() -> Self {
        Self {
            query: String::new(),
            selected: 0,
            commands: default_commands(),
        }
    }
}

impl Palette {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::QueryChanged(value) => {
                self.query = value;
                self.selected = 0;
                Task::none()
            }
            Message::SelectNext => {
                let count = self.visible_count();
                if count > 0 {
                    self.selected = (self.selected + 1).min(count - 1);
                }
                Task::none()
            }
            Message::SelectPrevious => {
                if self.selected > 0 {
                    self.selected -= 1;
                }
                Task::none()
            }
            Message::Execute => {
                if let Some(cmd) = self.selected_command() {
                    println!("→ exécuter: {}", cmd.name);
                }
                std::process::exit(0);
            }
            Message::Quit => {
                std::process::exit(0);
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
        search::filter_and_sort(&self.commands, &self.query)
    }

    pub fn visible_count(&self) -> usize {
        self.visible_commands().len()
    }

    pub fn selected_command(&self) -> Option<&Command> {
        self.visible_commands().get(self.selected).copied()
    }

    pub fn new() -> (Self, Task<Message>) {
    (
        Self::default(),
        iced::widget::text_input::focus(crate::view::header::INPUT_ID.clone()),
    )
}
}