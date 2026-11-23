//! The left navigation rail, which collapses into an icon strip on narrow
//! windows.

use iced::widget::{button, column, container, mouse_area, row, scrollable, text, Space};
use iced::{Alignment, Border, Color, Element, Fill, Padding};

use crate::app::Mochi;
use crate::message::{DropTarget, Hover, Message, NavId, Screen};
use crate::theme::{self, style, Palette};
use crate::widgets::components::{
    count_badge, dot, icon_button, nav_item, plain_button_style, section_label,
};
use crate::widgets::icons::{icon, Icon};
use crate::widgets::mascot;

pub fn sidebar<'a>(app: &'a Mochi, compact: bool) -> Element<'a, Message> {
    let p = app.palette;
    let width = if compact { 76.0 } else { 260.0 };

    let mut nav = column![].spacing(1).width(Fill);

    nav = nav.push(nav_item(
        Icon::Home,
        "Home",
        None,
        app.nav_selected(NavId::Home),
        app.hovered == Some(Hover::Nav(NavId::Home)),
        Hover::Nav(NavId::Home),
        Message::Navigate(Screen::Home),
        p,
        app.hover_progress(Hover::Nav(NavId::Home)),
    ));
    nav = nav.push(nav_item(
        Icon::Note,
        "All notes",
        Some(app.active_notes().count()),
        app.nav_selected(NavId::AllNotes),
        app.hovered == Some(Hover::Nav(NavId::AllNotes)),
        Hover::Nav(NavId::AllNotes),
        Message::Navigate(Screen::AllNotes),
        p,
        app.hover_progress(Hover::Nav(NavId::AllNotes)),
    ));
    nav = nav.push(nav_item(
        Icon::Star,
        "Favorites",
        Some(app.favorite_count()),
        app.nav_selected(NavId::Favorites),
        app.hovered == Some(Hover::Nav(NavId::Favorites)),
        Hover::Nav(NavId::Favorites),
        Message::Navigate(Screen::Favorites),
        p,
        app.hover_progress(Hover::Nav(NavId::Favorites)),
    ));

    if !compact {
        nav = nav.push(
            container(
                row![
                    section_label("FOLDERS", p),
                    Space::new().width(Fill),
                    icon_button(Icon::Plus, Message::OpenFolderForm(None), p, false),
                ]
                .align_y(Alignment::Center)
                .padding(Padding::from([0.0, 13.0])),
            )
            .width(Fill)
            .height(32.0)
            .align_y(Alignment::Center),
        );

        if app.folders.is_empty() {
            nav = nav.push(
                container(
                    text("No folders yet")
                        .size(11.5)
                        .color(p.text_faint),
                )
                .padding(Padding::from([0.0, 13.0]))
                .height(32.0)
                .align_y(Alignment::Center),
            );
        } else {
            for folder in app.folders.iter().take(6) {
                let color = theme::parse_hex(&folder.color);
                let selected = app.screen == Screen::Folder(folder.id);
                nav = nav.push(with_drop(
                    folder_item(
                        folder.name.clone(),
                        color,
                        app.folder_count(folder.id),
                        selected,
                        Hover::Folder(folder.id),
                        app.hover_progress(Hover::Folder(folder.id)),
                        Message::Navigate(Screen::Folder(folder.id)),
                        p,
                    ),
                    DropTarget::Folder(folder.id),
                ));
            }
        }

        nav = nav.push(manage_row(
            Icon::FolderOpen,
            "Manage folders",
            app.folders.len(),
            Hover::Generic(4),
            Message::Navigate(Screen::Folders),
            app.screen == Screen::Folders,
            app.hover_progress(Hover::Generic(4)),
            p,
        ));

        nav = nav.push(
            container(
                row![
                    section_label("TAGS", p),
                    Space::new().width(Fill),
                    icon_button(Icon::Plus, Message::OpenTagForm(None), p, false),
                ]
                .align_y(Alignment::Center)
                .padding(Padding::from([0.0, 13.0])),
            )
            .width(Fill)
            .height(32.0)
            .align_y(Alignment::Center),
        );

        if app.tags.is_empty() {
            nav = nav.push(
                container(text("No tags yet").size(11.5).color(p.text_faint))
                    .padding(Padding::from([0.0, 13.0]))
                    .height(32.0)
                    .align_y(Alignment::Center),
            );
        } else {
            for tag in app.tags.iter().take(6) {
                let color = theme::parse_hex(&tag.color);
                let selected = app.screen == Screen::Tag(tag.id);
                nav = nav.push(folder_item(
                    format!("#{}", tag.name),
                    color,
                    app.tag_count(tag.id),
                    selected,
                    Hover::Tag(tag.id),
                    app.hover_progress(Hover::Tag(tag.id)),
                    Message::Navigate(Screen::Tag(tag.id)),
                    p,
                ));
            }
        }

        nav = nav.push(manage_row(
            Icon::Tags,
            "Manage tags",
            app.tags.len(),
            Hover::Generic(5),
            Message::Navigate(Screen::Tags),
            app.screen == Screen::Tags,
            app.hover_progress(Hover::Generic(5)),
            p,
        ));
    }

    nav = nav.push(with_drop(
        nav_item(
            Icon::Archive,
            "Archive",
            Some(app.archived_count()),
            app.nav_selected(NavId::Archive),
            app.hovered == Some(Hover::Nav(NavId::Archive)),
            Hover::Nav(NavId::Archive),
            Message::Navigate(Screen::Archive),
            p,
            app.hover_progress(Hover::Nav(NavId::Archive)),
        ),
        DropTarget::Archive,
    ));
    nav = nav.push(with_drop(
        nav_item(
            Icon::Trash,
            "Trash",
            Some(app.trashed_count()),
            app.nav_selected(NavId::Trash),
            app.hovered == Some(Hover::Nav(NavId::Trash)),
            Hover::Nav(NavId::Trash),
            Message::Navigate(Screen::Trash),
            p,
            app.hover_progress(Hover::Nav(NavId::Trash)),
        ),
        DropTarget::Trash,
    ));

    let middle = scrollable(nav.padding(Padding::from([0.0, 12.0])))
        .style(move |_theme, status| style::scrollable(p, status))
        .height(Fill);

    let brand: Element<'a, Message> = if compact {
        container(mascot::mascot(app.phase, p, 44.0))
            .padding(Padding::from([18.0, 10.0]))
            .center_x(Fill)
            .into()
    } else {
        row![
            mascot::mascot(app.phase, p, 44.0),
            column![
                text("Mochi")
                    .size(17.0)
                    .font(theme::FONT_BOLD)
                    .color(p.text),
                text("Notes")
                    .size(10.5)
                    .font(theme::FONT_MEDIUM)
                    .color(p.primary)
            ]
            .spacing(0)
        ]
        .spacing(10)
        .align_y(Alignment::Center)
        .padding(Padding::from([18.0, 20.0]))
        .into()
    };

    let new_note: Element<'a, Message> = if compact {
        container(
            button(icon(Icon::Plus, 14.0, p.on_primary))
                .padding(Padding::from([10.0, 12.0]))
                .style(move |_theme, status| style::button_primary(p, status))
                .on_press(Message::NewNote),
        )
        .center_x(Fill)
        .padding(Padding::from([0.0, 0.0]))
        .into()
    } else {
        button(
            row![
                icon(Icon::Plus, 13.0, p.on_primary),
                text("New note")
                    .size(13.5)
                    .font(theme::FONT_BOLD)
                    .color(p.on_primary)
            ]
            .spacing(8)
            .align_y(Alignment::Center),
        )
        .padding(Padding::from([11.0, 13.0]))
        .width(Fill)
        .style(move |_theme, status| style::button_primary(p, status))
        .on_press(Message::NewNote)
        .into()
    };

    let settings_item: Element<'a, Message> = if compact {
        icon_button(
            Icon::Gear,
            Message::Navigate(Screen::Settings),
            p,
            app.screen == Screen::Settings,
        )
    } else {
        nav_item(
            Icon::Gear,
            "Settings",
            None,
            app.nav_selected(NavId::Settings),
            app.hovered == Some(Hover::Nav(NavId::Settings)),
            Hover::Nav(NavId::Settings),
            Message::Navigate(Screen::Settings),
            p,
            app.hover_progress(Hover::Nav(NavId::Settings)),
        )
    };

    let footer: Element<'a, Message> = if compact {
        column![
            container(
                button(icon(Icon::Palette, 13.0, p.primary))
                    .padding(Padding::from([8.0, 10.0]))
                    .style(move |_theme, status| style::button_soft(p, status))
                    .on_press(Message::Navigate(Screen::Settings)),
            )
            .center_x(Fill)
            .padding(Padding::from([4.0, 0.0])),
            container(settings_item).padding(Padding::from([0.0, 12.0])),
        ]
        .spacing(4)
        .padding(theme::pad(0.0, 0.0, 14.0, 0.0))
        .into()
    } else {
        column![
            theme_card(app),
            container(settings_item).padding(Padding::from([0.0, 12.0])),
        ]
        .spacing(6)
        .padding(theme::pad(0.0, 0.0, 14.0, 0.0))
        .into()
    };

    container(column![
        brand,
        container(new_note).padding(Padding::from([0.0, if compact { 10.0 } else { 14.0 }])),
        middle,
        container(
            container(Space::new().height(1))
                .width(Fill)
                .style(move |_theme| container::Style {
                    background: Some(iced::Background::Color(p.border)),
                    ..container::Style::default()
                }),
        )
        .padding(Padding::from([0.0, if compact { 10.0 } else { 14.0 }])),
        footer,
    ])
    .width(width)
    .height(Fill)
    .style(move |_theme| style::sidebar(p))
    .into()
}

