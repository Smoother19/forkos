use portable_pty::{CommandBuilder, MasterPty, NativePtySystem, PtySize, PtySystem, SlavePty};
use std::io::Read;

#[derive(Debug, Clone)]
pub struct PtyLine {
    pub spans: Vec<StyledSpan>,
}

#[derive(Debug, Clone)]
pub struct StyledSpan {
    pub text: String,
    pub color: SpanColor,
    pub bold: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SpanColor {
    Default,
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
    BrightBlack,
    BrightRed,
    BrightGreen,
    BrightYellow,
    BrightBlue,
    BrightMagenta,
    BrightCyan,
    BrightWhite,
}

impl SpanColor {
    pub fn to_iced_color(self) -> iced::Color {
        use forkos_shared::theme;
        match self {
            SpanColor::Default       => theme::TEXT,
            SpanColor::Black         => theme::OVERLAY,
            SpanColor::Red           => theme::LOVE,
            SpanColor::Green         => theme::PINE,
            SpanColor::Yellow        => theme::GOLD,
            SpanColor::Blue          => theme::FOAM,
            SpanColor::Magenta       => theme::IRIS,
            SpanColor::Cyan          => theme::FOAM,
            SpanColor::White         => theme::TEXT,
            SpanColor::BrightBlack   => theme::MUTED,
            SpanColor::BrightRed     => theme::LOVE,
            SpanColor::BrightGreen   => theme::PINE,
            SpanColor::BrightYellow  => theme::GOLD,
            SpanColor::BrightBlue    => theme::FOAM,
            SpanColor::BrightMagenta => theme::IRIS,
            SpanColor::BrightCyan    => theme::FOAM,
            SpanColor::BrightWhite   => theme::SUBTLE,
        }
    }
}

/// Spawne un PTY avec `$SHELL`, démarre la lecture en thread, retourne le writer stdin.
pub fn spawn_pty(
    cols: u16,
    rows: u16,
    tx: tokio::sync::mpsc::UnboundedSender<String>,
) -> anyhow::Result<Box<dyn std::io::Write + Send>> {
    let pty_system = NativePtySystem::default();
    let pair = pty_system.openpty(PtySize {
        rows,
        cols,
        pixel_width: 0,
        pixel_height: 0,
    })?;

    let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string());
    let mut cmd = CommandBuilder::new(&shell);
    cmd.env("TERM", "xterm-256color");
    cmd.env("COLORTERM", "truecolor");

    let child = pair.slave.spawn_command(cmd)?;
    std::mem::forget(child); // le shell continue sans être tué au drop

    let writer = pair.master.take_writer()?;
    let mut reader = pair.master.try_clone_reader()?;

    std::thread::spawn(move || {
        let mut buf = [0u8; 4096];
        loop {
            match reader.read(&mut buf) {
                Ok(0) | Err(_) => break,
                Ok(n) => {
                    let chunk = String::from_utf8_lossy(&buf[..n]).to_string();
                    if tx.send(chunk).is_err() {
                        break;
                    }
                }
            }
        }
    });

    Ok(writer)
}

/// Parse une string avec séquences ANSI en lignes de spans colorés.
/// Gère SGR 3/4-bit (couleurs 30–37, 90–97, bold 1, reset 0).
pub fn parse_ansi(raw: &str) -> Vec<PtyLine> {
    let mut lines: Vec<PtyLine> = vec![PtyLine { spans: vec![] }];
    let mut current_color = SpanColor::Default;
    let mut current_bold = false;
    let mut current_text = String::new();
    let bytes = raw.as_bytes();
    let mut i = 0;

    macro_rules! flush {
        () => {
            if !current_text.is_empty() {
                lines.last_mut().unwrap().spans.push(StyledSpan {
                    text: current_text.clone(),
                    color: current_color,
                    bold: current_bold,
                });
                current_text.clear();
            }
        };
    }

    while i < bytes.len() {
        // Séquence ESC [
        if bytes[i] == 0x1b && i + 1 < bytes.len() && bytes[i + 1] == b'[' {
            flush!();
            i += 2;
            let start = i;
            while i < bytes.len() && !bytes[i].is_ascii_alphabetic() {
                i += 1;
            }
            let cmd_char = if i < bytes.len() { bytes[i] as char } else { 'm' };
            let params_str = std::str::from_utf8(&bytes[start..i]).unwrap_or("");
            i += 1;

            if cmd_char == 'm' {
                let params: Vec<u8> =
                    params_str.split(';').filter_map(|s| s.parse().ok()).collect();
                if params.is_empty() {
                    current_color = SpanColor::Default;
                    current_bold = false;
                } else {
                    for &p in &params {
                        match p {
                            0  => { current_color = SpanColor::Default; current_bold = false; }
                            1  => current_bold = true,
                            22 => current_bold = false,
                            30 => current_color = SpanColor::Black,
                            31 => current_color = SpanColor::Red,
                            32 => current_color = SpanColor::Green,
                            33 => current_color = SpanColor::Yellow,
                            34 => current_color = SpanColor::Blue,
                            35 => current_color = SpanColor::Magenta,
                            36 => current_color = SpanColor::Cyan,
                            37 => current_color = SpanColor::White,
                            39 => current_color = SpanColor::Default,
                            90 => current_color = SpanColor::BrightBlack,
                            91 => current_color = SpanColor::BrightRed,
                            92 => current_color = SpanColor::BrightGreen,
                            93 => current_color = SpanColor::BrightYellow,
                            94 => current_color = SpanColor::BrightBlue,
                            95 => current_color = SpanColor::BrightMagenta,
                            96 => current_color = SpanColor::BrightCyan,
                            97 => current_color = SpanColor::BrightWhite,
                            _  => {}
                        }
                    }
                }
            }
            // Autres séquences (curseur, effacement) → ignorées
            continue;
        }

        if bytes[i] == b'\n' {
            flush!();
            lines.push(PtyLine { spans: vec![] });
            i += 1;
            continue;
        }

        if bytes[i] == b'\r' {
            i += 1;
            continue;
        }

        if let Some(c) = std::str::from_utf8(&bytes[i..]).ok().and_then(|s| s.chars().next()) {
            current_text.push(c);
            i += c.len_utf8();
        } else {
            i += 1;
        }
    }

    flush!();

    // Supprime les lignes vides en queue
    while lines.last().map(|l| l.spans.is_empty()).unwrap_or(false) {
        lines.pop();
    }

    lines
}
