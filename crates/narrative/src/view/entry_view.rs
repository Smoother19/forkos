use crate::app::Message;
use crate::entry::{Entry, EntryKind};
use forkos_shared::theme;
use iced::widget::{column, container, row, text, Space};
use iced::{Background, Color, Element, Length, Padding};

use super::{context_bar, left_border_block};

pub fn render(entry: &Entry) -> Element<'_, Message> {
    match &entry.kind {
        EntryKind::System { message } => render_system(message),
        EntryKind::AppLaunched { name, detail, icon, duration } => {
            render_app_launched(name, detail, icon, *duration)
        }
        EntryKind::Media { title, artist, progress_secs, duration_secs, playing } => {
            render_media(title, artist, *progress_secs, *duration_secs, *playing)
        }
        EntryKind::Notifications { source, count, items } => {
            render_notifications(source, *count, items)
        }
        EntryKind::FileEdit { path, lines, preview, modified_ago } => {
            render_file_edit(path, *lines, preview, modified_ago)
        }
        EntryKind::Shell { command, output_preview, exit_code } => {
            render_shell(command, output_preview, *exit_code)
        }
        EntryKind::PaletteSearch { query, result_chosen } => {
            render_palette_search(query, result_chosen.as_deref())
        }
    }
}

// ── Système ───────────────────────────────────────────────────────────────────

fn render_system(message: &str) -> Element<'_, Message> {
    row![
        text("›").size(13).color(theme::FOAM),
        text(message).size(13).color(theme::MUTED),
    ]
    .spacing(8)
    .padding(Padding { top: 6.0, right: 0.0, bottom: 6.0, left: 0.0 })
    .into()
}

// ── App lancée ────────────────────────────────────────────────────────────────

fn render_app_launched<'a>(
    name: &'a str,
    detail: &'a str,
    _icon: &'a str,
    duration: Option<u64>,
) -> Element<'a, Message> {
    let duration_str = duration
        .map(|s| format!(" · utilisé {}min", s / 60))
        .unwrap_or_default();

    let header = row![
        text("›").size(13).color(theme::PINE),
        text(format!("lancé {} · {}{}", name, detail, duration_str))
            .size(13)
            .color(theme::TEXT),
    ]
    .spacing(8);

    let actions = vec![
        crate::entry::ContextAction {
            label: "relancer".to_string(),
            command: name.to_string(),
        },
        crate::entry::ContextAction {
            label: "focus".to_string(),
            command: format!("niri msg action focus-window --app-id {}", name),
        },
        crate::entry::ContextAction {
            label: "fermer".to_string(),
            command: format!("niri msg action close-window --app-id {}", name),
        },
    ];

    column![
        header,
        context_bar::render(actions),
    ]
    .spacing(2)
    .padding(Padding { top: 4.0, right: 0.0, bottom: 4.0, left: 0.0 })
    .into()
}

// ── Média ─────────────────────────────────────────────────────────────────────

fn render_media<'a>(
    title: &'a str,
    artist: &'a str,
    progress_secs: u32,
    duration_secs: u32,
    playing: bool,
) -> Element<'a, Message> {
    let status = if playing { "lecture" } else { "pause" };
    let header = row![
        text("›").size(13).color(theme::GOLD),
        text(format!("musique [{}]", status)).size(13).color(theme::TEXT),
    ]
    .spacing(8);

    let progress = if duration_secs > 0 {
        progress_secs as f32 / duration_secs as f32
    } else {
        0.0
    };
    let bar = progress_bar_str(progress, 18);
    let time_str = format!("{} / {}", fmt_secs(progress_secs), fmt_secs(duration_secs));

    let block_content = column![
        row![
            text("♫").size(13).color(theme::GOLD),
            text(format!("{} — {}", title, artist)).size(13).color(theme::TEXT),
            Space::new(Length::Fill, Length::Shrink),
            text(time_str).size(11).color(theme::MUTED),
        ]
        .spacing(8),
        text(bar).size(11).color(theme::GOLD),
    ]
    .spacing(4);

    column![
        header,
        left_border_block(block_content.into(), theme::GOLD, theme::SURFACE),
        context_bar::media_controls(),
    ]
    .spacing(2)
    .padding(Padding { top: 4.0, right: 0.0, bottom: 4.0, left: 0.0 })
    .into()
}

fn progress_bar_str(p: f32, width: usize) -> String {
    let filled = ((p * width as f32).round() as usize).min(width);
    let empty = width - filled;
    "━".repeat(filled) + &"░".repeat(empty)
}

fn fmt_secs(secs: u32) -> String {
    format!("{:02}:{:02}", secs / 60, secs % 60)
}

// ── Notifications ─────────────────────────────────────────────────────────────

