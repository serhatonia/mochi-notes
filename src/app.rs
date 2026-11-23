//! Application state, boot sequence and update logic.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use chrono::Local;
use iced::time::Instant;
use iced::widget::{markdown, operation, text_editor, Id};
use iced::{keyboard, window, Element, Point, Subscription, Task, Theme};
use uuid::Uuid;

use crate::database::Database;
use crate::message::{
    ConfirmKind, DragState, DropTarget, Format, Hover, Message, Modal, NavId, Screen, SortMode,
    ToastKind,
};
use crate::models::{Folder, Note, Settings, Tag};
use crate::services::{backup, export, paths::Paths};
use crate::theme::{self, Palette};
use crate::utils::text as text_utils;
use crate::views;

pub const EDITOR_ID: &str = "editor";
pub const SEARCH_ID: &str = "search";
pub const TITLE_ID: &str = "title";

#[derive(Debug, Clone)]
pub enum SaveStatus {
    Idle,
    Editing,
    Saved(chrono::DateTime<Local>),
    Error(String),
}

pub struct EditorState {
    pub note_id: Uuid,
    pub content: text_editor::Content,
    pub dirty: bool,
    pub words: usize,
    pub chars: usize,
    pub preview: Option<markdown::Content>,
}

impl EditorState {
    fn new(note: &Note) -> Self {
        Self {
            note_id: note.id,
            content: text_editor::Content::with_text(&note.content),
            dirty: false,
            words: text_utils::word_count(&note.content),
            chars: note.content.chars().count(),
            preview: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Toast {
    pub text: String,
    pub kind: ToastKind,
    pub born: Instant,
}

pub struct Mochi {
    pub db: Option<Database>,
    pub db_error: Option<String>,
    pub paths: Paths,

    pub notes: Vec<Note>,
    pub folders: Vec<Folder>,
    pub tags: Vec<Tag>,
    pub settings: Settings,

    pub palette: Palette,
    pub iced_theme: Theme,

    pub screen: Screen,
    pub editor: Option<EditorState>,
    pub editor_return: Screen,
    /// Distraction-free writing mode (session only).
    pub focus_mode: bool,

    pub search: String,
    pub sort: SortMode,

    pub hovered: Option<Hover>,
    pub hover_t: f32,
    pub phase: f32,

    /// The id of the main (frameless) window, used by the custom title bar.
    pub window_id: Option<window::Id>,
    pub window_maximized: bool,
    pub last_tick: Instant,
    dirty_since: Option<Instant>,

    pub save_status: SaveStatus,
    pub toast: Option<Toast>,
    pub modal: Option<Modal>,
    pub backups: Vec<backup::BackupInfo>,

    /// Little messages Mochi says when you poke it.
    pub mascot_message: Option<String>,
    pub mascot_born: Instant,
    pub mascot_pokes: usize,

    /// Mouse cursor in window coordinates and the active note drag.
    pub cursor: Point,
    pub drag: Option<DragState>,
}

/// Global mouse/file events used for note dragging and file imports.
fn drag_event(
    event: iced::event::Event,
    _status: iced::event::Status,
    _window: window::Id,
) -> Option<Message> {
    match event {
        iced::event::Event::Mouse(iced::mouse::Event::CursorMoved { position }) => {
            Some(Message::CursorMoved(position))
        }
        iced::event::Event::Mouse(iced::mouse::Event::ButtonReleased(
            iced::mouse::Button::Left,
        )) => Some(Message::DragReleased),
        iced::event::Event::Window(window::Event::FileDropped(path)) => {
            Some(Message::FilesDropped(path))
        }
        _ => None,
    }
}

impl Mochi {
    pub fn new() -> (Self, Task<Message>) {
        let paths = Paths::discover();
        let mut app = Self {
            db: None,
            db_error: None,
            paths,
            notes: Vec::new(),
            folders: Vec::new(),
            tags: Vec::new(),
            settings: Settings::default(),
            palette: theme::ThemeId::default().palette(),
            iced_theme: theme::ThemeId::default().iced_theme(),
            screen: Screen::Home,
            editor: None,
            editor_return: Screen::Home,
            focus_mode: false,
            search: String::new(),
            sort: SortMode::Updated,
            hovered: None,
            hover_t: 0.0,
            phase: 0.0,
            window_id: None,
            window_maximized: false,
            last_tick: Instant::now(),
            dirty_since: None,
            save_status: SaveStatus::Idle,
            toast: None,
            modal: None,
            backups: Vec::new(),
            mascot_message: None,
            mascot_born: Instant::now(),
            mascot_pokes: 0,
            cursor: Point::new(0.0, 0.0),
            drag: None,
        };

        app.open_database();

        let mut tasks = vec![window::oldest().map(Message::WindowReady)];
        if app.db.is_some() && app.settings.backup_on_start {
            let stale = app
                .settings
                .last_backup_at
                .map(|date| Local::now().signed_duration_since(date).num_hours() >= 12)
                .unwrap_or(true);

            if stale {
                let db_path = app.paths.db_path.clone();
                let backups_dir = app.paths.backups_dir.clone();
                tasks.push(Task::perform(
                    async move {
                        backup::create_backup(&db_path, &backups_dir, 20)
                    },
                    Message::BackupCreated,
                ));
            }
        }

        (app, Task::batch(tasks))
    }

    fn open_database(&mut self) {
        match Database::open(&self.paths.db_path) {
            Ok(db) => {
                self.db = Some(db);
                self.db_error = None;
                self.reload();
            }
            Err(error) => {
                self.db = None;
                self.db_error = Some(error.to_string());
            }
        }
    }

    fn reload(&mut self) {
        let Some(db) = &self.db else {
            return;
        };

        match db.load_notes() {
            Ok(notes) => {
                self.notes = notes;
                for (index, note) in self.notes.iter_mut().enumerate() {
                    if note.sort_order == 0 {
                        note.sort_order = index as i64 + 1;
                    }
                }
            }
            Err(error) => self.db_error = Some(error.to_string()),
        }
        match db.load_folders() {
            Ok(folders) => self.folders = folders,
            Err(error) => self.db_error = Some(error.to_string()),
        }
        match db.load_tags() {
            Ok(tags) => self.tags = tags,
            Err(error) => self.db_error = Some(error.to_string()),
        }
        if let Ok(settings) = db.load_settings() {
            self.settings = settings;
        }

        self.backups = backup::list_backups(&self.paths.backups_dir);
        self.apply_theme();
    }

    fn apply_theme(&mut self) {
        self.palette = self.settings.theme.palette();
        self.iced_theme = self.settings.theme.iced_theme();
    }

    // -----------------------------------------------------------------
    // Iced integration
    // -----------------------------------------------------------------

    pub fn title(&self) -> String {
        match self.editor.as_ref().and_then(|editor| self.note(editor.note_id)) {
            Some(note) => format!("{} — Mochi Notes", note.display_title()),
            None => "Mochi Notes".to_string(),
        }
    }

    pub fn theme(&self) -> Theme {
        self.iced_theme.clone()
    }

    /// A short, human readable label for the current screen, shown in the
    /// custom title bar next to the app name.
    pub fn breadcrumb(&self) -> String {
        if let Some(note) = self.current_note() {
            return note.display_title();
        }

        match self.screen {
            Screen::Home => "Home".into(),
            Screen::AllNotes => "All notes".into(),
            Screen::Favorites => "Favorites".into(),
            Screen::Folder(id) => self
                .folder(id)
                .map(|folder| folder.name.clone())
                .unwrap_or_else(|| "Folder".into()),
            Screen::Tag(id) => self
                .tag(id)
                .map(|tag| format!("#{}", tag.name))
                .unwrap_or_else(|| "Tag".into()),
            Screen::Folders => "Folders".into(),
            Screen::Tags => "Tags".into(),
            Screen::Archive => "Archive".into(),
            Screen::Trash => "Trash".into(),
            Screen::Settings => "Settings".into(),
            Screen::Editor => "Editor".into(),
        }
    }

    pub fn subscription(&self) -> Subscription<Message> {
        let animating = self.hovered.is_some()
            || self.toast.is_some()
            || matches!(self.screen, Screen::Home)
            || self.editor.is_some();

        let interval = if animating { 33 } else { 250 };

        Subscription::batch([
            iced::time::every(Duration::from_millis(interval)).map(Message::Tick),
            keyboard::listen().map(Message::Keyboard),
            window::close_requests().map(Message::CloseRequested),
            window::open_events().map(|id| Message::WindowReady(Some(id))),
            iced::event::listen_with(drag_event),
        ])
    }

    pub fn view(&self) -> Element<'_, Message> {
        views::root(self)
    }

    // -----------------------------------------------------------------
    // Accessors used by views
    // -----------------------------------------------------------------

    pub fn note(&self, id: Uuid) -> Option<&Note> {
        self.notes.iter().find(|note| note.id == id)
    }

    pub fn folder(&self, id: Uuid) -> Option<&Folder> {
        self.folders.iter().find(|folder| folder.id == id)
    }

    pub fn tag(&self, id: Uuid) -> Option<&Tag> {
        self.tags.iter().find(|tag| tag.id == id)
    }

    pub fn current_note(&self) -> Option<&Note> {
        self.editor
            .as_ref()
            .and_then(|editor| self.note(editor.note_id))
    }

    pub fn active_notes(&self) -> impl Iterator<Item = &Note> {
        self.notes
            .iter()
            .filter(|note| !note.trashed && !note.archived)
    }

    pub fn favorite_count(&self) -> usize {
        self.active_notes().filter(|note| note.favorite).count()
    }

    pub fn archived_count(&self) -> usize {
        self.notes
            .iter()
            .filter(|note| note.archived && !note.trashed)
            .count()
    }

    pub fn trashed_count(&self) -> usize {
        self.notes.iter().filter(|note| note.trashed).count()
    }

    pub fn folder_count(&self, id: Uuid) -> usize {
        self.active_notes()
            .filter(|note| note.folder_id == Some(id))
            .count()
    }

    pub fn tag_count(&self, id: Uuid) -> usize {
        self.active_notes()
            .filter(|note| note.tags.contains(&id))
            .count()
    }

    pub fn total_words(&self) -> usize {
        self.active_notes().map(Note::word_count).sum()
    }

    pub fn hover_progress(&self, key: Hover) -> f32 {
        if self.hovered == Some(key) {
            self.hover_t
        } else {
            0.0
        }
    }

    // -----------------------------------------------------------------
    // Update
    // -----------------------------------------------------------------

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Tick(now) => self.tick(now),
            Message::Keyboard(event) => self.keyboard(event),
            Message::CloseRequested(id) => {
                self.window_id = Some(id);
                self.close_editor(true);
                window::close(id)
            }
            Message::Noop => Task::none(),

            // Custom window chrome
            Message::WindowReady(id) => {
                if let Some(id) = id {
                    self.window_id = Some(id);
                    return window::is_maximized(id).map(Message::WindowMaximized);
                }
                Task::none()
            }
            Message::WindowMaximized(value) => {
                self.window_maximized = value;
                Task::none()
            }
            Message::TitleBarDrag => self
                .window_id
                .map(window::drag)
                .unwrap_or_else(Task::none),
            Message::TitleBarDoubleClick | Message::ToggleMaximizeWindow => {
                self.window_toggle_maximize()
            }
            Message::WindowSystemMenu => self
                .window_id
                .map(window::show_system_menu)
                .unwrap_or_else(Task::none),
            Message::MinimizeWindow => self
                .window_id
                .map(|id| window::minimize(id, true))
                .unwrap_or_else(Task::none),
            Message::CloseWindow => {
                self.close_editor(true);
                self.window_id
                    .map(window::close)
                    .unwrap_or_else(Task::none)
            }

            Message::Navigate(screen) => self.navigate(screen),
            Message::Back => {
                if self.modal.is_some() {
                    self.modal = None;
                    Task::none()
                } else if self.screen == Screen::Editor {
                    self.close_editor(true);
                    Task::none()
                } else if !self.search.is_empty() {
                    self.search.clear();
                    Task::none()
                } else {
                    self.navigate(Screen::Home)
                }
            }
            Message::FocusSearch => self.focus_search(),
            Message::ClearSearch => {
                self.search.clear();
                Task::none()
            }
            Message::SearchChanged(value) => {
                self.search = value;
                Task::none()
            }
            Message::SetSort(sort) => {
                self.sort = sort;
                Task::none()
            }

            Message::NewNote => self.new_note(),
            Message::OpenNote(id) => self.open_note(id),
            Message::CloseEditor => {
                self.close_editor(true);
                Task::none()
            }
            Message::TitleChanged(title) => {
                let Some(editor) = &self.editor else {
                    return Task::none();
                };
                let id = editor.note_id;
                if let Some(note) = self.notes.iter_mut().find(|note| note.id == id) {
                    note.title = title;
                }
                self.mark_dirty();
                Task::none()
            }
            Message::ContentAction(action) => {
                if let Some(editor) = &mut self.editor {
                    editor.content.perform(action);
                }
                self.sync_editor_content();
                Task::none()
            }
            Message::SaveNow => {
                self.save_editor(true);
                Task::none()
            }
            Message::ToggleFocusMode => {
                if self.editor.is_some() {
                    self.focus_mode = !self.focus_mode;
                }
                Task::none()
            }
            Message::ExitFocus => {
                self.focus_mode = false;
                Task::none()
            }
            Message::TogglePreview => {
                self.toggle_preview();
                Task::none()
            }
            Message::ApplyFormat(format) => self.apply_format(format),

            Message::ToggleFavorite(id) => {
                if let Some(note) = self.notes.iter_mut().find(|note| note.id == id) {
                    note.favorite = !note.favorite;
                    let favorite = note.favorite;
                    self.persist_note(id);
                    if favorite {
                        self.show_toast("Added to favorites", ToastKind::Success);
                    }
                }
                Task::none()
            }
            Message::TogglePin(id) => {
                if let Some(note) = self.notes.iter_mut().find(|note| note.id == id) {
                    note.pinned = !note.pinned;
                }
                self.persist_note(id);
                Task::none()
            }
            Message::DuplicateNote(id) => self.duplicate_note(id),
            Message::ArchiveNote(id) => {
                self.archive_note(id);
                Task::none()
            }
            Message::UnarchiveNote(id) => {
                self.unarchive_note(id);
                Task::none()
            }
            Message::TrashNote(id) => {
                self.trash_note(id);
                Task::none()
            }
            Message::RestoreNote(id) => {
                self.restore_note(id);
                Task::none()
            }
            Message::RequestDeleteNote(id) => {
                self.modal = Some(Modal::confirm(ConfirmKind::DeleteNote(id)));
                Task::none()
            }
            Message::RequestEmptyTrash => {
                if self.trashed_count() == 0 {
                    self.show_toast("The trash is already empty", ToastKind::Info);
                } else {
                    self.modal = Some(Modal::confirm(ConfirmKind::EmptyTrash));
                }
                Task::none()
            }
            Message::EmptyTrash => {
                self.empty_trash();
                Task::none()
            }

            Message::ExportNote(id) => self.export_note(id),
            Message::Exported(result) => {
                match result {
                    Ok(path) => self.show_toast(
                        format!("Exported {}", file_name(&path)),
                        ToastKind::Success,
                    ),
                    Err(error) if error == "Export cancelled" => {}
                    Err(error) => self.show_toast(format!("Export failed: {error}"), ToastKind::Error),
                }
                Task::none()
            }
            Message::ExportAll => self.export_all(),
            Message::ExportAllFinished(result) => {
                match result {
                    Ok(path) => self.show_toast(
                        format!("Library exported to {}", file_name(&path)),
                        ToastKind::Success,
                    ),
                    Err(error) => {
                        self.show_toast(format!("Export failed: {error}"), ToastKind::Error)
                    }
                }
                Task::none()
            }

            Message::OpenFolderForm(id) => {
                let (name, color) = id
                    .and_then(|id| self.folder(id))
                    .map(|folder| {
                        let color = theme::SWATCHES
                            .iter()
                            .position(|swatch| *swatch == folder.color)
                            .unwrap_or(0);
                        (folder.name.clone(), color)
                    })
                    .unwrap_or((String::new(), 0));

                self.modal = Some(Modal::FolderForm { id, name, color });
                Task::none()
            }
            Message::OpenTagForm(id) => {
                let (name, color) = id
                    .and_then(|id| self.tag(id))
                    .map(|tag| {
                        let color = theme::SWATCHES
                            .iter()
                            .position(|swatch| *swatch == tag.color)
                            .unwrap_or(0);
                        (tag.name.clone(), color)
                    })
                    .unwrap_or((String::new(), 1));

                self.modal = Some(Modal::TagForm { id, name, color });
                Task::none()
            }
            Message::ModalNameChanged(value) => {
                match &mut self.modal {
                    Some(Modal::FolderForm { name, .. })
                    | Some(Modal::TagForm { name, .. }) => *name = value,
                    Some(Modal::TagPicker { query, .. }) => *query = value,
                    _ => {}
                }
                Task::none()
            }
            Message::ModalColorSelected(index) => {
                match &mut self.modal {
                    Some(Modal::FolderForm { color, .. })
                    | Some(Modal::TagForm { color, .. }) => *color = index,
                    _ => {}
                }
                Task::none()
            }
            Message::ModalConfirm => {
                self.confirm_modal();
                Task::none()
            }
            Message::ModalClose => {
                self.modal = None;
                Task::none()
            }
            Message::SetNoteFolder(folder_id) => {
                let Some(note_id) = self.editor.as_ref().map(|editor| editor.note_id) else {
                    return Task::none();
                };
                if let Some(note) = self.notes.iter_mut().find(|note| note.id == note_id) {
                    note.folder_id = folder_id;
                }
                self.persist_note(note_id);
                self.modal = None;
                self.show_toast(
                    match folder_id {
                        Some(_) => "Note moved to folder",
                        None => "Note removed from folder",
                    },
                    ToastKind::Success,
                );
                Task::none()
            }
            Message::RequestDeleteFolder(id) => {
                self.modal = Some(Modal::confirm(ConfirmKind::DeleteFolder(id)));
                Task::none()
            }
            Message::RequestDeleteTag(id) => {
                self.modal = Some(Modal::confirm(ConfirmKind::DeleteTag(id)));
                Task::none()
            }
            Message::OpenTagPicker => {
                if let Some(note_id) = self.editor.as_ref().map(|editor| editor.note_id) {
                    self.modal = Some(Modal::TagPicker {
                        note_id,
                        query: String::new(),
                    });
                }
                Task::none()
            }
            Message::TagPickerQueryChanged(value) => {
                if let Some(Modal::TagPicker { query, .. }) = &mut self.modal {
                    *query = value;
                }
                Task::none()
            }
            Message::ToggleTagOnEditor(tag_id) => {
                self.toggle_tag(tag_id);
                Task::none()
            }
            Message::CreateTagFromPicker => {
                self.create_tag_from_picker();
                Task::none()
            }
            Message::OpenFolderPicker => {
                if let Some(note_id) = self.editor.as_ref().map(|editor| editor.note_id) {
                    self.modal = Some(Modal::FolderPicker { note_id });
                }
                Task::none()
            }
            Message::OpenLink(url) => {
                let _ = std::process::Command::new("cmd")
                    .args(["/C", "start", "", &url])
                    .spawn();
                Task::none()
            }

            Message::SetUsername(value) => {
                self.settings.username = value;
                self.save_settings();
                Task::none()
            }
            Message::SetTheme(id) => {
                self.settings.theme = id;
                self.apply_theme();
                self.save_settings();
                Task::none()
            }
            Message::SetEditorFontSize(size) => {
                self.settings.editor_font_size = size.clamp(12.0, 22.0);
                self.save_settings();
                Task::none()
            }
            Message::SetAutosave(value) => {
                self.settings.autosave = value;
                self.save_settings();
                Task::none()
            }
            Message::SetAutosaveDelay(ms) => {
                self.settings.autosave_delay_ms = ms.clamp(300, 3000);
                self.save_settings();
                Task::none()
            }
            Message::SetBackupOnStart(value) => {
                self.settings.backup_on_start = value;
                self.save_settings();
                Task::none()
            }
            Message::CreateBackup => self.create_backup(),
            Message::BackupCreated(result) => {
                match result {
                    Ok(_) => {
                        self.settings.last_backup_at = Some(Local::now());
                        self.save_settings();
                        self.backups = backup::list_backups(&self.paths.backups_dir);
                        self.show_toast("Backup saved", ToastKind::Success);
                    }
                    Err(error) => {
                        self.show_toast(format!("Backup failed: {error}"), ToastKind::Error)
                    }
                }
                Task::none()
            }
            Message::RequestRestoreBackup(index) => {
                if index < self.backups.len() {
                    self.modal = Some(Modal::confirm(ConfirmKind::RestoreBackup(index)));
                }
                Task::none()
            }
            Message::RestoreBackup(path) => {
                self.restore_backup(path);
                Task::none()
            }
            Message::OpenDataFolder => {
                let _ = std::process::Command::new("explorer")
                    .arg(&self.paths.data_dir)
                    .spawn();
                Task::none()
            }
            Message::RefreshBackups => {
                self.backups = backup::list_backups(&self.paths.backups_dir);
                Task::none()
            }
            Message::RetryDatabase => {
                self.open_database();
                if self.db.is_some() {
                    self.show_toast("Connected to your library", ToastKind::Success);
                }
                Task::none()
            }

            // Drag and drop
            Message::CursorMoved(position) => {
                self.cursor = position;
                if let Some(drag) = &mut self.drag {
                    drag.position = position;
                }
                Task::none()
            }
            Message::DragNote(id) => {
                if self.drag.is_none() {
                    self.drag = Some(DragState {
                        note_id: id,
                        origin: self.cursor,
                        position: self.cursor,
                        target: None,
                    });
                }
                Task::none()
            }
            Message::DragMoved(position) => {
                if let Some(drag) = &mut self.drag {
                    drag.position = position;
                }
                Task::none()
            }
            Message::DropEnter(target) => {
                if let Some(drag) = &mut self.drag {
                    drag.target = Some(target);
                }
                Task::none()
            }
            Message::DropExit(target) => {
                if let Some(drag) = &mut self.drag {
                    if drag.target == Some(target) {
                        drag.target = None;
                    }
                }
                Task::none()
            }
            Message::DragReleased => self.finish_drag(),
            Message::FilesDropped(path) => self.import_dropped_file(path),

            Message::PokeMascot => {
                self.mascot_pokes += 1;
                const LINES: [&str; 8] = [
                    "You're doing great, really.",
                    "Small steps still count.",
                    "Mochi is proud of you!",
                    "Take a breath. Your notes will wait.",
                    "Ideas like yours deserve a home.",
                    "One page at a time.",
                    "Hi! I live in your notes now.",
                    "Every note starts with one word.",
                ];
                let special = self.mascot_pokes % 10 == 0;
                self.mascot_message = Some(if special {
                    format!("{} pokes! You're my favourite human.", self.mascot_pokes)
                } else {
                    LINES[(self.mascot_pokes - 1) % LINES.len()].to_string()
                });
                self.mascot_born = Instant::now();
                Task::none()
            }
            Message::HoverEnter(key) => {
                if self.hovered != Some(key) {
                    self.hovered = Some(key);
                    self.hover_t = 0.0;
                }
                Task::none()
            }
            Message::HoverExit(key) => {
                if self.hovered == Some(key) {
                    self.hovered = None;
                    self.hover_t = 0.0;
                }
                Task::none()
            }
            Message::DismissToast => {
                self.toast = None;
                Task::none()
            }
            Message::OpenShortcuts => {
                self.modal = Some(Modal::Shortcuts);
                Task::none()
            }
        }
    }

