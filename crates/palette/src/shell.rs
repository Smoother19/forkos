use std::process::Command as SysCommand;

#[derive(Debug, Clone)]
pub struct ShellEntry {
    pub command: String,
    pub output: String,
    pub success: bool,
}

/// Exécute une commande shell via `sh -c` et retourne le résultat.
/// Appelé depuis un Task::perform pour ne pas bloquer l'UI.
pub async fn execute(cmd: String) -> ShellEntry {
    let result = SysCommand::new("sh").arg("-c").arg(&cmd).output();

    match result {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout).to_string();
            let stderr = String::from_utf8_lossy(&out.stderr).to_string();
            let output = match (stdout.is_empty(), stderr.is_empty()) {
                (false, false) => format!("{}\n{}", stdout.trim_end(), stderr.trim_end()),
                (false, true) => stdout.trim_end().to_string(),
                (true, false) => stderr.trim_end().to_string(),
                (true, true) => "(aucune sortie)".to_string(),
            };
            ShellEntry { command: cmd, output, success: out.status.success() }
        }
        Err(e) => ShellEntry {
            command: cmd,
            output: format!("erreur: {}", e),
            success: false,
        },
    }
}
