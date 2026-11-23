use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

use crate::theme::ThemeId;

/// User preferences. Persisted as JSON in the SQLite settings table.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    /// Displayed in the dashboard greeting ("Good morning, <name>").
    pub username: String,
    pub theme: ThemeId,
    pub editor_font_size: f32,
    pub autosave: bool,
    /// Debounce before an automatic save, in milliseconds.
    pub autosave_delay_ms: u64,
    pub backup_on_start: bool,
    pub last_backup_at: Option<DateTime<Local>>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            username: String::new(),
            theme: ThemeId::StrawberryMilk,
            editor_font_size: 15.5,
            autosave: true,
            autosave_delay_ms: 900,
            backup_on_start: true,
            last_backup_at: None,
        }
    }
}
