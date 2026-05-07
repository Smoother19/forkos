mod app;
mod entry;
mod mpris;
mod store;
mod view;

use app::Narrative;
use iced_layershell::build_pattern::{MainSettings, application};
use iced_layershell::reexport::{Anchor, Layer};
use iced_layershell::settings::{LayerShellSettings, StartMode};

fn main() -> iced_layershell::Result {
    tracing_subscriber::fmt::init();

    application("narrative", Narrative::update, Narrative::view)
        .subscription(Narrative::subscription)
        .settings(MainSettings {
            layer_settings: LayerShellSettings {
                size: Some((0, 648)),  // 60% de 1080 — bureau ouvert au démarrage
                anchor: Anchor::Bottom | Anchor::Left | Anchor::Right,
                exclusive_zone: 48,
                layer: Layer::Top,
                start_mode: StartMode::Active,
                ..Default::default()
            },
            ..Default::default()
        })
        .run_with(Narrative::new)
}
