use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entry {
    pub id: u64,
    pub timestamp: DateTime<Local>,
    pub kind: EntryKind,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EntryKind {
    System {
        message: String,
    },
    AppLaunched {
        name: String,
        detail: String,
        icon: String,
        duration: Option<u64>,
    },
    Media {
        title: String,
        artist: String,
        progress_secs: u32,
        duration_secs: u32,
        playing: bool,
    },
    Notifications {
        source: String,
        count: usize,
        items: Vec<NotifItem>,
    },
    FileEdit {
        path: String,
        lines: usize,
        preview: String,
        modified_ago: String,
    },
    Shell {
        command: String,
        output_preview: String,
        exit_code: i32,
    },
    PaletteSearch {
        query: String,
        result_chosen: Option<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotifItem {
    pub sender: String,
    pub preview: String,
    pub actions: Vec<ContextAction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextAction {
    pub label: String,
    pub command: String,
}

impl Entry {
    pub fn timestamp_label(&self) -> String {
        self.timestamp.format("%H:%M").to_string()
    }
}
