//! Settings: appearance, editor preferences and local data management.

use iced::widget::{button, column, container, grid, row, slider, text, text_input, Space};
use iced::{Alignment, Border, Color, Element, Fill, Length};

use crate::app::Mochi;
use crate::message::Message;
use crate::services::backup;
use crate::theme::{self, style};
use crate::utils::text as text_utils;
use crate::views::{page, page_spacer};
use crate::widgets::components::{
    divider, ghost_button, icon_button, primary_button, setting_card, soft_button, stat,
    switch_row,
};
use crate::widgets::icons::{icon, Icon};
use crate::widgets::mascot;

pub fn view(app: &Mochi) -> Element<'_, Message> {
    let p = app.palette;

    let header = row![
        column![
            text("Settings")
                .size(theme::size::H1)
                .font(theme::FONT_BOLD)
                .color(p.text),
            text("Make Mochi feel like your own little corner.")
                .size(12.5)
                .color(p.text_muted),
        ]
        .spacing(4),
        Space::new().width(Fill),
        ghost_button("Keyboard shortcuts", Some(Icon::Keyboard), Message::OpenShortcuts, p),
    ]
    .align_y(Alignment::Center)
    .width(Fill);

    let mut content = column![header, page_spacer(20.0)].spacing(0).width(Fill);

    // ---------------------------------------------------------------
    // Profile
    // ---------------------------------------------------------------

    content = content.push(section_title(
        Icon::Smile,
        "Your name",
        "Mochi will use this to greet you on the home screen.",
        p,
    ));
    content = content.push(page_spacer(10.0));

    let name_card = setting_card(
        column![
            text("Display name")
                .size(13.5)
                .font(theme::FONT_MEDIUM)
                .color(p.text),
            text_input("Your name…", &app.settings.username)
                .id("username")
                .on_input(Message::SetUsername)
                .padding(iced::Padding::from([10.0, 13.0]))
                .size(13.0)
                .width(400)
                .style(move |_theme, status| style::input(p, status)),
        ]
        .spacing(9)
        .width(Fill),
        p,
    );

    content = content.push(name_card);
    content = content.push(page_spacer(22.0));

    // ---------------------------------------------------------------
    // Appearance
    // ---------------------------------------------------------------

    content = content.push(section_title(
        Icon::Palette,
        "Appearance",
        "Six hand-mixed pastel palettes. Pick the one that matches your mood.",
        p,
    ));
    content = content.push(page_spacer(10.0));

    let mut themes = grid::Grid::new()
        .fluid(232.0)
        .spacing(12)
        .height(grid::Sizing::EvenlyDistribute(Length::Shrink));

    for id in theme::ThemeId::ALL {
        themes = themes.push(theme_tile(app, id));
    }

    content = content.push(themes);
    content = content.push(page_spacer(22.0));

    // ---------------------------------------------------------------
    // Editor
    // ---------------------------------------------------------------

    content = content.push(section_title(
        Icon::Pen,
        "Editor",
        "How writing feels while you are in the flow.",
        p,
    ));
    content = content.push(page_spacer(10.0));

    let font_size = app.settings.editor_font_size;

    let font_row = column![
        row![
            column![
                text("Writing size")
                    .size(13.5)
                    .font(theme::FONT_MEDIUM)
                    .color(p.text),
                text("A comfortable size for long, cozy sessions.")
                    .size(11.5)
                    .color(p.text_muted),
            ]
            .spacing(3)
            .width(Fill),
            container(
                text(format!("{font_size:.1} px"))
                    .size(11.5)
                    .font(theme::FONT_MEDIUM)
                    .color(p.primary),
            )
            .padding(iced::Padding::from([3.0, 10.0]))
            .style(move |_theme| style::badge(p, p.primary_soft)),
        ]
        .align_y(Alignment::Center)
        .width(Fill),
        slider(12.0..=22.0, font_size, Message::SetEditorFontSize)
            .step(0.5_f32)
            .width(Fill)
            .style(move |_theme, status| style::slider(p, status)),
    ]
    .spacing(12)
    .width(Fill);

    let autosave_delay = app.settings.autosave_delay_ms;

    let editor_card = setting_card(
        column![
            switch_row(
                "Autosave",
                "Save quietly while you type, so nothing is ever lost.",
                app.settings.autosave,
                Message::SetAutosave,
                p,
            ),
            divider(p),
            font_row,
            divider(p),
            row![
                column![
                    text("Autosave pause")
                        .size(13.5)
                        .font(theme::FONT_MEDIUM)
                        .color(p.text),
                    text("How long Mochi waits after you stop typing.")
                        .size(11.5)
                        .color(p.text_muted),
                ]
                .spacing(3)
                .width(Fill),
                row![
                    delay_pill(500, autosave_delay, p),
                    delay_pill(900, autosave_delay, p),
                    delay_pill(1500, autosave_delay, p),
                    delay_pill(2500, autosave_delay, p),
                ]
                .spacing(6)
                .align_y(Alignment::Center),
            ]
            .align_y(Alignment::Center)
            .width(Fill),
        ]
        .spacing(16)
        .width(Fill),
        p,
    );

    content = content.push(editor_card);
    content = content.push(page_spacer(22.0));

    // ---------------------------------------------------------------
    // Data & backups
    // ---------------------------------------------------------------

    content = content.push(section_title(
        Icon::Archive,
        "Your data",
        "Everything lives on this computer. Nothing leaves your desk.",
        p,
    ));
    content = content.push(page_spacer(10.0));

    let db_size = app.db.as_ref().map(|db| db.size_bytes()).unwrap_or(0);
    let backup_count = app.backups.len();

    let data_card = setting_card(
        column![
            row![
                stat(Icon::Note, app.notes.len().to_string(), "notes", p.primary, p),
                stat(Icon::Folder, app.folders.len().to_string(), "folders", p.pastel[1], p),
                stat(Icon::Tags, app.tags.len().to_string(), "tags", p.pastel[4], p),
                stat(Icon::Archive, text_utils::human_bytes(db_size), "library size", p.pastel[2], p),
            ]
            .spacing(28)
            .align_y(Alignment::Center),
            divider(p),
            row![
                column![
                    text("Backups")
                        .size(13.5)
                        .font(theme::FONT_MEDIUM)
                        .color(p.text),
                    text(format!(
                        "{} kept in your local backups folder.",
                        text_utils::plural(backup_count, "copy", "copies")
                    ))
                    .size(11.5)
                    .color(p.text_muted),
                ]
                .spacing(3)
                .width(Fill),
                row![
                    soft_button("Backup now", Some(Icon::Save), Message::CreateBackup, p),
                    ghost_button("Open folder", Some(Icon::FolderOpen), Message::OpenDataFolder, p),
                ]
                .spacing(8),
            ]
            .align_y(Alignment::Center)
            .width(Fill),
            switch_row(
                "Backup on launch",
                "Make a fresh safety copy when Mochi starts, at most twice a day.",
                app.settings.backup_on_start,
                Message::SetBackupOnStart,
                p,
            ),
            divider(p),
            backup_list(app),
            divider(p),
            row![
                column![
                    text("Export")
                        .size(13.5)
                        .font(theme::FONT_MEDIUM)
                        .color(p.text),
                    text("Write every note out as Markdown files.")
                        .size(11.5)
                        .color(p.text_muted),
                ]
                .spacing(3)
                .width(Fill),
                primary_button("Export all notes", Some(Icon::Download), Message::ExportAll, p),
            ]
            .align_y(Alignment::Center)
            .width(Fill),
        ]
        .spacing(16)
        .width(Fill),
        p,
    );

    content = content.push(data_card);
    content = content.push(page_spacer(22.0));

    // ---------------------------------------------------------------
    // About
    // ---------------------------------------------------------------

    let about = container(
        row![
            mascot::mascot(app.phase, p, 116.0),
            column![
                text("Mochi Notes")
                    .size(17.0)
                    .font(theme::FONT_BOLD)
                    .color(p.text),
                text("Version 1.0 · made with love, Rust and Iced")
                    .size(11.5)
                    .color(p.text_muted),
                text("A soft place for thoughts, built to stay on your machine.")
                    .size(11.5)
                    .color(p.text_muted),
                Space::new().height(2),
                text("Developed with love by Serhat")
                    .size(11.5)
                    .font(theme::FONT_MEDIUM)
                    .color(p.text_muted),
                row![
                    ghost_button(
                        "github/serhatonia",
                        None,
                        Message::OpenLink("https://github.com/serhatonia".to_string()),
                        p,
                    ),
                    ghost_button(
                        "x/serhatonia",
                        None,
                        Message::OpenLink("https://x.com/serhatonia".to_string()),
                        p,
                    ),
                ]
                .spacing(4),
                Space::new().height(2),
                row![
                    ghost_button("Shortcuts", Some(Icon::Keyboard), Message::OpenShortcuts, p),
                    ghost_button("Data folder", Some(Icon::FolderOpen), Message::OpenDataFolder, p),
                ]
                .spacing(6),
            ]
            .spacing(3),
        ]
        .spacing(18)
        .align_y(Alignment::Center),
    )
    .padding(iced::Padding::from([18.0, 22.0]))
    .width(Fill)
    .style(move |_theme| style::panel(p));

    content = content.push(about);
    content = content.push(page_spacer(20.0));

    page(content, p)
}

