//! A small floating toast for save, export and backup feedback.

use iced::widget::{container, mouse_area, row, text, Space};
use iced::{Alignment, Border, Color, Element, Fill, Padding};

use crate::app::{Mochi, Toast};
use crate::message::{Message, ToastKind};
use crate::theme::{self, style};
use crate::widgets::icons::{icon, Icon};

pub fn view<'a>(app: &'a Mochi, toast: &'a Toast) -> Element<'a, Message> {
    let p = app.palette;

    let (glyph, tint) = match toast.kind {
        ToastKind::Success => (Icon::CheckCircle, p.success),
        ToastKind::Info => (Icon::Info, p.primary),
        ToastKind::Error => (Icon::CircleX, p.danger),
    };

    let card = container(
        row![
            container(icon(glyph, 12.0, tint))
                .width(26)
                .height(26)
                .align_x(Alignment::Center)
                .align_y(Alignment::Center)
                .style(move |_theme| style::badge(p, tint)),
            text(toast.text.clone())
                .size(12.0)
                .font(theme::FONT_MEDIUM)
                .color(p.text)
                .width(iced::Length::Shrink),
            Space::new().width(4),
            icon(Icon::Close, 9.0, p.text_faint),
        ]
        .spacing(9)
        .align_y(Alignment::Center),
    )
    .padding(Padding::from([10.0, 13.0]))
    .style(move |_theme| container::Style {
        background: Some(iced::Background::Color(p.surface)),
        text_color: Some(p.text),
        border: Border {
            color: p.border_strong,
            width: 1.0,
            radius: 14.0.into(),
        },
        shadow: p.shadow_lifted(),
        snap: false,
    });

    container(
        mouse_area(card).on_press(Message::DismissToast),
    )
    .width(Fill)
    .height(Fill)
    .padding(theme::pad(0.0, 22.0, 22.0, 0.0))
    .align_x(iced::Alignment::End)
    .align_y(iced::Alignment::End)
    .into()
}

/// A subtle divider used between toast sections.
#[allow(dead_code)]
fn hairline<'a>(color: Color) -> Element<'a, Message> {
    container(Space::new().height(1))
        .width(Fill)
        .style(move |_theme| container::Style {
            background: Some(iced::Background::Color(color)),
            ..container::Style::default()
        })
        .into()
}
