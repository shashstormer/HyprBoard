use iced::widget::{container, row, text, tooltip as iced_tooltip};
use iced::{Color, Element, Theme};

pub fn info_tooltip<'a, Message: Clone + 'a>(description: &'a str) -> Element<'a, Message> {
    let icon = text("ⓘ").size(14).style(|_: &Theme| text::Style {
        color: Some(Color::from_rgb8(137, 180, 250)),
    });

    let tip = container(text(description).size(12).style(|_: &Theme| text::Style {
        color: Some(Color::from_rgb8(205, 214, 244)),
    }))
    .padding(8)
    .max_width(300)
    .style(|_: &Theme| container::Style {
        background: Some(Color::from_rgb8(30, 30, 46).into()),
        border: iced::Border {
            color: Color::from_rgb8(69, 71, 90),
            width: 1.0,
            radius: 6.0.into(),
        },
        ..Default::default()
    });

    iced_tooltip(icon, tip, iced_tooltip::Position::Right)
        .gap(5)
        .into()
}

pub fn labeled_tooltip<'a, Message: Clone + 'a>(
    label: &'a str,
    description: &'a str,
) -> Element<'a, Message> {
    row![text(label).size(14), info_tooltip(description),]
        .spacing(6)
        .align_y(iced::Alignment::Center)
        .into()
}
