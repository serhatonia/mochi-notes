//! SQLite persistence layer built on `rusqlite`.
//!
//! The database is a single file in the user's local app-data directory. Notes
//! are soft-deleted and can be restored; tags and folders are normalized.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use chrono::{DateTime, Local};
use rusqlite::{params, Connection, OptionalExtension};
use uuid::Uuid;

use crate::models::{Folder, Note, Settings, Tag};

#[derive(Debug, thiserror::Error)]
pub enum DbError {
    #[error("database error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("file error: {0}")]
    Io(#[from] std::io::Error),
    #[error("settings error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("{0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, DbError>;

pub struct Database {
    conn: Connection,
    path: PathBuf,
}

impl Database {
    pub fn open(path: &Path) -> Result<Self> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let conn = Connection::open(path)?;
        conn.execute_batch(
            "PRAGMA journal_mode = DELETE;
             PRAGMA synchronous = NORMAL;
             PRAGMA foreign_keys = ON;
             PRAGMA busy_timeout = 3000;",
        )?;

        let db = Self {
            conn,
            path: path.to_path_buf(),
        };
        db.migrate()?;
        Ok(db)
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    fn migrate(&self) -> Result<()> {
        self.conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS folders (
                id          TEXT PRIMARY KEY,
                name        TEXT NOT NULL,
                color       TEXT NOT NULL,
                created_at  TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS notes (
                id          TEXT PRIMARY KEY,
                title       TEXT NOT NULL DEFAULT '',
                content     TEXT NOT NULL DEFAULT '',
                folder_id   TEXT REFERENCES folders(id) ON DELETE SET NULL,
                favorite    INTEGER NOT NULL DEFAULT 0,
                pinned      INTEGER NOT NULL DEFAULT 0,
                archived    INTEGER NOT NULL DEFAULT 0,
                trashed     INTEGER NOT NULL DEFAULT 0,
                sort_order  INTEGER NOT NULL DEFAULT 0,
                created_at  TEXT NOT NULL,
                updated_at  TEXT NOT NULL,
                trashed_at  TEXT
            );

            CREATE TABLE IF NOT EXISTS tags (
                id          TEXT PRIMARY KEY,
                name        TEXT NOT NULL COLLATE NOCASE UNIQUE,
                color       TEXT NOT NULL,
                created_at  TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS note_tags (
                note_id     TEXT NOT NULL REFERENCES notes(id) ON DELETE CASCADE,
                tag_id      TEXT NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
                PRIMARY KEY (note_id, tag_id)
            );

            CREATE TABLE IF NOT EXISTS settings (
                key         TEXT PRIMARY KEY,
                value       TEXT NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_notes_updated ON notes(updated_at DESC);
            CREATE INDEX IF NOT EXISTS idx_notes_folder ON notes(folder_id);
            CREATE INDEX IF NOT EXISTS idx_note_tags_tag ON note_tags(tag_id);
            "#,
        )?;

        // Older libraries predate manual ordering.
        let columns: Vec<String> = self
            .conn
            .prepare("PRAGMA table_info(notes)")?
            .query_map([], |row| row.get::<_, String>(1))?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        if !columns.iter().any(|column| column == "sort_order") {
            self.conn.execute(
                "ALTER TABLE notes ADD COLUMN sort_order INTEGER NOT NULL DEFAULT 0",
                [],
            )?;
        }

        Ok(())
    }

    // -----------------------------------------------------------------
    // Notes
    // -----------------------------------------------------------------

    pub fn load_notes(&self) -> Result<Vec<Note>> {
        struct RawNote {
            id: String,
            title: String,
            content: String,
            folder_id: Option<String>,
            favorite: i64,
            pinned: i64,
            archived: i64,
            trashed: i64,
            sort_order: i64,
            created_at: String,
            updated_at: String,
            trashed_at: Option<String>,
        }

        let mut statement = self.conn.prepare(
            "SELECT id, title, content, folder_id, favorite, pinned, archived, trashed,
                    sort_order, created_at, updated_at, trashed_at
             FROM notes
             ORDER BY updated_at DESC",
        )?;

        let rows = statement.query_map([], |row| {
            Ok(RawNote {
                id: row.get(0)?,
                title: row.get(1)?,
                content: row.get(2)?,
                folder_id: row.get(3)?,
                favorite: row.get(4)?,
                pinned: row.get(5)?,
                archived: row.get(6)?,
                trashed: row.get(7)?,
                sort_order: row.get(8)?,
                created_at: row.get(9)?,
                updated_at: row.get(10)?,
                trashed_at: row.get(11)?,
            })
        })?;

        let mut notes = Vec::new();
        for row in rows {
            let row = row?;
            let Ok(id) = Uuid::parse_str(&row.id) else {
                continue;
            };

            notes.push(Note {
                id,
                title: row.title,
                content: row.content,
                folder_id: row.folder_id.and_then(|id| Uuid::parse_str(&id).ok()),
                tags: Vec::new(),
                favorite: row.favorite != 0,
                pinned: row.pinned != 0,
                archived: row.archived != 0,
                trashed: row.trashed != 0,
                sort_order: row.sort_order,
                created_at: parse_datetime(&row.created_at),
                updated_at: parse_datetime(&row.updated_at),
                trashed_at: row.trashed_at.as_deref().map(parse_datetime),
            });
        }
        drop(statement);

        let mut tags = self.load_note_tags()?;
        for note in &mut notes {
            if let Some(note_tags) = tags.remove(&note.id) {
                note.tags = note_tags;
            }
        }

        Ok(notes)
    }

    fn load_note_tags(&self) -> Result<HashMap<Uuid, Vec<Uuid>>> {
        let mut statement = self
            .conn
            .prepare("SELECT note_id, tag_id FROM note_tags")?;
        let rows = statement.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?;

        let mut map: HashMap<Uuid, Vec<Uuid>> = HashMap::new();
        for row in rows {
            let (note_id, tag_id) = row?;
            if let (Ok(note_id), Ok(tag_id)) =
                (Uuid::parse_str(&note_id), Uuid::parse_str(&tag_id))
            {
                map.entry(note_id).or_default().push(tag_id);
            }
        }

        Ok(map)
    }

    pub fn save_note(&self, note: &Note) -> Result<()> {
        let transaction = self.conn.unchecked_transaction()?;

        transaction.execute(
            "INSERT INTO notes
                (id, title, content, folder_id, favorite, pinned, archived, trashed,
                 sort_order, created_at, updated_at, trashed_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
             ON CONFLICT(id) DO UPDATE SET
                title = excluded.title,
                content = excluded.content,
                folder_id = excluded.folder_id,
                favorite = excluded.favorite,
                pinned = excluded.pinned,
                archived = excluded.archived,
                trashed = excluded.trashed,
                sort_order = excluded.sort_order,
                updated_at = excluded.updated_at,
                trashed_at = excluded.trashed_at",
            params![
                note.id.to_string(),
                note.title,
                note.content,
                note.folder_id.map(|id| id.to_string()),
                note.favorite as i64,
                note.pinned as i64,
                note.archived as i64,
                note.trashed as i64,
                note.sort_order,
                note.created_at.to_rfc3339(),
                note.updated_at.to_rfc3339(),
                note.trashed_at.map(|date| date.to_rfc3339()),
            ],
        )?;

        transaction.execute(
            "DELETE FROM note_tags WHERE note_id = ?1",
            params![note.id.to_string()],
        )?;

        for tag in &note.tags {
            transaction.execute(
                "INSERT OR IGNORE INTO note_tags (note_id, tag_id) VALUES (?1, ?2)",
                params![note.id.to_string(), tag.to_string()],
            )?;
        }

        transaction.commit()?;
        Ok(())
    }

    pub fn delete_note(&self, id: Uuid) -> Result<()> {
        self.conn.execute(
            "DELETE FROM notes WHERE id = ?1",
            params![id.to_string()],
        )?;
        Ok(())
    }

    pub fn delete_all_trashed(&self) -> Result<usize> {
        let count = self
            .conn
            .execute("DELETE FROM notes WHERE trashed = 1", [])?;
        Ok(count)
    }

    // -----------------------------------------------------------------
    // Folders
    // -----------------------------------------------------------------

    pub fn load_folders(&self) -> Result<Vec<Folder>> {
        let mut statement = self
            .conn
            .prepare("SELECT id, name, color, created_at FROM folders ORDER BY name COLLATE NOCASE")?;
        let rows = statement.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
            ))
        })?;

        let mut folders = Vec::new();
        for row in rows {
            let (id, name, color, created_at) = row?;
            let Ok(id) = Uuid::parse_str(&id) else {
                continue;
            };
            folders.push(Folder {
                id,
                name,
                color,
                created_at: parse_datetime(&created_at),
            });
        }

        Ok(folders)
    }

    pub fn save_folder(&self, folder: &Folder) -> Result<()> {
        self.conn.execute(
            "INSERT INTO folders (id, name, color, created_at)
             VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT(id) DO UPDATE SET name = excluded.name, color = excluded.color",
            params![
                folder.id.to_string(),
                folder.name,
                folder.color,
                folder.created_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    pub fn delete_folder(&self, id: Uuid) -> Result<()> {
        self.conn.execute(
            "DELETE FROM folders WHERE id = ?1",
            params![id.to_string()],
        )?;
        Ok(())
    }

    // -----------------------------------------------------------------
    // Tags
    // -----------------------------------------------------------------

    pub fn load_tags(&self) -> Result<Vec<Tag>> {
        let mut statement = self
            .conn
            .prepare("SELECT id, name, color, created_at FROM tags ORDER BY name COLLATE NOCASE")?;
        let rows = statement.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
            ))
        })?;

        let mut tags = Vec::new();
        for row in rows {
            let (id, name, color, created_at) = row?;
            let Ok(id) = Uuid::parse_str(&id) else {
                continue;
            };
            tags.push(Tag {
                id,
                name,
                color,
                created_at: parse_datetime(&created_at),
            });
        }

        Ok(tags)
    }