    // -----------------------------------------------------------------
    // Tick, animation and autosave
    // -----------------------------------------------------------------

    fn tick(&mut self, now: Instant) -> Task<Message> {
        let delta = now
            .saturating_duration_since(self.last_tick)
            .as_secs_f32()
            .min(0.25);
        self.last_tick = now;
        self.phase += delta;

        if self.hovered.is_some() {
            self.hover_t = (self.hover_t + delta * 8.0).min(1.0);
        }

        if let Some(toast) = &self.toast {
            if toast.born.elapsed() > Duration::from_millis(3200) {
                self.toast = None;
            }
        }

        if self.mascot_message.is_some()
            && self.mascot_born.elapsed() > Duration::from_millis(4200)
        {
            self.mascot_message = None;
        }

        if self.settings.autosave {
            let should_save = match (&self.editor, self.dirty_since) {
                (Some(editor), Some(since)) if editor.dirty => {
                    now.duration_since(since)
                        >= Duration::from_millis(self.settings.autosave_delay_ms)
                }
                _ => false,
            };

            if should_save {
                self.save_editor(false);
            }
        }

        Task::none()
    }

    // -----------------------------------------------------------------
    // Navigation and editor lifecycle
    // -----------------------------------------------------------------

    fn navigate(&mut self, screen: Screen) -> Task<Message> {
        if screen == Screen::Editor {
            return Task::none();
        }

        if self.screen == Screen::Editor {
            self.close_editor(true);
        }

        if let Screen::Folder(id) = screen {
            if self.folder(id).is_none() {
                self.screen = Screen::AllNotes;
                return Task::none();
            }
        }
        if let Screen::Tag(id) = screen {
            if self.tag(id).is_none() {
                self.screen = Screen::AllNotes;
                return Task::none();
            }
        }

        self.screen = screen;
        self.search.clear();
        self.hovered = None;
        self.hover_t = 0.0;
        self.modal = None;

        if screen == Screen::Settings {
            self.backups = backup::list_backups(&self.paths.backups_dir);
        }

        Task::none()
    }