/// Marks a sidebar row as a drop target for dragged notes.
fn with_drop<'a>(content: Element<'a, Message>, target: DropTarget) -> Element<'a, Message> {
    mouse_area(content)
        .on_enter(Message::DropEnter(target))
        .on_exit(Message::DropExit(target))
        .into()
}

/// A quiet sidebar link that reveals a chevron and tints itself on hover.
fn manage_row<'a>(
    glyph: Icon,
    label: &str,
    count: usize,
    key: Hover,
    message: Message,
    selected: bool,
    hover_t: f32,
    p: Palette,
) -> Element<'a, Message> {
    let hovered = hover_t > 0.0;
    let tint = if selected {
        p.primary
    } else if hovered {
        p.text
    } else {
        p.text_muted
    };

    // Built exactly like the Home / All notes rows: same icon slot, same
    // typography, same hover background and the same count badge on the right.
    let content = row![
        container(icon(glyph, 14.0, tint))
            .width(16.0)
            .align_x(Alignment::Center),
        text(label.to_string()).size(13.0).color(tint),
        Space::new().width(Fill),
        count_badge(count, p, selected),
    ]
    .spacing(9)
    .align_y(Alignment::Center);

    mouse_area(
        button(
            container(content)
                .padding(Padding::from([7.0, 13.0]))
                .width(Fill)
                .style(move |_theme| style::nav(p, selected, hover_t)),
        )
        .padding(0)
        .width(Fill)
        .height(32.0)
        .style(plain_button_style)
        .on_press(message),
    )
    .on_enter(Message::HoverEnter(key))
    .on_exit(Message::HoverExit(key))
    .into()
}

