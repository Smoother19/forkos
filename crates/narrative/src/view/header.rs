use crate::app::{Message, Narrative};
use crate::theme;
use chrono::{Datelike, Local, Timelike};
use iced::widget::{column, container, row, text, Space};
use iced::{Background, Element, Length, Padding};

pub fn render(state: &Narrative) -> Element<'_, Message> {
    let now = Local::now();
    let day = format_french_date(&now);
    let time = format!("{:02}:{:02}", now.hour(), now.minute());
    let context = context_line(state);

    let title = column![
        text(day).size(28).color(theme::TEXT),
        text(context).size(12).color(theme::MUTED),
    ]
    .spacing(4);

    let clock = text(time).size(20).color(theme::SUBTLE);

    container(
        row![
            title,
            Space::new(Length::Fill, Length::Shrink),
            clock,
        ]
        .padding(Padding { top: 28.0, right: 32.0, bottom: 24.0, left: 32.0 })
        .align_y(iced::Alignment::Start),
    )
    .style(|_| container::Style {
        background: Some(Background::Color(theme::BASE)),
        ..Default::default()
    })
    .width(Length::Fill)
    .into()
}

fn context_line(state: &Narrative) -> String {
    let total = state.entries.len();
    if total == 0 {
        return "fil vide — première session".into();
    }
    let today_count = state
        .entries
        .iter()
        .filter(|e| {
            let d = e.timestamp.date_naive();
            d == Local::now().date_naive()
        })
        .count();
    if today_count == 0 {
        format!("{} entrées · rien aujourd'hui pour l'instant", total)
    } else {
        format!("{} entrées · {} aujourd'hui", total, today_count)
    }
}

/// Date « lundi 6 mai 2026 » en français.
fn format_french_date(dt: &chrono::DateTime<Local>) -> String {
    let weekday = match dt.weekday() {
        chrono::Weekday::Mon => "lundi",
        chrono::Weekday::Tue => "mardi",
        chrono::Weekday::Wed => "mercredi",
        chrono::Weekday::Thu => "jeudi",
        chrono::Weekday::Fri => "vendredi",
        chrono::Weekday::Sat => "samedi",
        chrono::Weekday::Sun => "dimanche",
    };
    let month = match dt.month() {
        1 => "janvier",
        2 => "février",
        3 => "mars",
        4 => "avril",
        5 => "mai",
        6 => "juin",
        7 => "juillet",
        8 => "août",
        9 => "septembre",
        10 => "octobre",
        11 => "novembre",
        12 => "décembre",
        _ => "?",
    };
    format!("{} {} {} {}", weekday, dt.day(), month, dt.year())
}
