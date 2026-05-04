use crate::app::{Message, Palette};
use crate::command::{Command, Section};
use crate::theme;
use iced::widget::{column, container, row, text, Space};
use iced::{Alignment, Background, Border, Color, Element, Length, Padding};

pub fn render(state: &Palette) -> Element<'_, Message> {
    let visible = state.visible_commands();

    let mut col = column![].spacing(0);
    let mut current_section: Option<Section> = None;

    for (index, cmd) in visible.iter().enumerate() {
        if current_section != Some(cmd.section) {
            col = col.push(section_header(cmd.section.label()));
            current_section = Some(cmd.section);
        }
        let is_selected = index == state.selected;
        col = col.push(command_row(cmd, is_selected, &state.query));
    }

    col.into()
}

fn section_header(label: &str) -> Element<'_, Message> {
    container(text(label.to_string()).size(10).color(theme::MUTED))
        .padding(Padding {
            top: 14.0,
            right: 22.0,
            bottom: 6.0,
            left: 22.0,
        })
        .into()
}

fn command_row<'a>(cmd: &'a Command, selected: bool, query: &str) -> Element<'a, Message> {
    let icon_box = container(text(cmd.icon).size(14).color(cmd.section.icon_color()))
        .width(Length::Fixed(32.0))
        .height(Length::Fixed(32.0))
        .center_x(Length::Fixed(32.0))
        .center_y(Length::Fixed(32.0))
        .style(|_| container::Style {
            background: Some(Background::Color(theme::OVERLAY)),
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 4.0.into(),
            },
            ..Default::default()
        });

    let info = column![
        highlighted_text(&cmd.name, query, 13.0, theme::TEXT),
        text(&cmd.description).size(11).color(theme::MUTED),
    ]
    .spacing(2);

    let shortcut_color = if selected { theme::LOVE } else { theme::MUTED };
    let shortcut = text(cmd.shortcut).size(10).color(shortcut_color);

    let line = row![
        icon_box,
        info,
        Space::new(Length::Fill, Length::Shrink),
        shortcut,
    ]
    .spacing(14)
    .align_y(Alignment::Center)
    .padding(Padding {
        top: 10.0,
        right: 22.0,
        bottom: 10.0,
        left: 22.0,
    });

    let bg = if selected {
        theme::OVERLAY
    } else {
        Color::TRANSPARENT
    };

    container(line)
        .width(Length::Fill)
        .style(move |_| container::Style {
            background: Some(Background::Color(bg)),
            ..Default::default()
        })
        .into()
}

fn highlighted_text<'a>(name: &'a str, query: &str, size: f32, color: Color) -> Element<'a, Message> {
    if query.is_empty() {
        return text(name).size(size).color(color).into();
    }

    let lower_name = name.to_lowercase();
    let lower_query = query.to_lowercase();

    if let Some(start) = lower_name.find(&lower_query) {
        let end = start + lower_query.len();
        let before = &name[..start];
        let middle = &name[start..end];
        let after = &name[end..];

        let mut segments = row![].spacing(0);
        if !before.is_empty() {
            segments = segments.push(text(before.to_string()).size(size).color(color));
        }
        let highlight = container(text(middle.to_string()).size(size).color(color)).style(|_| {
            container::Style {
                background: Some(Background::Color(theme::HIGHLIGHT_MED)),
                ..Default::default()
            }
        });
        segments = segments.push(highlight);
        if !after.is_empty() {
            segments = segments.push(text(after.to_string()).size(size).color(color));
        }
        segments.into()
    } else {
        text(name).size(size).color(color).into()
    }
}