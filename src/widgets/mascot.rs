//! Mochi, the little rice-cake mascot, drawn with a custom Iced canvas.

use std::f32::consts::PI;

use iced::widget::canvas::{self, Canvas, Frame, Geometry, LineCap, Path, Stroke};
use iced::widget::{container, mouse_area, text};
use iced::{
    mouse, Alignment, Background, Border, Color, Element, Padding, Point, Rectangle, Size, Theme,
};

use crate::message::Message;
use crate::theme::{self, Palette};

pub struct Mascot {
    phase: f32,
    palette: Palette,
}

impl Mascot {
    fn color(&self, value: u32) -> Color {
        theme::rgb(value)
    }
}

impl<Message> canvas::Program<Message> for Mascot {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &iced::Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: iced::mouse::Cursor,
    ) -> Vec<Geometry> {
        let scale = bounds.width.min(bounds.height) / 160.0;
        let offset_x = (bounds.width - 160.0 * scale) / 2.0;
        let offset_y = (bounds.height - 160.0 * scale) / 2.0;

        let bob = (self.phase * 1.5).sin() * 2.4 * scale;
        let breathe = 1.0 + (self.phase * 1.5).sin() * 0.012;

        let p = |x: f32, y: f32| {
            Point::new(
                offset_x + (80.0 + (x - 80.0) * breathe) * scale,
                offset_y + (80.0 + (y - 80.0) * breathe) * scale + bob,
            )
        };
        let r = |value: f32| value * scale * breathe;

        let mut frame = Frame::new(renderer, bounds.size());
        let palette = self.palette;

        let body = theme::mix(palette.surface, self.color(0xFFF4E6), 0.88);
        let ink = theme::mix(palette.text, self.color(0x5A4038), 0.55);
        let outline = theme::alpha(theme::mix(palette.text, body, 0.55), 0.5);
        let blush = theme::alpha(self.color(0xF59AAF), 0.42);
        let cream = self.color(0xFFFDF8);

        // Soft ground shadow
        frame.fill(
            &Path::rounded_rectangle(
                Point::new(offset_x + 44.0 * scale, offset_y + 139.0 * scale + bob),
                Size::new(72.0 * scale, 11.0 * scale),
                (6.0 * scale).into(),
            ),
            theme::alpha(palette.shadow, 0.16),
        );

        // Ears
        for center in [p(49.0, 51.0), p(111.0, 51.0)] {
            frame.fill(&Path::circle(center, r(17.0)), body);
            frame.stroke(
                &Path::circle(center, r(17.0)),
                Stroke::default().with_color(outline).with_width(1.4 * scale),
            );
        }
        frame.fill(&Path::circle(p(49.0, 51.0), r(8.0)), theme::alpha(palette.accent, 0.55));
        frame.fill(
            &Path::circle(p(111.0, 51.0), r(8.0)),
            theme::alpha(palette.accent, 0.55),
        );

        // Body
        frame.fill(&Path::circle(p(80.0, 89.0), r(54.0)), body);
        frame.stroke(
            &Path::circle(p(80.0, 89.0), r(54.0)),
            Stroke::default().with_color(outline).with_width(1.5 * scale),
        );

        // Belly highlight
        frame.fill(
            &Path::circle(p(80.0, 101.0), r(38.0)),
            theme::alpha(cream, 0.5),
        );

        // Blush
        frame.fill(&Path::circle(p(55.0, 102.0), r(8.5)), blush);
        frame.fill(&Path::circle(p(105.0, 102.0), r(8.5)), blush);

        // Eyes (blink for a couple of frames every few seconds)
        let cycle = self.phase % 4.2;
        let blinking = cycle > 3.92 && cycle < 4.08;

        if blinking {
            for x in [64.0, 96.0] {
                frame.stroke(
                    &Path::line(p(x - 5.0, 90.0), p(x + 5.0, 90.0)),
                    Stroke::default()
                        .with_color(ink)
                        .with_width(2.6 * scale)
                        .with_line_cap(LineCap::Round),
                );
            }
        } else {
            for x in [64.0, 96.0] {
                frame.fill(&Path::circle(p(x, 89.5), r(4.4)), ink);
                frame.fill(
                    &Path::circle(p(x + 1.4, 87.8), r(1.4)),
                    theme::alpha(Color::WHITE, 0.9),
                );
            }
        }

        // Mouth: a tiny cat-like "w"
        let mouth = Path::new(|path| {
            path.move_to(p(75.0, 99.0));
            path.quadratic_curve_to(p(78.0, 103.0), p(80.5, 99.5));
            path.quadratic_curve_to(p(83.0, 103.0), p(86.0, 99.0));
        });
        frame.stroke(
            &mouth,
            Stroke::default()
                .with_color(ink)
                .with_width(1.9 * scale)
                .with_line_cap(LineCap::Round),
        );

        // Strawberry topping
        let berry = self.color(0xEC7C8C);
        let berry_dark = self.color(0xD96276);
        let berry_path = Path::new(|path| {
            path.move_to(p(80.0, 38.0));
            path.bezier_curve_to(p(68.0, 30.0), p(66.0, 18.0), p(74.0, 14.0));
            path.bezier_curve_to(p(78.0, 12.0), p(80.0, 15.0), p(80.0, 17.0));
            path.bezier_curve_to(p(80.0, 15.0), p(82.0, 12.0), p(86.0, 14.0));
            path.bezier_curve_to(p(94.0, 18.0), p(92.0, 30.0), p(80.0, 38.0));
            path.close();
        });
        frame.fill(&berry_path, berry);
        frame.stroke(
            &berry_path,
            Stroke::default().with_color(berry_dark).with_width(1.2 * scale),
        );

        // Seeds
        for (x, y) in [(74.0, 23.0), (84.0, 22.0), (79.0, 29.0)] {
            frame.fill(
                &Path::circle(p(x, y), r(1.15)),
                theme::alpha(Color::WHITE, 0.8),
            );
        }

        // Leaf
        let leaf = Path::new(|path| {
            path.move_to(p(80.0, 14.0));
            path.quadratic_curve_to(p(86.0, 8.0), p(92.0, 10.0));
            path.quadratic_curve_to(p(89.0, 17.0), p(80.0, 14.0));
        });
        frame.fill(&leaf, self.color(0x88BC7C));

        // Sparkles
        for (index, (x, y, base)) in [(20.0, 66.0, 0.0), (141.0, 54.0, 2.1), (28.0, 118.0, 4.0)]
            .into_iter()
            .enumerate()
        {
            let twinkle = ((self.phase * 1.3 + base).sin() * 0.5 + 0.5).powf(2.0);
            let alpha = 0.25 + twinkle * 0.55;
            let radius = (4.5 + twinkle * 2.0) * scale;
            let center = Point::new(offset_x + x * scale, offset_y + y * scale + bob);
            let color = theme::alpha(palette.primary, alpha);

            frame.stroke(
                &Path::line(
                    Point::new(center.x - radius, center.y),
                    Point::new(center.x + radius, center.y),
                ),
                Stroke::default()
                    .with_color(color)
                    .with_width(1.7 * scale)
                    .with_line_cap(LineCap::Round),
            );
            frame.stroke(
                &Path::line(
                    Point::new(center.x, center.y - radius),
                    Point::new(center.x, center.y + radius),
                ),
                Stroke::default()
                    .with_color(color)
                    .with_width(1.7 * scale)
                    .with_line_cap(LineCap::Round),
            );

            let _ = index;
        }

        // A sleepy little sparkle arc above the head, very subtle
        let halo = Path::new(|path| {
            let center = Point::new(offset_x + 80.0 * scale, offset_y + 80.0 * scale + bob);
            let radius = 62.0 * scale;
            let start = -PI * 0.85;
            let end = -PI * 0.15;
            let steps = 24;
            for step in 0..=steps {
                let t = step as f32 / steps as f32;
                let angle = start + (end - start) * t;
                let point = Point::new(
                    center.x + radius * angle.cos(),
                    center.y + radius * angle.sin(),
                );
                if step == 0 {
                    path.move_to(point);
                } else {
                    path.line_to(point);
                }
            }
        });
        frame.stroke(
            &halo,
            Stroke::default()
                .with_color(theme::alpha(palette.primary, 0.10))
                .with_width(2.0 * scale),
        );

        vec![frame.into_geometry()]
    }
}

