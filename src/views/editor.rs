//! The note editor: title, markdown body, formatting toolbar and autosave
//! status.

use iced::widget::{button, column, container, markdown, row, scrollable, text, text_editor, text_input, Space};
use iced::{Alignment, Border, Element, Fill, Padding};

use crate::app::{EditorState, Mochi, SaveStatus};
use crate::models::Note;
use crate::message::{Format, Message};
use crate::theme::{self, style};
use crate::utils::text as text_utils;
use crate::widgets::components::icon_button;
use crate::widgets::icons::{icon, Icon};

pub fn view(app: &Mochi) -> Element<'_, Message> {
    let p = app.palette;

    let Some(editor) = &app.editor else {
        return container(text("No note is open").color(p.text_muted))
            .width(Fill)
            .height(Fill)
            .align_x(Alignment::Center)
            .align_y(Alignment::Center)
            .into();
    };

    let Some(note) = app.note(editor.note_id) else {
        return container(text("This note could not be found").color(p.text_muted))
            .width(Fill)
            .height(Fill)
            .align_x(Alignment::Center)
            .align_y(Alignment::Center)
            .into();
    };

    let id = note.id;

    if app.focus_mode {
        return focus_view(app, editor, note);
    }

    // -----------------------------------------------------------------
    // Top bar
    // -----------------------------------------------------------------

    let title = text_input("Untitled note", &note.title)
        .id(crate::app::TITLE_ID)
        .on_input(Message::TitleChanged)
        .padding(Padding::from([2.0, 0.0]))
        .size(21.0)
        .font(theme::FONT_BOLD)
        .width(Fill)
        .style(move |_theme, status| style::title_input(p, status));

    let top = row![
        icon_button(Icon::ArrowLeft, Message::CloseEditor, p, false),
        container(title).width(Fill),
        save_chip(app, p),
        icon_button(Icon::Star, Message::ToggleFavorite(id), p, note.favorite),
        icon_button(Icon::Pin, Message::TogglePin(id), p, note.pinned),
        icon_button(
            Icon::Expand,
            Message::ToggleFocusMode,
            p,
            false
        ),
        icon_button(
            Icon::Eye,
            Message::TogglePreview,
            p,
            editor.preview.is_some()
        ),
        icon_button(Icon::Download, Message::ExportNote(id), p, false),
        icon_button(Icon::Copy, Message::DuplicateNote(id), p, false),
        icon_button(Icon::Archive, Message::ArchiveNote(id), p, false),
        icon_button(Icon::Trash, Message::TrashNote(id), p, false),
    ]
    .spacing(3)
    .align_y(Alignment::Center)
    .width(Fill);

    // -----------------------------------------------------------------
    // Meta row: folder and tags
    // -----------------------------------------------------------------

    let folder_label = note
        .folder_id
        .and_then(|folder_id| app.folder(folder_id))
        .map(|folder| folder.name.clone())
        .unwrap_or_else(|| "Add to folder".to_string());

    let folder_color = note
        .folder_id
        .and_then(|folder_id| app.folder(folder_id))
        .map(|folder| theme::parse_hex(&folder.color))
        .unwrap_or(p.text_faint);

    let folder_button = button(
        row![
            icon(Icon::Folder, 10.5, folder_color),
            text(folder_label).size(11.0).color(p.text_muted),
        ]
        .spacing(6)
        .align_y(Alignment::Center),
    )
    .padding(Padding::from([4.0, 9.0]))
    .style(move |_theme, status| style::chip(p, false, status))
    .on_press(Message::OpenFolderPicker);

    let mut meta = row![folder_button].spacing(6).align_y(Alignment::Center);

    for tag_id in &note.tags {
        if let Some(tag) = app.tag(*tag_id) {
            let tint = theme::parse_hex(&tag.color);
            let chip = button(
                row![
                    text(format!("#{}", tag.name))
                        .size(10.5)
                        .font(theme::FONT_MEDIUM)
                        .color(theme::mix(p.text, tint, 0.45)),
                    icon(Icon::Close, 8.0, theme::mix(p.text, tint, 0.4)),
                ]
                .spacing(5)
                .align_y(Alignment::Center),
            )
            .padding(Padding::from([4.0, 8.0]))
            .style(move |_theme, status| style::tag_chip(p, tint, status))
            .on_press(Message::ToggleTagOnEditor(*tag_id));

            meta = meta.push(chip);
        }
    }

    meta = meta.push(
        button(
            row![
                icon(Icon::Plus, 9.0, p.text_faint),
                text("tag").size(10.5).color(p.text_faint),
            ]
            .spacing(4)
            .align_y(Alignment::Center),
        )
        .padding(Padding::from([4.0, 9.0]))
        .style(move |_theme, status| style::button_ghost(p, status))
        .on_press(Message::OpenTagPicker),
    );

    meta = meta.push(Space::new().width(Fill));
    meta = meta.push(
        text(format!(
            "Edited {} · {}",
            text_utils::relative_time(note.updated_at),
            text_utils::plural(note.word_count(), "word", "words")
        ))
        .size(10.5)
        .color(p.text_faint),
    );

    // -----------------------------------------------------------------
    // Formatting toolbar
    // -----------------------------------------------------------------

    let toolbar = row![
        fmt_icon(Icon::Bold, Format::Bold, "Bold  (Ctrl+B)", p),
        fmt_icon(Icon::Italic, Format::Italic, "Italic  (Ctrl+I)", p),
        fmt_icon(Icon::Strike, Format::Strike, "Strikethrough", p),
        fmt_icon(Icon::Code, Format::InlineCode, "Inline code", p),
        divider_dot(p),
        fmt_text("H1", Format::Heading1, "Heading 1", p),
        fmt_text("H2", Format::Heading2, "Heading 2", p),
        divider_dot(p),
        fmt_icon(Icon::Quote, Format::Quote, "Quote", p),
        fmt_icon(Icon::List, Format::Bullet, "Bullet list", p),
        fmt_text("1.", Format::Numbered, "Numbered list", p),
        fmt_icon(Icon::CheckSquare, Format::Checkbox, "Checklist", p),
        fmt_icon(Icon::Link, Format::Link, "Link", p),
        Space::new().width(Fill),
        row![
            icon(
                Icon::Clock,
                10.0,
                if editor.dirty { p.warning } else { p.text_faint }
            ),
            text(if editor.dirty {
                "Unsaved changes"
            } else {
                "All changes saved"
            })
            .size(10.5)
            .color(if editor.dirty { p.warning } else { p.text_faint }),
        ]
        .spacing(6)
        .align_y(Alignment::Center),
    ]
    .spacing(3)
    .align_y(Alignment::Center)
    .width(Fill);

    // -----------------------------------------------------------------
    // Body
    // -----------------------------------------------------------------

    let body = editor_body(app, editor, note);

    let body_card = container(body)
        .padding(Padding::from([22.0, 26.0]))
        .width(Fill)
        .height(Fill)
        .style(move |_theme| style::panel(p));

    // -----------------------------------------------------------------
    // Footer
    // -----------------------------------------------------------------

    let footer = row![
        text(format!(
            "{} · {} · {} read",
            text_utils::plural(editor.words, "word", "words"),
            text_utils::plural(editor.chars, "character", "characters"),
            text_utils::plural(
                text_utils::reading_minutes(editor.words).max(1),
                "minute",
                "minutes"
            )
        ))
        .size(10.5)
        .color(p.text_faint),
        Space::new().width(Fill),
        text("Markdown supported")
            .size(10.0)
            .color(p.text_faint),
        icon(Icon::Info, 9.5, p.text_faint),
    ]
    .spacing(8)
    .align_y(Alignment::Center)
    .width(Fill);

    container(
        column![
            container(top).padding(Padding::from([10.0, 18.0])),
            container(meta).padding(theme::pad(2.0, 20.0, 8.0, 20.0)),
            container(toolbar)
                .padding(Padding::from([6.0, 18.0]))
                .width(Fill)
                .style(move |_theme| style::toolbar(p)),
            container(body_card)
                .padding(Padding::from([14.0, 20.0]))
                .height(Fill),
            container(footer).padding(theme::pad(8.0, 22.0, 12.0, 22.0)),
        ]
        .spacing(0)
        .width(Fill)
        .height(Fill),
    )
    .width(Fill)
    .height(Fill)
    .style(move |_theme| style::app(p))
    .into()
}

