mod context_bar;
mod entry_view;
mod header;
mod palette_inline;

use crate::app::{Message, Narrative, BOTTOM_INPUT_ID};
use forkos_shared::theme;
use iced::widget::{column, container, row, scrollable, text, text_input, Space};
use iced::{Background, Border, Color, Element, Length, Padding};
use std::sync::LazyLock;

pub static FEED_SCROLL_ID: LazyLock<scrollable::Id> = LazyLock::new(scrollable::Id::unique);

pub fn render(state: &Narrative) -> Element<'_, Message> {
    let window = column![
        header::render(),
        separator(),
        scrollable(entries_column(state))
            .height(Length::Fill)
            .id(FEED_SCROLL_ID.clone())
            .on_scroll(Message::FeedScrolled),
        separator(),
        bottom_bar(state),
    ]
    .height(Length::Fill);

    container(window)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|_| container::Style {
            background: Some(Background::Color(theme::BASE)),
            ..Default::default()
        })
        .into()
}

fn entries_column(state: &Narrative) -> Element<'_, Message> {
    let mut col = column![].spacing(4);

    for entry in &state.entries {
        col = col.push(entry_view::render(entry));
    }

    // Zone vide en bas pour aérer
    col = col.push(Space::new(Length::Fill, Length::Fixed(32.0)));

    container(col)
        .max_width(600)
        .padding(Padding { top: 20.0, right: 24.0, bottom: 8.0, left: 24.0 })
        .center_x(Length::Fill)
        .into()
}

fn bottom_bar(state: &Narrative) -> Element<'_, Message> {
    let inner: Element<'_, Message> = if state.palette_open {
        palette_inline::render(state)
    } else {
        plain_input(state)
    };

    container(
        container(inner)
            .max_width(600)
            .center_x(Length::Fill)
            .padding(Padding { top: 0.0, right: 24.0, bottom: 0.0, left: 24.0 }),
    )
    .width(Length::Fill)
    .style(|_| container::Style {
        background: Some(Background::Color(theme::BASE)),
        ..Default::default()
    })
    .into()
}

fn plain_input(state: &Narrative) -> Element<'_, Message> {
    let prompt_row = row![
        text("›").size(14).color(theme::FOAM),
        text_input("", &state.bottom_query)
            .id(BOTTOM_INPUT_ID.clone())
            .on_input(Message::BottomInputChanged)
            .on_submit(Message::BottomInputSubmit)
            .padding(0)
            .size(13)
            .style(|_, _| iced::widget::text_input::Style {
                background: Background::Color(Color::TRANSPARENT),
                border: Border::default(),
                icon: theme::TEXT,
                placeholder: theme::MUTED,
                value: theme::TEXT,
                selection: theme::HIGHLIGHT_MED,
            }),
    ]
    .spacing(8)
    .padding(Padding { top: 14.0, right: 0.0, bottom: 14.0, left: 0.0 });

    prompt_row.into()
}

pub fn separator() -> Element<'static, Message> {
    container(Space::new(Length::Fill, Length::Fixed(1.0)))
        .style(|_| container::Style {
            background: Some(Background::Color(theme::HIGHLIGHT_MED)),
            ..Default::default()
        })
        .into()
}

/// Bloc avec border-left coloré (row: [bande 2px | contenu])
pub fn left_border_block<'a>(
    content: Element<'a, Message>,
    border_color: Color,
    bg: Color,
) -> Element<'a, Message> {
    let strip = container(Space::new(Length::Fixed(0.0), Length::Fixed(0.0)))
        .width(Length::Fixed(2.0))
        .height(Length::Fill)
        .style(move |_| container::Style {
            background: Some(Background::Color(border_color)),
            ..Default::default()
        });

    let body = container(content)
        .width(Length::Fill)
        .padding(Padding { top: 10.0, right: 16.0, bottom: 10.0, left: 12.0 })
        .style(move |_| container::Style {
            background: Some(Background::Color(bg)),
            ..Default::default()
        });

    row![strip, body].into()
}
