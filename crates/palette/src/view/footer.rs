use crate::app::Message;
use crate::theme;
use iced::widget::{row, text, Space};
use iced::{Element, Length, Padding};

pub fn render(count: usize) -> Element<'static, Message> {
    row![
        shortcut("↑↓", "naviguer"),
        shortcut("↵", "exécuter"),
        shortcut("tab", "mode"),
        Space::new(Length::Fill, Length::Shrink),
        text(format!("{} résultats", count)).size(11).color(theme::MUTED),
    ]
    .spacing(18)
    .padding(Padding {
        top: 10.0,
        right: 22.0,
        bottom: 10.0,
        left: 22.0,
    })
    .into()
}

fn shortcut(key: &'static str, label: &'static str) -> Element<'static, Message> {
    row![
        text(key).size(11).color(theme::LOVE),
        text(label).size(11).color(theme::MUTED),
    ]
    .spacing(6)
    .into()
}