fn section_title<'a>(
    glyph: Icon,
    title: &str,
    subtitle: &str,
    p: theme::Palette,
) -> Element<'a, Message> {
    row![
        container(icon(glyph, 13.0, p.primary))
            .width(30)
            .height(30)
            .align_x(Alignment::Center)
            .align_y(Alignment::Center)
            .style(move |_theme| style::badge(p, p.primary_soft)),
        column![
            text(title.to_string())
                .size(15.0)
                .font(theme::FONT_BOLD)
                .color(p.text),
            text(subtitle.to_string())
                .size(11.5)
                .color(p.text_muted),
        ]
        .spacing(2),
    ]
    .spacing(10)
    .align_y(Alignment::Center)
    .into()
}

fn theme_tile<'a>(app: &'a Mochi, id: theme::ThemeId) -> Element<'a, Message> {
    let p = app.palette;
    let palette = id.palette();
    let selected = app.settings.theme == id;

    let swatches: Vec<Element<'a, Message>> = [
        palette.primary,
        palette.primary_soft,
        palette.pastel[1],
        palette.bg_alt,
        palette.text,
    ]
    .into_iter()
    .map(|color| {
        container(Space::new().width(0))
            .width(Length::Fill)
            .height(22)
            .style(move |_theme| container::Style {
                background: Some(iced::Background::Color(color)),
                border: Border {
                    color: Color::TRANSPARENT,
                    width: 0.0,
                    radius: 5.0.into(),
                },
                ..container::Style::default()
            })
            .into()
    })
    .collect();

    let check: Element<'a, Message> = if selected {
        icon(Icon::CheckCircle, 13.0, p.primary).into()
    } else {
        Space::new().width(0).into()
    };

    button(
        column![
            row(swatches).spacing(3),
            Space::new().height(8),
            row![
                text(id.name())
                    .size(12.5)
                    .font(theme::FONT_BOLD)
                    .color(if selected { p.primary } else { p.text }),
                Space::new().width(Fill),
                check,
            ]
            .align_y(Alignment::Center),
            text(id.tagline()).size(10.0).color(p.text_faint),
        ]
        .spacing(2)
        .width(Fill),
    )
    .padding(iced::Padding::from([10.0, 11.0]))
    .width(Fill)
    .style(move |_theme, status| {
        let hovered = matches!(status, button::Status::Hovered | button::Status::Pressed);
        button::Style {
            background: Some(iced::Background::Color(if selected {
                theme::mix(p.surface, p.primary_soft, 0.55)
            } else if hovered {
                p.surface_alt
            } else {
                p.surface
            })),
            text_color: p.text,
            border: Border {
                color: if selected { p.primary } else { p.border },
                width: if selected { 1.8 } else { 1.0 },
                radius: 16.0.into(),
            },
            shadow: if selected { p.shadow_soft() } else { iced::Shadow::default() },
            snap: false,
        }
    })
    .on_press(Message::SetTheme(id))
    .into()
}

