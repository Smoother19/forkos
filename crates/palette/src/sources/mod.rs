pub mod contacts;
pub mod desktop;
pub mod icons;
pub mod path_cmds;
pub mod recent;
pub mod tags;

use crate::command::{Command, Section};

#[derive(Debug, Clone)]
pub struct LoadedSources {
    pub commands: Vec<Command>,
    pub path_commands: Vec<String>,
}

/// Point d'entrée unique : charge toutes les sources en parallèle via spawn_blocking.
/// Appelé depuis app.rs au démarrage via Task::perform.
pub async fn load_all() -> LoadedSources {
    tokio::task::spawn_blocking(load_blocking).await.unwrap_or_else(|_| LoadedSources {
        commands: vec![],
        path_commands: vec![],
    })
}

fn load_blocking() -> LoadedSources {
    let mut commands = Vec::new();

    // ── Apps depuis les fichiers .desktop ──────────────────────────────────
    for app in desktop::scan() {
        let icon_path = icons::resolve(&app.icon_name)
            .map(|p| p.to_string_lossy().into_owned());
        let icon = if app.is_flatpak { "⬡" } else { "⬢" };
        commands.push(Command {
            name: app.name,
            description: app.description,
            section: Section::Apps,
            icon: icon.to_string(),
            icon_path,
            shortcut: "↵".to_string(),
            exec: Some(app.exec),
        });
    }

    // ── Fichiers récents ───────────────────────────────────────────────────
    for file in recent::load() {
        commands.push(Command {
            name: file.name.clone(),
            description: format!("modifié {} · {}", file.modified, shorten_path(&file.path)),
            section: Section::Files,
            icon: icon_for_file(&file.name),
            icon_path: None,
            shortcut: "↵".to_string(),
            exec: Some(format!("xdg-open \"{}\"", file.path)),
        });
    }

    // ── Contacts (@) ──────────────────────────────────────────────────────
    for contact in contacts::load() {
        let description = build_contact_description(&contact);
        let exec = contact.email.as_ref().map(|e| format!("mailto:{}", e));
        commands.push(Command {
            name: contact.name,
            description,
            section: Section::Contacts,
            icon: "◎".to_string(),
            icon_path: None,
            shortcut: "↵".to_string(),
            exec,
        });
    }

    // ── Notes / Tags (#) ──────────────────────────────────────────────────
    for note in tags::load_notes() {
        let tag_display = if note.tags.is_empty() {
            note.preview.clone()
        } else {
            let tag_str = note.tags.iter().map(|t| format!("#{}", t)).collect::<Vec<_>>().join(" ");
            if note.preview.is_empty() {
                tag_str
            } else {
                format!("{} — {}", tag_str, note.preview)
            }
        };
        commands.push(Command {
            name: note.title,
            description: tag_display,
            section: Section::Tags,
            icon: "▤".to_string(),
            icon_path: None,
            shortcut: "↵".to_string(),
            exec: Some(format!("xdg-open \"{}\"", note.path)),
        });
    }

    // ── Commandes PATH pour l'autocomplétion shell ─────────────────────────
    let path_commands = path_cmds::scan();

    LoadedSources { commands, path_commands }
}

fn build_contact_description(contact: &contacts::Contact) -> String {
    let mut parts = Vec::new();
    if let Some(org) = &contact.org {
        parts.push(org.clone());
    }
    if let Some(email) = &contact.email {
        parts.push(email.clone());
    }
    if let Some(phone) = &contact.phone {
        parts.push(phone.clone());
    }
    parts.join(" · ")
}

fn icon_for_file(name: &str) -> String {
    let ext = name.rsplit('.').next().unwrap_or("").to_lowercase();
    match ext.as_str() {
        "md" | "txt" | "rst" => "▤",
        "pdf" => "▥",
        "rs" | "py" | "js" | "ts" | "go" | "c" | "cpp" | "h" => "◈",
        "png" | "jpg" | "jpeg" | "gif" | "svg" | "webp" => "⬚",
        "mp3" | "flac" | "ogg" | "wav" | "opus" => "♫",
        "mp4" | "mkv" | "avi" | "mov" => "▶",
        "zip" | "tar" | "gz" | "xz" | "zst" => "◫",
        _ => "▤",
    }
    .to_string()
}

fn shorten_path(path: &str) -> String {
    let home = std::env::var("HOME").unwrap_or_default();
    if !home.is_empty() && path.starts_with(&home) {
        return format!("~{}", &path[home.len()..]);
    }
    path.to_string()
}
