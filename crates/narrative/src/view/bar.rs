use crate::app::{Message, Narrative};
use forkos_shared::theme;
use iced::widget::{button, container, row, text, Space};
use iced::{Alignment, Background, Border, Element, Length, Padding};

/// Barre permanente du bas : bouton Super + apps actives + musique + heure
pub fn render(state: &Narrative) -> Element<'_, Message> {
    let super_icon = if state.bar_open { "⊟" } else { "⊞" };
    let super_bg = if state.bar_open { theme::IRIS } else { theme::OVERLAY };
    let super_color = if state.bar_open { theme::BASE } else { theme::IRIS };

    let super_btn = button(
        container(text(super_icon).size(15).color(super_color))
            .center_x(Length::Fixed(40.0))
            .center_y(Length::Fixed(32.0)),
    )
    .width(Length::Fixed(40.0))
    .height(Length::Fixed(32.0))
    .on_press(Message::BarToggle)
    .style(move |_, _| button::Style {
        background: Some(Background::Color(super_bg)),
        border: Border { radius: 6.0.into(), ..Default::default() },
        text_color: super_color,
        shadow: Default::default(),
    });

    let apps = apps_strip(state);
    let media = media_pill(state);

    let now = chrono::Local::now();
    let time_str = now.format("%H:%M").to_string();
    let time = text(time_str).size(11).color(theme::TEXT);

    let content = row![
        super_btn,
        Space::new(Length::Fixed(10.0), Length::Shrink),
        apps,
        Space::new(Length::Fill, Length::Shrink),
        media,
        Space::new(Length::Fixed(14.0), Length::Shrink),
        time,
    ]
    .align_y(Alignment::Center)
    .padding(Padding { top: 0.0, right: 14.0, bottom: 0.0, left: 8.0 });

    container(content)
        .width(Length::Fill)
        .height(Length::Fixed(48.0))
        .style(|_| container::Style {
            background: Some(Background::Color(theme::SURFACE)),
            border: Border {
                color: theme::HIGHLIGHT_MED,
                width: 1.0,
                radius: 0.0.into(),
            },
            ..Default::default()
        })
        .into()
}

fn apps_strip(state: &Narrative) -> Element<'_, Message> {
    let mut r = row![].spacing(4).align_y(Alignment::Center);

    let mut windows: Vec<(&u64, &(String, String))> = state.active_windows.iter().collect();
    windows.sort_by_key(|(id, _)| *id);

    for (id, (app_id, title)) in windows {
        let is_active = Some(*id) == state.active_window_id;

        let raw = if !title.is_empty() { title.as_str() } else { app_id.as_str() };
        let label = if raw.len() > 18 {
            format!("{}…", &raw[..17])
        } else {
            raw.to_string()
        };

        let bg = if is_active { theme::FOAM } else { theme::OVERLAY };
        let fg = if is_active { theme::BASE } else { theme::TEXT };
        let cmd = format!("niri msg action focus-window --id {}", id);

        let btn = button(text(label).size(11).color(fg))
            .padding(Padding { top: 4.0, right: 10.0, bottom: 4.0, left: 10.0 })
            .on_press(Message::ContextAction(cmd))
            .style(move |_, _| button::Style {
                background: Some(Background::Color(bg)),
                border: Border { radius: 5.0.into(), ..Default::default() },
                text_color: fg,
                shadow: Default::default(),
            });

        r = r.push(btn);
    }

    r.into()
}

fn media_pill(state: &Narrative) -> Element<'_, Message> {
    match &state.current_media {
        Some(m) if m.playing => {
            let raw = format!("♫ {} — {}", m.artist, m.title);
            let label = if raw.len() > 32 {
                format!("{}…", &raw[..31])
            } else {
                raw
            };
            text(label).size(11).color(theme::GOLD).into()
        }
        _ => Space::new(Length::Shrink, Length::Shrink).into(),
    }
}
