//! Note collections: all notes, favorites, folders, tags, archive and trash.

use iced::widget::{column, container, grid, row, text, Space};
use iced::{Alignment, Element, Fill, Length};

use crate::app::Mochi;
use crate::message::{Message, Screen, SortMode};
use crate::models::Note;
use crate::theme::{self, style};
use crate::views::{page, page_spacer};
use crate::widgets::components::{
    danger_button, empty_state, pill, primary_button, search_bar, soft_button,
};
use crate::widgets::icons::Icon;
use crate::widgets::note_card::{note_card, CardMode};

pub fn view(app: &Mochi) -> Element<'_, Message> {
    let p = app.palette;

    let bubble = app.mascot_message.as_deref().map(|message| {
        let elapsed = app.mascot_born.elapsed().as_secs_f32();
        (message, (1.0 - elapsed / 4.2).clamp(0.0, 1.0))
    });

    let (title, subtitle, mode, accent) = match app.screen {
        Screen::Favorites => (
            "Favorites".to_string(),
            "Every note you have marked with a little star.".to_string(),
            CardMode::Active,
            p.pastel[2],
        ),
        Screen::Folder(id) => {
            let folder = app.folder(id);
            (
                folder.map(|f| f.name.clone()).unwrap_or_else(|| "Folder".into()),
                "Everything you have gathered in this folder.".to_string(),
                CardMode::Active,
                folder
                    .map(|f| theme::parse_hex(&f.color))
                    .unwrap_or(p.primary_soft),
            )
        }
        Screen::Tag(id) => {
            let tag = app.tag(id);
            (
                tag.map(|t| format!("#{}", t.name))
                    .unwrap_or_else(|| "Tag".into()),
                "Notes sharing this little label.".to_string(),
                CardMode::Active,
                tag.map(|t| theme::parse_hex(&t.color))
                    .unwrap_or(p.primary_soft),
            )
        }
        Screen::Archive => (
            "Archive".to_string(),
            "Out of the way, but never gone.".to_string(),
            CardMode::Archive,
            p.pastel[3],
        ),
        Screen::Trash => (
            "Trash".to_string(),
            "Notes resting here can be restored or deleted forever.".to_string(),
            CardMode::Trash,
            p.danger_soft,
        ),
        _ => (
            "All notes".to_string(),
            "Your whole collection, in one calm place.".to_string(),
            CardMode::Active,
            p.pastel[1],
        ),
    };

    let mut notes: Vec<&Note> = app
        .notes
        .iter()
        .filter(|note| match app.screen {
            Screen::AllNotes => !note.trashed && !note.archived,
            Screen::Favorites => note.favorite && !note.trashed && !note.archived,
            Screen::Folder(id) => note.folder_id == Some(id) && !note.trashed && !note.archived,
            Screen::Tag(id) => note.tags.contains(&id) && !note.trashed && !note.archived,
            Screen::Archive => note.archived && !note.trashed,
            Screen::Trash => note.trashed,
            _ => !note.trashed && !note.archived,
        })
        .filter(|note| note.matches(&app.search))
        .collect();

    sort_notes(&mut notes, app.sort, mode);
    if app.screen == Screen::Favorites {
        notes.sort_by_key(|note| note.sort_order);
    }

    let searching = !app.search.trim().is_empty();

    let mut actions = row![].spacing(9).align_y(Alignment::Center);
    if mode == CardMode::Trash {
        if app.trashed_count() > 0 {
            actions = actions.push(danger_button(
                "Empty trash",
                Some(Icon::TrashCan),
                Message::RequestEmptyTrash,
                p,
            ));
        }
    } else {
        actions = actions.push(soft_button(
            "Export all",
            Some(Icon::Download),
            Message::ExportAll,
            p,
        ));
        actions = actions.push(primary_button(
            "New note",
            Some(Icon::Plus),
            Message::NewNote,
            p,
        ));
    }

    let header = row![
        column![
            row![
                text(title)
                    .size(theme::size::H1)
                    .font(theme::FONT_BOLD)
                    .color(p.text),
                container(
                    text(notes.len().to_string())
                        .size(11.0)
                        .font(theme::FONT_BOLD)
                        .color(theme::mix(p.text, accent, 0.5)),
                )
                .padding(iced::Padding::from([2.0, 9.0]))
                .style(move |_theme| style::badge(p, accent)),
            ]
            .spacing(10)
            .align_y(Alignment::Center),
            text(subtitle).size(12.5).color(p.text_muted),
        ]
        .spacing(4),
        Space::new().width(Fill),
        actions,
    ]
    .align_y(Alignment::Center)
    .width(Fill);

    let mut toolbar = row![
        search_bar(&app.search, 320.0, p),
        Space::new().width(Fill),
        sort_pills(app),
    ]
    .spacing(12)
    .align_y(Alignment::Center);

    if searching {
        toolbar = toolbar.push(
            iced::widget::button(
                row![
                    crate::widgets::icons::icon(Icon::Close, 10.0, p.text_muted),
                    text("Clear").size(11.0).color(p.text_muted),
                ]
                .spacing(5)
                .align_y(Alignment::Center),
            )
            .padding(iced::Padding::from([5.0, 9.0]))
            .style(move |_theme, status| style::button_ghost(p, status))
            .on_press(Message::ClearSearch),
        );
    }

    let mut content = column![
        header,
        page_spacer(18.0),
        container(toolbar)
            .padding(iced::Padding::from([10.0, 12.0]))
            .width(Fill)
            .style(move |_theme| style::toolbar(p)),
        page_spacer(18.0),
    ]
    .spacing(0)
    .width(Fill);

    if notes.is_empty() {
        let (empty_title, empty_body) = if searching {
            (
                "No notes match that search",
                "Try a different word, or clear the search to see everything again.",
            )
        } else {
            match app.screen {
                Screen::Favorites => (
                    "No favorites yet",
                    "Tap the little star on a note card to keep it close.",
                ),
                Screen::Archive => (
                    "Nothing archived",
                    "When a note is finished but worth keeping, archive it here.",
                ),
                Screen::Trash => (
                    "The trash is empty",
                    "Everything is tidy. Deleted notes will appear here first.",
                ),
                Screen::Folder(_) => (
                    "This folder is empty",
                    "Create a note while browsing this folder and it will land right here.",
                ),
                Screen::Tag(_) => (
                    "No notes with this tag",
                    "Add this tag to a note from the editor to see it here.",
                ),
                _ => (
                    "No notes yet",
                    "Your first note is only a click away. Mochi is ready when you are.",
                ),
            }
        };

        content = content.push(
            container(empty_state(
                app.phase,
                empty_title,
                empty_body,
                if searching {
                    Some((Icon::Close, "Clear search", Message::ClearSearch))
                } else if mode == CardMode::Trash {
                    None
                } else {
                    Some((Icon::Plus, "New note", Message::NewNote))
                },
                bubble,
                p,
            ))
            .height(380.0)
            .width(Fill),
        );
    } else {
        let mut cards = grid::Grid::new()
            .fluid(288.0)
            .spacing(14)
            .height(grid::Sizing::EvenlyDistribute(Length::Shrink));

        for note in &notes {
            let folder = note.folder_id.and_then(|id| app.folder(id));
            let hover = app.hover_progress(crate::message::Hover::Note(note.id));
            cards = cards.push(note_card(
                note,
                &app.tags,
                folder,
                p,
                hover,
                mode,
                if app.screen == Screen::Favorites {
                    Some(crate::message::DropTarget::Note(note.id))
                } else {
                    None
                },
            ));
        }

        content = content.push(cards);
    }

    content = content.push(page_spacer(16.0));

    page(content, p)
}

