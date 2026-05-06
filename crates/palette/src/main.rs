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
        .theme(|_| {
            iced::Theme::custom(
                "forkos".to_string(),
                iced::theme::Palette {
                    background: iced::Color::TRANSPARENT,
                    ..iced::theme::Palette::LIGHT
                },
            )
        })
        .decorations(false)
        .transparent(true)
        .run_with(Palette::new)
}