    fn new_note(&mut self) -> Task<Message> {
        if self.screen == Screen::Editor {
            self.close_editor(true);
        }

        let folder_id = match self.screen {
            Screen::Folder(id) => Some(id),
            _ => None,
        };

        let mut note = Note::new(folder_id);
        if let Screen::Tag(id) = self.screen {
            note.tags.push(id);
        }

        if let Some(db) = &self.db {
            if let Err(error) = db.save_note(&note) {
                self.show_toast(format!("Could not create note: {error}"), ToastKind::Error);
            }
        }

        self.notes.insert(0, note.clone());
        self.open_note(note.id)
    }

    fn open_note(&mut self, id: Uuid) -> Task<Message> {
        if self.editor.as_ref().map(|editor| editor.note_id) == Some(id) {
            return Task::none();
        }

        if self.screen == Screen::Editor {
            self.close_editor(true);
        }

        let Some(note) = self.note(id).cloned() else {
            return Task::none();
        };

        self.editor_return = match self.screen {
            Screen::Editor => Screen::Home,
            screen => screen,
        };

        self.editor = Some(EditorState::new(&note));
        self.screen = Screen::Editor;
        self.search.clear();
        self.save_status = SaveStatus::Saved(note.updated_at);
        self.dirty_since = None;

        let target = if note.title.trim().is_empty() {
            TITLE_ID
        } else {
            EDITOR_ID
        };

        operation::focus(Id::new(target))
    }

