use std::fs;
use std::path::Path;

pub struct Note {
    pub path: String,
    pub title: String,
    pub tags: Vec<String>,
    pub preview: String,
}

pub fn load_notes() -> Vec<Note> {
    let home = std::env::var("HOME").unwrap_or_default();
    let candidate_dirs = [
        format!("{}/notes", home),
        format!("{}/Notes", home),
        format!("{}/Documents/notes", home),
        format!("{}/Documents/Notes", home),
        format!("{}/obsidian", home),
        format!("{}/Obsidian", home),
        format!("{}/vault", home),
        format!("{}/Vault", home),
        format!("{}/wiki", home),
        format!("{}/knowledge", home),
    ];

    let mut notes = Vec::new();

    for dir in &candidate_dirs {
        let p = Path::new(dir);
        if p.is_dir() {
            scan_dir(p, &mut notes, 0);
        }
    }

    notes.sort_by(|a, b| a.title.cmp(&b.title));
    notes
}

fn scan_dir(dir: &Path, notes: &mut Vec<Note>, depth: u8) {
    if depth > 3 {
        return;
    }
    let Ok(entries) = fs::read_dir(dir) else { return };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            scan_dir(&path, notes, depth + 1);
            continue;
        }
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
        if !matches!(ext.as_str(), "md" | "txt" | "markdown" | "org") {
            continue;
        }
        if let Some(note) = parse_note(&path) {
            notes.push(note);
        }
    }
}

fn parse_note(path: &Path) -> Option<Note> {
    let content = fs::read_to_string(path).ok()?;
    let title = path.file_stem()?.to_str()?.to_string();
    let path_str = path.to_str()?.to_string();
    let (tags, preview) = extract_metadata(&content);
    Some(Note { path: path_str, title, tags, preview })
}

fn extract_metadata(content: &str) -> (Vec<String>, String) {
    let mut tags = Vec::new();

    // Parse YAML frontmatter (--- … ---)
    let body_start = if content.starts_with("---") {
        match content[3..].find("\n---") {
            Some(pos) => {
                let fm = &content[3..3 + pos];
                tags.extend(parse_yaml_tags(fm));
                3 + pos + 4 // skip past closing "\n---"
            }
            None => 0,
        }
    } else {
        0
    };

    let body = content.get(body_start..).unwrap_or(content);

    // Extract inline #hashtags from the first 200 words of body
    for word in body.split_whitespace().take(200) {
        // Strip leading punctuation to handle "text. #tag" or "(#tag)"
        let stripped = word.trim_start_matches(|c: char| !c.is_alphanumeric() && c != '#');
        if let Some(tag) = stripped.strip_prefix('#') {
            let tag: String = tag
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_' || *c == '-')
                .collect();
            if tag.len() >= 2 && !tags.contains(&tag) {
                tags.push(tag);
            }
        }
    }

    // Preview: first non-empty, non-heading, non-separator line
    let preview: String = body
        .lines()
        .map(|l| l.trim())
        .find(|l| !l.is_empty() && !l.starts_with('#') && !l.starts_with("---"))
        .unwrap_or("")
        .chars()
        .take(80)
        .collect();

    (tags, preview)
}

fn parse_yaml_tags(frontmatter: &str) -> Vec<String> {
    let mut tags = Vec::new();
    let mut in_tags_block = false;

    for line in frontmatter.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("tags:") {
            in_tags_block = true;
            let rest = trimmed["tags:".len()..].trim();

            if rest.starts_with('[') {
                // Inline array: tags: [foo, bar, "baz"]
                let inner = rest.trim_start_matches('[').trim_end_matches(']');
                for t in inner.split(',') {
                    let t = t.trim().trim_matches('"').trim_matches('\'').to_string();
                    if !t.is_empty() {
                        tags.push(t);
                    }
                }
                in_tags_block = false;
            } else if !rest.is_empty() {
                // Single value: tags: foo
                tags.push(rest.to_string());
                in_tags_block = false;
            }
        } else if in_tags_block {
            if trimmed.starts_with('-') {
                let t = trimmed[1..].trim().trim_matches('"').trim_matches('\'').to_string();
                if !t.is_empty() {
                    tags.push(t);
                }
            } else if !trimmed.is_empty() {
                // Non-indented line means we've left the tags block
                in_tags_block = false;
            }
        }
    }

    tags
}