    pub fn save_tag(&self, tag: &Tag) -> Result<()> {
        self.conn.execute(
            "INSERT INTO tags (id, name, color, created_at)
             VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT(id) DO UPDATE SET name = excluded.name, color = excluded.color",
            params![
                tag.id.to_string(),
                tag.name,
                tag.color,
                tag.created_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    pub fn delete_tag(&self, id: Uuid) -> Result<()> {
        self.conn.execute("DELETE FROM tags WHERE id = ?1", params![id.to_string()])?;
        Ok(())
    }

    // -----------------------------------------------------------------
    // Settings
    // -----------------------------------------------------------------

    pub fn load_settings(&self) -> Result<Settings> {
        let raw: Option<String> = self
            .conn
            .query_row(
                "SELECT value FROM settings WHERE key = 'app'",
                [],
                |row| row.get(0),
            )
            .optional()?;

        match raw {
            Some(json) => Ok(serde_json::from_str(&json).unwrap_or_default()),
            None => Ok(Settings::default()),
        }
    }

    pub fn save_settings(&self, settings: &Settings) -> Result<()> {
        let json = serde_json::to_string_pretty(settings)?;
        self.conn.execute(
            "INSERT INTO settings (key, value) VALUES ('app', ?1)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![json],
        )?;
        Ok(())
    }

    // -----------------------------------------------------------------
    // Maintenance
    // -----------------------------------------------------------------

    pub fn size_bytes(&self) -> u64 {
        fs::metadata(&self.path).map(|m| m.len()).unwrap_or(0)
    }

    /// Copies the database into `directory`, returning the new file.
    pub fn backup_to(&self, directory: &Path) -> Result<PathBuf> {
        fs::create_dir_all(directory)?;

        let stamp = Local::now().format("%Y%m%d-%H%M%S");
        let mut target = directory.join(format!("mochi-{stamp}.db"));
        let mut attempt = 2;
        while target.exists() {
            target = directory.join(format!("mochi-{stamp}-{attempt}.db"));
            attempt += 1;
        }

        fs::copy(&self.path, &target)?;
        Ok(target)
    }

    /// Runs `VACUUM` to keep the database tidy.
    pub fn vacuum(&self) -> Result<()> {
        self.conn.execute_batch("VACUUM;")?;
        Ok(())
    }
}

fn parse_datetime(value: &str) -> DateTime<Local> {
    DateTime::parse_from_rfc3339(value)
        .map(|date| date.with_timezone(&Local))
        .unwrap_or_else(|_| Local::now())
}
