mod context_bar;
mod entry_view;
mod header;
pub mod bar;
pub mod terminal;

use crate::app::{Message, Narrative};
use forkos_shared::theme;
use iced::widget::{button, column, container, row, text, Space};
use iced::{Background, Border, Color, Element, Length, Padding};

pub fn render(state: &Narrative) -> Element<'_, Message> {
    if state.bar_open {
        render_open(state)
    } else {
        render_closed(state)
    }
}

/// État fermé : uniquement la barre 48px
pub fn render_closed(state: &Narrative) -> Element<'_, Message> {
    container(bar::render(state))
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|_| container::Style {
            background: Some(Background::Color(theme::SURFACE)),
            ..Default::default()
        })
        .into()
}

/// État ouvert : header → apps (si présentes) → terminal PTY (ou palette) → barre 48px
pub fn render_open(state: &Narrative) -> Element<'_, Message> {
    let mut body = column![];

    body = body.push(header::render());
    body = body.push(separator());

    if !state.active_windows.is_empty() {
        body = body.push(active_apps_section(state));
        body = body.push(separator());
    }

    body = body.push(
        container(terminal::render(state))
            .height(Length::Fill)
            .width(Length::Fill),
    );

    body = body.push(separator());
    body = body.push(bar::render(state));

    container(body.height(Length::Fill))
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|_| container::Style {
            background: Some(Background::Color(theme::BASE)),
            ..Default::default()
        })
        .into()
}

/// Section apps actives : grille de cartes cliquables (uniquement si non vide)
fn active_apps_section(state: &Narrative) -> Element<'_, Message> {
    let mut r = row![].spacing(8);

    let mut windows: Vec<(&u64, &(String, String))> = state.active_windows.iter().collect();
    windows.sort_by_key(|(id, _)| *id);

    for (id, (app_id, title)) in windows {
        let is_active = Some(*id) == state.active_window_id;
        let label = if title.is_empty() { app_id.clone() } else { title.clone() };
        let label = if label.len() > 24 { format!("{}…", &label[..23]) } else { label };
        let sub = if app_id.len() > 18 { format!("{}…", &app_id[..17]) } else { app_id.clone() };

        let bg = if is_active { theme::FOAM } else { theme::OVERLAY };
        let fg = if is_active { theme::BASE } else { theme::TEXT };
        let fg_sub = if is_active { theme::BASE } else { theme::MUTED };
        let cmd = format!("niri msg action focus-window --id {}", id);

        let card = button(
            column![
                text(label).size(12).color(fg),
                text(sub).size(9).color(fg_sub),
            ]
            .spacing(2),
        )
        .padding(Padding { top: 8.0, right: 12.0, bottom: 8.0, left: 12.0 })
        .on_press(Message::ContextAction(cmd))
        .style(move |_, _| button::Style {
            background: Some(Background::Color(bg)),
            border: Border { radius: 6.0.into(), ..Default::default() },
            text_color: fg,
            shadow: Default::default(),
        });

        r = r.push(card);
    }

    let label_row = text(format!("APPS ACTIVES · {}", state.active_windows.len()))
        .size(9)
        .color(theme::MUTED);

    container(column![label_row, r.wrap()].spacing(6))
        .padding(Padding { top: 12.0, right: 24.0, bottom: 12.0, left: 24.0 })
        .width(Length::Fill)
        .style(|_| container::Style {
            background: Some(Background::Color(theme::HIGHLIGHT_LOW)),
            border: Border {
                color: theme::HIGHLIGHT_MED,
                width: 1.0,
                radius: 0.0.into(),
            },
            ..Default::default()
        })
        .into()
}

pub fn separator() -> Element<'static, Message> {
    container(Space::new(Length::Fill, Length::Fixed(1.0)))
        .style(|_| container::Style {
            background: Some(Background::Color(theme::HIGHLIGHT_MED)),
            ..Default::default()
        })
        .into()
}

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
