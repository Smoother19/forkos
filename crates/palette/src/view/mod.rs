mod footer;
pub mod header;
mod results;

use crate::app::{Message, Palette};
use crate::theme;
use iced::widget::{column, container, scrollable, Space};
use iced::{Background, Border, Element, Length};

pub fn render(state: &Palette) -> Element<'_, Message> {
    let separator = || {
        container(Space::new(Length::Fill, Length::Fixed(1.0)))
            .style(|_| container::Style {
                background: Some(Background::Color(theme::HIGHLIGHT_MED)),
                ..Default::default()
            })
    };

    let body = column![
        header::render(&state.query),
        separator(),
        scrollable(results::render(state)).height(Length::Fill),
        separator(),
        footer::render(state.visible_count()),
    ];

    let palette_box = container(body)
        .max_width(620)
        .style(|_| container::Style {
            background: Some(Background::Color(theme::SURFACE)),
            border: Border {
                color: theme::HIGHLIGHT_MED,
                width: 1.0,
                radius: 12.0.into(),
            },
            ..Default::default()
        });

    container(palette_box)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(40)
        .center_x(Length::Fill)
        .style(|_| container::Style {
            background: Some(Background::Color(theme::BASE)),
            ..Default::default()
        })
        .into()
}