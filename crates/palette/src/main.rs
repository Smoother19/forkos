use iced::widget::{column, container, row, scrollable, text, text_input, Space};
use iced::{Background, Border, Color, Element, Length, Padding, Task, Theme};
use nucleo::pattern::{CaseMatching, Normalization, Pattern};
use nucleo::{Config, Matcher, Utf32Str};

// === PALETTE ROSE PINE DAWN ===
mod palette_colors {
    use iced::Color;

    pub const BASE: Color = Color::from_rgb(0xfa as f32 / 255.0, 0xf4 as f32 / 255.0, 0xed as f32 / 255.0);
    pub const SURFACE: Color = Color::from_rgb(0xff as f32 / 255.0, 0xfa as f32 / 255.0, 0xf3 as f32 / 255.0);
    pub const OVERLAY: Color = Color::from_rgb(0xf2 as f32 / 255.0, 0xe9 as f32 / 255.0, 0xe1 as f32 / 255.0);
    pub const MUTED: Color = Color::from_rgb(0x9d as f32 / 255.0, 0x99 as f32 / 255.0, 0xa3 as f32 / 255.0);
    pub const SUBTLE: Color = Color::from_rgb(0x79 as f32 / 255.0, 0x76 as f32 / 255.0, 0x7e as f32 / 255.0);
    pub const TEXT: Color = Color::from_rgb(0x57 as f32 / 255.0, 0x52 as f32 / 255.0, 0x79 as f32 / 255.0);
    pub const LOVE: Color = Color::from_rgb(0xb4 as f32 / 255.0, 0x63 as f32 / 255.0, 0x7a as f32 / 255.0);
    pub const GOLD: Color = Color::from_rgb(0xea as f32 / 255.0, 0x9d as f32 / 255.0, 0x34 as f32 / 255.0);
    pub const ROSE: Color = Color::from_rgb(0xd7 as f32 / 255.0, 0x82 as f32 / 255.0, 0x7e as f32 / 255.0);
    pub const PINE: Color = Color::from_rgb(0x28 as f32 / 255.0, 0x65 as f32 / 255.0, 0x7b as f32 / 255.0);
    pub const FOAM: Color = Color::from_rgb(0x56 as f32 / 255.0, 0x94 as f32 / 255.0, 0x9f as f32 / 255.0);
    pub const IRIS: Color = Color::from_rgb(0x90 as f32 / 255.0, 0x7a as f32 / 255.0, 0xa9 as f32 / 255.0);
    pub const HIGHLIGHT_LOW: Color = Color::from_rgb(0xf4 as f32 / 255.0, 0xed as f32 / 255.0, 0xe8 as f32 / 255.0);
    pub const HIGHLIGHT_MED: Color = Color::from_rgb(0xdf as f32 / 255.0, 0xda as f32 / 255.0, 0xd9 as f32 / 255.0);
}

use palette_colors::*;

fn main() -> iced::Result {
    tracing_subscriber::fmt::init();

    iced::application("Palette", Palette::update, Palette::view)
        .theme(|_| Theme::Light)
        .run()
}

struct Palette {
    query: String,
    commands: Vec<Command>,
    selected: usize,
}

#[derive(Debug, Clone)]
struct Command {
    name: String,
    description: String,
    section: Section,
    icon: &'static str,
    shortcut: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Section {
    ActiveApps,
    Commands,
    Files,
}

impl Section {
    fn label(&self) -> &'static str {
        match self {
            Section::ActiveApps => "APPS ACTIVES · ⌘TAB POUR CYCLER",
            Section::Commands => "COMMANDES",
            Section::Files => "FICHIERS",
        }
    }

    fn order(&self) -> u8 {
        match self {
            Section::ActiveApps => 0,
            Section::Commands => 1,
            Section::Files => 2,
        }
    }

    fn icon_color(&self) -> Color {
        match self {
            Section::ActiveApps => LOVE,
            Section::Commands => FOAM,
            Section::Files => IRIS,
        }
    }
}

