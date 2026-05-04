mod app;
mod calculator;
mod command;
mod grep;
mod mode;
mod search;
mod shell;
mod sources;
mod theme;
mod view;

use app::Palette;

fn main() -> iced::Result {
    tracing_subscriber::fmt::init();

    iced::application("Palette", Palette::update, Palette::view)
        .subscription(Palette::subscription)
        .theme(|_| iced::Theme::Light)
        .decorations(false)
        .run_with(Palette::new)
}
