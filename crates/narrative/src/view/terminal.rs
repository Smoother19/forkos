use crate::app::{Message, Narrative, TERMINAL_INPUT_ID};
use crate::pty::PtyLine;
use forkos_shared::theme;
use iced::widget::{column, container, row, scrollable, text, text_input, Space};
use iced::{Background, Border, Color, Element, Font, Length, Padding};
use std::sync::LazyLock;

pub static TERMINAL_SCROLL: LazyLock<scrollable::Id> = LazyLock::new(scrollable::Id::unique);

pub fn render(state: &Narrative) -> Element<'_, Message> {
    let visible_lines = if state.pty_lines.len() > 500 {
        &state.pty_lines[state.pty_lines.len() - 500..]
    } else {
        &state.pty_lines
    };

    let mut output_col = column![].spacing(0);
    for line in visible_lines {
        output_col = output_col.push(render_line(line));
    }

    let output = scrollable(
        container(output_col)
            .padding(Padding { top: 8.0, right: 12.0, bottom: 8.0, left: 12.0 })
            .width(Length::Fill),
    )
    .height(Length::Fill)
    .id(TERMINAL_SCROLL.clone());

    let input_row = row![
        text("❯ ").size(12).color(theme::PINE).font(Font::MONOSPACE),
        text_input("", &state.pty_input)
            .id(TERMINAL_INPUT_ID.clone())
            .on_input(Message::PtyInputChanged)
            .on_submit(Message::PtySubmit)
            .padding(0)
            .size(12)
            .font(Font::MONOSPACE)
            .style(|_, _| iced::widget::text_input::Style {
                background: Background::Color(Color::TRANSPARENT),
                border: Border::default(),
                icon: theme::TEXT,
                placeholder: theme::MUTED,
                value: theme::TEXT,
                selection: theme::HIGHLIGHT_MED,
            }),
    ]
    .padding(Padding { top: 6.0, right: 12.0, bottom: 8.0, left: 12.0 });

    container(column![output, separator(), input_row])
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|_| container::Style {
            background: Some(Background::Color(theme::BASE)),
            ..Default::default()
        })
        .into()
}

fn render_line(line: &PtyLine) -> Element<'_, Message> {
    if line.spans.is_empty() {
        return container(Space::new(Length::Fill, Length::Fixed(14.0))).into();
    }

    let mut r = row![].spacing(0);
    for span in &line.spans {
        let base_color = span.color.to_iced_color();
        // Simule le bold par légère surbrillance (iced 0.13 n'expose pas font-weight facilement)
        let color = if span.bold {
            iced::Color {
                r: (base_color.r * 1.15).min(1.0),
                g: (base_color.g * 1.15).min(1.0),
                b: (base_color.b * 1.15).min(1.0),
                a: base_color.a,
            }
        } else {
            base_color
        };
        r = r.push(text(span.text.clone()).size(12).font(Font::MONOSPACE).color(color));
    }
    r.into()
}

fn separator() -> Element<'static, Message> {
    container(Space::new(Length::Fill, Length::Fixed(1.0)))
        .style(|_| container::Style {
            background: Some(Background::Color(theme::HIGHLIGHT_MED)),
            ..Default::default()
        })
        .into()
}
