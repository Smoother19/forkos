use crate::app::Message;
use crate::entry::ContextAction;
use forkos_shared::theme;
use iced::widget::{button, container, row, text};
use iced::{Background, Border, Element, Padding};

pub fn render(actions: Vec<ContextAction>) -> Element<'static, Message> {
    if actions.is_empty() {
        return row![].into();
    }

    let mut btns = row![].spacing(4);
    for action in actions {
        let cmd = action.command;
        let btn = button(
            text(action.label).size(10).color(theme::MUTED),
        )
        .padding(Padding { top: 3.0, right: 8.0, bottom: 3.0, left: 8.0 })
        .on_press(Message::ContextAction(cmd))
        .style(|_, status| {
            let bg = match status {
                button::Status::Hovered | button::Status::Pressed => {
                    Some(Background::Color(theme::OVERLAY))
                }
                _ => Some(Background::Color(theme::HIGHLIGHT_LOW)),
            };
            button::Style {
                background: bg,
                text_color: theme::MUTED,
                border: Border { radius: 3.0.into(), ..Default::default() },
                shadow: Default::default(),
            }
        });
        btns = btns.push(btn);
    }

    container(btns)
        .padding(Padding { top: 4.0, right: 0.0, bottom: 2.0, left: 16.0 })
        .into()
}

pub fn media_controls() -> Element<'static, Message> {
    let prev = action_btn("⏮", Message::MediaCommand(crate::mpris::MediaCommand::Previous));
    let play = action_btn("⏸", Message::MediaCommand(crate::mpris::MediaCommand::PlayPause));
    let next = action_btn("⏭", Message::MediaCommand(crate::mpris::MediaCommand::Next));
    let queue = label_btn("file d'attente");
    let fullscreen = label_btn("plein écran");

    container(row![prev, play, next, queue, fullscreen].spacing(4))
        .padding(Padding { top: 4.0, right: 0.0, bottom: 2.0, left: 16.0 })
        .into()
}

fn action_btn(icon: &'static str, msg: Message) -> Element<'static, Message> {
    button(text(icon).size(12).color(theme::MUTED))
        .padding(Padding { top: 3.0, right: 6.0, bottom: 3.0, left: 6.0 })
        .on_press(msg)
        .style(|_, status| {
            let bg = match status {
                button::Status::Hovered | button::Status::Pressed => {
                    Some(Background::Color(theme::OVERLAY))
                }
                _ => Some(Background::Color(theme::HIGHLIGHT_LOW)),
            };
            button::Style {
                background: bg,
                text_color: theme::MUTED,
                border: Border { radius: 3.0.into(), ..Default::default() },
                shadow: Default::default(),
            }
        })
        .into()
}

fn label_btn(label: &'static str) -> Element<'static, Message> {
    button(text(label).size(10).color(theme::MUTED))
        .padding(Padding { top: 3.0, right: 8.0, bottom: 3.0, left: 8.0 })
        .on_press(Message::ContextAction(String::new()))
        .style(|_, status| {
            let bg = match status {
                button::Status::Hovered | button::Status::Pressed => {
                    Some(Background::Color(theme::OVERLAY))
                }
                _ => Some(Background::Color(theme::HIGHLIGHT_LOW)),
            };
            button::Style {
                background: bg,
                text_color: theme::MUTED,
                border: Border { radius: 3.0.into(), ..Default::default() },
                shadow: Default::default(),
            }
        })
        .into()
}
