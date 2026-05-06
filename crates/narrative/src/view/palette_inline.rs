use crate::app::{Message, Narrative, PALETTE_INPUT_ID};
use forkos_shared::command::{Command, Section};
use forkos_shared::theme;
use iced::widget::{column, container, row, text, text_input, Space};
use iced::{Background, Border, Color, Element, Length, Padding};

pub fn render(state: &Narrative) -> Element<'_, Message> {
    let input_row = row![
        text("⌘").size(14).color(theme::IRIS),
        text("›").size(14).color(theme::LOVE),
        text_input(
            forkos_shared::mode::Mode::Universal.placeholder(),
            &state.palette_query,
        )
        .id(PALETTE_INPUT_ID.clone())
        .on_input(Message::PaletteQueryChanged)
        .on_submit(Message::PaletteExecute)
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
        Space::new(Length::Fill, Length::Shrink),
        esc_badge(),
    ]
    .spacing(8)
    .padding(Padding { top: 14.0, right: 0.0, bottom: 8.0, left: 0.0 });

    let results = results_list(state);

    let footer = footer_line(state);

    column![input_row, results, footer].spacing(0).into()
}

fn results_list(state: &Narrative) -> Element<'_, Message> {
    if state.palette_filtered.is_empty() {
        return container(text("aucun résultat").size(11).color(theme::MUTED))
            .padding(Padding { top: 4.0, right: 0.0, bottom: 8.0, left: 26.0 })
            .into();
    }

    let mut col = column![].spacing(0);
    let mut current_section: Option<Section> = None;

    for (i, cmd) in state.palette_filtered.iter().enumerate() {
        if current_section != Some(cmd.section) {
            col = col.push(section_header(cmd.section.label()));
            current_section = Some(cmd.section);
        }
        col = col.push(command_row(cmd, i == state.palette_selected));
    }

    col.into()
}

fn section_header(label: &str) -> Element<'_, Message> {
    container(text(label).size(9).color(theme::MUTED))
        .padding(Padding { top: 8.0, right: 0.0, bottom: 4.0, left: 26.0 })
        .into()
}

fn command_row(cmd: &Command, selected: bool) -> Element<'_, Message> {
    let bg = if selected { theme::OVERLAY } else { Color::TRANSPARENT };
    let border_color = if selected { theme::FOAM } else { Color::TRANSPARENT };

    let left_strip = container(Space::new(Length::Fixed(0.0), Length::Fixed(0.0)))
        .width(Length::Fixed(2.0))
        .height(Length::Fill)
        .style(move |_| container::Style {
            background: Some(Background::Color(border_color)),
            ..Default::default()
        });

    let icon_box = container(text(cmd.icon.clone()).size(13).color(cmd.section.icon_color()))
        .width(Length::Fixed(24.0))
        .height(Length::Fixed(24.0))
        .center_x(Length::Fixed(24.0))
        .center_y(Length::Fixed(24.0));

    let info = column![
        text(cmd.name.clone()).size(12).color(theme::TEXT),
        text(cmd.description.clone()).size(10).color(theme::MUTED),
    ]
    .spacing(2)
    .width(Length::Fill);

    let shortcut_color = if selected { theme::LOVE } else { theme::MUTED };
    let shortcut = text(cmd.shortcut.clone()).size(10).color(shortcut_color);

    let content = row![icon_box, info, shortcut]
        .spacing(10)
        .padding(Padding { top: 7.0, right: 12.0, bottom: 7.0, left: 4.0 });

    let body = container(content).width(Length::Fill).style(move |_| container::Style {
        background: Some(Background::Color(bg)),
        ..Default::default()
    });

    row![left_strip, body].into()
}

fn footer_line(state: &Narrative) -> Element<'_, Message> {
    let count = state.palette_filtered.len();
    let hint = "↑↓ nav  ↵ ouvrir  esc fermer";
    let count_label = format!("{} résultats", count);

    row![
        text(hint).size(10).color(theme::MUTED),
        Space::new(Length::Fill, Length::Shrink),
        text(count_label).size(10).color(theme::SUBTLE),
    ]
    .padding(Padding { top: 6.0, right: 0.0, bottom: 12.0, left: 26.0 })
    .into()
}

fn esc_badge() -> Element<'static, Message> {
    container(text("esc").size(10).color(theme::MUTED))
        .padding(Padding { top: 2.0, right: 6.0, bottom: 2.0, left: 6.0 })
        .style(|_| container::Style {
            background: Some(Background::Color(theme::OVERLAY)),
            border: Border { radius: 3.0.into(), ..Default::default() },
            ..Default::default()
        })
        .into()
}
