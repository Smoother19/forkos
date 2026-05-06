//! Rendu d'une entrée du fil narratif — un bloc différencié par `Kind`.

use crate::app::Message;
use crate::models::{Entry, Kind, Payload};
use crate::theme;
use crate::view::block_card;
use iced::widget::{button, column, container, row, text, Space};
use iced::{Alignment, Background, Border, Color, Element, Length, Padding};

pub fn render(entry: &Entry) -> Element<'_, Message> {
    let inner = match &entry.payload {
        Payload::Note { text: t } => render_note(entry, t),
        Payload::Task { text: t, done } => render_task(entry, t, *done),
        Payload::App { name, exec } => render_app(entry, name, exec),
        Payload::Web { url, title } => render_web(entry, url, title.as_deref()),
        Payload::Music { track, artist } => render_music(entry, track, artist.as_deref()),
        Payload::Git { repo, message, sha } => render_git(entry, repo, message, sha.as_deref()),
        Payload::Search { query, engine } => render_search(entry, query, engine),
        Payload::System { message } => render_system(entry, message),
    };

    container(inner)
        .padding(Padding { top: 4.0, right: 0.0, bottom: 4.0, left: 0.0 })
        .into()
}

// ── Note ───────────────────────────────────────────────────────────────────────
fn render_note(entry: &Entry, body: &str) -> Element<'static, Message> {
    let kind_marker = small_marker(entry);
    let body = text(body.to_string()).size(14).color(theme::TEXT);
    let r = row![kind_marker, body]
        .spacing(12)
        .align_y(Alignment::Start);
    container(r)
        .padding(Padding { top: 8.0, right: 6.0, bottom: 8.0, left: 6.0 })
        .into()
}

// ── Task ───────────────────────────────────────────────────────────────────────
fn render_task(entry: &Entry, body: &str, done: bool) -> Element<'static, Message> {
    let id = entry.id;
    let checkbox = if done { "▣" } else { "▢" };
    let body_color = if done { theme::MUTED } else { theme::TEXT };
    let body_text = text(body.to_string()).size(14).color(body_color);

    let toggle = button(text(checkbox).size(16).color(theme::FOAM))
        .on_press(Message::ToggleTask(id))
        .padding(Padding { top: 0.0, right: 4.0, bottom: 0.0, left: 0.0 })
        .style(|_, _| button::Style {
            background: None,
            text_color: theme::FOAM,
            border: Border::default(),
            shadow: Default::default(),
        });

    let r = row![toggle, body_text]
        .spacing(10)
        .align_y(Alignment::Center);
    container(r)
        .padding(Padding { top: 6.0, right: 6.0, bottom: 6.0, left: 6.0 })
        .into()
}

// ── App ────────────────────────────────────────────────────────────────────────
fn render_app(entry: &Entry, name: &str, exec: &str) -> Element<'static, Message> {
    let exec_string = exec.to_string();
    let inner = column![
        text(name.to_string()).size(14).color(theme::PINE),
        text(exec.to_string()).size(11).color(theme::MUTED),
    ]
    .spacing(2);

    let r = row![small_marker(entry), inner]
        .spacing(12)
        .align_y(Alignment::Start);

    let card = block_card(r.into());

    button(card)
        .on_press(Message::LaunchApp(exec_string))
        .padding(0)
        .style(|_, _| button::Style {
            background: None,
            text_color: theme::TEXT,
            border: Border::default(),
            shadow: Default::default(),
        })
        .into()
}

// ── Web ────────────────────────────────────────────────────────────────────────
fn render_web(entry: &Entry, url: &str, title: Option<&str>) -> Element<'static, Message> {
    let url_string = url.to_string();
    let primary = title.unwrap_or(url);
    let inner = column![
        text(primary.to_string()).size(14).color(theme::IRIS),
        text(url.to_string()).size(11).color(theme::MUTED),
    ]
    .spacing(2);

    let r = row![small_marker(entry), inner]
        .spacing(12)
        .align_y(Alignment::Start);

    let card = block_card(r.into());

    button(card)
        .on_press(Message::OpenUrl(url_string))
        .padding(0)
        .style(|_, _| button::Style {
            background: None,
            text_color: theme::TEXT,
            border: Border::default(),
            shadow: Default::default(),
        })
        .into()
}