impl Default for Palette {
    fn default() -> Self {
        Self {
            query: String::from("ma"),
            selected: 0,
            commands: vec![
                Command {
                    name: "mail".into(),
                    description: "3 nouveaux · marc.l, github, newsletter".into(),
                    section: Section::ActiveApps,
                    icon: "♪",
                    shortcut: "↵ aller",
                },
                Command {
                    name: "mana · musique".into(),
                    description: "en lecture · Discovery — Daft Punk".into(),
                    section: Section::ActiveApps,
                    icon: "♫",
                    shortcut: "↵",
                },
                Command {
                    name: "nouveau mail".into(),
                    description: "composer · ouvrir mail en mode rédaction".into(),
                    section: Section::Commands,
                    icon: "✎",
                    shortcut: "⇧↵",
                },
                Command {
                    name: "installer mattermost".into(),
                    description: "flathub · 124 MB · sandbox".into(),
                    section: Section::Commands,
                    icon: "⊕",
                    shortcut: "⌃↵",
                },
                Command {
                    name: "~/notes/mardi-meeting.md".into(),
                    description: "modifié hier · 12 lignes".into(),
                    section: Section::Files,
                    icon: "▤",
                    shortcut: "↵",
                },
            ],
        }
    }
}

#[derive(Debug, Clone)]
enum Message {
    QueryChanged(String),
}

impl Palette {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::QueryChanged(value) => {
                self.query = value;
                self.selected = 0;
                Task::none()
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let header = row![
            text("⌘").size(15).color(IRIS),
            text("›").size(15).color(LOVE),
            text_input("tape une commande...", &self.query)
                .on_input(Message::QueryChanged)
                .padding(0)
                .size(15)
                .style(|_theme, _status| iced::widget::text_input::Style {
                    background: Background::Color(Color::TRANSPARENT),
                    border: Border {
                        color: Color::TRANSPARENT,
                        width: 0.0,
                        radius: 0.0.into(),
                    },
                    icon: TEXT,
                    placeholder: MUTED,
                    value: TEXT,
                    selection: HIGHLIGHT_MED,
                }),
            esc_badge(),
        ]
        .spacing(14)
        .align_y(iced::Alignment::Center)
        .padding(Padding::from([18, 22]));

        let separator = container(Space::new(Length::Fill, Length::Fixed(1.0)))
            .style(|_| container::Style {
                background: Some(Background::Color(HIGHLIGHT_MED)),
                ..Default::default()
            });

        let results = compute_results(&self.commands, &self.query, self.selected);

        let footer = row![
            row![
                text("↑↓").size(11).color(LOVE),
                text("naviguer").size(11).color(MUTED),
            ].spacing(6),
            row![
                text("↵").size(11).color(LOVE),
                text("exécuter").size(11).color(MUTED),
            ].spacing(6),
            row![
                text("tab").size(11).color(LOVE),
                text("mode").size(11).color(MUTED),
            ].spacing(6),
            Space::new(Length::Fill, Length::Shrink),
            text(format!("{} résultats", self.commands.iter().filter(|c| matches_query(&c.name, &self.query)).count()))
                .size(11)
                .color(MUTED),
        ]
        .spacing(18)
        .padding(Padding::from([10, 22]));

        let footer_separator = container(Space::new(Length::Fill, Length::Fixed(1.0)))
            .style(|_| container::Style {
                background: Some(Background::Color(HIGHLIGHT_MED)),
                ..Default::default()
            });

        let body = column![
            header,
            separator,
            scrollable(results).height(Length::Fill),
            footer_separator,
            footer,
        ];

        let palette_box = container(body)
            .max_width(620)
            .style(|_| container::Style {
                background: Some(Background::Color(SURFACE)),
                border: Border {
                    color: HIGHLIGHT_MED,
                    width: 1.0,
                    radius: 12.0.into(),
                },
                ..Default::default()
            });

        container(palette_box)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(40)
            .center_x(Length::Fill)
            .style(|_| container::Style {
                background: Some(Background::Color(BASE)),
                ..Default::default()
            })
            .into()
    }
}

fn matches_query(name: &str, query: &str) -> bool {
    if query.is_empty() {
        return true;
    }
    let mut matcher = Matcher::new(Config::DEFAULT);
    let pattern = Pattern::parse(query, CaseMatching::Ignore, Normalization::Smart);
    let mut buf = Vec::new();
    let haystack = Utf32Str::new(name, &mut buf);
    pattern.score(haystack, &mut matcher).is_some()
}

