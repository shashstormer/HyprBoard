use iced::widget::{container, text};
use iced::{Color, Element, Theme};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Style {
    Neutral,
    Success,
    Warning,
    Danger,
    Info,
}

pub fn badge<'a, Message: 'a>(label: impl ToString, style: Style) -> Element<'a, Message> {
    let (text_col, bg_col) = match style {
        Style::Neutral => (
            Color::from_rgb(0.8, 0.8, 0.8),
            Color::from_rgb(0.2, 0.2, 0.2),
        ),
        Style::Success => (
            Color::from_rgb(0.0, 0.0, 0.0),
            Color::from_rgb(0.6, 0.9, 0.6),
        ),
        Style::Warning => (
            Color::from_rgb(0.0, 0.0, 0.0),
            Color::from_rgb(0.9, 0.8, 0.4),
        ),
        Style::Danger => (
            Color::from_rgb(1.0, 1.0, 1.0),
            Color::from_rgb(0.9, 0.4, 0.4),
        ),
        Style::Info => (
            Color::from_rgb(1.0, 1.0, 1.0),
            Color::from_rgb(0.4, 0.6, 0.9),
        ),
    };

    container(
        text(label.to_string())
            .size(12)
            .style(move |_theme: &Theme| text::Style {
                color: Some(text_col),
            }),
    )
    .padding([2, 8])
    .style(move |_theme: &Theme| container::Style {
        background: Some(bg_col.into()),
        border: iced::Border {
            radius: 10.0.into(),
            ..Default::default()
        },
        ..Default::default()
    })
    .into()
}
