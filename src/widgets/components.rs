//! Reusable UI building blocks with the Mochi look and feel.

use iced::widget::{
    button, column, container, mouse_area, row, text, text_input, toggler, Button, Space, Text,
};
use iced::{Alignment, Background, Border, Color, Element, Fill, Padding, Shadow};

use crate::message::{Hover, Message};
use crate::theme::{self, style, Palette};
use crate::widgets::icons::{icon, Icon};
use crate::widgets::mascot;

pub fn plain_button_style(_theme: &iced::Theme, _status: button::Status) -> button::Style {
    button::Style {
        background: None,
        text_color: Color::TRANSPARENT,
        border: Border::default(),
        shadow: Shadow::default(),
        snap: false,
    }
}

pub fn icon_button<'a>(
    glyph: Icon,
    message: Message,
    palette: Palette,
    active: bool,
) -> Element<'a, Message> {
    button(icon(
        glyph,
        14.0,
        if active { palette.primary } else { palette.text_muted },
    ))
    .padding(Padding::from([8.0, 9.0]))
    .style(move |_theme, status| style::button_icon(palette, active, status))
    .on_press(message)
    .into()
}

pub fn icon_button_lg<'a>(
    glyph: Icon,
    message: Message,
    palette: Palette,
    active: bool,
) -> Element<'a, Message> {
    button(icon(
        glyph,
        16.0,
        if active { palette.primary } else { palette.text_muted },
    ))
    .padding(Padding::from([9.0, 10.0]))
    .style(move |_theme, status| style::button_icon(palette, active, status))
    .on_press(message)
    .into()
}

pub fn soft_icon_button<'a>(
    glyph: Icon,
    message: Message,
    palette: Palette,
) -> Element<'a, Message> {
    button(icon(glyph, 14.0, palette.primary))
        .padding(Padding::from([8.0, 9.0]))
        .style(move |_theme, status| style::button_soft(palette, status))
        .on_press(message)
        .into()
}

pub fn primary_button<'a>(
    label: impl Into<String>,
    glyph: Option<Icon>,
    message: Message,
    palette: Palette,
) -> Element<'a, Message> {
    let label = label.into();
    let content: Element<'a, Message> = if let Some(glyph) = glyph {
        row![
            icon(glyph, 13.0, palette.on_primary),
            text(label).size(13.5).font(theme::FONT_BOLD).color(palette.on_primary)
        ]
        .spacing(8)
        .align_y(Alignment::Center)
        .into()
    } else {
        text(label)
            .size(13.5)
            .font(theme::FONT_BOLD)
            .color(palette.on_primary)
            .into()
    };

    button(content)
        .padding(Padding::from([10.0, 18.0]))
        .height(40)
        .style(move |_theme, status| style::button_primary(palette, status))
        .on_press(message)
        .into()
}

pub fn soft_button<'a>(
    label: impl Into<String>,
    glyph: Option<Icon>,
    message: Message,
    palette: Palette,
) -> Element<'a, Message> {
    let label = label.into();
    let content: Element<'a, Message> = if let Some(glyph) = glyph {
        row![
            icon(glyph, 12.5, palette.primary),
            text(label)
                .size(13.0)
                .font(theme::FONT_MEDIUM)
                .color(palette.primary)
        ]
        .spacing(7)
        .align_y(Alignment::Center)
        .into()
    } else {
        text(label)
            .size(13.0)
            .font(theme::FONT_MEDIUM)
            .color(palette.primary)
            .into()
    };

    button(content)
        .padding(Padding::from([9.0, 15.0]))
        .style(move |_theme, status| style::button_soft(palette, status))
        .on_press(message)
        .into()
}

pub fn ghost_button<'a>(
    label: impl Into<String>,
    glyph: Option<Icon>,
    message: Message,
    palette: Palette,
) -> Element<'a, Message> {
    let label = label.into();
    let content: Element<'a, Message> = if let Some(glyph) = glyph {
        row![
            icon(glyph, 12.5, palette.text_muted),
            text(label).size(13.0).color(palette.text_muted)
        ]
        .spacing(7)
        .align_y(Alignment::Center)
        .into()
    } else {
        text(label).size(13.0).color(palette.text_muted).into()
    };

    button(content)
        .padding(Padding::from([9.0, 14.0]))
        .style(move |_theme, status| style::button_ghost(palette, status))
        .on_press(message)
        .into()
}

