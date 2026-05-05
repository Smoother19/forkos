use std::path::PathBuf;

/// Resolves a freedesktop icon name to an absolute PNG or SVG file path.
/// Returns None if the icon cannot be found.
pub fn resolve(name: &str) -> Option<PathBuf> {
    if name.is_empty() {
        return None;
    }

    // Already an absolute path
    if name.starts_with('/') {
        let p = PathBuf::from(name);
        if p.exists() {
            return Some(p);
        }
        return None;
    }

    let home = std::env::var("HOME").unwrap_or_default();

    // Preferred sizes in lookup order (48px first for crisp display)
    let sizes = ["48x48", "32x32", "64x64", "scalable", "24x24", "22x22", "16x16", "128x128", "256x256"];
    let categories = ["apps", "applications"];
    let exts = ["png", "svg"];

    let icon_bases: Vec<PathBuf> = vec![
        PathBuf::from(format!("{}/.local/share/icons", home)),
        PathBuf::from("/usr/share/icons"),
        PathBuf::from("/usr/local/share/icons"),
    ];

    // Themes in priority order; hicolor is the mandatory fallback theme
    let themes = ["hicolor", "Papirus", "Adwaita", "gnome", "breeze", "oxygen"];

    for base in &icon_bases {
        for theme in &themes {
            let theme_dir = base.join(theme);
            if !theme_dir.is_dir() {
                continue;
            }
            for size in &sizes {
                for cat in &categories {
                    for ext in &exts {
                        let p = theme_dir.join(size).join(cat).join(format!("{}.{}", name, ext));
                        if p.exists() {
                            return Some(p);
                        }
                    }
                }
            }
        }
        // Flat fallback inside base dir
        for ext in &exts {
            let p = base.join(format!("{}.{}", name, ext));
            if p.exists() {
                return Some(p);
            }
        }
    }

    // /usr/share/pixmaps — last resort
    for ext in &exts {
        let p = PathBuf::from(format!("/usr/share/pixmaps/{}.{}", name, ext));
        if p.exists() {
            return Some(p);
        }
    }
    // Some pixmaps have no extension
    let p = PathBuf::from(format!("/usr/share/pixmaps/{}", name));
    if p.exists() {
        return Some(p);
    }

    None
}
