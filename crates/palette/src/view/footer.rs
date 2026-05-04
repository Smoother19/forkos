use crate::app::Message;
use crate::mode::Mode;
use crate::theme;
use iced::widget::{row, text, Space};
use iced::{Element, Length, Padding};

pub fn render(count: usize, mode: Mode, is_loading: bool) -> Element<'static, Message> {
    let hint_text = mode.hint();

    let right_label = if is_loading {
        "⟳ chargement des apps…".to_string()
    } else {
        match mode {
            Mode::Shell => "historique".to_string(),
            Mode::FileContent => format!("{} correspondances", count),
            Mode::Universal | Mode::Commands => format!("{} résultats", count),
            _ => String::new(),
        }
    };
    let right_color = if is_loading { theme::GOLD } else { theme::MUTED };

    row![
        text(hint_text).size(11).color(theme::MUTED),
        Space::new(Length::Fill, Length::Shrink),
        text(right_label).size(11).color(right_color),
    ]
    .spacing(18)
    .padding(Padding { top: 10.0, right: 22.0, bottom: 10.0, left: 22.0 })
    .into()
}