pub fn danger_button<'a>(
    label: impl Into<String>,
    glyph: Option<Icon>,
    message: Message,
    palette: Palette,
) -> Element<'a, Message> {
    let label = label.into();
    let content: Element<'a, Message> = if let Some(glyph) = glyph {
        row![
            icon(glyph, 12.5, palette.danger),
            text(label).size(13.0).font(theme::FONT_MEDIUM).color(palette.danger)
        ]
        .spacing(7)
        .align_y(Alignment::Center)
        .into()
    } else {
        text(label)
            .size(13.0)
            .font(theme::FONT_MEDIUM)
            .color(palette.danger)
            .into()
    };

    button(content)
        .padding(Padding::from([9.0, 15.0]))
        .style(move |_theme, status| style::button_danger(palette, status))
        .on_press(message)
        .into()
}

pub fn pill<'a>(
    label: impl Into<String>,
    selected: bool,
    message: Message,
    palette: Palette,
) -> Element<'a, Message> {
    button(
        text(label.into())
            .size(12.0)
            .font(if selected {
                theme::FONT_BOLD
            } else {
                theme::FONT
            })
            .color(if selected {
                palette.on_primary
            } else {
                palette.text_muted
            }),
    )
    .padding(Padding::from([6.0, 12.0]))
    .style(move |_theme, status| style::chip(palette, selected, status))
    .on_press(message)
    .into()
}

/// A colored tag chip. When `message` is `None` it is purely decorative.
pub fn tag_chip<'a>(
    label: impl Into<String>,
    tint: Color,
    message: Option<Message>,
    palette: Palette,
) -> Element<'a, Message> {
    let label = label.into();
    let content = row![
        icon(Icon::Tag, 9.5, theme::mix(palette.text, tint, 0.45)),
        text(label)
            .size(11.0)
            .font(theme::FONT_MEDIUM)
            .color(theme::mix(palette.text, tint, 0.45))
    ]
    .spacing(5)
    .align_y(Alignment::Center);

    let chip: Element<'a, Message> = container(content)
        .padding(Padding::from([3.0, 8.0]))
        .style(move |_theme| container::Style {
            background: Some(Background::Color(theme::mix(palette.surface, tint, 0.22))),
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 999.0.into(),
            },
            ..container::Style::default()
        })
        .into();

    if let Some(message) = message {
        button(chip)
            .padding(0)
            .style(plain_button_style)
            .on_press(message)
            .into()
    } else {
        chip
    }
}

pub fn count_badge<'a>(count: usize, palette: Palette, active: bool) -> Element<'a, Message> {
    container(
        text(count.to_string())
            .size(10.5)
            .font(theme::FONT_MEDIUM)
            .color(if active {
                palette.primary
            } else {
                palette.text_faint
            }),
    )
    .padding(Padding::from([1.0, 8.0]))
    .style(move |_theme| container::Style {
        background: Some(Background::Color(if active {
            theme::mix(palette.primary_soft, palette.surface, 0.4)
        } else {
            theme::alpha(palette.text_faint, 0.10)
        })),
        border: Border {
            color: iced::Color::TRANSPARENT,
            width: 0.0,
            radius: 999.0.into(),
        },
        ..container::Style::default()
    })
    .into()
}

pub fn nav_item<'a>(
    glyph: Icon,
    label: impl Into<String>,
    count: Option<usize>,
    selected: bool,
    hovered: bool,
    key: Hover,
    message: Message,
    palette: Palette,
    hover_t: f32,
) -> Element<'a, Message> {
    let label = label.into();
    let color = if selected {
        palette.primary
    } else if hovered {
        palette.text
    } else {
        palette.text_muted
    };

    // A fixed-width slot keeps every glyph's left edge on the same line,
    // regardless of the font's individual side bearings.
    let mut content = row![
        container(icon(glyph, 14.0, color))
            .width(16.0)
            .align_x(Alignment::Center),
        text(label)
            .size(13.0)
            .font(if selected {
                theme::FONT_BOLD
            } else {
                theme::FONT
            })
            .color(color)
    ]
    .spacing(9)
    .align_y(Alignment::Center);

    if let Some(count) = count {
        content = content.push(Space::new().width(Fill));
        content = content.push(count_badge(count, palette, selected));
    }

    let hover = if hovered { hover_t } else { 0.0 };

    mouse_area(
        button(
            container(content)
                .padding(Padding::from([7.0, 13.0]))
                .width(Fill)
                .style(move |_theme| style::nav(palette, selected, hover)),
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

pub fn folder_row<'a>(
    name: impl Into<String>,
    color: Color,
    count: usize,
    selected: bool,
    hover_t: f32,
    message: Message,
    palette: Palette,
) -> Element<'a, Message> {
    let name = name.into();
    let hover = if selected { 1.0 } else { hover_t };
    let content = row![
        container(Space::new().width(7).height(7))
            .style(move |_theme| container::Style {
                background: Some(Background::Color(color)),
                border: Border {
                    color: Color::TRANSPARENT,
                    width: 0.0,
                    radius: 999.0.into(),
                },
                ..container::Style::default()
            }),
        text(name)
            .size(12.5)
            .font(if selected {
                theme::FONT_MEDIUM
            } else {
                theme::FONT
            })
            .color(if selected { palette.text } else { palette.text_muted }),
        Space::new().width(Fill),
        text(count.to_string())
            .size(10.5)
            .color(palette.text_faint)
    ]
    .spacing(9)
    .align_y(Alignment::Center);

    button(
        container(content)
            .padding(Padding::from([7.0, 11.0]))
            .width(Fill)
            .style(move |_theme| style::nav(palette, false, hover * 0.55)),
    )
    .padding(0)
    .width(Fill)
    .style(plain_button_style)
    .on_press(message)
    .into()
}

