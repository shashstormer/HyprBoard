use iced::widget::{Container, container};
use iced::{Element, Length, Shadow, Theme};

pub fn card<'a, Message>(content: impl Into<Element<'a, Message>>) -> Container<'a, Message> {
    container(content)
        .padding(20)
        .style(|theme: &Theme| {
            let palette = crate::view::components::theme::get_palette(theme);
            container::Style {
                background: Some(palette.base.into()),
                border: iced::Border {
                    color: palette.surface1,
                    width: 1.0,
                    radius: 12.0.into(),
                },
                shadow: Shadow {
                    color: crate::view::components::theme::Palette::with_alpha(
                        iced::Color::BLACK,
                        0.3,
                    ),
                    offset: iced::Vector::new(0.0, 4.0),
                    blur_radius: 12.0,
                },
                ..Default::default()
            }
        })
        .width(Length::Fill)
}
