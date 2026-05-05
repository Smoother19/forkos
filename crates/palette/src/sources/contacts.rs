use std::fs;

pub struct Contact {
    pub name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub org: Option<String>,
}

pub fn load() -> Vec<Contact> {
    let home = std::env::var("HOME").unwrap_or_default();
    let dirs = [
        format!("{}/.local/share/gnome-contacts", home),
        format!("{}/contacts", home),
        format!("{}/Contacts", home),
        format!("{}/Documents/contacts", home),
        format!("{}/vCards", home),
        format!("{}/vcards", home),
    ];

    let mut contacts = Vec::new();

    for dir in &dirs {
        let Ok(entries) = fs::read_dir(dir) else { continue };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("vcf") {
                continue;
            }
            if let Ok(content) = fs::read_to_string(&path) {
                contacts.extend(parse_vcard_file(&content));
            }
        }
    }

    // Single contacts.vcf in home
    let single = format!("{}/contacts.vcf", home);
    if let Ok(content) = fs::read_to_string(&single) {
        contacts.extend(parse_vcard_file(&content));
    }

    contacts.sort_by(|a, b| a.name.cmp(&b.name));
    contacts.dedup_by(|a, b| a.name == b.name);
    contacts
}

/// A .vcf file can contain multiple vCards (BEGIN:VCARD … END:VCARD blocks).
fn parse_vcard_file(content: &str) -> Vec<Contact> {
    let mut contacts = Vec::new();
    let mut block = String::new();
    let mut in_vcard = false;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.eq_ignore_ascii_case("BEGIN:VCARD") {
            in_vcard = true;
            block.clear();
        } else if trimmed.eq_ignore_ascii_case("END:VCARD") {
            if let Some(c) = parse_vcard_block(&block) {
                contacts.push(c);
            }
            in_vcard = false;
        } else if in_vcard {
            block.push_str(line);
            block.push('\n');
        }
    }

    contacts
}

fn parse_vcard_block(content: &str) -> Option<Contact> {
    let mut name: Option<String> = None;
    let mut email: Option<String> = None;
    let mut phone: Option<String> = None;
    let mut org: Option<String> = None;

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        if let Some(val) = line.strip_prefix("FN:") {
            // FN: is the preferred display name
            if name.is_none() {
                let v = val.trim().to_string();
                if !v.is_empty() {
                    name = Some(v);
                }
            }
        } else if line.starts_with("EMAIL") {
            // EMAIL: or EMAIL;TYPE=work: etc.
            if email.is_none() {
                if let Some(val) = line.splitn(2, ':').nth(1) {
                    let v = val.trim().to_string();
                    if !v.is_empty() {
                        email = Some(v);
                    }
                }
            }
        } else if line.starts_with("TEL") {
            // TEL: or TEL;TYPE=cell: etc.
            if phone.is_none() {
                if let Some(val) = line.splitn(2, ':').nth(1) {
                    let v = val.trim().to_string();
                    if !v.is_empty() {
                        phone = Some(v);
                    }
                }
            }
        } else if let Some(val) = line.strip_prefix("ORG:") {
            if org.is_none() {
                // ORG has components separated by ;
                let v = val.split(';').next().unwrap_or(val).trim().to_string();
                if !v.is_empty() {
                    org = Some(v);
                }
            }
        } else if let Some(val) = line.strip_prefix("N:") {
            // N: Last;First;Middle;Prefix;Suffix — fallback if FN: absent
            if name.is_none() {
                let parts: Vec<&str> = val.split(';').collect();
                let last = parts.first().map(|s| s.trim()).unwrap_or("");
                let first = parts.get(1).map(|s| s.trim()).unwrap_or("");
                let combined = match (first.is_empty(), last.is_empty()) {
                    (false, false) => format!("{} {}", first, last),
                    (true, false) => last.to_string(),
                    (false, true) => first.to_string(),
                    _ => String::new(),
                };
                if !combined.is_empty() {
                    name = Some(combined);
                }
            }
        }
    }

    let name = name.filter(|n| !n.is_empty())?;
    Some(Contact { name, email, phone, org })
}
