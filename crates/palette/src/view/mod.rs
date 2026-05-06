mod footer;
pub mod header;
mod results;

use crate::app::{Message, Palette};
use crate::theme;
use iced::widget::{column, container, scrollable, Space};
use iced::{Background, Border, Color, Element, Length};

pub fn render(state: &Palette) -> Element<'_, Message> {
    let mode = state.mode();

    let separator = || {
        container(Space::new(Length::Fill, Length::Fixed(1.0))).style(|_| container::Style {
            background: Some(Background::Color(theme::HIGHLIGHT_MED)),
            ..Default::default()
        })
    };

    let visible_count = match mode {
        crate::mode::Mode::FileContent => state.grep_results.len(),
        _ => state.visible_count(),
    };

    let body = column![
        header::render(&state.query, mode),
        separator(),
        scrollable(results::render(state)).height(Length::Fill),
        separator(),
        footer::render(visible_count, mode, state.is_loading),
    ];

    let palette_box = container(body)
        .max_width(620)
        .max_height(520)
        .style(|_| container::Style {
            background: Some(Background::Color(theme::SURFACE)),
            border: Border { color: theme::HIGHLIGHT_MED, width: 1.0, radius: 12.0.into() },
            ..Default::default()
        });

    container(palette_box)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .style(|_| container::Style {
            background: Some(Background::Color(Color::TRANSPARENT)),
            ..Default::default()
        })
        .into()
}
