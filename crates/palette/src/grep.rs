#[derive(Debug, Clone)]
pub struct GrepMatch {
    pub file: String,
    pub line: u32,
    pub content: String,
}

/// Cherche `pattern` dans les fichiers texte sous `search_dir`.
/// Utilise `rg` (ripgrep) si disponible, sinon `grep -r`.
pub async fn search(pattern: String, search_dir: String) -> Vec<GrepMatch> {
    if pattern.is_empty() {
        return vec![];
    }

    // Essaie ripgrep, sinon grep
    let output = std::process::Command::new("rg")
        .args(["--line-number", "--max-count=50", "--color=never", &pattern, &search_dir])
        .output()
        .or_else(|_| {
            std::process::Command::new("grep")
                .args(["-rn", "--include=*.*", "-m", "50", &pattern, &search_dir])
                .output()
        });

    match output {
        Ok(out) if out.status.success() || out.status.code() == Some(1) => {
            // code 1 = pas de résultats pour grep, on continue
            parse_grep_output(&String::from_utf8_lossy(&out.stdout))
        }
        _ => vec![],
    }
}

fn parse_grep_output(raw: &str) -> Vec<GrepMatch> {
    let mut results = Vec::new();

    for line in raw.lines().take(50) {
        // Format: "chemin/fichier:N:contenu"
        let mut parts = line.splitn(3, ':');
        let file = match parts.next() {
            Some(f) => f.to_string(),
            None => continue,
        };
        let line_num: u32 = match parts.next().and_then(|n| n.parse().ok()) {
            Some(n) => n,
            None => continue,
        };
        let content = parts.next().unwrap_or("").trim().to_string();

        // Raccourcit le chemin de fichier pour l'affichage
        let display_file = shorten_path(&file);

        results.push(GrepMatch { file: display_file, line: line_num, content });
    }

    results
}

fn shorten_path(path: &str) -> String {
    // Remplace le $HOME par ~
    if let Ok(home) = std::env::var("HOME") {
        if path.starts_with(&home) {
            return format!("~{}", &path[home.len()..]);
        }
    }
    // Garde seulement les 3 dernières composantes
    let parts: Vec<&str> = path.split('/').collect();
    if parts.len() > 3 {
        format!(".../{}", parts[parts.len() - 3..].join("/"))
    } else {
        path.to_string()
    }
}
