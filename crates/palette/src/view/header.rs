use crate::app::Message;
use crate::theme;
use iced::widget::{container, row, text, text_input};
use iced::{Alignment, Background, Border, Color, Element, Padding};
use std::sync::LazyLock;

pub static INPUT_ID: LazyLock<text_input::Id> = LazyLock::new(text_input::Id::unique);

pub fn render(query: &str) -> Element<'_, Message> {
    row![
        text("⌘").size(15).color(theme::IRIS),
        text("›").size(15).color(theme::LOVE),
        text_input("tape une commande...", query)
            .id(INPUT_ID.clone())
            .on_input(Message::QueryChanged)
            .padding(0)
            .size(15)
            .style(|_, _| iced::widget::text_input::Style {
                background: Background::Color(Color::TRANSPARENT),
                border: Border {
                    color: Color::TRANSPARENT,
                    width: 0.0,
                    radius: 0.0.into(),
                },
                icon: theme::TEXT,
                placeholder: theme::MUTED,
                value: theme::TEXT,
                selection: theme::HIGHLIGHT_MED,
            }),
        esc_badge(),
    ]
    .spacing(14)
    .align_y(Alignment::Center)
    .padding(Padding {
        top: 18.0,
        right: 22.0,
        bottom: 18.0,
        left: 22.0,
    })
    .into()
}

fn esc_badge() -> Element<'static, Message> {
    container(text("esc").size(11).color(theme::MUTED))
        .padding(Padding {
            top: 3.0,
            right: 8.0,
            bottom: 3.0,
            left: 8.0,
        })
        .style(|_| container::Style {
            background: Some(Background::Color(theme::OVERLAY)),
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 4.0.into(),
            },
            ..Default::default()
        })
        .into()
}