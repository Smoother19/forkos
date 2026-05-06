use crate::command::{Command, Section};
use serde::Deserialize;

#[derive(Deserialize)]
struct NiriWindow {
    id: u64,
    title: String,
    app_id: String,
    #[serde(default)]
    is_focused: bool,
}

pub fn load() -> Vec<Command> {
    let output = std::process::Command::new("niri").args(["msg", "-j", "windows"]).output();

    let bytes = match output {
        Ok(o) if o.status.success() && !o.stdout.is_empty() => o.stdout,
        _ => return vec![],
    };

    let windows: Vec<NiriWindow> = match serde_json::from_slice(&bytes) {
        Ok(w) => w,
        Err(_) => return vec![],
    };

    windows
        .into_iter()
        .map(|w| Command {
            name: w.title,
            description: if w.is_focused {
                format!("{} · actif", w.app_id)
            } else {
                w.app_id.clone()
            },
            section: Section::ActiveApps,
            icon: "◉".into(),
            shortcut: if w.is_focused { "actif".into() } else { "↵".into() },
            exec: Some(format!("niri msg action focus-window --id {}", w.id)),
        })
        .collect()
}
