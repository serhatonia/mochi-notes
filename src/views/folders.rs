//! Folder management screen.

use iced::widget::{button, column, container, grid, row, text, Space};
use iced::{Alignment, Element, Fill, Length};

use crate::app::Mochi;
use crate::message::Message;
use crate::theme::{self, style};
use crate::utils::text as text_utils;
use crate::views::library::management_header;
use crate::views::{page, page_spacer};
use crate::widgets::components::{
    dot, icon_button, icon_in_circle, plain_button_style, primary_button,
};
use crate::widgets::icons::{icon, Icon};

pub fn view(app: &Mochi) -> Element<'_, Message> {
    let p = app.palette;

    let header = management_header(
        "Folders",
        "Little homes for your notes. Give each one its own color.",
        primary_button("New folder", Some(Icon::FolderPlus), Message::OpenFolderForm(None), p),
        p,
    );

    let mut cards = grid::Grid::new()
        .fluid(272.0)
        .spacing(14)
        .height(grid::Sizing::EvenlyDistribute(Length::Shrink));

    for folder in &app.folders {
        let color = theme::parse_hex(&folder.color);
        let count = app.folder_count(folder.id);
        let hover = app.hover_progress(crate::message::Hover::Folder(folder.id));

        let card = container(
            column![
                row![
                    icon_in_circle(Icon::FolderOpen, color, 38.0, p),
                    Space::new().width(Fill),
                    icon_button(Icon::Edit, Message::OpenFolderForm(Some(folder.id)), p, false),
                    icon_button(Icon::Trash, Message::RequestDeleteFolder(folder.id), p, false),
                ]
                .align_y(Alignment::Center),
                Space::new().height(10),
                button(
                    column![
                        text(folder.name.clone())
                            .size(15.5)
                            .font(theme::FONT_BOLD)
                            .color(p.text)
                            .wrapping(iced::widget::text::Wrapping::Word),
                        text(text_utils::plural(count, "note", "notes"))
                            .size(11.5)
                            .color(p.text_muted),
                    ]
                    .spacing(3)
                    .width(Fill),
                )
                .padding(0)
                .width(Fill)
                .style(plain_button_style)
                .on_press(Message::Navigate(crate::message::Screen::Folder(folder.id))),
                Space::new().height(8),
                row![
                    dot(color, 7.0),
                    text(format!("Created {}", text_utils::relative_time(folder.created_at)))
                        .size(10.0)
                        .color(p.text_faint),
                ]
                .spacing(6)
                .align_y(Alignment::Center),
            ]
            .spacing(0),
        )
        .padding(iced::Padding::from([16.0, 18.0]))
        .width(Fill)
        .style(move |_theme| style::card(p, hover));

        cards = cards.push(iced::widget::mouse_area(card)
            .on_enter(Message::HoverEnter(crate::message::Hover::Folder(folder.id)))
            .on_exit(Message::HoverExit(crate::message::Hover::Folder(folder.id))));
    }

    cards = cards.push(new_folder_card(p));

    let mut content = column![header, page_spacer(20.0)].spacing(0).width(Fill);

    if app.folders.is_empty() {
        content = content.push(
            container(
                column![
                    icon(Icon::Folder, 26.0, p.text_faint),
                    text("No folders yet")
                        .size(16.0)
                        .font(theme::FONT_BOLD)
                        .color(p.text),
                    text("Folders keep related notes together, like little labeled boxes.")
                        .size(12.0)
                        .color(p.text_muted),
                ]
                .spacing(8)
                .align_x(Alignment::Center),
            )
            .padding(40)
            .width(Fill)
            .style(move |_theme| style::dashed(p)),
        );
        content = content.push(page_spacer(16.0));
    } else {
        content = content.push(cards);
        content = content.push(page_spacer(16.0));
    }

    page(content, p)
}

fn new_folder_card<'a>(p: theme::Palette) -> Element<'a, Message> {
    button(
        column![
            container(icon(Icon::Plus, 16.0, p.primary))
                .width(38)
                .height(38)
                .align_x(Alignment::Center)
                .align_y(Alignment::Center)
                .style(move |_theme| style::badge(p, p.primary_soft)),
            Space::new().height(8),
            text("New folder")
                .size(13.5)
                .font(theme::FONT_MEDIUM)
                .color(p.primary),
            text("Group a few notes together")
                .size(10.5)
                .color(p.text_faint),
        ]
        .spacing(0)
        .align_x(Alignment::Center),
    )
    .padding(iced::Padding::from([20.0, 18.0]))
    .width(Fill)
    .height(Length::Fill)
    .style(move |_theme, _status| button::Style {
        background: Some(iced::Background::Color(theme::alpha(p.primary, 0.035))),
        text_color: p.primary,
        border: iced::Border {
            color: theme::mix(p.border_strong, p.primary, 0.35),
            width: 1.5,
            radius: theme::radius::CARD.into(),
        },
        shadow: iced::Shadow::default(),
        snap: false,
    })
    .on_press(Message::OpenFolderForm(None))
    .into()
}
