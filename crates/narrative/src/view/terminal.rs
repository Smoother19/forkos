use crate::app::{Message, Narrative};
use crate::pty::PtyLine;
use iced::widget::{column, container, row, scrollable, text, Space};
use iced::{Element, Font, Length, Padding};
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

    output_col = output_col.push(Space::new(Length::Fill, Length::Fixed(8.0)));

    scrollable(
        container(output_col)
            .padding(Padding { top: 8.0, right: 12.0, bottom: 0.0, left: 12.0 })
            .width(Length::Fill),
    )
    .height(Length::Fill)
    .id(TERMINAL_SCROLL.clone())
    .into()
}

fn render_line(line: &PtyLine) -> Element<'_, Message> {
    if line.spans.is_empty() {
        return container(Space::new(Length::Fill, Length::Fixed(14.0))).into();
    }

    let mut r = row![].spacing(0);
    for span in &line.spans {
        let base_color = span.color.to_iced_color();
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