fn render_notifications<'a>(
    source: &'a str,
    count: usize,
    items: &'a [crate::entry::NotifItem],
) -> Element<'a, Message> {
    let header = row![
        text("›").size(13).color(theme::FOAM),
        text(format!("{} · {} nouveaux", source, count)).size(13).color(theme::TEXT),
    ]
    .spacing(8);

    let mut items_col = column![].spacing(0);
    for (i, item) in items.iter().enumerate() {
        // Initiales pour l'avatar
        let initials: String = item.sender
            .split_whitespace()
            .filter_map(|w| w.chars().next())
            .take(2)
            .collect::<String>()
            .to_uppercase();
        let initials = if initials.is_empty() { "?".to_string() } else { initials };

        let avatar = container(text(initials).size(9).color(theme::FOAM))
            .width(Length::Fixed(24.0))
            .height(Length::Fixed(24.0))
            .center_x(Length::Fixed(24.0))
            .center_y(Length::Fixed(24.0))
            .style(|_| container::Style {
                background: Some(Background::Color(theme::HIGHLIGHT_LOW)),
                border: iced::Border { radius: 12.0.into(), color: Color::TRANSPARENT, width: 0.0 },
                ..Default::default()
            });

        let item_row = row![
            avatar,
            column![
                text(item.sender.clone()).size(12).color(theme::TEXT),
                text(item.preview.clone()).size(11).color(theme::MUTED),
            ]
            .spacing(2)
            .width(Length::Fill),
            context_bar::render(item.actions.clone()),
        ]
        .spacing(8)
        .padding(Padding { top: 8.0, right: 12.0, bottom: 8.0, left: 0.0 });

        items_col = items_col.push(item_row);

        if i < items.len() - 1 {
            items_col = items_col.push(notif_divider());
        }
    }

    let global_actions = vec![
        crate::entry::ContextAction {
            label: "✓ tout lire".to_string(),
            command: String::new(),
        },
        crate::entry::ContextAction {
            label: "📥 ouvrir".to_string(),
            command: format!("xdg-open {}", source),
        },
    ];

    column![
        header,
        left_border_block(items_col.into(), theme::FOAM, theme::SURFACE),
        context_bar::render(global_actions),
    ]
    .spacing(2)
    .padding(Padding { top: 4.0, right: 0.0, bottom: 4.0, left: 0.0 })
    .into()
}

fn notif_divider() -> Element<'static, Message> {
    container(Space::new(Length::Fill, Length::Fixed(1.0)))
        .style(|_| container::Style {
            background: Some(Background::Color(theme::HIGHLIGHT_MED)),
            ..Default::default()
        })
        .into()
}

// ── Fichier édité ─────────────────────────────────────────────────────────────

fn render_file_edit<'a>(
    path: &'a str,
    lines: usize,
    preview: &'a str,
    modified_ago: &'a str,
) -> Element<'a, Message> {
    let header = row![
        text("›").size(13).color(theme::IRIS),
        text(format!("édité · {} · {} lignes · {}", path, lines, modified_ago))
            .size(13)
            .color(theme::TEXT),
    ]
    .spacing(8);

    let preview_lines = preview.lines().take(3);
    let mut preview_col = column![].spacing(1);
    for line in preview_lines {
        preview_col = preview_col.push(text(line.to_string()).size(11).color(theme::SUBTLE));
    }

    let actions = vec![
        crate::entry::ContextAction {
            label: "ouvrir".to_string(),
            command: format!("xdg-open {}", path),
        },
        crate::entry::ContextAction {
            label: "copier chemin".to_string(),
            command: format!(
                "echo -n '{}' | xclip -selection clipboard 2>/dev/null || echo -n '{}' | xsel --clipboard 2>/dev/null",
                path, path
            ),
        },
    ];

    column![
        header,
        left_border_block(preview_col.into(), theme::IRIS, theme::SURFACE),
        context_bar::render(actions),
    ]
    .spacing(2)
    .padding(Padding { top: 4.0, right: 0.0, bottom: 4.0, left: 0.0 })
    .into()
}

// ── Shell ─────────────────────────────────────────────────────────────────────

fn render_shell<'a>(
    command: &'a str,
    output_preview: &'a str,
    exit_code: i32,
) -> Element<'a, Message> {
    let prompt_color = if exit_code == 0 { theme::PINE } else { theme::LOVE };

    let mut block = column![
        text(format!("$ {}", command)).size(12).color(prompt_color),
    ]
    .spacing(2);

    for line in output_preview.lines().take(4) {
        block = block.push(text(line.to_string()).size(11).color(theme::MUTED));
    }

    let actions = vec![
        crate::entry::ContextAction {
            label: "relancer".to_string(),
            command: command.to_string(),
        },
        crate::entry::ContextAction {
            label: "copier sortie".to_string(),
            command: format!(
                "echo -n '{}' | xclip -selection clipboard 2>/dev/null",
                output_preview.replace('\'', "")
            ),
        },
    ];

    column![
        left_border_block(
            block.into(),
            if exit_code == 0 { theme::PINE } else { theme::LOVE },
            theme::SURFACE,
        ),
        context_bar::render(actions),
    ]
    .spacing(2)
    .padding(Padding { top: 4.0, right: 0.0, bottom: 4.0, left: 0.0 })
    .into()
}

// ── Recherche palette ─────────────────────────────────────────────────────────

fn render_palette_search<'a>(
    query: &'a str,
    result_chosen: Option<&'a str>,
) -> Element<'a, Message> {
    let result_str = result_chosen
        .map(|r| format!(" → {}", r))
        .unwrap_or_else(|| " (annulé)".to_string());

    row![
        text("⌘").size(13).color(theme::IRIS),
        text("›").size(13).color(theme::LOVE),
        text(format!("{}{}", query, result_str)).size(13).color(theme::MUTED),
    ]
    .spacing(6)
    .padding(Padding { top: 6.0, right: 0.0, bottom: 6.0, left: 0.0 })
    .into()
}