pub fn section_label<'a>(label: impl Into<String>, palette: Palette) -> Element<'a, Message> {
    text(label.into())
        .size(10.0)
        .font(theme::FONT_BOLD)
        .color(palette.text_faint)
        .into()
}

pub fn section_header<'a>(
    title: impl Into<String>,
    subtitle: Option<String>,
    action: Option<Element<'a, Message>>,
    palette: Palette,
) -> Element<'a, Message> {
    let mut info = column![text(title.into())
        .size(theme::size::H2)
        .font(theme::FONT_BOLD)
        .color(palette.text)]
    .spacing(3);

    if let Some(subtitle) = subtitle {
        info = info.push(
            text(subtitle)
                .size(12.5)
                .color(palette.text_muted)
                .line_height(1.4),
        );
    }

    let mut header = row![info, Space::new().width(Fill)].align_y(Alignment::Center);
    if let Some(action) = action {
        header = header.push(action);
    }

    header.into()
}

pub fn divider<'a>(palette: Palette) -> Element<'a, Message> {
    container(Space::new().height(1))
        .width(Fill)
        .style(move |_theme| container::Style {
            background: Some(Background::Color(palette.border)),
            ..container::Style::default()
        })
        .into()
}

pub fn search_bar<'a>(
    value: &str,
    width: impl Into<iced::Length>,
    palette: Palette,
) -> Element<'a, Message> {
    text_input("Search notes…", value)
        .id("search")
        .on_input(Message::SearchChanged)
        .padding(Padding::from([10.0, 14.0]))
        .size(13.0)
        .width(width)
        .icon(text_input::Icon {
            font: theme::FONT_ICON,
            code_point: Icon::Search.glyph(),
            size: Some(iced::Pixels(13.0)),
            spacing: 9.0,
            side: text_input::Side::Left,
        })
        .style(move |_theme, status| style::input_soft(palette, status))
        .into()
}

pub fn stat<'a>(
    glyph: Icon,
    value: impl Into<String>,
    label: impl Into<String>,
    tint: Color,
    palette: Palette,
) -> Element<'a, Message> {
    let icon_circle = container(icon(glyph, 13.0, theme::mix(palette.text, tint, 0.6)))
        .width(32)
        .height(32)
        .align_x(Alignment::Center)
        .align_y(Alignment::Center)
        .style(move |_theme| container::Style {
            background: Some(Background::Color(theme::mix(palette.surface, tint, 0.4))),
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 10.0.into(),
            },
            ..container::Style::default()
        });

    row![
        icon_circle,
        column![
            text(value.into())
                .size(15.0)
                .font(theme::FONT_BOLD)
                .color(palette.text),
            text(label.into())
                .size(10.5)
                .color(palette.text_muted)
        ]
        .spacing(1)
    ]
    .spacing(10)
    .align_y(Alignment::Center)
    .into()
}

