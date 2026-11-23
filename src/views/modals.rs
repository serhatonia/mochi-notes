//! Modal dialogs: folder and tag forms, pickers, confirmations and the
//! keyboard shortcut cheat sheet.

use iced::widget::{button, column, container, row, scrollable, text, text_input, Space};
use iced::{Alignment, Border, Color, Element, Fill, Length, Padding};

use crate::app::Mochi;
use crate::message::{Message, Modal};
use crate::theme::{self, style};
use crate::utils::text as text_utils;
use crate::widgets::components::{
    danger_button, dot, ghost_button, icon_button, key_hint, plain_button_style,
    primary_button, soft_button,
};
use crate::widgets::icons::{icon, Icon};

pub fn view<'a>(app: &'a Mochi, modal: &'a Modal) -> Element<'a, Message> {
    let p = app.palette;

    let card: Element<'a, Message> = match modal {
        Modal::FolderForm { id, name, color } => form(
            app,
            if id.is_some() { "Edit folder" } else { "New folder" },
            "Give this little home a name and a color.",
            "Folder name",
            name,
            *color,
            if id.is_some() { "Save folder" } else { "Create folder" },
            Icon::FolderPlus,
        ),
        Modal::TagForm { id, name, color } => form(
            app,
            if id.is_some() { "Edit tag" } else { "New tag" },
            "A short label, like ideas or recipes.",
            "Tag name",
            name,
            *color,
            if id.is_some() { "Save tag" } else { "Create tag" },
            Icon::Hashtag,
        ),
        Modal::TagPicker { note_id, query } => tag_picker(app, *note_id, query),
        Modal::FolderPicker { note_id } => folder_picker(app, *note_id),
        Modal::Confirm {
            title,
            body,
            confirm_label,
            danger,
            ..
        } => confirm(app, title, body, confirm_label, *danger),
        Modal::Shortcuts => shortcuts(app),
    };

    container(
        container(card)
            .padding(Padding::from([20.0, 22.0]))
            .max_width(540)
            .style(move |_theme| style::panel(p)),
    )
    .width(Fill)
    .height(Fill)
    .align_x(Alignment::Center)
    .align_y(Alignment::Center)
    .style(move |_theme| style::modal_backdrop(p))
    .into()
}

fn header<'a>(
    glyph: Icon,
    title: &str,
    subtitle: &str,
    p: theme::Palette,
) -> Element<'a, Message> {
    row![
        container(icon(glyph, 13.0, p.primary))
            .width(34)
            .height(34)
            .align_x(Alignment::Center)
            .align_y(Alignment::Center)
            .style(move |_theme| style::badge(p, p.primary_soft)),
        column![
            text(title.to_string())
                .size(15.5)
                .font(theme::FONT_BOLD)
                .color(p.text),
            text(subtitle.to_string())
                .size(11.0)
                .color(p.text_muted),
        ]
        .spacing(2)
        .width(Fill),
        icon_button(Icon::Close, Message::ModalClose, p, false),
    ]
    .spacing(10)
    .align_y(Alignment::Center)
    .width(Fill)
    .into()
}

#[allow(clippy::too_many_arguments)]
fn form<'a>(
    app: &'a Mochi,
    title: &str,
    subtitle: &str,
    placeholder: &str,
    name: &str,
    color: usize,
    confirm_label: &str,
    glyph: Icon,
) -> Element<'a, Message> {
    let p = app.palette;

    let input = text_input(placeholder, name)
        .id("modal-name")
        .on_input(Message::ModalNameChanged)
        .on_submit(Message::ModalConfirm)
        .padding(Padding::from([10.0, 13.0]))
        .size(13.0)
        .width(Fill)
        .style(move |_theme, status| style::input(p, status));

    let swatches: Vec<Element<'a, Message>> = theme::SWATCHES
        .iter()
        .enumerate()
        .map(|(index, hex)| {
            let swatch = theme::parse_hex(hex);
            let selected = color == index;
            button(
                container(Space::new().width(22).height(22)).style(move |_theme| container::Style {
                    background: Some(iced::Background::Color(swatch)),
                    border: Border {
                        color: if selected { p.text } else { Color::TRANSPARENT },
                        width: if selected { 2.5 } else { 0.0 },
                        radius: 8.0.into(),
                    },
                    ..container::Style::default()
                }),
            )
            .padding(2)
            .style(plain_button_style)
            .on_press(Message::ModalColorSelected(index))
            .into()
        })
        .collect();

    column![
        header(glyph, title, subtitle, p),
        Space::new().height(4),
        input,
        column![
            text("Color")
                .size(10.5)
                .font(theme::FONT_BOLD)
                .color(p.text_faint),
            row(swatches).spacing(3),
        ]
        .spacing(6),
        Space::new().height(2),
        row![
            Space::new().width(Fill),
            ghost_button("Cancel", None, Message::ModalClose, p),
            primary_button(confirm_label, Some(Icon::Check), Message::ModalConfirm, p),
        ]
        .spacing(8)
        .align_y(Alignment::Center),
    ]
    .spacing(12)
    .width(Fill)
    .into()
}

