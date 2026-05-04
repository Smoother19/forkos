use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub struct DesktopApp {
    pub name: String,
    pub description: String,
    pub exec: String,
    pub is_flatpak: bool,
}

/// Scanne tous les répertoires .desktop standards et retourne les apps installées.
pub fn scan() -> Vec<DesktopApp> {
    let home = std::env::var("HOME").unwrap_or_default();
    let dirs = [
        "/usr/share/applications".to_string(),
        "/usr/local/share/applications".to_string(),
        format!("{}/.local/share/applications", home),
        "/var/lib/flatpak/exports/share/applications".to_string(),
        format!("{}/.local/share/flatpak/exports/share/applications", home),
    ];

    let mut apps = Vec::new();
    let mut seen_names = std::collections::HashSet::new();

    for dir in &dirs {
        let Ok(entries) = fs::read_dir(dir) else { continue };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("desktop") {
                continue;
            }
            if let Some(app) = parse_desktop_file(&path) {
                // Deduplique par nom (première occurrence gagne)
                if seen_names.insert(app.name.to_lowercase()) {
                    apps.push(app);
                }
            }
        }
    }

    apps.sort_by(|a, b| a.name.cmp(&b.name));
    apps
}

fn parse_desktop_file(path: &Path) -> Option<DesktopApp> {
    let content = fs::read_to_string(path).ok()?;
    let mut in_entry = false;
    let mut fields: HashMap<String, String> = HashMap::new();

    for line in content.lines() {
        let line = line.trim();
        if line == "[Desktop Entry]" {
            in_entry = true;
            continue;
        }
        if line.starts_with('[') {
            in_entry = false;
            continue;
        }
        if !in_entry || line.starts_with('#') || line.is_empty() {
            continue;
        }
        if let Some((k, v)) = line.split_once('=') {
            fields.insert(k.trim().to_string(), v.trim().to_string());
        }
    }

    // Seulement les applications non-cachées
    if fields.get("Type").map(|s| s.as_str()) != Some("Application") {
        return None;
    }
    if fields.get("NoDisplay").map(|s| s == "true").unwrap_or(false) {
        return None;
    }
    if fields.get("Hidden").map(|s| s == "true").unwrap_or(false) {
        return None;
    }

    // Préfère la localisation française si disponible
    let name = fields
        .get("Name[fr]")
        .or_else(|| fields.get("Name"))?
        .clone();
    let exec = fields.get("Exec")?.clone();
    let description = fields
        .get("Comment[fr]")
        .or_else(|| fields.get("Comment"))
        .cloned()
        .unwrap_or_default();

    let is_flatpak = path
        .to_str()
        .map(|s| s.contains("flatpak"))
        .unwrap_or(false);

    Some(DesktopApp {
        name,
        description,
        exec: clean_exec(&exec),
        is_flatpak,
    })
}

/// Supprime les placeholders %u %f %U %F etc. du champ Exec
pub fn clean_exec(exec: &str) -> String {
    exec.split_whitespace()
        .filter(|s| !s.starts_with('%'))
        .collect::<Vec<_>>()
        .join(" ")
}
