//! Tag management screen.

use iced::widget::{button, column, container, grid, row, text, Space};
use iced::{Alignment, Element, Fill, Length};

use crate::app::Mochi;
use crate::message::{Message, Screen};
use crate::theme::{self, style};
use crate::utils::text as text_utils;
use crate::views::library::management_header;
use crate::views::{page, page_spacer};
use crate::widgets::components::{
    icon_button, icon_in_circle, plain_button_style, primary_button,
};
use crate::widgets::icons::{icon, Icon};

pub fn view(app: &Mochi) -> Element<'_, Message> {
    let p = app.palette;

    let header = management_header(
        "Tags",
        "Small labels that connect notes across folders.",
        primary_button("New tag", Some(Icon::Hashtag), Message::OpenTagForm(None), p),
        p,
    );

    let mut cards = grid::Grid::new()
        .fluid(240.0)
        .spacing(14)
        .height(grid::Sizing::EvenlyDistribute(Length::Shrink));

    for tag in &app.tags {
        let color = theme::parse_hex(&tag.color);
        let count = app.tag_count(tag.id);
        let hover = app.hover_progress(crate::message::Hover::Tag(tag.id));

        let card = container(
            column![
                row![
                    icon_in_circle(Icon::Hashtag, color, 36.0, p),
                    Space::new().width(Fill),
                    icon_button(Icon::Edit, Message::OpenTagForm(Some(tag.id)), p, false),
                    icon_button(Icon::Trash, Message::RequestDeleteTag(tag.id), p, false),
                ]
                .align_y(Alignment::Center),
                Space::new().height(10),
                button(
                    column![
                        text(format!("#{}", tag.name))
                            .size(15.0)
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
                .on_press(Message::Navigate(Screen::Tag(tag.id))),
                Space::new().height(8),
                container(
                    text("Browse tagged notes")
                        .size(10.0)
                        .color(theme::mix(p.text, color, 0.5)),
                )
                .padding(iced::Padding::from([2.0, 8.0]))
                .style(move |_theme| style::badge(p, color)),
            ]
            .spacing(0),
        )
        .padding(iced::Padding::from([16.0, 18.0]))
        .width(Fill)
        .style(move |_theme| style::card(p, hover));

        cards = cards.push(
            iced::widget::mouse_area(card)
                .on_enter(Message::HoverEnter(crate::message::Hover::Tag(tag.id)))
                .on_exit(Message::HoverExit(crate::message::Hover::Tag(tag.id))),
        );
    }

    cards = cards.push(new_tag_card(p));

    let mut content = column![header, page_spacer(20.0)].spacing(0).width(Fill);

    if app.tags.is_empty() {
        content = content.push(
            container(
                column![
                    icon(Icon::Tags, 26.0, p.text_faint),
                    text("No tags yet")
                        .size(16.0)
                        .font(theme::FONT_BOLD)
                        .color(p.text),
                    text("Tag a note with #ideas or #recipes to find it again in a blink.")
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

fn new_tag_card<'a>(p: theme::Palette) -> Element<'a, Message> {
    button(
        column![
            container(icon(Icon::Plus, 15.0, p.primary))
                .width(36)
                .height(36)
                .align_x(Alignment::Center)
                .align_y(Alignment::Center)
                .style(move |_theme| style::badge(p, p.primary_soft)),
            Space::new().height(8),
            text("New tag")
                .size(13.0)
                .font(theme::FONT_MEDIUM)
                .color(p.primary),
            text("Label notes your way")
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
    .on_press(Message::OpenTagForm(None))
    .into()
}
