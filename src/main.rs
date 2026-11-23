//! Mochi Notes — a cozy, pastel note-taking app written in pure Rust with Iced.
//!
//! The application is intentionally split into small modules:
//!
//! * [`app`] holds all state and the update loop.
//! * [`message`] describes every interaction.
//! * [`theme`] is the single source of truth for colors, fonts and styles.
//! * [`database`] persists notes, folders, tags and settings in SQLite.
//! * [`views`] and [`widgets`] draw the interface.
//! * [`services`] handle files, exports and backups.

mod app;
mod database;
mod message;
mod models;
mod services;
mod theme;
mod utils;
mod views;
mod widgets;

use iced::window;
use iced::Size;

use crate::app::Mochi;

fn main() -> iced::Result {
    iced::application(Mochi::new, Mochi::update, Mochi::view)
        .title(Mochi::title)
        .theme(Mochi::theme)
        .subscription(Mochi::subscription)
        .window(window::Settings {
            size: Size::new(1260.0, 860.0),
            min_size: Some(Size::new(900.0, 620.0)),
            decorations: false,
            transparent: true,
            exit_on_close_request: false,
            ..window::Settings::default()
        })
        .font(include_bytes!("../assets/fonts/MPLUSRounded1c-Regular.ttf").as_slice())
        .font(include_bytes!("../assets/fonts/MPLUSRounded1c-Medium.ttf").as_slice())
        .font(include_bytes!("../assets/fonts/MPLUSRounded1c-Bold.ttf").as_slice())
        .font(include_bytes!("../assets/fonts/fa-solid-900.ttf").as_slice())
        .default_font(theme::FONT)
        .antialiasing(true)
        .run()
}
