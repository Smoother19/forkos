use crate::app::{Message, Narrative};
use crate::theme;
use iced::widget::{container, row, text, text_input};
use iced::{Background, Border, Color, Element, Length, Padding};

pub fn render(state: &Narrative) -> Element<'_, Message> {
    let prompt = text("›").size(15).color(theme::ROSE);

    let input = text_input("note libre  ·  [ ] tâche  ·  [x] fait", &state.composer)
        .on_input(Message::ComposerChanged)
        .on_submit(Message::SubmitComposer)
        .padding(Padding { top: 0.0, right: 0.0, bottom: 0.0, left: 0.0 })
        .size(14)
        .style(|_, _| iced::widget::text_input::Style {
            background: Background::Color(Color::TRANSPARENT),
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 0.0.into(),
            },
            icon: theme::TEXT,
            placeholder: theme::MUTED,
            value: theme::TEXT,
            selection: theme::HIGHLIGHT_MED,
        });

    let r = row![prompt, input]
        .spacing(12)
        .align_y(iced::Alignment::Center)
        .padding(Padding { top: 14.0, right: 32.0, bottom: 14.0, left: 32.0 });

    container(r)
        .style(|_| container::Style {
            background: Some(Background::Color(theme::OVERLAY)),
            border: Border {
                color: theme::HIGHLIGHT_MED,
                width: 1.0,
                radius: 0.0.into(),
            },
            ..Default::default()
        })
        .width(Length::Fill)
        .into()
}