fn sort_pills(app: &Mochi) -> Element<'_, Message> {
    let p = app.palette;
    let mut row = row![
        text("Sort")
            .size(10.5)
            .font(theme::FONT_BOLD)
            .color(p.text_faint)
    ]
    .spacing(5)
    .align_y(Alignment::Center);

    for mode in SortMode::ALL {
        row = row.push(pill(
            mode.label(),
            app.sort == mode,
            Message::SetSort(mode),
            p,
        ));
    }

    row.into()
}

fn sort_notes(notes: &mut [&Note], sort: SortMode, mode: CardMode) {
    notes.sort_by(|a, b| {
        if mode == CardMode::Active {
            let pinned = b.pinned.cmp(&a.pinned);
            if pinned != std::cmp::Ordering::Equal {
                return pinned;
            }
        }

        match sort {
            SortMode::Updated => b.updated_at.cmp(&a.updated_at),
            SortMode::Created => b.created_at.cmp(&a.created_at),
            SortMode::Title => a
                .display_title()
                .to_lowercase()
                .cmp(&b.display_title().to_lowercase()),
        }
    });
}

/// A small header used by folder and tag management screens.
pub fn management_header<'a>(
    title: &str,
    subtitle: &str,
    action: Element<'a, Message>,
    p: theme::Palette,
) -> Element<'a, Message> {
    row![
        column![
            text(title.to_string())
                .size(theme::size::H1)
                .font(theme::FONT_BOLD)
                .color(p.text),
            text(subtitle.to_string()).size(12.5).color(p.text_muted),
        ]
        .spacing(4),
        Space::new().width(Fill),
        action,
    ]
    .align_y(Alignment::Center)
    .width(Fill)
    .into()
}
