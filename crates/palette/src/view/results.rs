use crate::app::{Message, Palette};
use crate::calculator;
use crate::command::{Command, Section};
use crate::grep::GrepMatch;
use crate::mode::Mode;
use crate::shell::ShellEntry;
use crate::theme;
use iced::widget::image as img;
use iced::widget::svg;
use iced::widget::{column, container, row, text, Space};
use iced::{Alignment, Background, Border, Color, ContentFit, Element, Length, Padding};

pub fn render(state: &Palette) -> Element<'_, Message> {
    let mode = state.mode();
    let eq = state.effective_query();

    match mode {
        Mode::Shell => render_shell(state),
        Mode::Calculator => render_calculator(eq),
        Mode::Web => render_web(eq),
        Mode::Contacts => render_contacts(state),
        Mode::Tags => render_tags(state),
        Mode::FileContent => render_grep(state),
        Mode::Narrative => render_narrative(eq),
        Mode::Universal | Mode::Commands => render_commands(state),
    }
}

// ── Mode commandes (Universal + Commands) ──────────────────────────────────────

fn render_commands(state: &Palette) -> Element<'_, Message> {
    let visible = state.visible_commands();

    if visible.is_empty() {
        return empty_state("Aucun résultat");
    }

    let mut col = column![].spacing(0);
    let mut current_section: Option<Section> = None;

    for (index, cmd) in visible.iter().enumerate() {
        if current_section != Some(cmd.section) {
            col = col.push(section_header(cmd.section.label()));
            current_section = Some(cmd.section);
        }
        col = col.push(command_row(
            cmd,
            index == state.selected,
            state.effective_query(),
            index,
        ));
    }

    col.into()
}

fn section_header(label: &str) -> Element<'_, Message> {
    container(text(label.to_string()).size(10).color(theme::MUTED))
        .padding(Padding { top: 14.0, right: 22.0, bottom: 6.0, left: 22.0 })
        .into()
}

fn icon_box<'a>(cmd: &'a Command) -> Element<'a, Message> {
    let inner: Element<'a, Message> = match &cmd.icon_path {
        Some(path) if path.ends_with(".svg") => svg::Svg::new(svg::Handle::from_path(path))
            .width(Length::Fixed(22.0))
            .height(Length::Fixed(22.0))
            .content_fit(ContentFit::Contain)
            .into(),
        Some(path) => img::Image::new(img::Handle::from_path(path))
            .width(Length::Fixed(22.0))
            .height(Length::Fixed(22.0))
            .content_fit(ContentFit::Contain)
            .into(),
        None => text(cmd.icon.clone()).size(14).color(cmd.section.icon_color()).into(),
    };

    container(inner)
        .width(Length::Fixed(32.0))
        .height(Length::Fixed(32.0))
        .center_x(Length::Fixed(32.0))
        .center_y(Length::Fixed(32.0))
        .style(|_| container::Style {
            background: Some(Background::Color(theme::OVERLAY)),
            border: Border { color: Color::TRANSPARENT, width: 0.0, radius: 4.0.into() },
            ..Default::default()
        })
        .into()
}

fn command_row<'a>(
    cmd: &'a Command,
    selected: bool,
    query: &str,
    index: usize,
) -> Element<'a, Message> {
    let icon_box = icon_box(cmd);

    let info = column![
        highlighted_text(&cmd.name, query, 13.0, theme::TEXT),
        text(&cmd.description).size(11).color(theme::MUTED),
    ]
    .spacing(2);

    let shortcut_color = if selected { theme::LOVE } else { theme::MUTED };
    // Pour les 9 premiers résultats, on affiche ⌘N à la place du raccourci générique
    let shortcut_label = if index < 9 {
        format!("⌘{}", index + 1)
    } else {
        cmd.shortcut.clone()
    };
    let line = row![
        icon_box,
        info,
        Space::new(Length::Fill, Length::Shrink),
        text(shortcut_label).size(10).color(shortcut_color),
    ]
    .spacing(14)
    .align_y(Alignment::Center)
    .padding(Padding { top: 10.0, right: 22.0, bottom: 10.0, left: 22.0 });

    let bg = if selected { theme::OVERLAY } else { Color::TRANSPARENT };
    container(line)
        .width(Length::Fill)
        .style(move |_| container::Style {
            background: Some(Background::Color(bg)),
            ..Default::default()
        })
        .into()
}