    fn close_editor(&mut self, discard_blank: bool) {
        self.focus_mode = false;
        self.save_editor(false);
        self.sync_editor_content();

        if let Some(editor) = &self.editor {
            let id = editor.note_id;
            if discard_blank {
                if let Some(note) = self.note(id) {
                    if note.is_blank() && !note.favorite && !note.pinned {
                        if let Some(db) = &self.db {
                            let _ = db.delete_note(id);
                        }
                        self.notes.retain(|note| note.id != id);
                        self.show_toast("Empty note discarded", ToastKind::Info);
                    }
                }
            }
        }

        self.editor = None;
        self.dirty_since = None;
        self.save_status = SaveStatus::Idle;
        self.screen = match self.editor_return {
            Screen::Editor | Screen::Trash => Screen::AllNotes,
            screen => screen,
        };
    }

    fn focus_search(&mut self) -> Task<Message> {
        if self.screen == Screen::Editor {
            self.close_editor(true);
            self.screen = Screen::AllNotes;
        }
        operation::focus(Id::new(SEARCH_ID))
    }

    fn sync_editor_content(&mut self) {
        let Some(editor) = &mut self.editor else {
            return;
        };

        let content = editor.content.text();
        editor.words = text_utils::word_count(&content);
        editor.chars = content.chars().count();
        let id = editor.note_id;

        let mut changed = false;
        if let Some(note) = self.notes.iter_mut().find(|note| note.id == id) {
            if note.content != content {
                note.content = content;
                changed = true;
            }
        }

        if changed {
            if let Some(editor) = &mut self.editor {
                editor.dirty = true;
            }
            self.dirty_since = Some(Instant::now());
            self.save_status = SaveStatus::Editing;
        }
    }

