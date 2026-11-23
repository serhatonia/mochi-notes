//! Every interaction in Mochi Notes is expressed as a [`Message`].

use std::path::PathBuf;

use iced::time::Instant;
use iced::window;
use iced::{keyboard, widget::text_editor};
use uuid::Uuid;

use crate::theme::ThemeId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Home,
    AllNotes,
    Favorites,
    Folder(Uuid),
    Folders,
    Tags,
    Tag(Uuid),
    Archive,
    Trash,
    Settings,
    Editor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NavId {
    Home,
    AllNotes,
    Favorites,
    Folders,
    Tags,
    Archive,
    Trash,
    Settings,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Hover {
    Nav(NavId),
    Note(Uuid),
    Folder(Uuid),
    Tag(Uuid),
    Theme(ThemeId),
    Generic(u8),
}

/// A place where a dragged note can be dropped.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DropTarget {
    Folder(Uuid),
    Archive,
    Trash,
    Note(Uuid),
}

/// Live state of a note being dragged with the mouse.
#[derive(Debug, Clone, Copy)]
pub struct DragState {
    pub note_id: Uuid,
    pub origin: iced::Point,
    pub position: iced::Point,
    pub target: Option<DropTarget>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortMode {
    Updated,
    Created,
    Title,
}

impl SortMode {
    pub const ALL: [SortMode; 3] = [SortMode::Updated, SortMode::Created, SortMode::Title];

    pub fn label(self) -> &'static str {
        match self {
            SortMode::Updated => "Last edited",
            SortMode::Created => "Created",
            SortMode::Title => "Title",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    Bold,
    Italic,
    Strike,
    InlineCode,
    Heading1,
    Heading2,
    Quote,
    Bullet,
    Numbered,
    Link,
    Checkbox,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToastKind {
    Success,
    Info,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfirmKind {
    TrashNote(Uuid),
    DeleteNote(Uuid),
    DeleteFolder(Uuid),
    DeleteTag(Uuid),
    EmptyTrash,
    RestoreBackup(usize),
}

#[derive(Debug, Clone)]
pub enum Modal {
    FolderForm {
        id: Option<Uuid>,
        name: String,
        color: usize,
    },
    TagForm {
        id: Option<Uuid>,
        name: String,
        color: usize,
    },
    TagPicker {
        note_id: Uuid,
        query: String,
    },
    FolderPicker {
        note_id: Uuid,
    },
    Confirm {
        kind: ConfirmKind,
        title: String,
        body: String,
        confirm_label: String,
        danger: bool,
    },
    Shortcuts,
}

impl Modal {
    pub fn confirm(kind: ConfirmKind) -> Self {
        let (title, body, confirm_label, danger) = match &kind {
            ConfirmKind::TrashNote(_) => (
                "Move this note to the trash?",
                "You can restore it later from the trash. Nothing is lost.",
                "Move to trash",
                false,
            ),
            ConfirmKind::DeleteNote(_) => (
                "Delete this note forever?",
                "This action cannot be undone. The note and its tags will be gone for good.",
                "Delete forever",
                true,
            ),
            ConfirmKind::DeleteFolder(_) => (
                "Delete this folder?",
                "The folder will disappear, but the notes inside it will be kept.",
                "Delete folder",
                true,
            ),
            ConfirmKind::DeleteTag(_) => (
                "Delete this tag?",
                "The tag will be removed from every note that uses it.",
                "Delete tag",
                true,
            ),
            ConfirmKind::EmptyTrash => (
                "Empty the trash?",
                "Every note in the trash will be permanently deleted. This cannot be undone.",
                "Empty trash",
                true,
            ),
            ConfirmKind::RestoreBackup(_) => (
                "Restore this backup?",
                "Your current library will be replaced by the contents of this backup. A safety copy is made first.",
                "Restore backup",
                true,
            ),
        };

        Modal::Confirm {
            kind,
            title: title.into(),
            body: body.into(),
            confirm_label: confirm_label.into(),
            danger,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    /// Animation and autosave heartbeat.
    Tick(Instant),
    Keyboard(keyboard::Event),
    CloseRequested(window::Id),
    Noop,

    // Window chrome (custom, frameless title bar)
    WindowReady(Option<window::Id>),
    WindowMaximized(bool),
    TitleBarDrag,
    TitleBarDoubleClick,
    WindowSystemMenu,
    MinimizeWindow,
    ToggleMaximizeWindow,
    CloseWindow,

    // Navigation
    Navigate(Screen),
    Back,
    FocusSearch,
    ClearSearch,
    SearchChanged(String),
    SetSort(SortMode),

    // Notes
    NewNote,
    OpenNote(Uuid),
    CloseEditor,
    TitleChanged(String),
    ContentAction(text_editor::Action),
    SaveNow,
    ToggleFocusMode,
    ExitFocus,
    TogglePreview,
    ApplyFormat(Format),
    ToggleFavorite(Uuid),
    TogglePin(Uuid),
    DuplicateNote(Uuid),
    ArchiveNote(Uuid),
    UnarchiveNote(Uuid),
    TrashNote(Uuid),
    RestoreNote(Uuid),
    RequestDeleteNote(Uuid),
    RequestEmptyTrash,
    EmptyTrash,
    ExportNote(Uuid),
    Exported(Result<PathBuf, String>),
    ExportAll,
    ExportAllFinished(Result<PathBuf, String>),

    // Folders and tags
    OpenFolderForm(Option<Uuid>),
    OpenTagForm(Option<Uuid>),
    ModalNameChanged(String),
    ModalColorSelected(usize),
    ModalConfirm,
    ModalClose,
    SetNoteFolder(Option<Uuid>),
    RequestDeleteFolder(Uuid),
    RequestDeleteTag(Uuid),
    OpenTagPicker,
    TagPickerQueryChanged(String),
    ToggleTagOnEditor(Uuid),
    CreateTagFromPicker,
    OpenFolderPicker,
    OpenLink(String),

    // Settings and data
    SetUsername(String),
    SetTheme(ThemeId),
    SetEditorFontSize(f32),
    SetAutosave(bool),
    SetAutosaveDelay(u64),
    SetBackupOnStart(bool),
    CreateBackup,
    BackupCreated(Result<PathBuf, String>),
    RequestRestoreBackup(usize),
    RestoreBackup(PathBuf),
    OpenDataFolder,
    RefreshBackups,
    RetryDatabase,

    // Drag and drop
    CursorMoved(iced::Point),
    DragNote(Uuid),
    DragMoved(iced::Point),
    DragReleased,
    DropEnter(DropTarget),
    DropExit(DropTarget),
    FilesDropped(PathBuf),

    // UI feedback
    PokeMascot,
    HoverEnter(Hover),
    HoverExit(Hover),
    DismissToast,
    OpenShortcuts,
}