// ── Mode Shell ($) ──────────────────────────────────────────────────────────────

fn render_shell(state: &Palette) -> Element<'_, Message> {
    if state.shell_history.is_empty() {
        return empty_state("Tape une commande shell et appuie sur ↵");
    }

    let mut col = column![].spacing(0);
    col = col.push(section_header("HISTORIQUE SHELL"));

    // Affiche les entrées les plus récentes en bas
    for entry in &state.shell_history {
        col = col.push(shell_entry_row(entry));
    }

    col.into()
}

fn shell_entry_row(entry: &ShellEntry) -> Element<'_, Message> {
    let prompt_color = if entry.success { theme::PINE } else { theme::LOVE };
    let prompt = text(format!("$ {}", entry.command)).size(12).color(prompt_color);

    let output_lines: Vec<Element<'_, Message>> = entry
        .output
        .lines()
        .map(|l| text(l.to_string()).size(11).color(theme::MUTED).into())
        .collect();

    let mut block = column![prompt].spacing(2);
    for line in output_lines {
        block = block.push(line);
    }

    container(block)
        .width(Length::Fill)
        .padding(Padding { top: 8.0, right: 22.0, bottom: 8.0, left: 22.0 })
        .style(|_| container::Style {
            background: Some(Background::Color(Color { r: 0.0, g: 0.0, b: 0.0, a: 0.15 })),
            border: Border { color: Color::TRANSPARENT, width: 0.0, radius: 4.0.into() },
            ..Default::default()
        })
        .into()
}

// ── Mode Calculatrice (:) ───────────────────────────────────────────────────────

fn render_calculator(query: &str) -> Element<'_, Message> {
    if query.is_empty() {
        return empty_state("Ex : 2 + 2 · 1km in miles · 100°C in °F");
    }

    match calculator::evaluate(query) {
        Some(result) => {
            let result_display = container(
                column![
                    text(result).size(28).color(theme::GOLD),
                    text("↵ pour copier").size(11).color(theme::MUTED),
                ]
                .spacing(6)
                .align_x(iced::Alignment::Center),
            )
            .width(Length::Fill)
            .padding(Padding { top: 28.0, right: 22.0, bottom: 28.0, left: 22.0 })
            .center_x(Length::Fill);

            result_display.into()
        }
        None => empty_state("Expression non reconnue"),
    }
}

// ── Mode Web (?) ───────────────────────────────────────────────────────────────

fn render_web(query: &str) -> Element<'_, Message> {
    if query.is_empty() {
        return empty_state("Tape ta recherche et appuie sur ↵");
    }

    let label = format!("Rechercher « {} » sur DuckDuckGo", query);
    let row = row![
        container(text("🔍").size(14).color(theme::IRIS))
            .width(Length::Fixed(32.0))
            .height(Length::Fixed(32.0))
            .center_x(Length::Fixed(32.0))
            .center_y(Length::Fixed(32.0))
            .style(|_| container::Style {
                background: Some(Background::Color(theme::OVERLAY)),
                border: Border { color: Color::TRANSPARENT, width: 0.0, radius: 4.0.into() },
                ..Default::default()
            }),
        column![
            text(label).size(13).color(theme::TEXT),
            text("appuie sur ↵ pour ouvrir le navigateur").size(11).color(theme::MUTED),
        ]
        .spacing(2),
    ]
    .spacing(14)
    .align_y(Alignment::Center)
    .padding(Padding { top: 10.0, right: 22.0, bottom: 10.0, left: 22.0 });

    container(row)
        .width(Length::Fill)
        .style(|_| container::Style {
            background: Some(Background::Color(theme::OVERLAY)),
            ..Default::default()
        })
        .into()
}

// ── Mode Grep (/) ──────────────────────────────────────────────────────────────

fn render_grep(state: &Palette) -> Element<'_, Message> {
    if state.effective_query().is_empty() {
        return empty_state("Tape un motif pour chercher dans les fichiers");
    }

    if state.grep_loading {
        return empty_state("Recherche en cours…");
    }

    if state.grep_results.is_empty() {
        return empty_state("Aucune correspondance trouvée");
    }

    let mut col = column![].spacing(0);
    col = col.push(section_header("RÉSULTATS DANS LES FICHIERS"));

    for (i, m) in state.grep_results.iter().enumerate() {
        col = col.push(grep_match_row(m, i == state.selected));
    }

    col.into()
}