fn folder_item<'a>(
    name: impl Into<String>,
    color: Color,
    count: usize,
    selected: bool,
    key: Hover,
    hover_t: f32,
    message: Message,
    p: Palette,
) -> Element<'a, Message> {
    let name = name.into();
    let hover = if selected { 1.0 } else { hover_t };

    mouse_area(
        button(
            container(
                row![
                    container(dot(color, 7.0))
                        .width(16.0)
                        .align_x(Alignment::Center),
                    text(name)
                        .size(12.5)
                        .font(if selected {
                            theme::FONT_MEDIUM
                        } else {
                            theme::FONT
                        })
                        .color(if selected { p.text } else { p.text_muted })
                        .wrapping(iced::widget::text::Wrapping::None),
                    Space::new().width(Fill),
                    text(count.to_string()).size(10.5).color(p.text_faint),
                ]
                .spacing(9)
                .align_y(Alignment::Center),
            )
            .padding(Padding::from([7.0, 13.0]))
            .width(Fill)
            .style(move |_theme| style::nav(p, false, hover * 0.55)),
        )
        .padding(0)
        .width(Fill)
        .height(32.0)
        .style(plain_button_style)
        .on_press(message),
    )
    .on_enter(Message::HoverEnter(key))
    .on_exit(Message::HoverExit(key))
    .into()
}

fn theme_card<'a>(app: &'a Mochi) -> Element<'a, Message> {
    let p = app.palette;
    let theme = app.settings.theme;

    let swatch_children: Vec<Element<'a, Message>> = theme::ThemeId::ALL
        .iter()
        .map(|id| {
            let palette = id.palette();
            let active = *id == theme;
            button(
                container(Space::new().width(13).height(13)).style(move |_theme| container::Style {
                    background: Some(iced::Background::Color(palette.primary)),
                    border: Border {
                        color: if active { p.text } else { Color::TRANSPARENT },
                        width: if active { 2.0 } else { 0.0 },
                        radius: 999.0.into(),
                    },
                    ..container::Style::default()
                }),
            )
            .padding(3)
            .style(plain_button_style)
            .on_press(Message::SetTheme(*id))
            .into()
        })
        .collect();

    let swatches = row(swatch_children).spacing(1).align_y(Alignment::Center);

    container(
        button(
            column![
                row![
                    icon(Icon::Palette, 11.5, p.primary),
                    text("Theme").size(10.0).font(theme::FONT_BOLD).color(p.text_faint),
                    Space::new().width(Fill),
                ]
                .spacing(5)
                .align_y(Alignment::Center),
                row![
                    text(theme.name())
                        .size(11.5)
                        .font(theme::FONT_MEDIUM)
                        .color(p.text),
                    Space::new().width(Fill),
                ],
                swatches,
            ]
            .spacing(6),
        )
        .padding(Padding::from([9.0, 12.0]))
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
                    radius: 12.0.into(),
                },
                shadow: iced::Shadow::default(),
                snap: false,
            }
        })
        .on_press(Message::Navigate(Screen::Settings)),
    )
    .padding(Padding::from([0.0, 12.0]))
    .into()
}

/// Small helper used by a few views to show a count next to a label.
pub fn count<'a>(value: usize, active: bool, p: Palette) -> Element<'a, Message> {
    count_badge(value, p, active)
}
