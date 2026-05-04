mod app;
mod command;
mod search;
mod theme;
mod view;

use app::Palette;

fn main() -> iced::Result {
    tracing_subscriber::fmt::init();

    iced::application("Palette", Palette::update, Palette::view)
        .subscription(Palette::subscription)
        .theme(|_| iced::Theme::Light)
        .run_with(Palette::new)
}