fn delay_pill<'a>(ms: u64, current: u64, p: theme::Palette) -> Element<'a, Message> {
    let selected = current == ms;
    let label = if ms < 1000 {
        format!("{ms} ms")
    } else {
        format!("{:.1} s", ms as f32 / 1000.0)
    };

    button(
        text(label)
            .size(11.0)
            .font(if selected {
                theme::FONT_BOLD
            } else {
                theme::FONT
            })
            .color(if selected { p.on_primary } else { p.text_muted }),
    )
    .padding(iced::Padding::from([5.0, 10.0]))
    .style(move |_theme, status| style::chip(p, selected, status))
    .on_press(Message::SetAutosaveDelay(ms))
    .into()
}

fn backup_list(app: &Mochi) -> Element<'_, Message> {
    let p = app.palette;

    let mut column = column![row![
        text("Recent backups")
            .size(12.0)
            .font(theme::FONT_BOLD)
            .color(p.text_muted),
        Space::new().width(Fill),
        icon_button(Icon::Refresh, Message::RefreshBackups, p, false),
    ]
    .align_y(Alignment::Center)
    .width(Fill)]
    .spacing(6);

    if app.backups.is_empty() {
        column = column.push(
            container(
                text("No backups yet — Mochi will make one when you ask.")
                    .size(11.0)
                    .color(p.text_faint),
            )
            .padding(iced::Padding::from([10.0, 12.0]))
            .width(Fill)
            .style(move |_theme| style::toolbar(p)),
        );
    } else {
        for (index, entry) in app.backups.iter().take(4).enumerate() {
            column = column.push(backup_row(index, entry, p));
        }
    }

    column.width(Fill).into()
}

fn backup_row<'a>(
    index: usize,
    entry: &backup::BackupInfo,
    p: theme::Palette,
) -> Element<'a, Message> {
    row![
        icon(Icon::Save, 11.0, p.text_faint),
        text(text_utils::format_short_date(entry.modified))
            .size(11.5)
            .color(p.text),
        text(text_utils::human_bytes(entry.size))
            .size(10.5)
            .color(p.text_faint),
        Space::new().width(Fill),
        ghost_button(
            "Restore",
            Some(Icon::RotateLeft),
            Message::RequestRestoreBackup(index),
            p,
        ),
    ]
    .spacing(10)
    .align_y(Alignment::Center)
    .width(Fill)
    .into()
}
