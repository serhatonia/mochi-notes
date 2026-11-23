//! Screen composition. Every view is a pure function of [`Mochi`] state.

pub mod dashboard;
pub mod editor;
pub mod folders;
pub mod library;
pub mod modals;
pub mod settings;
pub mod sidebar;
pub mod tags;
pub mod titlebar;
pub mod toast;

use iced::widget::{column, container, responsive, row, stack, text, Space};
use iced::{Alignment, Background, Border, Element, Fill, Padding};

use crate::app::Mochi;
use crate::message::{DragState, Message, Screen};
use crate::theme::{self, style};
use crate::widgets::components::soft_button;
use crate::widgets::icons::{icon, Icon};
use crate::widgets::mascot;

pub fn root(app: &Mochi) -> Element<'_, Message> {
    let base: Element<'_, Message> = if let Some(error) = &app.db_error {
        database_error(app, error)
    } else {
        responsive(|size| {
            let compact = size.width < 1020.0;
            let sidebar = sidebar::sidebar(app, compact);

            let content: Element<'_, Message> = match app.screen {
                Screen::Home => dashboard::view(app),
                Screen::AllNotes
                | Screen::Favorites
                | Screen::Folder(_)
                | Screen::Tag(_)
                | Screen::Archive
                | Screen::Trash => library::view(app),
                Screen::Folders => folders::view(app),
                Screen::Tags => tags::view(app),
                Screen::Settings => settings::view(app),
                Screen::Editor => editor::view(app),
            };

            row![sidebar, content].height(Fill).into()
        })
        .into()
    };

    // The frameless window is drawn as one clipped shell so the OS corners
    // stay soft and every screen inherits the current palette.
    let shell: Element<'_, Message> = container(
        column![titlebar::view(app), base]
            .width(Fill)
            .height(Fill),
    )
    .width(Fill)
    .height(Fill)
    .padding(1)
    .clip(true)
    .style(move |_theme| {
        let p = app.palette;
        container::Style {
            background: Some(Background::Color(p.bg)),
            text_color: Some(p.text),
            border: Border {
                color: p.border_strong,
                width: 1.0,
                radius: 14.0.into(),
            },
            shadow: iced::Shadow::default(),
            snap: false,
        }
    })
    .into();

    let mut layers: Vec<Element<'_, Message>> = vec![shell];

    if let Some(modal) = &app.modal {
        layers.push(modals::view(app, modal));
    }

    if let Some(toast) = &app.toast {
        layers.push(toast::view(app, toast));
    }

    if let Some(drag) = &app.drag {
        layers.push(drag_ghost(app, drag));
    }

    stack(layers).width(Fill).height(Fill).into()
}

/// A small floating card that follows the cursor while a note is dragged.
fn drag_ghost<'a>(app: &'a Mochi, drag: &DragState) -> Element<'a, Message> {
    let p = app.palette;
    let title = app
        .note(drag.note_id)
        .map(|note| note.display_title())
        .unwrap_or_else(|| "Note".to_string());

    let x = (drag.position.x - 28.0).max(0.0);
    let y = (drag.position.y - 16.0).max(0.0);

    container(
        container(
            row![
                icon(Icon::Note, 11.0, p.primary),
                text(title)
                    .size(11.5)
                    .font(theme::FONT_MEDIUM)
                    .color(p.text)
                    .wrapping(iced::widget::text::Wrapping::None),
            ]
            .spacing(7)
            .align_y(Alignment::Center),
        )
        .padding(Padding::from([9.0, 12.0]))
        .max_width(220)
        .style(move |_theme| container::Style {
            background: Some(Background::Color(theme::alpha(p.surface, 0.94))),
            text_color: Some(p.text),
            border: Border {
                color: p.primary,
                width: 1.0,
                radius: 12.0.into(),
            },
            shadow: p.shadow_lifted(),
            snap: false,
        }),
    )
    .width(Fill)
    .height(Fill)
    .padding(theme::pad(y, 0.0, 0.0, x))
    .align_x(Alignment::Start)
    .align_y(Alignment::Start)
    .into()
}

fn database_error<'a>(app: &'a Mochi, error: &str) -> Element<'a, Message> {
    let palette = app.palette;

    let card = container(
        column![
            mascot::mascot(app.phase, palette, 150.0),
            iced::widget::text("Mochi could not open your library")
                .size(theme::size::H2)
                .font(theme::FONT_BOLD)
                .color(palette.text),
            iced::widget::text(
                "The database file could not be read. It may be locked by another \
                 copy of Mochi Notes, or the folder may be read-only."
            )
            .size(12.5)
            .color(palette.text_muted)
            .align_x(iced::Alignment::Center)
            .width(420),
            container(
                iced::widget::text(error.to_string())
                    .size(11.0)
                    .font(theme::FONT_MONO)
                    .color(palette.danger)
                    .wrapping(iced::widget::text::Wrapping::Word)
            )
            .padding(14)
            .width(460)
            .style(move |_theme| style::toolbar(palette)),
            row![
                soft_button("Try again", Some(Icon::Refresh), Message::RetryDatabase, palette),
                soft_button(
                    "Open data folder",
                    Some(Icon::FolderOpen),
                    Message::OpenDataFolder,
                    palette
                ),
            ]
            .spacing(10)
            .align_y(iced::Alignment::Center),
        ]
        .spacing(16)
        .align_x(iced::Alignment::Center),
    )
    .padding(44)
    .max_width(560)
    .style(move |_theme| style::panel(palette));

    container(card)
        .width(Fill)
        .height(Fill)
        .align_x(Alignment::Center)
        .align_y(Alignment::Center)
        .style(move |_theme| style::app(palette))
        .into()
}

/// A vertically scrolling page with consistent padding.
pub fn page<'a>(content: impl Into<Element<'a, Message>>, palette: theme::Palette) -> Element<'a, Message> {
    iced::widget::scrollable(container(content.into()).padding(iced::Padding::from([26.0, 30.0])))
        .style(move |_theme, status| style::scrollable(palette, status))
        .width(Fill)
        .height(Fill)
        .into()
}

pub fn page_spacer<'a>(height: f32) -> Element<'a, Message> {
    Space::new().height(height).into()
}