fn fmt_icon<'a>(glyph: Icon, format: Format, tip: &'static str, p: theme::Palette) -> Element<'a, Message> {
    let _ = tip;
    button(icon(glyph, 12.0, p.text_muted))
        .padding(Padding::from([6.0, 8.0]))
        .style(move |_theme, status| style::button_icon(p, false, status))
        .on_press(Message::ApplyFormat(format))
        .into()
}

fn fmt_text<'a>(label: &'static str, format: Format, tip: &'static str, p: theme::Palette) -> Element<'a, Message> {
    let _ = tip;
    button(
        text(label)
            .size(11.0)
            .font(theme::FONT_BOLD)
            .color(p.text_muted),
    )
    .padding(Padding::from([6.0, 8.0]))
    .style(move |_theme, status| style::button_icon(p, false, status))
    .on_press(Message::ApplyFormat(format))
    .into()
}

fn divider_dot<'a>(p: theme::Palette) -> Element<'a, Message> {
    container(
        container(Space::new().width(1))
            .height(16)
            .style(move |_theme| container::Style {
                background: Some(iced::Background::Color(p.border)),
                ..container::Style::default()
            }),
    )
    .padding(Padding::from([0.0, 5.0]))
    .into()
}

fn editor_body<'a>(
    app: &'a Mochi,
    editor: &'a EditorState,
    note: &Note,
) -> Element<'a, Message> {
    let p = app.palette;

    if let Some(preview) = &editor.preview {
        let mut markdown_style = markdown::Style::from(&app.iced_theme);
        markdown_style.font = theme::FONT;
        markdown_style.inline_code_highlight.background =
            iced::Background::Color(p.surface_alt);
        markdown_style.inline_code_highlight.border = Border {
            color: p.border,
            width: 1.0,
            radius: 5.0.into(),
        };
        markdown_style.inline_code_padding = Padding::from([1.0, 5.0]);
        markdown_style.inline_code_color = p.primary;
        markdown_style.inline_code_font = theme::FONT_MONO;
        markdown_style.code_block_font = theme::FONT_MONO;
        markdown_style.link_color = p.primary;

        let settings =
            markdown::Settings::with_text_size(app.settings.editor_font_size, markdown_style);

        let rendered: Element<'_, Message> = if note.content.trim().is_empty() {
            column![
                text("Nothing to preview yet")
                    .size(15.0)
                    .font(theme::FONT_BOLD)
                    .color(p.text_muted),
                text("Write a few words in the editor, then peek again.")
                    .size(12.0)
                    .color(p.text_faint),
            ]
            .spacing(4)
            .into()
        } else {
            markdown::view(preview.items(), settings).map(Message::OpenLink)
        };

        scrollable(container(rendered).padding(Padding::from([4.0, 2.0])))
            .style(move |_theme, status| style::scrollable(p, status))
            .height(Fill)
            .into()
    } else {
        text_editor(&editor.content)
            .id(crate::app::EDITOR_ID)
            .placeholder("Start writing something lovely…")
            .on_action(Message::ContentAction)
            .padding(0)
            .size(app.settings.editor_font_size)
            .font(theme::FONT)
            .height(Fill)
            .style(move |_theme, status| style::editor(p, status))
            .into()
    }
}

