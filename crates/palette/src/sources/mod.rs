pub mod desktop;
pub mod path_cmds;
pub mod recent;

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
        let (icon, section) = if app.is_flatpak {
            ("⬡", Section::Apps)
        } else {
            ("⬢", Section::Apps)
        };
        commands.push(Command {
            name: app.name,
            description: app.description,
            section,
            icon: icon.to_string(),
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
            shortcut: "↵".to_string(),
            exec: Some(format!("xdg-open \"{}\"", file.path)),
        });
    }

    // ── Commandes PATH pour l'autocomplétion shell ─────────────────────────
    let path_commands = path_cmds::scan();

    LoadedSources { commands, path_commands }
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
