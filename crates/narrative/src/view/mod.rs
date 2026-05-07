mod context_bar;
mod entry_view;
mod header;
mod palette_inline;
pub mod bar;

use crate::app::{Message, Narrative, BOTTOM_INPUT_ID};
use forkos_shared::theme;
use iced::widget::{button, column, container, row, scrollable, text, text_input, Space};
use iced::{Background, Border, Color, Element, Length, Padding};
use std::sync::LazyLock;

pub static FEED_SCROLL_ID: LazyLock<scrollable::Id> = LazyLock::new(scrollable::Id::unique);

pub fn render(state: &Narrative) -> Element<'_, Message> {
    if state.bar_open {
        render_open(state)
    } else {
        render_closed(state)
    }
}

/// État fermé : juste la barre 48px avec son champ texte intégré
fn render_closed(state: &Narrative) -> Element<'_, Message> {
    container(bar::render(state))
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|_| container::Style {
            background: Some(Background::Color(theme::SURFACE)),
            ..Default::default()
        })
        .into()
}

/// État ouvert : header → apps actives → fil → input shell → barre 48px
fn render_open(state: &Narrative) -> Element<'_, Message> {
    let body = column![
        header::render(),
        separator(),
        active_apps_section(state),
        separator(),
        scrollable(entries_column(state))
            .height(Length::Fill)
            .id(FEED_SCROLL_ID.clone())
            .on_scroll(Message::FeedScrolled),
        separator(),
        bottom_bar(state),
        separator(),
        bar::render(state),
    ]
    .height(Length::Fill);

    container(body)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|_| container::Style {
            background: Some(Background::Color(theme::BASE)),
            ..Default::default()
        })
        .into()
}

/// Section apps actives : grille wrappée de cartes cliquables
fn active_apps_section(state: &Narrative) -> Element<'_, Message> {
    if state.active_windows.is_empty() {
        return container(
            text("aucune app active · tape > pour ouvrir une app")
                .size(11)
                .color(theme::MUTED),
        )
        .padding(Padding { top: 12.0, right: 24.0, bottom: 12.0, left: 24.0 })
        .into();
    }

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

    container(
        column![label_row, r.wrap()].spacing(6),
    )
    .padding(Padding { top: 14.0, right: 24.0, bottom: 14.0, left: 24.0 })
    .max_width(720)
    .center_x(Length::Fill)
    .into()
}

fn entries_column(state: &Narrative) -> Element<'_, Message> {
    let mut col = column![].spacing(4);

    for entry in &state.entries {
        col = col.push(entry_view::render(entry));
    }

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