    fn mark_dirty(&mut self) {
        if let Some(editor) = &mut self.editor {
            editor.dirty = true;
        }
        self.dirty_since = Some(Instant::now());
        self.save_status = SaveStatus::Editing;
    }

    fn save_editor(&mut self, announce: bool) {
        let Some(editor) = &self.editor else {
            return;
        };
        if !editor.dirty {
            return;
        }

        let id = editor.note_id;
        let content = editor.content.text();
        let now = Local::now();

        let note = {
            let Some(note) = self.notes.iter_mut().find(|note| note.id == id) else {
                return;
            };
            note.content = content;
            note.updated_at = now;
            note.clone()
        };

        match &self.db {
            Some(db) => match db.save_note(&note) {
                Ok(()) => {
                    if let Some(editor) = &mut self.editor {
                        editor.dirty = false;
                    }
                    self.dirty_since = None;
                    self.save_status = SaveStatus::Saved(now);
                    if announce {
                        self.show_toast("Note saved", ToastKind::Success);
                    }
                }
                Err(error) => {
                    let message = error.to_string();
                    self.save_status = SaveStatus::Error(message.clone());
                    self.show_toast(format!("Could not save: {message}"), ToastKind::Error);
                }
            },
            None => {
                self.save_status = SaveStatus::Error("Database unavailable".into());
            }
        }
    }

    fn persist_note(&mut self, id: Uuid) {
        let Some(note) = self.note(id).cloned() else {
            return;
        };

        if let Some(db) = &self.db {
            if let Err(error) = db.save_note(&note) {
                self.show_toast(format!("Could not save note: {error}"), ToastKind::Error);
            }
        }
    }

    fn save_settings(&mut self) {
        if let Some(db) = &self.db {
            let _ = db.save_settings(&self.settings);
        }
    }

    // -----------------------------------------------------------------
    // Drag and drop
    // -----------------------------------------------------------------

    fn finish_drag(&mut self) -> Task<Message> {
        let Some(drag) = self.drag.take() else {
            return Task::none();
        };

        let moved = drag.origin.distance(drag.position) > 6.0;

        match drag.target {
            Some(DropTarget::Folder(folder_id)) => {
                if let Some(note) = self.notes.iter_mut().find(|note| note.id == drag.note_id) {
                    note.folder_id = Some(folder_id);
                }
                self.persist_note(drag.note_id);
                self.show_toast("Moved to folder", ToastKind::Success);
                Task::none()
            }
            Some(DropTarget::Archive) => {
                self.archive_note(drag.note_id);
                Task::none()
            }
            Some(DropTarget::Trash) => {
                self.trash_note(drag.note_id);
                Task::none()
            }
            Some(DropTarget::Note(target)) => self.reorder_favorite(drag.note_id, target),
            None if !moved => self.open_note(drag.note_id),
            None => Task::none(),
        }
    }

    fn reorder_favorite(&mut self, source: Uuid, target: Uuid) -> Task<Message> {
        let source_index = self.notes.iter().position(|note| note.id == source);
        let target_index = self.notes.iter().position(|note| note.id == target);

        if let (Some(source_index), Some(target_index)) = (source_index, target_index) {
            if source_index != target_index {
                let order = self.notes[source_index].sort_order;
                self.notes[source_index].sort_order = self.notes[target_index].sort_order;
                self.notes[target_index].sort_order = order;
                self.persist_note(source);
                self.persist_note(target);
            }
        }

        Task::none()
    }

    fn import_dropped_file(&mut self, path: PathBuf) -> Task<Message> {
        let supported = path
            .extension()
            .and_then(|extension| extension.to_str())
            .map(|extension| {
                matches!(extension.to_ascii_lowercase().as_str(), "md" | "markdown" | "txt")
            })
            .unwrap_or(false);

        if !supported {
            self.show_toast("Only .md and .txt files can be imported", ToastKind::Error);
            return Task::none();
        }

        let Ok(content) = std::fs::read_to_string(&path) else {
            self.show_toast("Could not read that file", ToastKind::Error);
            return Task::none();
        };

        let title = path
            .file_stem()
            .map(|stem| stem.to_string_lossy().to_string())
            .unwrap_or_else(|| "Imported note".to_string());

        let mut note = Note::new(None);
        note.title = title;
        note.content = content;

        if let Some(db) = &self.db {
            if let Err(error) = db.save_note(&note) {
                self.show_toast(format!("Could not import: {error}"), ToastKind::Error);
                return Task::none();
            }
        }

        self.show_toast("File imported as a new note", ToastKind::Success);
        self.notes.push(note);
        Task::none()
    }

