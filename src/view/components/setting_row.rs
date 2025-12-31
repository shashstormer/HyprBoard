use super::tooltip::info_tooltip;
use iced::widget::{Container, column, row, text};
use iced::{Element, Length, Theme};

pub fn setting_row<'a, Message: 'a>(
    title: impl Into<String>,
    description: impl Into<String>,
    control: impl Into<Element<'a, Message>>,
) -> Element<'a, Message> {
    let title = title.into();
    let description = description.into();
    row![
        column![
            text(title)
                .size(14)
                .font(iced::font::Font {
                    weight: iced::font::Weight::Bold,
                    ..Default::default()
                })
                .style(|theme: &Theme| text::Style {
                    color: Some(theme.palette().text)
                }),
            text(description)
                .size(12)
                .style(|theme: &Theme| text::Style {
                    color: Some({
                        let mut c = theme.palette().text;
                        c.a = 0.7;
                        c
                    }),
                }),
        ]
        .width(Length::FillPortion(1)),
        Container::new(control)
            .width(Length::FillPortion(1))
            .align_x(iced::alignment::Horizontal::Right)
    ]
    .padding([10, 0])
    .align_y(iced::Alignment::Center)
    .into()
}

pub fn setting_row_with_tooltip<'a, Message: Clone + 'a>(
    title: impl Into<String>,
    description: impl Into<String>,
    tooltip_text: impl Into<String>,
    control: impl Into<Element<'a, Message>>,
) -> Element<'a, Message> {
    let title = title.into();
    let description = description.into();
    let tooltip_text = tooltip_text.into();

    let title_clone = title.clone();
    row![
        column![
            row![
                text(title_clone)
                    .size(14)
                    .font(iced::font::Font {
                        weight: iced::font::Weight::Bold,
                        ..Default::default()
                    })
                    .style(|theme: &Theme| text::Style {
                        color: Some(theme.palette().text)
                    }),
                info_tooltip(Box::leak(tooltip_text.into_boxed_str())),
            ]
            .spacing(6)
            .align_y(iced::Alignment::Center),
            text(description)
                .size(12)
                .style(|theme: &Theme| text::Style {
                    color: Some({
                        let mut c = theme.palette().text;
                        c.a = 0.7;
                        c
                    }),
                }),
        ]
        .width(Length::FillPortion(1)),
        Container::new(control)
            .width(Length::FillPortion(1))
            .align_x(iced::alignment::Horizontal::Right)
    ]
    .padding([10, 0])
    .align_y(iced::Alignment::Center)
    .into()
}

pub fn setting_row_compact<'a, Message: 'a>(
    title: impl Into<String>,
    control: impl Into<Element<'a, Message>>,
) -> Element<'a, Message> {
    let title = title.into();
    row![
        text(title).size(14).style(|theme: &Theme| text::Style {
            color: Some(theme.palette().text)
        }),
        Container::new(control)
            .width(Length::Fill)
            .align_x(iced::alignment::Horizontal::Right)
    ]
    .padding([8, 0])
    .align_y(iced::Alignment::Center)
    .into()
}
