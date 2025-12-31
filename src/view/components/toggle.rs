use iced::widget::{button, container, row};
use iced::{Border, Element, Length, Theme};

pub fn toggle<'a, Message: Clone + 'a>(is_on: bool, on_toggle: Message) -> Element<'a, Message> {
    let knob = container("")
        .width(18)
        .height(18)
        .style(move |theme: &Theme| container::Style {
            background: Some(theme.palette().background.into()), // Knob color -> background to contrast
            border: Border {
                radius: 9.0.into(),
                ..Default::default()
            },
            ..Default::default()
        });

    let track_content = if is_on {
        row![container("").width(Length::Fill), knob,].padding([2, 3])
    } else {
        row![knob, container("").width(Length::Fill),].padding([2, 3])
    };

    button(
        container(track_content)
            .width(44)
            .height(24)
            .style(move |theme: &Theme| {
                let color = if is_on {
                    theme.palette().primary
                } else {
                    // Dimmed text or a neutral color for off state
                    let mut c = theme.palette().text;
                    c.a = 0.3;
                    c
                };

                container::Style {
                    background: Some(color.into()),
                    border: Border {
                        radius: 12.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                }
            }),
    )
    .on_press(on_toggle)
    .padding(0)
    .style(|_: &Theme, _| button::Style {
        background: None,
        ..Default::default()
    })
    .into()
}

pub fn toggle_labeled<'a, Message: Clone + 'a>(
    label: &'a str,
    is_on: bool,
    on_toggle: Message,
) -> Element<'a, Message> {
    row![
        iced::widget::text(label).width(Length::Fill),
        toggle(is_on, on_toggle),
    ]
    .spacing(10)
    .align_y(iced::Alignment::Center)
    .into()
}
