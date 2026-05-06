use crate::app::Message;
use forkos_shared::theme;
use iced::widget::{container, row, text, Space};
use iced::{Element, Length, Padding};

pub fn render() -> Element<'static, Message> {
    let now = chrono::Local::now();
    let session_str = format!("~ · session {} · {}", now.format("%d %b %Y"), now.format("%H:%M"));
    let os_str = format!("os {}", now.format("%Y.%m"));

    let content = row![
        text(session_str).size(10).color(theme::MUTED),
        Space::new(Length::Fill, Length::Shrink),
        text(os_str).size(10).color(theme::SUBTLE),
    ]
    .padding(Padding { top: 10.0, right: 24.0, bottom: 10.0, left: 24.0 });

    container(content)
        .width(Length::Fill)
        .into()
}