    // -----------------------------------------------------------------
    // Note operations
    // -----------------------------------------------------------------

    fn duplicate_note(&mut self, id: Uuid) -> Task<Message> {
        let Some(source) = self.note(id).cloned() else {
            return Task::none();
        };

        let mut copy = source.clone();
        copy.id = Uuid::new_v4();
        copy.title = format!("{} copy", source.display_title());
        copy.created_at = Local::now();
        copy.updated_at = Local::now();
        copy.favorite = false;
        copy.pinned = false;
        copy.archived = false;
        copy.trashed = false;
        copy.trashed_at = None;

        if let Some(db) = &self.db {
            if let Err(error) = db.save_note(&copy) {
                self.show_toast(format!("Could not duplicate: {error}"), ToastKind::Error);
                return Task::none();
            }
        }

        self.notes.insert(0, copy.clone());
        self.show_toast("Note duplicated", ToastKind::Success);
        self.open_note(copy.id)
    }

    fn archive_note(&mut self, id: Uuid) {
        self.save_editor(false);
        if let Some(note) = self.notes.iter_mut().find(|note| note.id == id) {
            note.archived = true;
            note.trashed = false;
            note.trashed_at = None;
        }
        self.persist_note(id);

        if self.editor.as_ref().map(|editor| editor.note_id) == Some(id) {
            self.editor = None;
            self.screen = Screen::AllNotes;
            self.save_status = SaveStatus::Idle;
        }

        self.show_toast("Note archived", ToastKind::Success);
    }

    fn unarchive_note(&mut self, id: Uuid) {
        if let Some(note) = self.notes.iter_mut().find(|note| note.id == id) {
            note.archived = false;
        }
        self.persist_note(id);
        self.show_toast("Note restored from archive", ToastKind::Success);
    }

    fn trash_note(&mut self, id: Uuid) {
        self.save_editor(false);
        let now = Local::now();
        if let Some(note) = self.notes.iter_mut().find(|note| note.id == id) {
            note.trashed = true;
            note.archived = false;
            note.trashed_at = Some(now);
        }
        self.persist_note(id);

        if self.editor.as_ref().map(|editor| editor.note_id) == Some(id) {
            self.editor = None;
            self.screen = Screen::AllNotes;
            self.save_status = SaveStatus::Idle;
        }

        self.show_toast("Moved to trash", ToastKind::Info);
    }

    fn restore_note(&mut self, id: Uuid) {
        if let Some(note) = self.notes.iter_mut().find(|note| note.id == id) {
            note.trashed = false;
            note.trashed_at = None;
        }
        self.persist_note(id);
        self.show_toast("Note restored", ToastKind::Success);
    }

    fn delete_note_forever(&mut self, id: Uuid) {
        if let Some(db) = &self.db {
            if let Err(error) = db.delete_note(id) {
                self.show_toast(format!("Could not delete: {error}"), ToastKind::Error);
                return;
            }
        }

        self.notes.retain(|note| note.id != id);
        if self.editor.as_ref().map(|editor| editor.note_id) == Some(id) {
            self.editor = None;
            self.screen = Screen::Trash;
            self.save_status = SaveStatus::Idle;
        }
        self.show_toast("Note deleted forever", ToastKind::Info);
    }

    fn empty_trash(&mut self) {
        if let Some(db) = &self.db {
            if let Err(error) = db.delete_all_trashed() {
                self.show_toast(format!("Could not empty trash: {error}"), ToastKind::Error);
                return;
            }
        }

        let removed = self.trashed_count();
        self.notes.retain(|note| !note.trashed);
        self.show_toast(
            format!(
                "{} permanently deleted",
                text_utils::plural(removed, "note", "notes")
            ),
            ToastKind::Info,
        );
    }

    fn toggle_tag(&mut self, tag_id: Uuid) {
        let Some(note_id) = self.editor.as_ref().map(|editor| editor.note_id) else {
            return;
        };

        if let Some(note) = self.notes.iter_mut().find(|note| note.id == note_id) {
            if let Some(index) = note.tags.iter().position(|id| *id == tag_id) {
                note.tags.remove(index);
            } else {
                note.tags.push(tag_id);
            }
        }

        self.persist_note(note_id);
    }

    fn create_tag_from_picker(&mut self) {
        let Some((note_id, query)) = (match &self.modal {
            Some(Modal::TagPicker { note_id, query }) => Some((*note_id, query.trim().to_string())),
            _ => None,
        }) else {
            return;
        };

        if query.is_empty() {
            self.show_toast("Type a name for the new tag", ToastKind::Info);
            return;
        }

        let existing = self
            .tags
            .iter()
            .find(|tag| tag.name.eq_ignore_ascii_case(&query))
            .cloned();

        let tag = if let Some(tag) = existing {
            tag
        } else {
            let color = theme::SWATCHES[(query.len() + query.chars().next().unwrap_or('a') as usize)
                % theme::SWATCHES.len()]
            .to_string();
            let tag = Tag::new(query.clone(), color);
            if let Some(db) = &self.db {
                if let Err(error) = db.save_tag(&tag) {
                    self.show_toast(format!("Could not create tag: {error}"), ToastKind::Error);
                    return;
                }
            }
            self.tags.push(tag.clone());
            self.tags.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
            tag
        };

        if let Some(note) = self.notes.iter_mut().find(|note| note.id == note_id) {
            if !note.tags.contains(&tag.id) {
                note.tags.push(tag.id);
            }
        }
        self.persist_note(note_id);

        if let Some(Modal::TagPicker { query, .. }) = &mut self.modal {
            query.clear();
        }
        self.show_toast(format!("Tagged #{}", tag.name), ToastKind::Success);
    }

    // -----------------------------------------------------------------
    // Editor helpers
    // -----------------------------------------------------------------

    fn toggle_preview(&mut self) {
        let Some(note_id) = self.editor.as_ref().map(|editor| editor.note_id) else {
            return;
        };

        let preview_open = self
            .editor
            .as_ref()
            .map(|editor| editor.preview.is_some())
            .unwrap_or(false);

        if preview_open {
            if let Some(editor) = &mut self.editor {
                editor.preview = None;
            }
        } else {
            let content = self
                .note(note_id)
                .map(|note| note.content.clone())
                .unwrap_or_default();

            if let Some(editor) = &mut self.editor {
                editor.preview = Some(markdown::Content::parse(&content));
            }
        }
    }

