use crate::theme;
use iced::Color;

#[derive(Debug, Clone)]
pub struct Command {
    pub name: String,
    pub description: String,
    pub section: Section,
    pub icon: String,
    pub shortcut: String,
    /// Commande à exécuter quand l'entrée est sélectionnée.
    /// None = commande de démo sans action réelle.
    pub exec: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum Section {
    Apps,       // Apps installées (depuis .desktop)
    ActiveApps, // Apps actives (branché plus tard via compositor)
    Commands,
    Files,
    System,
    Settings,
}

impl Section {
    pub fn label(&self) -> &'static str {
        match self {
            Section::Apps => "APPLICATIONS",
            Section::ActiveApps => "APPS ACTIVES · ⌘TAB POUR CYCLER",
            Section::Commands => "COMMANDES",
            Section::Files => "FICHIERS RÉCENTS",
            Section::System => "SYSTÈME",
            Section::Settings => "PARAMÈTRES",
        }
    }

    pub fn order(&self) -> u8 {
        match self {
            Section::ActiveApps => 0,
            Section::Commands => 1,
            Section::Files => 2,
            Section::Apps => 3,
            Section::System => 4,
            Section::Settings => 5,
        }
    }

    pub fn icon_color(&self) -> Color {
        match self {
            Section::Apps => theme::FOAM,
            Section::ActiveApps => theme::LOVE,
            Section::Commands => theme::FOAM,
            Section::Files => theme::IRIS,
            Section::System => theme::PINE,
            Section::Settings => theme::GOLD,
        }
    }
}

/// Commandes disponibles en mode `>` (commandes système pures)
pub fn system_commands() -> Vec<Command> {
    vec![
        Command {
            name: "gestionnaire de fichiers".into(),
            description: "ouvrir Nautilus".into(),
            section: Section::System,
            icon: "📁".into(),
            shortcut: "↵".into(),
            exec: Some("nautilus".into()),
        },
        Command {
            name: "terminal".into(),
            description: "ouvrir un terminal".into(),
            section: Section::System,
            icon: "⬛".into(),
            shortcut: "↵".into(),
            exec: Some("xterm".into()),
        },
        Command {
            name: "éditeur de texte".into(),
            description: "ouvrir Helix / Gedit".into(),
            section: Section::System,
            icon: "✏".into(),
            shortcut: "↵".into(),
            exec: Some("helix".into()),
        },
        Command {
            name: "navigateur".into(),
            description: "ouvrir Firefox".into(),
            section: Section::System,
            icon: "🌐".into(),
            shortcut: "↵".into(),
            exec: Some("firefox".into()),
        },
        Command {
            name: "paramètres système".into(),
            description: "ouvrir GNOME Settings".into(),
            section: Section::Settings,
            icon: "⚙".into(),
            shortcut: "↵".into(),
            exec: Some("gnome-control-center".into()),
        },
        Command {
            name: "bluetooth".into(),
            description: "gérer les appareils bluetooth".into(),
            section: Section::Settings,
            icon: "⟡".into(),
            shortcut: "↵".into(),
            exec: Some("gnome-control-center bluetooth".into()),
        },
        Command {
            name: "réseau & wifi".into(),
            description: "gérer les connexions réseau".into(),
            section: Section::Settings,
            icon: "⋯".into(),
            shortcut: "↵".into(),
            exec: Some("gnome-control-center network".into()),
        },
        Command {
            name: "verrouiller l'écran".into(),
            description: "verrouille la session immédiatement".into(),
            section: Section::System,
            icon: "🔒".into(),
            shortcut: "↵".into(),
            exec: Some("loginctl lock-session".into()),
        },
        Command {
            name: "éteindre".into(),
            description: "arrêter le système".into(),
            section: Section::System,
            icon: "⏻".into(),
            shortcut: "↵".into(),
            exec: Some("systemctl poweroff".into()),
        },
        Command {
            name: "redémarrer".into(),
            description: "redémarrer le système".into(),
            section: Section::System,
            icon: "↺".into(),
            shortcut: "↵".into(),
            exec: Some("systemctl reboot".into()),
        },
    ]
}
