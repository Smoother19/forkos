use crate::theme;
use iced::Color;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Universal,
    Commands,
    Shell,
    Web,
    Calculator,
    Contacts,
    Tags,
    FileContent,
}

impl Mode {
    pub fn detect(query: &str) -> (Mode, &str) {
        match query.chars().next() {
            Some('>') => (Mode::Commands, query[1..].trim_start()),
            Some('$') => (Mode::Shell, query[1..].trim_start()),
            Some('?') => (Mode::Web, query[1..].trim_start()),
            Some(':') => (Mode::Calculator, query[1..].trim_start()),
            Some('@') => (Mode::Contacts, query[1..].trim_start()),
            Some('#') => (Mode::Tags, query[1..].trim_start()),
            Some('/') => (Mode::FileContent, query[1..].trim_start()),
            _ => (Mode::Universal, query),
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Mode::Universal => "recherche",
            Mode::Commands => "commandes",
            Mode::Shell => "shell",
            Mode::Web => "web",
            Mode::Calculator => "calcul",
            Mode::Contacts => "contacts",
            Mode::Tags => "tags",
            Mode::FileContent => "contenu",
        }
    }

    pub fn placeholder(&self) -> &'static str {
        match self {
            Mode::Universal => "apps, commandes, fichiers...",
            Mode::Commands => "commande système...",
            Mode::Shell => "commande shell à exécuter...",
            Mode::Web => "recherche web...",
            Mode::Calculator => "2 + 2, 1km in miles, 100°C in °F...",
            Mode::Contacts => "nom, email...",
            Mode::Tags => "tag de note...",
            Mode::FileContent => "motif à chercher dans les fichiers...",
        }
    }

    pub fn color(&self) -> Color {
        match self {
            Mode::Universal => theme::TEXT,
            Mode::Commands => theme::FOAM,
            Mode::Shell => theme::PINE,
            Mode::Web => theme::IRIS,
            Mode::Calculator => theme::GOLD,
            Mode::Contacts => theme::LOVE,
            Mode::Tags => theme::ROSE,
            Mode::FileContent => theme::TEXT,
        }
    }

    pub fn hint(&self) -> &'static str {
        match self {
            Mode::Universal => "tape > commandes  $ shell  ? web  : calcul",
            Mode::Commands => "> commandes  ↑↓ naviguer  ↵ exécuter",
            Mode::Shell => "↵ exécuter  ↑↓ historique  esc quitter",
            Mode::Web => "↵ ouvrir DuckDuckGo  esc quitter",
            Mode::Calculator => "résultat en temps réel  ↵ copier",
            Mode::Contacts => "↑↓ naviguer  ↵ ouvrir  esc quitter",
            Mode::Tags => "↑↓ naviguer  ↵ ouvrir note  esc quitter",
            Mode::FileContent => "↵ ouvrir fichier  esc quitter",
        }
    }
}
