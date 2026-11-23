//! Centralized visual design system for Mochi Notes.
//!
//! Every color, font and reusable style lives here so the whole application
//! can be re-skinned from a single place.

use iced::widget::{button, container, scrollable, slider, text_editor, text_input, toggler};
use iced::{font, theme as iced_theme, Background, Border, Color, Font, Shadow, Theme, Vector};
// ---------------------------------------------------------------------------
// Fonts
// ---------------------------------------------------------------------------

pub const FONT_FAMILY: &str = "Rounded Mplus 1c";
pub const ICON_FAMILY: &str = "Font Awesome 6 Free Solid";

pub const FONT: Font = Font {
    family: font::Family::Name(FONT_FAMILY),
    weight: font::Weight::Normal,
    stretch: font::Stretch::Normal,
    style: font::Style::Normal,
};

pub const FONT_MEDIUM: Font = Font {
    weight: font::Weight::Medium,
    ..FONT
};

pub const FONT_BOLD: Font = Font {
    weight: font::Weight::Bold,
    ..FONT
};

pub const FONT_ICON: Font = Font {
    family: font::Family::Name(ICON_FAMILY),
    weight: font::Weight::Black,
    stretch: font::Stretch::Normal,
    style: font::Style::Normal,
};

pub const FONT_MONO: Font = Font {
    family: font::Family::Monospace,
    ..FONT
};

// ---------------------------------------------------------------------------
// Type scale
// ---------------------------------------------------------------------------

pub mod size {
    pub const DISPLAY: f32 = 34.0;
    pub const H1: f32 = 26.0;
    pub const H2: f32 = 20.0;
    pub const H3: f32 = 16.0;
    pub const BODY: f32 = 14.0;
    pub const SMALL: f32 = 12.5;
    pub const MICRO: f32 = 11.0;
    pub const ICON: f32 = 15.0;
    pub const ICON_LG: f32 = 19.0;
}

pub mod space {
    pub const XS: f32 = 4.0;
    pub const SM: f32 = 8.0;
    pub const MD: f32 = 12.0;
    pub const LG: f32 = 16.0;
    pub const XL: f32 = 20.0;
    pub const XXL: f32 = 28.0;
    pub const HUGE: f32 = 40.0;
}

pub mod radius {
    pub const CARD: f32 = 22.0;
    pub const PANEL: f32 = 18.0;
    pub const INPUT: f32 = 14.0;
    pub const BUTTON: f32 = 13.0;
    pub const CHIP: f32 = 999.0;
}

// ---------------------------------------------------------------------------
// Color helpers
// ---------------------------------------------------------------------------

pub const fn rgb(hex: u32) -> Color {
    Color {
        r: ((hex >> 16) & 0xff) as f32 / 255.0,
        g: ((hex >> 8) & 0xff) as f32 / 255.0,
        b: (hex & 0xff) as f32 / 255.0,
        a: 1.0,
    }
}

pub const fn alpha(color: Color, a: f32) -> Color {
    Color { a, ..color }
}

/// Linear interpolation between two colors in sRGB space.
pub fn mix(a: Color, b: Color, t: f32) -> Color {
    let t = t.clamp(0.0, 1.0);
    Color {
        r: a.r + (b.r - a.r) * t,
        g: a.g + (b.g - a.g) * t,
        b: a.b + (b.b - a.b) * t,
        a: a.a + (b.a - a.a) * t,
    }
}

pub fn darken(color: Color, amount: f32) -> Color {
    mix(color, Color::BLACK, amount)
}

pub fn lighten(color: Color, amount: f32) -> Color {
    mix(color, Color::WHITE, amount)
}

/// Parses a `#rrggbb` string, falling back to a pleasant pastel pink.
pub fn parse_hex(value: &str) -> Color {
    let value = value.trim().trim_start_matches('#');
    if value.len() == 6 {
        if let Ok(hex) = u32::from_str_radix(value, 16) {
            return rgb(hex);
        }
    }
    rgb(0xf2a7bb)
}

