mod app;
mod models;
mod storage;
mod theme;
mod view;

use app::Narrative;

fn main() -> iced::Result {
    tracing_subscriber::fmt::init();

    iced::application("forkOS — fil narratif", Narrative::update, Narrative::view)
        .subscription(Narrative::subscription)
        .theme(|_| iced::Theme::Light)
        .decorations(false)
        .run_with(Narrative::new)
}