/// Distraction-free editor: no sidebar, no toolbar, no chrome — just the note.
fn focus_view<'a>(app: &'a Mochi, editor: &'a EditorState, note: &Note) -> Element<'a, Message> {
    let p = app.palette;

    let badge = container(
        row![
            icon(Icon::Expand, 10.0, p.primary),
            text("Focus mode")
                .size(11.5)
                .font(theme::FONT_BOLD)
                .color(p.text_muted),
        ]
        .spacing(7)
        .align_y(Alignment::Center),
    )
    .padding(Padding::from([4.0, 10.0]))
    .style(move |_theme| style::badge(p, p.primary_soft));

    let top = row![
        badge,
        Space::new().width(Fill),
        text("Esc or Ctrl+Shift+F to exit")
            .size(10.5)
            .color(p.text_faint),
        icon_button(Icon::Compress, Message::ExitFocus, p, false),
    ]
    .spacing(12)
    .align_y(Alignment::Center)
    .width(Fill);

    let title = text_input("Untitled note", &note.title)
        .id(crate::app::TITLE_ID)
        .on_input(Message::TitleChanged)
        .padding(Padding::from([2.0, 0.0]))
        .size(24.0)
        .font(theme::FONT_BOLD)
        .width(Fill)
        .style(move |_theme, status| style::title_input(p, status));

    let footer = row![
        text(format!(
            "{} · {}",
            text_utils::plural(editor.words, "word", "words"),
            text_utils::plural(editor.chars, "character", "characters")
        ))
        .size(10.5)
        .color(p.text_faint),
        Space::new().width(Fill),
        save_chip(app, p),
    ]
    .spacing(10)
    .align_y(Alignment::Center)
    .width(Fill);

    let content = column![
        top,
        Space::new().height(20),
        title,
        Space::new().height(20),
        container(editor_body(app, editor, note)).height(Fill),
        Space::new().height(14),
        footer,
    ]
    .spacing(0)
    .max_width(760)
    .width(Fill)
    .height(Fill);

    container(content)
        .width(Fill)
        .height(Fill)
        .padding(Padding::from([20.0, 46.0]))
        .style(move |_theme| style::app(p))
        .into()
}

fn save_chip<'a>(app: &'a Mochi, p: theme::Palette) -> Element<'a, Message> {
    let (glyph, label, color) = match &app.save_status {
        SaveStatus::Idle => (Icon::Check, "Ready".to_string(), p.text_faint),
        SaveStatus::Editing => (Icon::Clock, "Saving…".to_string(), p.warning),
        SaveStatus::Saved(time) => (
            Icon::CheckCircle,
            format!("Saved {}", text_utils::relative_time(*time)),
            p.success,
        ),
        SaveStatus::Error(_) => (Icon::CircleX, "Not saved".to_string(), p.danger),
    };

    container(
        row![icon(glyph, 9.5, color), text(label).size(10.0).color(color)]
            .spacing(5)
            .align_y(Alignment::Center),
    )
    .padding(Padding::from([4.0, 9.0]))
    .style(move |_theme| style::badge(p, color))
    .into()
}