pub fn mascot<'a>(phase: f32, palette: Palette, size: f32) -> Element<'a, Message> {
    Canvas::new(Mascot { phase, palette })
        .width(size)
        .height(size)
        .into()
}

/// The mascot with a pointer cursor and a poke handler.
pub fn clickable<'a>(phase: f32, palette: Palette, size: f32) -> Element<'a, Message> {
    mouse_area(mascot(phase, palette, size))
        .on_press(Message::PokeMascot)
        .interaction(mouse::Interaction::Pointer)
        .into()
}

/// A soft speech bubble for the mascot's little messages. `alpha` is used for
/// a gentle fade-out driven by the application tick.
pub fn bubble<'a>(message: &str, p: Palette, alpha: f32) -> Element<'a, Message> {
    let alpha = alpha.clamp(0.0, 1.0);

    container(
        container(
            text(message.to_string())
                .size(11.5)
                .font(theme::FONT_MEDIUM)
                .color(theme::alpha(p.text, alpha))
                .align_x(Alignment::Center)
                .width(190),
        )
        .padding(Padding::from([8.0, 12.0]))
        .style(move |_theme| container::Style {
            background: Some(Background::Color(theme::alpha(p.surface, alpha))),
            text_color: Some(p.text),
            border: Border {
                color: theme::alpha(p.border_strong, alpha),
                width: 1.0,
                radius: 14.0.into(),
            },
            shadow: iced::Shadow::default(),
            snap: false,
        }),
    )
    .width(190)
    .align_x(Alignment::Center)
    .into()
}
