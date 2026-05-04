use crate::app::Message;
use crate::mode::Mode;
use crate::theme;
use iced::widget::{container, row, text, text_input};
use iced::{Alignment, Background, Border, Color, Element, Padding};
use std::sync::LazyLock;

pub static INPUT_ID: LazyLock<text_input::Id> = LazyLock::new(text_input::Id::unique);

pub fn render(query: &str, mode: Mode) -> Element<'_, Message> {
    let placeholder = mode.placeholder();

    let mut r = row![]
        .spacing(10)
        .align_y(Alignment::Center)
        .padding(Padding { top: 18.0, right: 22.0, bottom: 18.0, left: 22.0 });

    // Prompt de base
    r = r.push(text("⌘").size(15).color(theme::IRIS));
    r = r.push(text("›").size(15).color(theme::LOVE));

    // Badge de mode (sauf Universal)
    if mode != Mode::Universal {
        r = r.push(mode_badge(mode));
    }

    r = r.push(
        text_input(placeholder, query)
            .id(INPUT_ID.clone())
            .on_input(Message::QueryChanged)
            .padding(0)
            .size(15)
            .style(|_, _| iced::widget::text_input::Style {
                background: Background::Color(Color::TRANSPARENT),
                border: Border { color: Color::TRANSPARENT, width: 0.0, radius: 0.0.into() },
                icon: theme::TEXT,
                placeholder: theme::MUTED,
                value: theme::TEXT,
                selection: theme::HIGHLIGHT_MED,
            }),
    );

    r = r.push(esc_badge());

    r.into()
}

fn mode_badge(mode: Mode) -> Element<'static, Message> {
    let color = mode.color();
    container(text(mode.label()).size(11).color(color))
        .padding(Padding { top: 3.0, right: 8.0, bottom: 3.0, left: 8.0 })
        .style(move |_| container::Style {
            background: Some(Background::Color(with_alpha(color, 0.15))),
            border: Border { color: with_alpha(color, 0.4), width: 1.0, radius: 4.0.into() },
            ..Default::default()
        })
        .into()
}

fn esc_badge() -> Element<'static, Message> {
    container(text("esc").size(11).color(theme::MUTED))
        .padding(Padding { top: 3.0, right: 8.0, bottom: 3.0, left: 8.0 })
        .style(|_| container::Style {
            background: Some(Background::Color(theme::OVERLAY)),
            border: Border { color: Color::TRANSPARENT, width: 0.0, radius: 4.0.into() },
            ..Default::default()
        })
        .into()
}

fn with_alpha(c: Color, alpha: f32) -> Color {
    Color { r: c.r, g: c.g, b: c.b, a: alpha }
}
