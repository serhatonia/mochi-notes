//! The cozy home dashboard: greeting, quick stats, pinned and recent notes.

use iced::widget::{column, container, grid, row, text, Space};
use iced::{Alignment, Element, Fill, Length};

use crate::app::Mochi;
use crate::message::{Message, Screen};
use crate::models::Note;
use crate::theme::{self, style};
use crate::utils::text as text_utils;
use crate::views::{page, page_spacer};
use crate::widgets::components::{
    empty_state, icon_button, primary_button, search_bar, stat,
};
use crate::widgets::icons::Icon;
use crate::widgets::mascot;
use crate::widgets::note_card::{note_card, CardMode};

pub fn view(app: &Mochi) -> Element<'_, Message> {
    let p = app.palette;

    let mut active: Vec<&Note> = app.active_notes().collect();
    active.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));

    let recent: Vec<&Note> = active.iter().copied().take(6).collect();
    let pinned: Vec<&Note> = active.iter().copied().filter(|note| note.pinned).take(3).collect();
    let mut favorites: Vec<&Note> = active.iter().copied().filter(|note| note.favorite).collect();
    favorites.sort_by_key(|note| note.sort_order);
    let favorites: Vec<&Note> = favorites.into_iter().take(3).collect();

    let greeting = text_utils::greeting();
    let name = app.settings.username.trim();
    let who = if name.is_empty() { "friend" } else { name };

    let bubble = app.mascot_message.as_deref().map(|message| {
        let elapsed = app.mascot_born.elapsed().as_secs_f32();
        (message, (1.0 - elapsed / 4.2).clamp(0.0, 1.0))
    });

    // The speech bubble sits to the left of the mascot.
    let mut mascot_area = row![].spacing(8).align_y(Alignment::Center);
    if let Some((message, alpha)) = bubble {
        mascot_area = mascot_area.push(mascot::bubble(message, p, alpha));
    }
    mascot_area = mascot_area.push(mascot::clickable(app.phase, p, 158.0));

    let summary = if active.is_empty() {
        "Your little library is waiting for its very first thought.".to_string()
    } else {
        format!(
            "You have {} and {} tucked away. Take your time.",
            text_utils::plural(active.len(), "note", "notes"),
            text_utils::plural(app.total_words(), "word", "words")
        )
    };

    let hero = container(
        row![
            column![
                text(format!("{greeting}, {who}"))
                    .size(30.0)
                    .font(theme::FONT_BOLD)
                    .color(p.text),
                text(summary)
                    .size(13.0)
                    .color(p.text_muted)
                    .line_height(1.5)
                    .width(440),
                Space::new().height(6),
                row![
                    stat(Icon::Note, active.len().to_string(), "notes", p.primary, p),
                    stat(
                        Icon::Star,
                        app.favorite_count().to_string(),
                        "favorites",
                        p.pastel[2],
                        p
                    ),
                    stat(
                        Icon::Folder,
                        app.folders.len().to_string(),
                        "folders",
                        p.pastel[1],
                        p
                    ),
                    stat(
                        Icon::Feather,
                        text_utils::compact_number(app.total_words()),
                        "words",
                        p.pastel[4],
                        p
                    ),
                ]
                .spacing(26)
                .align_y(Alignment::Center),
            ]
            .spacing(6)
            .width(Fill),
            container(mascot_area).padding(theme::pad(0.0, 6.0, 0.0, 0.0)),
        ]
        .align_y(Alignment::Center)
        .spacing(16),
    )
    .padding(iced::Padding::from([26.0, 30.0]))
    .width(Fill)
    .style(move |_theme| style::hero(p));

    let quick = row![
        search_bar(&app.search, 340.0, p),
        Space::new().width(Fill),
        icon_button(Icon::Keyboard, Message::OpenShortcuts, p, false),
        icon_button(Icon::Download, Message::ExportAll, p, false),
        primary_button("New note", Some(Icon::Plus), Message::NewNote, p),
    ]
    .spacing(10)
    .align_y(Alignment::Center);

    let mut content = column![hero, Space::new().height(22), quick, Space::new().height(18)]
        .spacing(0)
        .width(Fill);

    if active.is_empty() {
        content = content.push(container(empty_state(
            app.phase,
            "A fresh page, just for you",
            "Write a thought, a recipe, a tiny list — anything at all. Mochi will keep it safe.",
            Some((Icon::Plus, "Create your first note", Message::NewNote)),
            bubble,
            p,
        ))
        .height(360.0)
        .width(Fill));
    } else {
        if !pinned.is_empty() {
            content = content.push(section(
                "Pinned",
                Some("Little things you want close by.".into()),
                None,
                p,
            ));
            content = content.push(note_grid(app, &pinned, CardMode::Active, false));
            content = content.push(page_spacer(24.0));
        }

        content = content.push(section(
            "Recent notes",
            Some("Picked up right where you left off.".into()),
            Some(iced::widget::button(
                text("See all")
                    .size(11.5)
                    .font(theme::FONT_MEDIUM)
                    .color(p.primary),
            )
            .padding(iced::Padding::from([4.0, 8.0]))
            .style(move |_theme, status| style::button_ghost(p, status))
            .on_press(Message::Navigate(Screen::AllNotes))
            .into()),
            p,
        ));
        content = content.push(note_grid(app, &recent, CardMode::Active, false));

        if !favorites.is_empty() {
            content = content.push(page_spacer(24.0));
            content = content.push(section(
                "Favorites",
                Some("The notes your heart keeps returning to.".into()),
                Some(iced::widget::button(
                    text("All favorites")
                        .size(11.5)
                        .font(theme::FONT_MEDIUM)
                        .color(p.primary),
                )
                .padding(iced::Padding::from([4.0, 8.0]))
                .style(move |_theme, status| style::button_ghost(p, status))
                .on_press(Message::Navigate(Screen::Favorites))
                .into()),
                p,
            ));
            content = content.push(note_grid(app, &favorites, CardMode::Active, true));
        }
    }

    content = content.push(page_spacer(20.0));

    page(content, p)
}

fn section<'a>(
    title: impl Into<String>,
    subtitle: Option<String>,
    action: Option<Element<'a, Message>>,
    p: theme::Palette,
) -> Element<'a, Message> {
    let mut info = column![text(title.into())
        .size(17.0)
        .font(theme::FONT_BOLD)
        .color(p.text)]
    .spacing(2);

    if let Some(subtitle) = subtitle {
        info = info.push(text(subtitle).size(11.5).color(p.text_muted));
    }

    let mut header = row![info, Space::new().width(Fill)].align_y(Alignment::Center);
    if let Some(action) = action {
        header = header.push(action);
    }

    container(header)
        .width(Fill)
        .padding(theme::pad(0.0, 2.0, 10.0, 2.0))
        .into()
}

fn note_grid<'a>(
    app: &'a Mochi,
    notes: &[&'a Note],
    mode: CardMode,
    reorder: bool,
) -> Element<'a, Message> {
    let mut grid = grid::Grid::new()
        .fluid(288.0)
        .spacing(14)
        .height(grid::Sizing::EvenlyDistribute(Length::Shrink));

    for note in notes {
        let folder = note.folder_id.and_then(|id| app.folder(id));
        let hover = app.hover_progress(crate::message::Hover::Note(note.id));
        grid = grid.push(note_card(
            note,
            &app.tags,
            folder,
            app.palette,
            hover,
            mode,
            if reorder {
                Some(crate::message::DropTarget::Note(note.id))
            } else {
                None
            },
        ));
    }

    grid.into()
}