fn tag_picker<'a>(app: &'a Mochi, note_id: uuid::Uuid, query: &str) -> Element<'a, Message> {
    let p = app.palette;
    let attached = app
        .note(note_id)
        .map(|note| note.tags.clone())
        .unwrap_or_default();

    let input = text_input("Search or type a new tag…", query)
        .id("tag-picker")
        .on_input(Message::TagPickerQueryChanged)
        .on_submit(Message::CreateTagFromPicker)
        .padding(Padding::from([9.0, 13.0]))
        .size(12.5)
        .width(Fill)
        .icon(text_input::Icon {
            font: theme::FONT_ICON,
            code_point: Icon::Search.glyph(),
            size: Some(iced::Pixels(12.0)),
            spacing: 8.0,
            side: text_input::Side::Left,
        })
        .style(move |_theme, status| style::input(p, status));

    let needle = query.trim().to_lowercase();
    let mut list = column![].spacing(3);

    let mut shown = 0;
    for tag in &app.tags {
        if !needle.is_empty() && !tag.name.to_lowercase().contains(&needle) {
            continue;
        }
        shown += 1;

        let tint = theme::parse_hex(&tag.color);
        let active = attached.contains(&tag.id);
        let count = app.tag_count(tag.id);

        let row_content = row![
            dot(tint, 9.0),
            text(format!("#{}", tag.name))
                .size(12.5)
                .font(if active {
                    theme::FONT_MEDIUM
                } else {
                    theme::FONT
                })
                .color(p.text),
            text(text_utils::plural(count, "note", "notes"))
                .size(10.0)
                .color(p.text_faint),
            Space::new().width(Fill),
            check_mark(active, p),
        ]
        .spacing(8)
        .align_y(Alignment::Center)
        .padding(Padding::from([8.0, 10.0]))
        .width(Fill);

        list = list.push(
            button(row_content)
                .padding(0)
                .width(Fill)
                .style(move |_theme, status| {
                    let bg = match status {
                        button::Status::Hovered => p.surface_alt,
                        _ => Color::TRANSPARENT,
                    };
                    button::Style {
                        background: Some(iced::Background::Color(bg)),
                        text_color: p.text,
                        border: Border {
                            color: Color::TRANSPARENT,
                            width: 0.0,
                            radius: 10.0.into(),
                        },
                        shadow: iced::Shadow::default(),
                        snap: false,
                    }
                })
                .on_press(Message::ToggleTagOnEditor(tag.id)),
        );
    }

    if shown == 0 {
        list = list.push(
            container(
                text(if needle.is_empty() {
                    "No tags yet — type a name to create one."
                } else {
                    "No matching tag. Press enter to create it."
                })
                .size(11.5)
                .color(p.text_faint),
            )
            .padding(Padding::from([14.0, 12.0]))
            .width(Fill)
            .center_x(Fill),
        );
    }

    let exact = app
        .tags
        .iter()
        .any(|tag| tag.name.eq_ignore_ascii_case(query.trim()));

    let mut footer = row![Space::new().width(Fill)].align_y(Alignment::Center);
    if !needle.is_empty() && !exact {
        footer = footer.push(soft_button(
            format!("Create #{}", query.trim()),
            Some(Icon::Plus),
            Message::CreateTagFromPicker,
            p,
        ));
    }
    footer = footer.push(ghost_button("Done", None, Message::ModalClose, p));

    column![
        header(
            Icon::Tags,
            "Tags",
            "Click a tag to add or remove it from this note.",
            p
        ),
        input,
        scrollable(list).height(Length::Fixed(250.0)).style(move |_theme, status| style::scrollable(p, status)),
        footer.spacing(8),
    ]
    .spacing(12)
    .width(Fill)
    .into()
}

fn folder_picker<'a>(app: &'a Mochi, note_id: uuid::Uuid) -> Element<'a, Message> {
    let p = app.palette;
    let current = app.note(note_id).and_then(|note| note.folder_id);

    let mut list = column![].spacing(3);

    list = list.push(folder_row(None, "No folder", None, current.is_none(), p));

    for folder in &app.folders {
        list = list.push(folder_row(
            Some(folder.id),
            &folder.name,
            Some(theme::parse_hex(&folder.color)),
            current == Some(folder.id),
            p,
        ));
    }

    column![
        header(
            Icon::FolderOpen,
            "Move to folder",
            "Choose where this note should live.",
            p
        ),
        scrollable(list)
            .height(Length::Fixed(240.0))
            .style(move |_theme, status| style::scrollable(p, status)),
        row![
            Space::new().width(Fill),
            ghost_button("Cancel", None, Message::ModalClose, p),
        ],
    ]
    .spacing(12)
    .width(Fill)
    .into()
}