pub fn to_hex(color: Color) -> String {
    format!(
        "#{:02x}{:02x}{:02x}",
        (color.r * 255.0).round() as u8,
        (color.g * 255.0).round() as u8,
        (color.b * 255.0).round() as u8
    )
}

/// Builds a four-sided padding. Iced 0.14 only implements `From<[f32; 2]>`,
/// so this keeps the call sites readable.
pub const fn pad(top: f32, right: f32, bottom: f32, left: f32) -> iced::Padding {
    iced::Padding {
        top,
        right,
        bottom,
        left,
    }
}

/// Pastel colors offered when creating folders and tags.
pub const SWATCHES: [&str; 10] = [
    "#F49AB6", "#F2B27E", "#F3D37A", "#A9D3A1", "#8FD0C6", "#93BFEF", "#B6A7EE", "#E3A9D4",
    "#C7B39E", "#9FB1C9",
];

// ---------------------------------------------------------------------------
// Themes
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ThemeId {
    #[default]
    StrawberryMilk,
    MatchaLatte,
    LavenderDream,
    BlueberryCloud,
    VanillaCream,
    MidnightMochi,
}

impl ThemeId {
    pub const ALL: [ThemeId; 6] = [
        ThemeId::StrawberryMilk,
        ThemeId::MatchaLatte,
        ThemeId::LavenderDream,
        ThemeId::BlueberryCloud,
        ThemeId::VanillaCream,
        ThemeId::MidnightMochi,
    ];

