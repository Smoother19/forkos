use std::os::unix::fs::PermissionsExt;

pub fn scan() -> Vec<String> {
    let path_var = std::env::var("PATH").unwrap_or_default();
    let mut cmds = Vec::new();

    for dir in path_var.split(':') {
        let Ok(entries) = std::fs::read_dir(dir) else { continue };
        for entry in entries.flatten() {
            let Ok(meta) = entry.metadata() else { continue };
            if meta.is_file() && meta.permissions().mode() & 0o111 != 0 {
                if let Some(name) = entry.file_name().to_str() {
                    cmds.push(name.to_string());
                }
            }
        }
    }

    cmds.sort_unstable();
    cmds.dedup();
    cmds
}

pub fn complete(all: &[String], prefix: &str) -> Vec<String> {
    if prefix.is_empty() {
        return vec![];
    }
    all.iter().filter(|c| c.starts_with(prefix)).take(20).cloned().collect()
}