fn esc_badge() -> Element<'static, Message> {
    container(text("esc").size(11).color(MUTED))
        .padding(Padding::from([3, 8]))
        .style(|_| container::Style {
            background: Some(Background::Color(OVERLAY)),
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 4.0.into(),
            },
            ..Default::default()
        })
        .into()
}

fn compute_results<'a>(
    commands: &'a [Command],
    query: &str,
    selected: usize,
) -> Element<'a, Message> {
    // Filtrage + scoring
    let mut filtered: Vec<&Command> = if query.is_empty() {
        commands.iter().collect()
    } else {
        let mut matcher = Matcher::new(Config::DEFAULT);
        let pattern = Pattern::parse(query, CaseMatching::Ignore, Normalization::Smart);
        let mut buf = Vec::new();
        let mut scored: Vec<(u32, &Command)> = commands
            .iter()
            .filter_map(|cmd| {
                let haystack = Utf32Str::new(&cmd.name, &mut buf);
                pattern.score(haystack, &mut matcher).map(|s| (s, cmd))
            })
            .collect();
        scored.sort_by_key(|item| std::cmp::Reverse(item.0));
        scored.into_iter().map(|(_, c)| c).collect()
    };

    // On regroupe par section, en gardant l'ordre des sections
    filtered.sort_by_key(|c| c.section.order());

    let mut col = column![].spacing(0);
    let mut current_section: Option<Section> = None;
    let mut global_index = 0usize;

    for cmd in filtered {
        if current_section != Some(cmd.section) {
            col = col.push(section_header(cmd.section.label()));
            current_section = Some(cmd.section);
        }
        let is_selected = global_index == selected;
        col = col.push(command_row(cmd, is_selected, query));
        global_index += 1;
    }

    col.into()
}

fn section_header(label: &str) -> Element<'_, Message> {
    container(
        text(label)
            .size(10)
            .color(MUTED),
    )
    .padding(Padding {
        top: 14.0,
        right: 22.0,
        bottom: 6.0,
        left: 22.0,
    })
    .into()
}

fn command_row<'a>(cmd: &'a Command, selected: bool, query: &str) -> Element<'a, Message> {
    let icon_box = container(
        text(cmd.icon)
            .size(14)
            .color(cmd.section.icon_color()),
    )
    .width(Length::Fixed(32.0))
    .height(Length::Fixed(32.0))
    .center_x(Length::Fixed(32.0))
    .center_y(Length::Fixed(32.0))
    .style(|_| container::Style {
        background: Some(Background::Color(OVERLAY)),
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: 4.0.into(),
        },
        ..Default::default()
    });

    let name_line = highlighted_text(&cmd.name, query, 13.0, TEXT);

    let info = column![
        name_line,
        text(&cmd.description).size(11).color(MUTED),
    ]
    .spacing(2);

    let shortcut_color = if selected { LOVE } else { MUTED };
    let shortcut = text(cmd.shortcut).size(10).color(shortcut_color);

    let line = row![
        icon_box,
        info,
        Space::new(Length::Fill, Length::Shrink),
        shortcut,
    ]
    .spacing(14)
    .align_y(iced::Alignment::Center)
    .padding(Padding::from([10, 22]));

    let bg = if selected { OVERLAY } else { Color::TRANSPARENT };
    let border_left = if selected { LOVE } else { Color::TRANSPARENT };

    container(line)
        .width(Length::Fill)
        .style(move |_| container::Style {
            background: Some(Background::Color(bg)),
            border: Border {
                color: border_left,
                width: 0.0,
                radius: 0.0.into(),
            },
            ..Default::default()
        })
        .into()
}

/// Surligne les caractères du `query` trouvés dans `name`.
fn highlighted_text<'a>(name: &'a str, query: &str, size: f32, color: Color) -> Element<'a, Message> {
    if query.is_empty() {
        return text(name).size(size).color(color).into();
    }

    // Trouver la sous-chaîne contiguë (insensible à la casse) du query dans name
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
        let highlight = container(text(middle.to_string()).size(size).color(color))
            .style(|_| container::Style {
                background: Some(Background::Color(HIGHLIGHT_MED)),
                ..Default::default()
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