pub fn switch_row<'a>(
    title: impl Into<String>,
    description: impl Into<String>,
    value: bool,
    on_toggle: impl Fn(bool) -> Message + 'a,
    palette: Palette,
) -> Element<'a, Message> {
    row![
        column![
            text(title.into())
                .size(13.5)
                .font(theme::FONT_MEDIUM)
                .color(palette.text),
            text(description.into())
                .size(11.5)
                .color(palette.text_muted)
                .line_height(1.4)
        ]
        .spacing(3)
        .width(Fill),
        toggler(value)
            .on_toggle(on_toggle)
            .style(move |_theme, status| style::toggler(palette, status))
    ]
    .spacing(16)
    .align_y(Alignment::Center)
    .width(Fill)
    .into()
}

pub fn setting_card<'a>(
    content: impl Into<Element<'a, Message>>,
    palette: Palette,
) -> Element<'a, Message> {
    container(content.into())
        .padding(20)
        .width(Fill)
        .style(move |_theme| style::panel(palette))
        .into()
}

pub fn empty_state<'a>(
    phase: f32,
    title: impl Into<String>,
    body: impl Into<String>,
    action: Option<(Icon, &'static str, Message)>,
    bubble: Option<(&str, f32)>,
    palette: Palette,
) -> Element<'a, Message> {
    let mut mascot_row = row![].spacing(8).align_y(Alignment::Center);
    if let Some((message, alpha)) = bubble {
        mascot_row = mascot_row.push(mascot::bubble(message, palette, alpha));
    }
    mascot_row = mascot_row.push(mascot::clickable(phase, palette, 132.0));

    let mut content = column![].spacing(8).align_x(Alignment::Center);
    content = content.push(mascot_row);
    content = content.push(
        text(title.into())
            .size(19.0)
            .font(theme::FONT_BOLD)
            .color(palette.text),
    );
    content = content.push(
        text(body.into())
            .size(12.5)
            .color(palette.text_muted)
            .line_height(1.5)
            .align_x(Alignment::Center)
            .width(360),
    );

    if let Some((glyph, label, message)) = action {
        content = content.push(Space::new().height(8));
        content = content.push(soft_button(label, Some(glyph), message, palette));
    }

    container(content)
        .padding(Padding::from([20.0, 32.0]))
        .center_x(Fill)
        .center_y(Fill)
        .into()
}

pub fn key_hint<'a>(keys: impl Into<String>, palette: Palette) -> Element<'a, Message> {
    container(
        text(keys.into())
            .size(10.5)
            .font(theme::FONT_MEDIUM)
            .color(palette.text_faint),
    )
    .padding(Padding::from([2.0, 7.0]))
    .style(move |_theme| container::Style {
        background: Some(Background::Color(palette.surface_alt)),
        border: Border {
            color: palette.border,
            width: 1.0,
            radius: 6.0.into(),
        },
        ..container::Style::default()
    })
    .into()
}

pub fn dot<'a>(color: Color, size: f32) -> Element<'a, Message> {
    container(Space::new().width(size).height(size))
        .style(move |_theme| container::Style {
            background: Some(Background::Color(color)),
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 999.0.into(),
            },
            ..container::Style::default()
        })
        .into()
}

pub fn hoverable<'a>(
    content: impl Into<Element<'a, Message>>,
    key: Hover,
) -> Element<'a, Message> {
    iced::widget::mouse_area(content.into())
        .on_enter(Message::HoverEnter(key))
        .on_exit(Message::HoverExit(key))
        .into()
}

/// A text widget with a custom font and color, used a lot in views.
pub fn label<'a>(
    value: impl Into<String>,
    size: f32,
    color: Color,
    font: iced::Font,
) -> Text<'a> {
    text(value.into()).size(size).color(color).font(font)
}

pub fn icon_in_circle<'a>(
    glyph: Icon,
    tint: Color,
    size: f32,
    palette: Palette,
) -> Element<'a, Message> {
    container(icon(glyph, size * 0.45, theme::mix(palette.text, tint, 0.6)))
        .width(size)
        .height(size)
        .align_x(Alignment::Center)
        .align_y(Alignment::Center)
        .style(move |_theme| container::Style {
            background: Some(Background::Color(theme::mix(palette.surface, tint, 0.45))),
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: (size * 0.34).into(),
            },
            ..container::Style::default()
        })
        .into()
}

pub fn icon_button_maybe<'a>(
    glyph: Icon,
    message: Option<Message>,
    palette: Palette,
    active: bool,
) -> Button<'a, Message> {
    button(icon(
        glyph,
        14.0,
        if active { palette.primary } else { palette.text_muted },
    ))
    .padding(Padding::from([8.0, 9.0]))
    .style(move |_theme, status| style::button_icon(palette, active, status))
    .on_press_maybe(message)
}
