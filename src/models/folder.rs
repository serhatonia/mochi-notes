use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Folder {
    pub id: Uuid,
    pub name: String,
    /// A `#rrggbb` color used for the folder dot and accents.
    pub color: String,
    pub created_at: DateTime<Local>,
}

impl Folder {
    pub fn new(name: impl Into<String>, color: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            color: color.into(),
            created_at: Local::now(),
        }
    }
}