    fn apply_format(&mut self, format: Format) -> Task<Message> {
        let Some(editor) = &mut self.editor else {
            return Task::none();
        };

        let selection = editor.content.selection();
        let (text, move_back) = match format {
            Format::Bold => wrap("**", selection),
            Format::Italic => wrap("*", selection),
            Format::Strike => wrap("~~", selection),
            Format::InlineCode => wrap("`", selection),
            Format::Heading1 => prefix("# ", selection),
            Format::Heading2 => prefix("## ", selection),
            Format::Quote => prefix("> ", selection),
            Format::Bullet => prefix("- ", selection),
            Format::Numbered => prefix("1. ", selection),
            Format::Checkbox => prefix("- [ ] ", selection),
            Format::Link => match selection {
                Some(selected) if !selected.is_empty() => {
                    (format!("[{selected}](https://)"), 0)
                }
                _ => ("[link](https://)".to_string(), 0),
            },
        };

        editor
            .content
            .perform(text_editor::Action::Edit(text_editor::Edit::Paste(
                Arc::new(text),
            )));

        for _ in 0..move_back {
            editor
                .content
                .perform(text_editor::Action::Move(text_editor::Motion::Left));
        }

        self.sync_editor_content();
        operation::focus(Id::new(EDITOR_ID))
    }

    // -----------------------------------------------------------------
    // Keyboard shortcuts
    // -----------------------------------------------------------------

    fn window_toggle_maximize(&self) -> Task<Message> {
        let Some(id) = self.window_id else {
            return Task::none();
        };

        Task::batch([
            window::toggle_maximize(id),
            window::is_maximized(id).map(Message::WindowMaximized),
        ])
    }

    fn keyboard(&mut self, event: keyboard::Event) -> Task<Message> {
        let keyboard::Event::KeyPressed {
            key,
            modifiers,
            ..
        } = event
        else {
            return Task::none();
        };

        let control = modifiers.control();
        let shift = modifiers.shift();

        match &key {
            keyboard::Key::Named(keyboard::key::Named::Escape) => {
                if self.modal.is_some() {
                    self.modal = None;
                } else if self.focus_mode {
                    self.focus_mode = false;
                } else if self.screen == Screen::Editor {
                    self.close_editor(true);
                } else if !self.search.is_empty() {
                    self.search.clear();
                }
                Task::none()
            }
            keyboard::Key::Named(keyboard::key::Named::F1) => {
                self.modal = Some(Modal::Shortcuts);
                Task::none()
            }
            keyboard::Key::Character(character) if control => {
                let key = character.to_lowercase();
                match key.as_str() {
                    "n" => self.new_note(),
                    "s" => {
                        self.save_editor(true);
                        Task::none()
                    }
                    "f" if shift => {
                        if self.editor.is_some() {
                            self.focus_mode = !self.focus_mode;
                        }
                        Task::none()
                    }
                    "f" => self.focus_search(),
                    "e" => match self.editor.as_ref().map(|editor| editor.note_id) {
                        Some(id) => self.export_note(id),
                        None => Task::none(),
                    },
                    "d" => match self.editor.as_ref().map(|editor| editor.note_id) {
                        Some(id) => self.duplicate_note(id),
                        None => Task::none(),
                    },
                    "b" if self.editor.is_some() => self.apply_format(Format::Bold),
                    "i" if self.editor.is_some() => self.apply_format(Format::Italic),
                    "p" => {
                        if let Some(id) = self.editor.as_ref().map(|editor| editor.note_id) {
                            if let Some(note) = self.notes.iter_mut().find(|note| note.id == id) {
                                note.pinned = !note.pinned;
                            }
                            self.persist_note(id);
                        }
                        Task::none()
                    }
                    "h" => self.navigate(Screen::Home),
                    "1" => self.navigate(Screen::AllNotes),
                    "2" => self.navigate(Screen::Favorites),
                    "3" => self.navigate(Screen::Archive),
                    "4" => self.navigate(Screen::Trash),
                    "," => self.navigate(Screen::Settings),
                    "/" => {
                        self.modal = Some(Modal::Shortcuts);
                        Task::none()
                    }
                    _ => Task::none(),
                }
            }
            _ => Task::none(),
        }
    }

    // -----------------------------------------------------------------
    // Modals
    // -----------------------------------------------------------------

    fn confirm_modal(&mut self) {
        let Some(modal) = self.modal.take() else {
            return;
        };

        match modal {
            Modal::FolderForm { id, name, color } => {
                let name = name.trim().to_string();
                if name.is_empty() {
                    self.show_toast("Folders need a name", ToastKind::Error);
                    self.modal = Some(Modal::FolderForm { id, name, color });
                    return;
                }

                let color = theme::SWATCHES[color % theme::SWATCHES.len()].to_string();
                let folder = if let Some(id) = id {
                    match self.folders.iter_mut().find(|folder| folder.id == id) {
                        Some(folder) => {
                            folder.name = name.clone();
                            folder.color = color.clone();
                            folder.clone()
                        }
                        None => Folder::new(name, color),
                    }
                } else {
                    Folder::new(name, color)
                };

                if let Some(db) = &self.db {
                    if let Err(error) = db.save_folder(&folder) {
                        self.show_toast(format!("Could not save folder: {error}"), ToastKind::Error);
                        return;
                    }
                }

                if let Some(existing) = self.folders.iter_mut().find(|f| f.id == folder.id) {
                    *existing = folder.clone();
                } else {
                    self.folders.push(folder.clone());
                }
                self.folders
                    .sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

                self.show_toast(
                    if id.is_some() {
                        "Folder updated"
                    } else {
                        "Folder created"
                    },
                    ToastKind::Success,
                );
            }
            Modal::TagForm { id, name, color } => {
                let name = name.trim().to_string();
                if name.is_empty() {
                    self.show_toast("Tags need a name", ToastKind::Error);
                    self.modal = Some(Modal::TagForm { id, name, color });
                    return;
                }

                let duplicate = self.tags.iter().any(|tag| {
                    tag.id != id.unwrap_or(Uuid::nil()) && tag.name.eq_ignore_ascii_case(&name)
                });
                if duplicate {
                    self.show_toast("A tag with that name already exists", ToastKind::Error);
                    self.modal = Some(Modal::TagForm { id, name, color });
                    return;
                }

                let color = theme::SWATCHES[color % theme::SWATCHES.len()].to_string();
                let tag = if let Some(id) = id {
                    match self.tags.iter_mut().find(|tag| tag.id == id) {
                        Some(tag) => {
                            tag.name = name.clone();
                            tag.color = color.clone();
                            tag.clone()
                        }
                        None => Tag::new(name, color),
                    }
                } else {
                    Tag::new(name, color)
                };

                if let Some(db) = &self.db {
                    if let Err(error) = db.save_tag(&tag) {
                        self.show_toast(format!("Could not save tag: {error}"), ToastKind::Error);
                        return;
                    }
                }

                if let Some(existing) = self.tags.iter_mut().find(|t| t.id == tag.id) {
                    *existing = tag.clone();
                } else {
                    self.tags.push(tag.clone());
                }
                self.tags
                    .sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

                self.show_toast(
                    if id.is_some() {
                        "Tag updated"
                    } else {
                        "Tag created"
                    },
                    ToastKind::Success,
                );
            }
            Modal::Confirm { kind, .. } => match kind {
                ConfirmKind::TrashNote(id) => self.trash_note(id),
                ConfirmKind::DeleteNote(id) => self.delete_note_forever(id),
                ConfirmKind::DeleteFolder(id) => self.delete_folder(id),
                ConfirmKind::DeleteTag(id) => self.delete_tag(id),
                ConfirmKind::EmptyTrash => self.empty_trash(),
                ConfirmKind::RestoreBackup(index) => {
                    if let Some(backup) = self.backups.get(index).cloned() {
                        self.restore_backup(backup.path);
                    }
                }
            },
            Modal::TagPicker { .. } | Modal::FolderPicker { .. } | Modal::Shortcuts => {}
        }
    }