// ── Music ──────────────────────────────────────────────────────────────────────
fn render_music(entry: &Entry, track: &str, artist: Option<&str>) -> Element<'static, Message> {
    let artist = artist.unwrap_or("(artiste inconnu)");
    let inner = column![
        text(track.to_string()).size(14).color(theme::ROSE),
        text(artist.to_string()).size(11).color(theme::MUTED),
    ]
    .spacing(2);
    let r = row![small_marker(entry), inner]
        .spacing(12)
        .align_y(Alignment::Start);
    block_card(r.into())
}

// ── Git ────────────────────────────────────────────────────────────────────────
fn render_git(entry: &Entry, repo: &str, msg: &str, sha: Option<&str>) -> Element<'static, Message> {
    let sub = match sha {
        Some(s) => format!("{} · {}", repo, &s[..s.len().min(7)]),
        None => repo.to_string(),
    };
    let inner = column![
        text(msg.to_string()).size(14).color(theme::GOLD),
        text(sub).size(11).color(theme::MUTED),
    ]
    .spacing(2);
    let r = row![small_marker(entry), inner]
        .spacing(12)
        .align_y(Alignment::Start);
    block_card(r.into())
}

// ── Search ─────────────────────────────────────────────────────────────────────
fn render_search(entry: &Entry, query: &str, engine: &str) -> Element<'static, Message> {
    let inner = column![
        text(format!("« {} »", query)).size(13).color(theme::TEXT),
        text(format!("via {}", engine)).size(11).color(theme::MUTED),
    ]
    .spacing(2);
    let r = row![small_marker(entry), inner]
        .spacing(12)
        .align_y(Alignment::Start);
    container(r)
        .padding(Padding { top: 6.0, right: 6.0, bottom: 6.0, left: 6.0 })
        .into()
}

// ── System ─────────────────────────────────────────────────────────────────────
fn render_system(entry: &Entry, message: &str) -> Element<'static, Message> {
    let r = row![
        small_marker(entry),
        text(message.to_string()).size(12).color(theme::SUBTLE),
    ]
    .spacing(12)
    .align_y(Alignment::Center);
    container(r)
        .padding(Padding { top: 4.0, right: 6.0, bottom: 4.0, left: 6.0 })
        .into()
}

// ── Helpers ────────────────────────────────────────────────────────────────────

/// Petit marker à gauche d'un bloc : icône + heure.
fn small_marker(entry: &Entry) -> Element<'static, Message> {
    let time = entry.timestamp.format("%H:%M").to_string();
    let icon = entry.kind.icon().to_string();
    let color = theme::kind_color(entry.kind);

    column![
        text(icon).size(14).color(color),
        text(time).size(9).color(theme::MUTED),
    ]
    .spacing(2)
    .width(Length::Fixed(36.0))
    .align_x(Alignment::Center)
    .into()
}

#[allow(dead_code)]
fn pill(label: &str, color: Color) -> Element<'static, Message> {
    container(text(label.to_string()).size(9).color(color))
        .padding(Padding { top: 1.0, right: 6.0, bottom: 1.0, left: 6.0 })
        .style(move |_| container::Style {
            background: Some(Background::Color(with_alpha(color, 0.1))),
            border: Border {
                color: with_alpha(color, 0.3),
                width: 1.0,
                radius: 3.0.into(),
            },
            ..Default::default()
        })
        .into()
}

#[allow(dead_code)]
fn with_alpha(c: Color, a: f32) -> Color {
    Color { r: c.r, g: c.g, b: c.b, a }
}

#[allow(dead_code)]
fn spacer(width: f32) -> Element<'static, Message> {
    Space::new(Length::Fixed(width), Length::Shrink).into()
}

/// Renvoie le label visible court d'un Kind (pour l'instant non utilisé en
/// rendu mais peut servir à un futur filtre).
#[allow(dead_code)]
pub fn kind_pill(kind: Kind) -> Element<'static, Message> {
    pill(kind.label(), theme::kind_color(kind))
}
