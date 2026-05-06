use crate::app::{Message, Narrative};
use crate::theme;
use iced::widget::{column, container, scrollable, Space};
use iced::{Background, Color, Element, Length, Padding};

mod blocks;
mod composer;
mod header;

pub fn render(state: &Narrative) -> Element<'_, Message> {
    let header = header::render(state);
    let composer = composer::render(state);
    let timeline = render_timeline(state);
    let footer = render_footer(state);

    let body = column![header, composer, timeline]
        .spacing(0);

    let body = container(body)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|_| container::Style {
            background: Some(Background::Color(theme::BASE)),
            ..Default::default()
        });

    column![body, footer].into()
}

fn render_timeline(state: &Narrative) -> Element<'_, Message> {
    if state.entries.is_empty() {
        return container(
            iced::widget::text("le fil est vide. tape une note ci-dessus pour commencer.")
                .size(13)
                .color(theme::MUTED),
        )
        .width(Length::Fill)
        .padding(Padding {
            top: 60.0,
            right: 32.0,
            bottom: 60.0,
            left: 32.0,
        })
        .center_x(Length::Fill)
        .into();
    }

    let mut col = column![]
        .spacing(0)
        .padding(Padding {
            top: 8.0,
            right: 24.0,
            bottom: 24.0,
            left: 24.0,
        });

    let mut last_date: Option<String> = None;

    for entry in &state.entries {
        let date_label = entry.timestamp.format("%A %-d %B").to_string();
        if last_date.as_ref() != Some(&date_label) {
            col = col.push(date_separator(date_label.clone()));
            last_date = Some(date_label);
        }
        col = col.push(blocks::render(entry));
    }

    // Espace en bas pour respirer
    let col = col.push(Space::new(Length::Fill, Length::Fixed(40.0)));

    scrollable(col)
        .height(Length::Fill)
        .width(Length::Fill)
        .into()
}

fn date_separator(label: String) -> Element<'static, Message> {
    container(
        iced::widget::text(label)
            .size(11)
            .color(theme::MUTED),
    )
    .padding(Padding {
        top: 24.0,
        right: 12.0,
        bottom: 8.0,
        left: 12.0,
    })
    .into()
}

fn render_footer(state: &Narrative) -> Element<'_, Message> {
    use iced::widget::{row, text};

    let count = state.entries.len();
    let left_label = format!("{} entrées", count);

    let right: Element<'_, Message> = if let Some(err) = &state.error {
        text(format!("⚠ {}", err))
            .size(10)
            .color(theme::LOVE)
            .into()
    } else {
        text("↵ ajouter  esc quitter")
            .size(10)
            .color(theme::MUTED)
            .into()
    };

    container(
        row![
            text(left_label).size(10).color(theme::MUTED),
            Space::new(Length::Fill, Length::Shrink),
            right,
        ]
        .spacing(14)
        .padding(Padding {
            top: 8.0,
            right: 24.0,
            bottom: 8.0,
            left: 24.0,
        }),
    )
    .style(|_| container::Style {
        background: Some(Background::Color(theme::SURFACE)),
        ..Default::default()
    })
    .width(Length::Fill)
    .into()
}

/// Conteneur stylé avec background overlay et bord arrondi — utilisé par
/// les blocs riches du fil.
pub(crate) fn block_card(content: Element<'_, Message>) -> Element<'_, Message> {
    container(content)
        .width(Length::Fill)
        .padding(Padding {
            top: 10.0,
            right: 14.0,
            bottom: 10.0,
            left: 14.0,
        })
        .style(|_| container::Style {
            background: Some(Background::Color(theme::SURFACE)),
            border: iced::Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 6.0.into(),
            },
            ..Default::default()
        })
        .into()
}
