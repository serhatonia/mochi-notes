//! The pastel note card used across dashboards and libraries.

use iced::widget::{column, container, mouse_area, row, text, Space};
use iced::{mouse, Alignment, Border, Color, Element, Fill, Padding};

use crate::message::{DropTarget, Hover, Message};
use crate::models::{Folder, Note, Tag};
use crate::theme::{self, style, Palette};
use crate::utils::{color, text as text_utils};
use crate::widgets::components::{dot, icon_button};
use crate::widgets::icons::{icon, Icon};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CardMode {
    Active,
    Archive,
    Trash,
}

pub fn note_card<'a>(
    note: &'a Note,
    tags: &'a [Tag],
    folder: Option<&'a Folder>,
    palette: Palette,
    hover: f32,
    mode: CardMode,
    drop_target: Option<DropTarget>,
) -> Element<'a, Message> {
    let tint_index = color::note_tint(note.id.as_u128(), palette.pastel.len());
    let tint = palette.tint(tint_index);

    let mut top = row![].spacing(6).align_y(Alignment::Center);

    match mode {
        CardMode::Trash => {
            top = top.push(icon(Icon::Trash, 11.0, palette.danger));
            top = top.push(
                text("In trash")
                    .size(10.5)
                    .font(theme::FONT_MEDIUM)
                    .color(palette.danger),
            );
        }
        CardMode::Archive => {
            top = top.push(icon(Icon::Archive, 11.0, palette.text_muted));
            top = top.push(
                text("Archived")
                    .size(10.5)
                    .color(palette.text_muted),
            );
        }
        CardMode::Active => {
            if note.pinned {
                top = top.push(icon(Icon::Pin, 10.5, palette.primary));
                top = top.push(
                    text("Pinned")
                        .size(10.5)
                        .font(theme::FONT_MEDIUM)
                        .color(palette.primary),
                );
            } else if let Some(folder) = folder {
                top = top.push(dot(theme::parse_hex(&folder.color), 7.0));
                top = top.push(
                    text(folder.name.clone())
                        .size(10.5)
                        .color(palette.text_muted),
                );
            } else {
                top = top.push(
                    text("Note")
                        .size(10.5)
                        .color(palette.text_faint),
                );
            }
        }
    }

    top = top.push(Space::new().width(Fill));

    if mode == CardMode::Active {
        top = top.push(icon_button(
            Icon::Star,
            Message::ToggleFavorite(note.id),
            palette,
            note.favorite,
        ));
        top = top.push(icon_button(
            Icon::Pin,
            Message::TogglePin(note.id),
            palette,
            note.pinned,
        ));
    }

    let title = text_utils::truncate_chars(&note.display_title(), 64);
    let excerpt = note.excerpt(150);
    let preview_text = if excerpt.is_empty() {
        "No words yet — just a cozy blank page.".to_string()
    } else {
        excerpt
    };

    let content = column![
        text(title)
            .size(16.0)
            .font(theme::FONT_BOLD)
            .color(palette.text)
            .line_height(1.3)
            .wrapping(iced::widget::text::Wrapping::Word),
        text(preview_text)
            .size(11.8)
            .color(if note.excerpt(1).is_empty() {
                palette.text_faint
            } else {
                palette.text_muted
            })
            .line_height(1.5)
            .wrapping(iced::widget::text::Wrapping::Word)
    ]
    .spacing(7);

    // Dragging is handled by a mouse area around the text. Opening the note
    // happens on mouse release in the application (a tiny movement = a click).
    let content: Element<'a, Message> = mouse_area(
        container(content).padding(0).width(Fill),
    )
    .on_press(Message::DragNote(note.id))
    .interaction(mouse::Interaction::Pointer)
    .into();

    // Tags, at most two plus an overflow badge.
    let mut tag_row = row![].spacing(5).align_y(Alignment::Center);
    let note_tags: Vec<&Tag> = tags
        .iter()
        .filter(|tag| note.tags.contains(&tag.id))
        .take(2)
        .collect();

    for tag in &note_tags {
        tag_row = tag_row.push(container(
            text(tag.name.clone())
                .size(9.5)
                .font(theme::FONT_MEDIUM)
                .color(theme::mix(palette.text, theme::parse_hex(&tag.color), 0.45)),
        )
        .padding(Padding::from([2.0, 7.0]))
        .style({
            let tint = theme::parse_hex(&tag.color);
            move |_theme| container::Style {
                background: Some(iced::Background::Color(theme::mix(
                    palette.surface,
                    tint,
                    0.22,
                ))),
                border: Border {
                    color: Color::TRANSPARENT,
                    width: 0.0,
                    radius: 999.0.into(),
                },
                ..container::Style::default()
            }
        }));
    }

    if note.tags.len() > note_tags.len() {
        tag_row = tag_row.push(
            text(format!("+{}", note.tags.len() - note_tags.len()))
                .size(9.5)
                .color(palette.text_faint),
        );
    }

    let footer_actions: Element<'a, Message> = match mode {
        CardMode::Active => row![
            icon_button(Icon::Copy, Message::DuplicateNote(note.id), palette, false),
            icon_button(Icon::Archive, Message::ArchiveNote(note.id), palette, false),
            icon_button(Icon::Trash, Message::TrashNote(note.id), palette, false),
        ]
        .spacing(1)
        .into(),
        CardMode::Archive => row![
            icon_button(Icon::Undo, Message::UnarchiveNote(note.id), palette, false),
            icon_button(Icon::Trash, Message::TrashNote(note.id), palette, false),
        ]
        .spacing(1)
        .into(),
        CardMode::Trash => row![
            icon_button(Icon::RotateLeft, Message::RestoreNote(note.id), palette, false),
            icon_button(
                Icon::TrashCan,
                Message::RequestDeleteNote(note.id),
                palette,
                false,
            ),
        ]
        .spacing(1)
        .into(),
    };

    let footer = row![
        tag_row,
        Space::new().width(Fill),
        text(text_utils::relative_time(note.updated_at))
            .size(10.0)
            .color(palette.text_faint),
        footer_actions
    ]
    .spacing(4)
    .align_y(Alignment::Center)
    .width(Fill);

    let card = container(
        column![top, Space::new().height(2), content, Space::new().width(Fill), footer]
            .spacing(8)
            .height(Fill),
    )
    .padding(Padding::from([14.0, 16.0]))
    .width(Fill)
    .height(196)
    .clip(true)
    .style(move |_theme| style::tinted_card(palette, tint, hover));

    let card: Element<'a, Message> = mouse_area(card)
        .on_enter(Message::HoverEnter(Hover::Note(note.id)))
        .on_exit(Message::HoverExit(Hover::Note(note.id)))
        .into();

    if let Some(target) = drop_target {
        mouse_area(card)
            .on_enter(Message::DropEnter(target))
            .on_exit(Message::DropExit(target))
            .into()
    } else {
        card
    }
}