fn folder_row<'a>(
    target: Option<uuid::Uuid>,
    name: &str,
    color: Option<Color>,
    active: bool,
    p: theme::Palette,
) -> Element<'a, Message> {
    let content = row![
        if let Some(color) = color {
            dot(color, 9.0)
        } else {
            icon(Icon::Folder, 10.0, p.text_faint).into()
        },
        text(name.to_string())
            .size(12.5)
            .font(if active {
                theme::FONT_MEDIUM
            } else {
                theme::FONT
            })
            .color(p.text),
        Space::new().width(Fill),
        check_mark(active, p),
    ]
    .spacing(8)
    .align_y(Alignment::Center)
    .padding(Padding::from([9.0, 10.0]))
    .width(Fill);

    button(content)
        .padding(0)
        .width(Fill)
        .style(move |_theme, status| {
            let bg = match status {
                button::Status::Hovered => p.surface_alt,
                _ => Color::TRANSPARENT,
            };
            button::Style {
                background: Some(iced::Background::Color(bg)),
                text_color: p.text,
                border: Border {
                    color: Color::TRANSPARENT,
                    width: 0.0,
                    radius: 10.0.into(),
                },
                shadow: iced::Shadow::default(),
                snap: false,
            }
        })
        .on_press(Message::SetNoteFolder(target))
        .into()
}

fn confirm<'a>(
    app: &Mochi,
    title: &str,
    body: &str,
    confirm_label: &str,
    danger: bool,
) -> Element<'a, Message> {
    let p = app.palette;

    let icon_badge = container(icon(
        if danger { Icon::TrashCan } else { Icon::Info },
        17.0,
        if danger { p.danger } else { p.primary },
    ))
    .width(46)
    .height(46)
    .align_x(Alignment::Center)
    .align_y(Alignment::Center)
    .style(move |_theme| {
        style::badge(p, if danger { p.danger_soft } else { p.primary_soft })
    });

    let actions: Element<'a, Message> = if danger {
        danger_button(confirm_label, Some(Icon::TrashCan), Message::ModalConfirm, p)
    } else {
        primary_button(confirm_label, Some(Icon::Check), Message::ModalConfirm, p)
    };

    column![
        column![
            icon_badge,
            text(title.to_string())
                .size(17.0)
                .font(theme::FONT_BOLD)
                .color(p.text)
                .align_x(Alignment::Center),
            text(body.to_string())
                .size(12.0)
                .color(p.text_muted)
                .line_height(1.5)
                .align_x(Alignment::Center),
        ]
        .spacing(10)
        .align_x(Alignment::Center)
        .width(Fill),
        Space::new().height(2),
        row![
            Space::new().width(Fill),
            ghost_button("Cancel", None, Message::ModalClose, p),
            actions,
        ]
        .spacing(8)
        .align_y(Alignment::Center),
    ]
    .spacing(14)
    .width(Fill)
    .into()
}

fn check_mark<'a>(active: bool, p: theme::Palette) -> Element<'a, Message> {
    if active {
        icon(Icon::CheckCircle, 13.0, p.primary).into()
    } else {
        icon(Icon::Circle, 13.0, p.border_strong).into()
    }
}

fn shortcuts(app: &Mochi) -> Element<'_, Message> {
    let p = app.palette;

    let items: [(&str, &str); 15] = [
        ("New note", "Ctrl + N"),
        ("Save now", "Ctrl + S"),
        ("Focus mode", "Ctrl + Shift + F"),
        ("Search notes", "Ctrl + F"),
        ("Export note", "Ctrl + E"),
        ("Duplicate note", "Ctrl + D"),
        ("Pin note", "Ctrl + P"),
        ("Bold", "Ctrl + B"),
        ("Italic", "Ctrl + I"),
        ("Home", "Ctrl + H"),
        ("All notes", "Ctrl + 1"),
        ("Favorites", "Ctrl + 2"),
        ("Archive", "Ctrl + 3"),
        ("Trash", "Ctrl + 4"),
        ("Settings", "Ctrl + ,"),
    ];

    let mut rows = column![].spacing(2);
    for (index, (label, keys)) in items.iter().enumerate() {
        rows = rows.push(
            row![
                text(*label)
                    .size(12.0)
                    .color(p.text_muted)
                    .width(Fill),
                key_hint(*keys, p),
            ]
            .spacing(12)
            .align_y(Alignment::Center)
            .padding(Padding::from([5.0, 8.0]))
            .width(Fill),
        );

        if index == 5 {
            rows = rows.push(
                container(Space::new().height(1))
                    .width(Fill)
                    .style(move |_theme| container::Style {
                        background: Some(iced::Background::Color(p.border)),
                        ..container::Style::default()
                    }),
            );
        }
    }

    column![
        header(
            Icon::Keyboard,
            "Keyboard shortcuts",
            "A few keys to keep your hands on the keyboard.",
            p
        ),
        rows,
        row![
            Space::new().width(Fill),
            ghost_button("Esc to close", None, Message::ModalClose, p),
        ],
    ]
    .spacing(12)
    .width(Fill)
    .into()
}
