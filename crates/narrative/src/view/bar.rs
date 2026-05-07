use crate::app::{Message, Narrative, BOTTOM_INPUT_ID};
use forkos_shared::theme;
use iced::widget::{button, container, row, text, text_input, Space};
use iced::{Alignment, Background, Border, Element, Length, Padding};

pub fn render(state: &Narrative) -> Element<'_, Message> {
    let super_icon = if state.bar_open { "⊟" } else { "⊞" };
    let super_bg   = if state.bar_open { theme::IRIS } else { theme::OVERLAY };
    let super_fg   = if state.bar_open { theme::BASE } else { theme::IRIS };

    let super_btn = button(
        container(text(super_icon).size(15).color(super_fg))
            .center_x(Length::Fixed(40.0))
            .center_y(Length::Fixed(32.0)),
    )
    .width(Length::Fixed(40.0))
    .height(Length::Fixed(32.0))
    .on_press(Message::BarToggle)
    .style(move |_, _| button::Style {
        background: Some(Background::Color(super_bg)),
        border: Border { radius: 6.0.into(), ..Default::default() },
        text_color: super_fg,
        shadow: Default::default(),
    });

    let shell_input = text_input("$ tape une commande...", &state.bottom_query)
        .id(BOTTOM_INPUT_ID.clone())
        .on_input(Message::BottomInputChanged)
        .on_submit(Message::BottomInputSubmit)
        .padding(Padding { top: 6.0, right: 10.0, bottom: 6.0, left: 10.0 })
        .size(12)
        .style(|_, _| iced::widget::text_input::Style {
            background: Background::Color(theme::OVERLAY),
            border: Border {
                color: theme::HIGHLIGHT_MED,
                width: 1.0,
                radius: 5.0.into(),
            },
            icon: theme::TEXT,
            placeholder: theme::MUTED,
            value: theme::TEXT,
            selection: theme::HIGHLIGHT_MED,
        });

    let now = chrono::Local::now();
    let time_str = now.format("%H:%M").to_string();
    let time_widget = text(time_str).size(11).color(theme::TEXT);

    let content = row![
        super_btn,
        Space::new(Length::Fixed(8.0), Length::Shrink),
        shell_input,
        Space::new(Length::Fixed(12.0), Length::Shrink),
        time_widget,
        Space::new(Length::Fixed(14.0), Length::Shrink),
    ]
    .align_y(Alignment::Center)
    .height(Length::Fixed(48.0));

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
