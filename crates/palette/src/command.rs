use crate::theme;
use iced::Color;

#[derive(Debug, Clone)]
pub struct Command {
    pub name: String,
    pub description: String,
    pub section: Section,
    pub icon: &'static str,
    pub shortcut: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Section {
    ActiveApps,
    Commands,
    Files,
}

impl Section {
    pub fn label(&self) -> &'static str {
        match self {
            Section::ActiveApps => "APPS ACTIVES · ⌘TAB POUR CYCLER",
            Section::Commands => "COMMANDES",
            Section::Files => "FICHIERS",
        }
    }

    pub fn order(&self) -> u8 {
        match self {
            Section::ActiveApps => 0,
            Section::Commands => 1,
            Section::Files => 2,
        }
    }

    pub fn icon_color(&self) -> Color {
        match self {
            Section::ActiveApps => theme::LOVE,
            Section::Commands => theme::FOAM,
            Section::Files => theme::IRIS,
        }
    }
}

/// Source de commandes en dur pour le prototype.
/// Plus tard on remplacera ça par de vraies sources : apps installées,
/// fichiers récents, commandes shell, etc.
pub fn default_commands() -> Vec<Command> {
    vec![
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
    ]
}