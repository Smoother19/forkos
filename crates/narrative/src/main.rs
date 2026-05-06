mod app;
mod entry;
mod mpris;
mod store;
mod view;

use app::Narrative;
use forkos_shared::theme;

fn main() -> iced::Result {
    tracing_subscriber::fmt::init();

    iced::application("forkOS", Narrative::update, Narrative::view)
        .subscription(Narrative::subscription)
        .default_font(iced::Font::MONOSPACE)
        .theme(|_| {
            iced::Theme::custom(
                "forkos".to_string(),
                iced::theme::Palette {
                    background: theme::BASE,
                    text: theme::TEXT,
                    primary: theme::FOAM,
                    success: theme::PINE,
                    danger: theme::LOVE,
                },
            )
        })
        .run_with(Narrative::new)
}
