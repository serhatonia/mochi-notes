use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::utils::text;

/// A single note. Notes are soft-deleted into the trash and can be archived.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Note {
    pub id: Uuid,
    pub title: String,
    pub content: String,
    pub folder_id: Option<Uuid>,
    pub tags: Vec<Uuid>,
    pub favorite: bool,
    pub pinned: bool,
    pub archived: bool,
    pub trashed: bool,
    /// Manual ordering index, used when favorites are rearranged by dragging.
    #[serde(default)]
    pub sort_order: i64,
    pub created_at: DateTime<Local>,
    pub updated_at: DateTime<Local>,
    pub trashed_at: Option<DateTime<Local>>,
}

impl Note {
    pub fn new(folder_id: Option<Uuid>) -> Self {
        let now = Local::now();
        Self {
            id: Uuid::new_v4(),
            title: String::new(),
            content: String::new(),
            folder_id,
            tags: Vec::new(),
            favorite: false,
            pinned: false,
            archived: false,
            trashed: false,
            sort_order: 0,
            created_at: now,
            updated_at: now,
            trashed_at: None,
        }
    }

    /// A title that is always safe to display.
    pub fn display_title(&self) -> String {
        let title = self.title.trim();
        if !title.is_empty() {
            return title.to_string();
        }

        let first_line = self
            .content
            .lines()
            .map(|line| line.trim_start_matches(['#', '>', '-', '*', ' ']).trim())
            .find(|line| !line.is_empty());

        match first_line {
            Some(line) => text::truncate_chars(line, 48),
            None => "Untitled note".to_string(),
        }
    }

    /// A plain-text preview of the note body, without markdown noise.
    pub fn excerpt(&self, max_chars: usize) -> String {
        let plain = text::strip_markdown(&self.content);
        if plain.is_empty() {
            return String::new();
        }

        text::truncate_chars(&plain, max_chars)
    }

    pub fn word_count(&self) -> usize {
        text::word_count(&self.content)
    }

    pub fn char_count(&self) -> usize {
        self.content.chars().count()
    }

    pub fn reading_minutes(&self) -> usize {
        text::reading_minutes(self.word_count())
    }

    pub fn is_blank(&self) -> bool {
        self.title.trim().is_empty() && self.content.trim().is_empty() && self.tags.is_empty()
    }

    pub fn matches(&self, query: &str) -> bool {
        let query = query.trim().to_lowercase();
        if query.is_empty() {
            return true;
        }

        self.title.to_lowercase().contains(&query)
            || self.content.to_lowercase().contains(&query)
    }
}
