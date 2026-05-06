use std::path::Path;

pub struct RecentFile {
    pub path: String,
    pub name: String,
    pub modified: String,
}

pub fn load() -> Vec<RecentFile> {
    let home = std::env::var("HOME").unwrap_or_default();
    let xbel = format!("{}/.local/share/recently-used.xbel", home);
    let content = match std::fs::read_to_string(&xbel) {
        Ok(c) => c,
        Err(_) => return vec![],
    };
    parse_xbel(&content)
}

fn parse_xbel(xml: &str) -> Vec<RecentFile> {
    let mut results = Vec::new();
    let mut pos = 0;

    while let Some(start) = xml[pos..].find("<bookmark") {
        let abs_start = pos + start;
        let tag_end = match xml[abs_start..].find('>') {
            Some(i) => abs_start + i,
            None => break,
        };
        let tag = &xml[abs_start..=tag_end];
        if let Some(file) = parse_bookmark_tag(tag) {
            results.push(file);
        }
        pos = tag_end + 1;
        if pos >= xml.len() {
            break;
        }
    }

    results.reverse();
    results.truncate(30);
    results
}

fn parse_bookmark_tag(tag: &str) -> Option<RecentFile> {
    let href = extract_attr(tag, "href")?;
    if !href.starts_with("file://") {
        return None;
    }
    let raw_path = &href["file://".len()..];
    let decoded_path = percent_decode(raw_path);
    if !Path::new(&decoded_path).exists() {
        return None;
    }

    let name = Path::new(&decoded_path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(&decoded_path)
        .to_string();

    let modified = extract_attr(tag, "modified")
        .map(|m| format_relative_time(&m))
        .unwrap_or_else(|| "récemment".to_string());

    Some(RecentFile { path: decoded_path, name, modified })
}

fn extract_attr<'a>(tag: &'a str, attr: &str) -> Option<&'a str> {
    let needle = format!("{}=\"", attr);
    let start = tag.find(&needle)? + needle.len();
    let end = tag[start..].find('"')? + start;
    Some(&tag[start..end])
}

fn percent_decode(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let Ok(hex) = std::str::from_utf8(&bytes[i + 1..i + 3]) {
                if let Ok(byte) = u8::from_str_radix(hex, 16) {
                    result.push(byte as char);
                    i += 3;
                    continue;
                }
            }
        }
        result.push(bytes[i] as char);
        i += 1;
    }
    result
}

fn format_relative_time(iso: &str) -> String {
    let date_part = iso.split('T').next().unwrap_or("");
    if date_part.is_empty() {
        return "récemment".to_string();
    }
    let parts: Vec<&str> = date_part.split('-').collect();
    if parts.len() == 3 {
        format!("{}/{}/{}", parts[2], parts[1], parts[0])
    } else {
        "récemment".to_string()
    }
}
