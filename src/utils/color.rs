use iced::Color;

use crate::theme;

/// A stable color for a note card, derived from its identifier.
pub fn note_tint(seed: u128, count: usize) -> usize {
    if count == 0 {
        0
    } else {
        (seed % count as u128) as usize
    }
}

pub fn tint(seed: u128) -> Color {
    let index = note_tint(seed, theme::SWATCHES.len());
    theme::parse_hex(theme::SWATCHES[index])
}
