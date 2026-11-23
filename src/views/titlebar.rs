//! A custom, theme-aware title bar for the frameless window.
//!
//! The OS decorations are disabled in `main.rs`; this widget provides the
//! drag handle, the system menu and the familiar minimize / maximize / close
//! controls while still following the active pastel palette.

use iced::widget::canvas::{Frame, Geometry, LineCap, LineJoin, Path, Stroke, Style as StrokeStyle};
use iced::widget::{button, canvas, column, container, mouse_area, row, text, Space};
use iced::{
    mouse, Alignment, Color, Element, Fill, Point, Rectangle, Renderer, Size, Theme,
};

use crate::app::Mochi;
use crate::message::{Hover, Message};
use crate::theme::{self, style, Palette};
use crate::widgets::icons::{icon, Icon};

/// Height of the title bar in logical pixels.
pub const HEIGHT: f32 = 36.0;

/// Combined width of the three window control buttons (3 × 44 px).
const CONTROLS_WIDTH: f32 = 132.0;

pub fn view<'a>(app: &'a Mochi) -> Element<'a, Message> {
    let p = app.palette;

    // Plain, decor-free wordmark: no circle, no pill. It should read like a
    // regular application title, not like a badge.
    let brand = row![
        icon(Icon::Heart, 11.0, p.primary),
        text("Mochi Notes")
            .size(12.5)
            .font(theme::FONT_BOLD)
            .color(p.text),
    ]
    .spacing(7)
    .align_y(Alignment::Center);

    // Only the app name lives in the centre of the title bar. The 132 px
    // spacer on the left balances the three window buttons on the right, and
    // the name itself stays left-aligned inside its own (shrink-wrapped) box.
    // Everything except the three buttons is a native window drag handle.
    let handle: Element<'a, Message> = container(
        mouse_area(
            row![
                Space::new().width(CONTROLS_WIDTH),
                Space::new().width(Fill),
                brand,
                Space::new().width(Fill),
            ]
            .height(Fill)
            .align_y(Alignment::Center),
        )
        .on_press(Message::TitleBarDrag)
        .on_double_click(Message::TitleBarDoubleClick)
        .on_right_press(Message::WindowSystemMenu),
    )
    .width(Fill)
    .height(Fill)
    .into();

    let controls = row![
        control(app, 1, Control::Minimize, Message::MinimizeWindow, p, false),
        control(
            app,
            2,
            if app.window_maximized {
                Control::Restore
            } else {
                Control::Maximize
            },
            Message::ToggleMaximizeWindow,
            p,
            false,
        ),
        control(app, 3, Control::Close, Message::CloseWindow, p, true),
    ]
    .width(CONTROLS_WIDTH)
    .height(Fill);

    container(
        column![
            row![handle, controls].width(Fill).height(HEIGHT),
            container(Space::new().height(1.0))
                .width(Fill)
                .style(move |_theme| container::Style {
                    background: Some(iced::Background::Color(p.border)),
                    ..container::Style::default()
                }),
        ]
        .width(Fill),
    )
    .width(Fill)
    .style(move |_theme| style::titlebar(p))
    .into()
}

/// The four glyphs used by the window controls. Drawing them on a tiny canvas
/// keeps their stroke weight and optical size perfectly consistent — font
/// glyphs vary far too much for controls this small.
#[derive(Debug, Clone, Copy)]
enum Control {
    Minimize,
    Maximize,
    Restore,
    Close,
}

#[derive(Debug, Clone, Copy)]
struct ControlGlyph {
    kind: Control,
    color: Color,
}

impl canvas::Program<Message> for ControlGlyph {
    type State = ();

    fn draw(
        &self,
        _state: &(),
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());
        let center = Point::new(bounds.width / 2.0, bounds.height / 2.0);

        let stroke = Stroke {
            style: StrokeStyle::Solid(self.color),
            width: 1.4,
            line_cap: LineCap::Round,
            line_join: LineJoin::Round,
            line_dash: iced::widget::canvas::LineDash::default(),
        };

        match self.kind {
            Control::Minimize => {
                frame.stroke(
                    &Path::line(
                        Point::new(center.x - 5.0, center.y),
                        Point::new(center.x + 5.0, center.y),
                    ),
                    stroke,
                );
            }
            Control::Maximize => {
                frame.stroke(
                    &Path::rectangle(
                        Point::new(center.x - 5.0, center.y - 5.0),
                        Size::new(10.0, 10.0),
                    ),
                    stroke,
                );
            }
            Control::Restore => {
                frame.stroke(
                    &Path::rectangle(
                        Point::new(center.x - 3.5, center.y - 5.5),
                        Size::new(9.0, 9.0),
                    ),
                    stroke,
                );
                frame.stroke(
                    &Path::rectangle(
                        Point::new(center.x - 5.5, center.y - 3.5),
                        Size::new(9.0, 9.0),
                    ),
                    stroke,
                );
            }
            Control::Close => {
                frame.stroke(
                    &Path::line(
                        Point::new(center.x - 4.5, center.y - 4.5),
                        Point::new(center.x + 4.5, center.y + 4.5),
                    ),
                    stroke,
                );
                frame.stroke(
                    &Path::line(
                        Point::new(center.x + 4.5, center.y - 4.5),
                        Point::new(center.x - 4.5, center.y + 4.5),
                    ),
                    stroke,
                );
            }
        }

        vec![frame.into_geometry()]
    }
}

fn control<'a>(
    app: &'a Mochi,
    slot: u8,
    kind: Control,
    message: Message,
    p: Palette,
    danger: bool,
) -> Element<'a, Message> {
    let key = Hover::Generic(slot);
    let hovered = app.hovered == Some(key);
    let color = if hovered {
        if danger {
            Color::WHITE
        } else {
            p.text
        }
    } else {
        p.text_muted
    };

    let glyph = canvas(ControlGlyph { kind, color })
        .width(16.0)
        .height(16.0);

    let button = button(
        container(glyph)
            .width(Fill)
            .height(Fill)
            .align_x(Alignment::Center)
            .align_y(Alignment::Center),
    )
    .padding(0)
    .width(44.0)
    .height(Fill)
    .style(move |_theme, status| style::title_button(p, danger, status))
    .on_press(message);

    mouse_area(button)
        .on_enter(Message::HoverEnter(key))
        .on_exit(Message::HoverExit(key))
        .into()
}