fn grep_match_row(m: &GrepMatch, selected: bool) -> Element<'static, Message> {
    let file_line = format!("{}:{}", m.file, m.line);
    let content_text = m.content.clone();
    let content = column![
        text(file_line).size(11).color(theme::IRIS),
        text(content_text).size(12).color(theme::TEXT),
    ]
    .spacing(2);

    let line = row![
        container(text("◈").size(14).color(theme::FOAM))
            .width(Length::Fixed(32.0))
            .height(Length::Fixed(32.0))
            .center_x(Length::Fixed(32.0))
            .center_y(Length::Fixed(32.0))
            .style(|_| container::Style {
                background: Some(Background::Color(theme::OVERLAY)),
                border: Border { color: Color::TRANSPARENT, width: 0.0, radius: 4.0.into() },
                ..Default::default()
            }),
        content,
    ]
    .spacing(14)
    .align_y(Alignment::Center)
    .padding(Padding { top: 10.0, right: 22.0, bottom: 10.0, left: 22.0 });

    let bg = if selected { theme::OVERLAY } else { Color::TRANSPARENT };
    container(line)
        .width(Length::Fill)
        .style(move |_| container::Style {
            background: Some(Background::Color(bg)),
            ..Default::default()
        })
        .into()
}

// ── Mode Contacts (@) ──────────────────────────────────────────────────────────

fn render_contacts(state: &Palette) -> Element<'_, Message> {
    if state.is_loading {
        return empty_state("Chargement des contacts…");
    }
    let visible = state.visible_commands();
    if visible.is_empty() {
        return empty_state(
            "Aucun contact — place des fichiers .vcf dans ~/contacts/ ou ~/.local/share/gnome-contacts/",
        );
    }
    render_commands(state)
}

// ── Mode Tags (#) ──────────────────────────────────────────────────────────────

fn render_tags(state: &Palette) -> Element<'_, Message> {
    if state.is_loading {
        return empty_state("Chargement des notes…");
    }
    let visible = state.visible_commands();
    if visible.is_empty() {
        return empty_state(
            "Aucune note — place tes fichiers .md/.txt dans ~/notes/ ou ~/Documents/notes/",
        );
    }
    render_commands(state)
}

// ── Utilitaires ────────────────────────────────────────────────────────────────

fn empty_state(msg: impl Into<String>) -> Element<'static, Message> {
    container(text(msg.into()).size(13).color(theme::MUTED))
        .width(Length::Fill)
        .padding(Padding { top: 28.0, right: 22.0, bottom: 28.0, left: 22.0 })
        .center_x(Length::Fill)
        .into()
}

fn highlighted_text<'a>(
    name: &'a str,
    query: &str,
    size: f32,
    color: Color,
) -> Element<'a, Message> {
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

// ── Mode Narrative ─────────────────────────────────────────────────────────────

fn render_narrative(eq: &str) -> Element<'_, Message> {
    use iced::widget::{column, container, row, text};
    use iced::{Background, Length, Padding};

    if eq.trim().is_empty() {
        return container(
            text("tape une note ou une tâche…")
                .size(13)
                .color(theme::MUTED),
        )
        .padding(Padding {
            top: 20.0,
            right: 24.0,
            bottom: 20.0,
            left: 24.0,
        })
        .into();
    }

    // Détecte le type d'entrée à créer
    let (icon, kind_label, preview_text) = if let Some(rest) = eq.trim().strip_prefix("[ ]") {
        ("◇", "tâche", rest.trim())
    } else if let Some(rest) = eq.trim().strip_prefix("[x]").or_else(|| eq.trim().strip_prefix("[X]")) {
        ("◆", "tâche (faite)", rest.trim())
    } else {
        ("✎", "note", eq.trim())
    };

    let preview = row![
        text(icon).size(16).color(theme::FOAM),
        column![
            text(kind_label).size(10).color(theme::MUTED),
            text(preview_text).size(13).color(theme::TEXT),
        ]
        .spacing(2),
    ]
    .spacing(12)
    .align_y(iced::Alignment::Center)
    .padding(Padding {
        top: 14.0,
        right: 24.0,
        bottom: 14.0,
        left: 24.0,
    });

    container(preview)
        .width(Length::Fill)
        .style(|_| container::Style {
            background: Some(Background::Color(theme::SURFACE)),
            ..Default::default()
        })
        .into()
}
