use iced::widget::{column, container, text};
use iced::{Color, Element, Length, Theme};

pub fn section<'a, Message: 'a>(title: &'a str, description: &'a str) -> Element<'a, Message> {
    let divider = container(text(""))
        .width(Length::Fill)
        .height(1)
        .style(|theme: &Theme| {
            let palette = crate::view::components::theme::get_palette(theme);
            container::Style {
                background: Some(palette.surface0.into()),
                ..Default::default()
            }
        });

    column![
        text(title)
            .size(18)
            .font(iced::font::Font {
                weight: iced::font::Weight::Bold,
                ..Default::default()
            })
            .style(|theme: &Theme| {
                let palette = crate::view::components::theme::get_palette(theme);
                text::Style {
                    color: Some(palette.blue),
                }
            }),
        text(description).size(13).style(|_: &Theme| text::Style {
            color: Some(Color::from_rgb8(127, 132, 156)),
        }),
        divider,
    ]
    .spacing(8)
    .padding(16)
    .into()
}

pub fn section_compact<'a, Message: 'a>(title: &'a str) -> Element<'a, Message> {
    column![
        text(title)
            .size(15)
            .font(iced::font::Font {
                weight: iced::font::Weight::Bold,
                ..Default::default()
            })
            .style(|_: &Theme| text::Style {
                color: Some(Color::from_rgb8(166, 173, 200)),
            }),
    ]
    .padding(12)
    .into()
}
