use iced::{
    Border, Color, Element, Length, Shadow,
    widget::{button, column, container, stack},
};

pub fn overlay<'a, Message>(
    content: Element<'a, Message>,
    on_close: Message,
    is_open: bool,
) -> Element<'a, Message>
where
    Message: Clone + 'a,
{
    if !is_open {
        return content;
    }

    let backdrop = button(
        container(column![])
            .width(Length::Fill)
            .height(Length::Fill),
    )
    .on_press(on_close)
    .padding(0)
    .width(Length::Fill)
    .height(Length::Fill)
    .style(|_, _| iced::widget::button::Style {
        background: Some(iced::Background::Color(Color::from_rgba8(0, 0, 0, 0.6))),
        ..Default::default()
    });

    let content_wrapper = container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .padding(20);

    stack![backdrop, content_wrapper].into()
}

pub fn container_style(_theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(iced::Background::Color(Color::from_rgb8(30, 30, 46))),
        border: Border {
            color: Color::from_rgba8(255, 255, 255, 0.08),
            width: 1.0,
            radius: 16.0.into(),
        },
        shadow: Shadow {
            color: Color::from_rgba8(0, 0, 0, 0.5),
            offset: iced::Vector::new(0.0, 10.0),
            blur_radius: 40.0,
        },
        ..Default::default()
    }
}