    pub fn name(self) -> &'static str {
        match self {
            ThemeId::StrawberryMilk => "Strawberry Milk",
            ThemeId::MatchaLatte => "Matcha Latte",
            ThemeId::LavenderDream => "Lavender Dream",
            ThemeId::BlueberryCloud => "Blueberry Cloud",
            ThemeId::VanillaCream => "Vanilla Cream",
            ThemeId::MidnightMochi => "Midnight Mochi",
        }
    }

    pub fn tagline(self) -> &'static str {
        match self {
            ThemeId::StrawberryMilk => "Sweet, soft and a little bit dreamy.",
            ThemeId::MatchaLatte => "Calm sage greens for slow mornings.",
            ThemeId::LavenderDream => "Airy lilac for wandering thoughts.",
            ThemeId::BlueberryCloud => "Baby blue skies and fluffy ideas.",
            ThemeId::VanillaCream => "Warm, buttery and comforting.",
            ThemeId::MidnightMochi => "A moonlit corner for night owls.",
        }
    }

    pub fn palette(self) -> Palette {
        match self {
            ThemeId::StrawberryMilk => Palette {
                id: self,
                dark: false,
                bg: rgb(0xFFF7F9),
                bg_alt: rgb(0xFFF1F5),
                surface: rgb(0xFFFFFF),
                surface_alt: rgb(0xFFF2F6),
                sidebar: rgb(0xFFEFF4),
                text: rgb(0x4A3440),
                text_muted: rgb(0x8F7480),
                text_faint: rgb(0xB9A3AD),
                primary: rgb(0xE8799B),
                primary_hover: rgb(0xD9628A),
                primary_soft: rgb(0xFFDDE8),
                on_primary: rgb(0xFFFFFF),
                border: rgb(0xF6E1E8),
                border_strong: rgb(0xEFCEDA),
                danger: rgb(0xD9536F),
                danger_soft: rgb(0xFFE1E6),
                success: rgb(0x6FAE8B),
                warning: rgb(0xE3A857),
                shadow: rgb(0xB4788C),
                accent: rgb(0xF7B7C9),
                pastel: [
                    rgb(0xFFE3EC),
                    rgb(0xFFF0E2),
                    rgb(0xFDE7F5),
                    rgb(0xEAF3FF),
                    rgb(0xF0E9FF),
                    rgb(0xE6F7F0),
                ],
            },
            ThemeId::MatchaLatte => Palette {
                id: self,
                dark: false,
                bg: rgb(0xF7FAF2),
                bg_alt: rgb(0xF1F6EA),
                surface: rgb(0xFFFFFF),
                surface_alt: rgb(0xF0F5E9),
                sidebar: rgb(0xEDF4E3),
                text: rgb(0x34402E),
                text_muted: rgb(0x78876D),
                text_faint: rgb(0xA8B49E),
                primary: rgb(0x7CA56A),
                primary_hover: rgb(0x6B9359),
                primary_soft: rgb(0xE0EDD6),
                on_primary: rgb(0xFFFFFF),
                border: rgb(0xE3ECD9),
                border_strong: rgb(0xD3E0C6),
                danger: rgb(0xC4675C),
                danger_soft: rgb(0xFBE6E2),
                success: rgb(0x6FAE8B),
                warning: rgb(0xD3A44C),
                shadow: rgb(0x6E825A),
                accent: rgb(0xA8C98D),
                pastel: [
                    rgb(0xE7F1DC),
                    rgb(0xF4F0DA),
                    rgb(0xE2EFE6),
                    rgb(0xF0E9D8),
                    rgb(0xE9F0FF),
                    rgb(0xF7E4E0),
                ],
            },
            ThemeId::LavenderDream => Palette {
                id: self,
                dark: false,
                bg: rgb(0xFAF7FF),
                bg_alt: rgb(0xF4EEFF),
                surface: rgb(0xFFFFFF),
                surface_alt: rgb(0xF5F0FF),
                sidebar: rgb(0xF1EAFE),
                text: rgb(0x403552),
                text_muted: rgb(0x85779B),
                text_faint: rgb(0xB3A7C4),
                primary: rgb(0x9A7CE0),
                primary_hover: rgb(0x8668CE),
                primary_soft: rgb(0xE9E0FF),
                on_primary: rgb(0xFFFFFF),
                border: rgb(0xECE4F7),
                border_strong: rgb(0xDDD1EF),
                danger: rgb(0xC96A7E),
                danger_soft: rgb(0xFBE5EA),
                success: rgb(0x74AE92),
                warning: rgb(0xD7A65C),
                shadow: rgb(0x785FA0),
                accent: rgb(0xC4A7F0),
                pastel: [
                    rgb(0xEFE6FF),
                    rgb(0xF7E9F7),
                    rgb(0xE6EDFF),
                    rgb(0xF3EBDD),
                    rgb(0xE4F3F1),
                    rgb(0xFFE9F0),
                ],
            },
            ThemeId::BlueberryCloud => Palette {
                id: self,
                dark: false,
                bg: rgb(0xF5F9FF),
                bg_alt: rgb(0xEDF5FF),
                surface: rgb(0xFFFFFF),
                surface_alt: rgb(0xF0F7FF),
                sidebar: rgb(0xE8F2FF),
                text: rgb(0x2F3E52),
                text_muted: rgb(0x7285A0),
                text_faint: rgb(0xA6B6C9),
                primary: rgb(0x6C9FD8),
                primary_hover: rgb(0x5A8DC6),
                primary_soft: rgb(0xDCEBFF),
                on_primary: rgb(0xFFFFFF),
                border: rgb(0xE1ECFA),
                border_strong: rgb(0xCFDFF2),
                danger: rgb(0xD06A7C),
                danger_soft: rgb(0xFBE6E9),
                success: rgb(0x6FAE9B),
                warning: rgb(0xD9A75B),
                shadow: rgb(0x5A78A0),
                accent: rgb(0xA5C8EE),
                pastel: [
                    rgb(0xE2EFFF),
                    rgb(0xEDF6FF),
                    rgb(0xE7F0FB),
                    rgb(0xF0EAFF),
                    rgb(0xE4F4F6),
                    rgb(0xFFF1E8),
                ],
            },
            ThemeId::VanillaCream => Palette {
                id: self,
                dark: false,
                bg: rgb(0xFDFAF4),
                bg_alt: rgb(0xFAF4E8),
                surface: rgb(0xFFFFFF),
                surface_alt: rgb(0xFBF5EA),
                sidebar: rgb(0xF8F1E2),
                text: rgb(0x4A3F30),
                text_muted: rgb(0x8D7D68),
                text_faint: rgb(0xB8AA96),
                primary: rgb(0xC2914A),
                primary_hover: rgb(0xAD7E3B),
                primary_soft: rgb(0xF6E7CB),
                on_primary: rgb(0xFFFDF8),
                border: rgb(0xF0E6D4),
                border_strong: rgb(0xE4D6BF),
                danger: rgb(0xC56A5B),
                danger_soft: rgb(0xFBE7E2),
                success: rgb(0x7FA878),
                warning: rgb(0xD0A24F),
                shadow: rgb(0x8C7350),
                accent: rgb(0xE7C894),
                pastel: [
                    rgb(0xF8EBD3),
                    rgb(0xF6E2D8),
                    rgb(0xF2EADB),
                    rgb(0xEFE6D2),
                    rgb(0xF9F0E2),
                    rgb(0xEBF0E4),
                ],
            },
            ThemeId::MidnightMochi => Palette {
                id: self,
                dark: true,
                bg: rgb(0x1C1725),
                bg_alt: rgb(0x211A2C),
                surface: rgb(0x292134),
                surface_alt: rgb(0x332A41),
                sidebar: rgb(0x171220),
                text: rgb(0xF2EAF8),
                text_muted: rgb(0xB4A5C4),
                text_faint: rgb(0x7E708F),
                primary: rgb(0xC79AF2),
                primary_hover: rgb(0xD6B2FA),
                primary_soft: rgb(0x3B2E50),
                on_primary: rgb(0x241A33),
                border: rgb(0x382E47),
                border_strong: rgb(0x4A3D5E),
                danger: rgb(0xF08A9E),
                danger_soft: rgb(0x4A2A36),
                success: rgb(0x8FD3AE),
                warning: rgb(0xF0C27A),
                shadow: rgb(0x000000),
                accent: rgb(0xE8B7F5),
                pastel: [
                    rgb(0x352B44),
                    rgb(0x3C2F3E),
                    rgb(0x2F3549),
                    rgb(0x3A3145),
                    rgb(0x2C3B3B),
                    rgb(0x44302F),
                ],
            },
        }
    }

    pub fn swatches(self) -> [Color; 4] {
        let p = self.palette();
        [p.primary, p.primary_soft, p.pastel[1], p.bg_alt]
    }

    pub fn iced_theme(self) -> Theme {
        let p = self.palette();
        Theme::custom(
            self.name(),
            iced_theme::Palette {
                background: p.bg,
                text: p.text,
                primary: p.primary,
                success: p.success,
                warning: p.warning,
                danger: p.danger,
            },
        )
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Palette {
    pub id: ThemeId,
    pub dark: bool,
    pub bg: Color,
    pub bg_alt: Color,
    pub surface: Color,
    pub surface_alt: Color,
    pub sidebar: Color,
    pub text: Color,
    pub text_muted: Color,
    pub text_faint: Color,
    pub primary: Color,
    pub primary_hover: Color,
    pub primary_soft: Color,
    pub on_primary: Color,
    pub border: Color,
    pub border_strong: Color,
    pub danger: Color,
    pub danger_soft: Color,
    pub success: Color,
    pub warning: Color,
    pub shadow: Color,
    pub accent: Color,
    pub pastel: [Color; 6],
}

impl Palette {
    pub fn shadow_soft(&self) -> Shadow {
        Shadow {
            color: alpha(self.shadow, if self.dark { 0.42 } else { 0.14 }),
            offset: Vector::new(0.0, 5.0),
            blur_radius: 18.0,
        }
    }

    pub fn shadow_lifted(&self) -> Shadow {
        Shadow {
            color: alpha(self.shadow, if self.dark { 0.55 } else { 0.20 }),
            offset: Vector::new(0.0, 10.0),
            blur_radius: 30.0,
        }
    }

    pub fn tint(&self, index: usize) -> Color {
        let base = self.pastel[index % self.pastel.len()];
        mix(self.surface, base, if self.dark { 0.85 } else { 0.62 })
    }

    pub fn tint_strong(&self, index: usize) -> Color {
        let base = self.pastel[index % self.pastel.len()];
        mix(self.surface, base, if self.dark { 0.95 } else { 0.9 })
    }

    pub fn is_dark(&self) -> bool {
        self.dark
    }
}

// ---------------------------------------------------------------------------
// Reusable styles
// ---------------------------------------------------------------------------

pub mod style {
    use super::*;

    pub fn app(p: Palette) -> container::Style {
        container::Style {
            background: Some(Background::Color(p.bg)),
            text_color: Some(p.text),
            ..container::Style::default()
        }
    }

    /// Background of the frameless window's custom title bar.
    pub fn titlebar(p: Palette) -> container::Style {
        container::Style {
            background: Some(Background::Color(if p.dark {
                mix(p.sidebar, p.bg, 0.16)
            } else {
                mix(p.sidebar, p.surface, 0.22)
            })),
            text_color: Some(p.text_muted),
            ..container::Style::default()
        }
    }

    /// The small square buttons on the right of the title bar.
    pub fn title_button(p: Palette, danger: bool, status: button::Status) -> button::Style {
        let hovered = matches!(status, button::Status::Hovered | button::Status::Pressed);
        let background = if danger {
            match status {
                button::Status::Hovered | button::Status::Pressed => p.danger,
                _ => Color::TRANSPARENT,
            }
        } else {
            match status {
                button::Status::Hovered => p.surface_alt,
                button::Status::Pressed => mix(p.surface_alt, p.primary_soft, 0.7),
                _ => Color::TRANSPARENT,
            }
        };

        button::Style {
            background: Some(Background::Color(background)),
            text_color: if danger && hovered {
                Color::WHITE
            } else {
                p.text_muted
            },
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 0.0.into(),
            },
            shadow: iced::Shadow::default(),
            snap: false,
        }
    }

    pub fn sidebar(p: Palette) -> container::Style {
        container::Style {
            background: Some(Background::Color(p.sidebar)),
            text_color: Some(p.text),
            border: Border {
                color: p.border,
                width: 1.0,
                radius: 0.0.into(),
            },
            ..container::Style::default()
        }
    }

    pub fn panel(p: Palette) -> container::Style {
        container::Style {
            background: Some(Background::Color(p.surface)),
            text_color: Some(p.text),
            border: Border {
                color: p.border,
                width: 1.0,
                radius: radius::PANEL.into(),
            },
            shadow: p.shadow_soft(),
            snap: false,
        }
    }

    pub fn card(p: Palette, hover: f32) -> container::Style {
        let background = mix(p.surface, p.primary_soft, hover * 0.18);
        container::Style {
            background: Some(Background::Color(background)),
            text_color: Some(p.text),
            border: Border {
                color: mix(p.border, p.primary, hover * 0.45),
                width: 1.0,
                radius: radius::CARD.into(),
            },
            shadow: Shadow {
                color: alpha(p.shadow, if p.dark { 0.35 + hover * 0.2 } else { 0.10 + hover * 0.10 }),
                offset: Vector::new(0.0, 5.0 + hover * 5.0),
                blur_radius: 18.0 + hover * 14.0,
            },
            snap: false,
        }
    }

    pub fn tinted_card(p: Palette, tint: Color, hover: f32) -> container::Style {
        container::Style {
            background: Some(Background::Color(mix(tint, p.primary_soft, hover * 0.22))),
            text_color: Some(p.text),
            border: Border {
                color: mix(p.border, p.primary, hover * 0.4),
                width: 1.0,
                radius: radius::CARD.into(),
            },
            shadow: Shadow {
                color: alpha(p.shadow, if p.dark { 0.3 + hover * 0.2 } else { 0.08 + hover * 0.12 }),
                offset: Vector::new(0.0, 4.0 + hover * 6.0),
                blur_radius: 16.0 + hover * 16.0,
            },
            snap: false,
        }
    }

    pub fn hero(p: Palette) -> container::Style {
        container::Style {
            background: Some(Background::Color(if p.dark {
                mix(p.surface, p.primary_soft, 0.55)
            } else {
                mix(p.primary_soft, p.surface, 0.35)
            })),
            text_color: Some(p.text),
            border: Border {
                color: mix(p.primary_soft, p.border, 0.5),
                width: 1.0,
                radius: 28.0.into(),
            },
            shadow: Shadow {
                color: alpha(p.shadow, if p.dark { 0.4 } else { 0.12 }),
                offset: Vector::new(0.0, 10.0),
                blur_radius: 32.0,
            },
            snap: false,
        }
    }

    pub fn toolbar(p: Palette) -> container::Style {
        container::Style {
            background: Some(Background::Color(p.surface_alt)),
            text_color: Some(p.text),
            border: Border {
                color: p.border,
                width: 1.0,
                radius: radius::INPUT.into(),
            },
            ..container::Style::default()
        }
    }

    pub fn nav(p: Palette, selected: bool, hover: f32) -> container::Style {
        let background = if selected {
            p.primary_soft
        } else if hover > 0.0 {
            mix(Color::TRANSPARENT, p.surface_alt, hover)
        } else {
            Color::TRANSPARENT
        };

        container::Style {
            background: Some(Background::Color(background)),
            text_color: Some(if selected { p.primary } else { p.text_muted }),
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: radius::BUTTON.into(),
            },
            ..container::Style::default()
        }
    }

    pub fn button_primary(p: Palette, status: button::Status) -> button::Style {
        let bg = match status {
            button::Status::Hovered => p.primary_hover,
            button::Status::Pressed => darken(p.primary, 0.08),
            button::Status::Disabled => mix(p.primary, p.surface, 0.5),
            button::Status::Active => p.primary,
        };

        button::Style {
            background: Some(Background::Color(bg)),
            text_color: p.on_primary,
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: radius::BUTTON.into(),
            },
            // Flat by design: no glow around primary actions.
            shadow: Shadow::default(),
            snap: false,
        }
    }

    pub fn button_soft(p: Palette, status: button::Status) -> button::Style {
        let bg = match status {
            button::Status::Hovered => mix(p.primary_soft, p.primary, 0.16),
            button::Status::Pressed => mix(p.primary_soft, p.primary, 0.26),
            button::Status::Disabled => mix(p.surface_alt, p.primary_soft, 0.4),
            button::Status::Active => p.primary_soft,
        };

        button::Style {
            background: Some(Background::Color(bg)),
            text_color: if p.dark { p.text } else { darken(p.primary, 0.18) },
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: radius::BUTTON.into(),
            },
            shadow: Shadow::default(),
            snap: false,
        }
    }

    pub fn button_ghost(p: Palette, status: button::Status) -> button::Style {
        let bg = match status {
            button::Status::Hovered => p.surface_alt,
            button::Status::Pressed => mix(p.surface_alt, p.primary_soft, 0.6),
            _ => Color::TRANSPARENT,
        };

        button::Style {
            background: Some(Background::Color(bg)),
            text_color: p.text_muted,
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: radius::BUTTON.into(),
            },
            shadow: Shadow::default(),
            snap: false,
        }
    }

    pub fn button_icon(p: Palette, active: bool, status: button::Status) -> button::Style {
        let base = if active { p.primary_soft } else { Color::TRANSPARENT };
        let bg = match status {
            button::Status::Hovered => mix(base, p.primary, 0.16),
            button::Status::Pressed => mix(base, p.primary, 0.28),
            _ => base,
        };

        button::Style {
            background: Some(Background::Color(bg)),
            text_color: if active { p.primary } else { p.text_muted },
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 11.0.into(),
            },
            shadow: Shadow::default(),
            snap: false,
        }
    }

    pub fn button_danger(p: Palette, status: button::Status) -> button::Style {
        let bg = match status {
            button::Status::Hovered => mix(p.danger_soft, p.danger, 0.16),
            button::Status::Pressed => mix(p.danger_soft, p.danger, 0.28),
            _ => p.danger_soft,
        };

        button::Style {
            background: Some(Background::Color(bg)),
            text_color: p.danger,
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: radius::BUTTON.into(),
            },
            shadow: Shadow::default(),
            snap: false,
        }
    }

    pub fn chip(p: Palette, selected: bool, status: button::Status) -> button::Style {
        let (bg, text) = if selected {
            let bg = match status {
                button::Status::Hovered => p.primary_hover,
                _ => p.primary,
            };
            (bg, p.on_primary)
        } else {
            let bg = match status {
                button::Status::Hovered => mix(p.surface_alt, p.primary_soft, 0.7),
                _ => p.surface_alt,
            };
            (bg, p.text_muted)
        };

        button::Style {
            background: Some(Background::Color(bg)),
            text_color: text,
            border: Border {
                color: if selected { Color::TRANSPARENT } else { p.border },
                width: 1.0,
                radius: radius::CHIP.into(),
            },
            shadow: Shadow::default(),
            snap: false,
        }
    }

    pub fn tag_chip(p: Palette, tint: Color, status: button::Status) -> button::Style {
        let bg = match status {
            button::Status::Hovered => mix(p.surface, tint, 0.34),
            _ => mix(p.surface, tint, 0.22),
        };
        let text = mix(p.text, tint, 0.45);

        button::Style {
            background: Some(Background::Color(bg)),
            text_color: text,
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: radius::CHIP.into(),
            },
            shadow: Shadow::default(),
            snap: false,
        }
    }

    pub fn input(p: Palette, status: text_input::Status) -> text_input::Style {
        let focused = matches!(status, text_input::Status::Focused { .. });
        text_input::Style {
            background: Background::Color(p.surface),
            border: Border {
                color: if focused { p.primary } else { p.border },
                width: if focused { 1.5 } else { 1.0 },
                radius: radius::INPUT.into(),
            },
            icon: p.text_faint,
            placeholder: p.text_faint,
            value: p.text,
            selection: alpha(p.primary, 0.3),
        }
    }

    pub fn input_soft(p: Palette, status: text_input::Status) -> text_input::Style {
        let focused = matches!(status, text_input::Status::Focused { .. });
        text_input::Style {
            background: Background::Color(p.surface_alt),
            border: Border {
                color: if focused { p.primary } else { Color::TRANSPARENT },
                width: 1.5,
                radius: radius::INPUT.into(),
            },
            icon: p.text_faint,
            placeholder: p.text_faint,
            value: p.text,
            selection: alpha(p.primary, 0.3),
        }
    }

    pub fn title_input(p: Palette, _status: text_input::Status) -> text_input::Style {
        text_input::Style {
            background: Background::Color(Color::TRANSPARENT),
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 0.0.into(),
            },
            icon: p.text_faint,
            placeholder: p.text_faint,
            value: p.text,
            selection: alpha(p.primary, 0.3),
        }
    }

    pub fn editor(p: Palette, _status: text_editor::Status) -> text_editor::Style {
        text_editor::Style {
            background: Background::Color(Color::TRANSPARENT),
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 0.0.into(),
            },
            placeholder: p.text_faint,
            value: p.text,
            selection: alpha(p.primary, 0.28),
        }
    }

    pub fn scrollable(p: Palette, _status: scrollable::Status) -> scrollable::Style {
        let rail = scrollable::Rail {
            background: Some(Background::Color(Color::TRANSPARENT)),
            border: Border::default(),
            scroller: scrollable::Scroller {
                background: Background::Color(if p.dark {
                    p.border_strong
                } else {
                    mix(p.border_strong, p.text_faint, 0.35)
                }),
                border: Border {
                    color: Color::TRANSPARENT,
                    width: 0.0,
                    radius: 8.0.into(),
                },
            },
        };

        scrollable::Style {
            container: container::Style::default(),
            vertical_rail: rail,
            horizontal_rail: rail,
            gap: None,
            auto_scroll: scrollable::AutoScroll {
                background: Background::Color(p.surface),
                border: Border {
                    color: p.border,
                    width: 1.0,
                    radius: 10.0.into(),
                },
                shadow: p.shadow_soft(),
                icon: p.text_muted,
            },
        }
    }

    pub fn toggler(p: Palette, status: toggler::Status) -> toggler::Style {
        let checked = matches!(status, toggler::Status::Active { is_toggled: true });
        let hovered = matches!(
            status,
            toggler::Status::Hovered { is_toggled: true } | toggler::Status::Hovered { is_toggled: false }
        );
        let _ = hovered;
        toggler::Style {
            background: Background::Color(if checked { p.primary } else { p.surface_alt }),
            background_border_width: 1.0,
            background_border_color: if checked { p.primary } else { p.border_strong },
            foreground: Background::Color(if checked { p.on_primary } else { p.text_faint }),
            foreground_border_width: 0.0,
            foreground_border_color: Color::TRANSPARENT,
            text_color: Some(p.text),
            border_radius: Some(radius::CHIP.into()),
            padding_ratio: 0.18,
        }
    }

    pub fn slider(p: Palette, _status: slider::Status) -> slider::Style {
        slider::Style {
            rail: slider::Rail {
                backgrounds: (
                    Background::Color(p.primary),
                    Background::Color(p.border_strong),
                ),
                width: 5.0,
                border: Border {
                    color: Color::TRANSPARENT,
                    width: 0.0,
                    radius: 3.0.into(),
                },
            },
            handle: slider::Handle {
                shape: slider::HandleShape::Circle { radius: 9.0 },
                background: Background::Color(p.surface),
                border_width: 3.0,
                border_color: p.primary,
            },
        }
    }

    pub fn modal_backdrop(_p: Palette) -> container::Style {
        container::Style {
            background: Some(Background::Color(Color { a: 0.45, ..rgb(0x1A1220) })),
            ..container::Style::default()
        }
    }

    pub fn tooltip(p: Palette) -> container::Style {
        container::Style {
            background: Some(Background::Color(p.text)),
            text_color: Some(p.surface),
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 10.0.into(),
            },
            shadow: p.shadow_soft(),
            snap: false,
        }
    }

    pub fn badge(p: Palette, tint: Color) -> container::Style {
        container::Style {
            background: Some(Background::Color(mix(p.surface, tint, 0.2))),
            text_color: Some(mix(p.text, tint, 0.5)),
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: radius::CHIP.into(),
            },
            ..container::Style::default()
        }
    }

    pub fn dashed(p: Palette) -> container::Style {
        container::Style {
            background: Some(Background::Color(Color::TRANSPARENT)),
            text_color: Some(p.text_muted),
            border: Border {
                color: mix(p.border_strong, p.primary, 0.2),
                width: 1.5,
                radius: radius::CARD.into(),
            },
            ..container::Style::default()
        }
    }
}