    fn delete_folder(&mut self, id: Uuid) {
        if let Some(db) = &self.db {
            if let Err(error) = db.delete_folder(id) {
                self.show_toast(format!("Could not delete folder: {error}"), ToastKind::Error);
                return;
            }
        }

        self.folders.retain(|folder| folder.id != id);
        for note in &mut self.notes {
            if note.folder_id == Some(id) {
                note.folder_id = None;
            }
        }

        if self.screen == Screen::Folder(id) {
            self.screen = Screen::AllNotes;
        }

        self.show_toast("Folder deleted", ToastKind::Info);
    }

    fn delete_tag(&mut self, id: Uuid) {
        if let Some(db) = &self.db {
            if let Err(error) = db.delete_tag(id) {
                self.show_toast(format!("Could not delete tag: {error}"), ToastKind::Error);
                return;
            }
        }

        self.tags.retain(|tag| tag.id != id);
        for note in &mut self.notes {
            note.tags.retain(|tag| *tag != id);
        }

        if self.screen == Screen::Tag(id) {
            self.screen = Screen::AllNotes;
        }

        self.show_toast("Tag deleted", ToastKind::Info);
    }

    // -----------------------------------------------------------------
    // Export and backups
    // -----------------------------------------------------------------

    fn export_note(&mut self, id: Uuid) -> Task<Message> {
        let Some(note) = self.note(id).cloned() else {
            return Task::none();
        };
        let folder = note.folder_id.and_then(|id| self.folder(id)).cloned();
        let tags = self.tags.clone();
        let exports_dir = self.paths.exports_dir.clone();

        Task::perform(
            async move { export::save_note_dialog(&note, folder.as_ref(), &tags, &exports_dir) },
            Message::Exported,
        )
    }

    fn export_all(&mut self) -> Task<Message> {
        let notes: Vec<Note> = self
            .notes
            .iter()
            .filter(|note| !note.trashed)
            .cloned()
            .collect();
        let folders = self.folders.clone();
        let tags = self.tags.clone();
        let exports_dir = self.paths.exports_dir.clone();

        Task::perform(
            async move { export::export_all(&notes, &folders, &tags, &exports_dir) },
            Message::ExportAllFinished,
        )
    }

    fn create_backup(&mut self) -> Task<Message> {
        let db_path = self.paths.db_path.clone();
        let backups_dir = self.paths.backups_dir.clone();

        Task::perform(
            async move { backup::create_backup(&db_path, &backups_dir, 20) },
            Message::BackupCreated,
        )
    }

    fn restore_backup(&mut self, path: PathBuf) {
        self.save_editor(false);

        // Keep a safety copy of the current library before replacing it.
        let _ = backup::create_backup(&self.paths.db_path, &self.paths.backups_dir, 20);

        self.db = None;
        self.db_error = None;

        match std::fs::copy(&path, &self.paths.db_path) {
            Ok(_) => match Database::open(&self.paths.db_path) {
                Ok(db) => {
                    self.db = Some(db);
                    self.editor = None;
                    self.screen = Screen::Home;
                    self.save_status = SaveStatus::Idle;
                    self.reload();
                    self.show_toast("Backup restored", ToastKind::Success);
                }
                Err(error) => {
                    self.db_error = Some(error.to_string());
                    self.show_toast("Backup could not be opened", ToastKind::Error);
                }
            },
            Err(error) => {
                self.db_error = Some(error.to_string());
                if let Ok(db) = Database::open(&self.paths.db_path) {
                    self.db = Some(db);
                    self.reload();
                }
                self.show_toast("Backup could not be restored", ToastKind::Error);
            }
        }
    }

    // -----------------------------------------------------------------
    // Feedback
    // -----------------------------------------------------------------

    pub fn show_toast(&mut self, text: impl Into<String>, kind: ToastKind) {
        self.toast = Some(Toast {
            text: text.into(),
            kind,
            born: Instant::now(),
        });
    }

    pub fn nav_selected(&self, nav: NavId) -> bool {
        match (nav, self.screen) {
            (NavId::Home, Screen::Home) => true,
            (NavId::AllNotes, Screen::AllNotes) => true,
            (NavId::Favorites, Screen::Favorites) => true,
            (NavId::Folders, Screen::Folders) | (NavId::Folders, Screen::Folder(_)) => true,
            (NavId::Tags, Screen::Tags) | (NavId::Tags, Screen::Tag(_)) => true,
            (NavId::Archive, Screen::Archive) => true,
            (NavId::Trash, Screen::Trash) => true,
            (NavId::Settings, Screen::Settings) => true,
            _ => false,
        }
    }
}

fn wrap(marker: &str, selection: Option<String>) -> (String, usize) {
    match selection {
        Some(selected) if !selected.is_empty() => {
            (format!("{marker}{selected}{marker}"), 0)
        }
        _ => (
            format!("{marker}{marker}"),
            marker.chars().count(),
        ),
    }
}

fn prefix(marker: &str, selection: Option<String>) -> (String, usize) {
    match selection {
        Some(selected) if !selected.is_empty() => (format!("{marker}{selected}"), 0),
        _ => (marker.to_string(), 0),
    }
}

fn file_name(path: &std::path::Path) -> String {
    path.file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("export")
        .to_string()
}
