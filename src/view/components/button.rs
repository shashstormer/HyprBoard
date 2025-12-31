use iced::widget::button;
use iced::{Border, Element, Length, Shadow, Theme};

fn base_button<'a, Message: Clone + 'a>(
    content: impl Into<Element<'a, Message>>,
    on_press: Message,
) -> button::Button<'a, Message> {
    button(content)
        .on_press(on_press)
        .padding(10)
        .width(Length::Fill)
}

fn small_base_button<'a, Message: Clone + 'a>(
    content: impl Into<Element<'a, Message>>,
    on_press: Message,
) -> button::Button<'a, Message> {
    button(content).on_press(on_press).padding([8, 14])
}

pub fn primary<'a, Message: 'a + Clone>(
    content: impl Into<Element<'a, Message>>,
    on_press: Message,
) -> Element<'a, Message> {
    base_button(content, on_press)
        .style(|theme: &Theme, status| {
            let palette = crate::view::components::theme::get_palette(theme);
            let base = button::Style {
                background: Some(palette.teal.into()),
                text_color: palette.crust,
                border: Border {
                    radius: 8.0.into(),
                    ..Default::default()
                },
                shadow: Shadow {
                    color: crate::view::components::theme::Palette::with_alpha(palette.teal, 0.3),
                    offset: iced::Vector::new(0.0, 2.0),
                    blur_radius: 8.0,
                },
                snap: false,
            };
            match status {
                button::Status::Hovered => button::Style {
                    background: Some(
                        crate::view::components::theme::Palette::with_alpha(palette.teal, 0.8)
                            .into(),
                    ),
                    shadow: Shadow {
                        color: crate::view::components::theme::Palette::with_alpha(
                            palette.teal,
                            0.5,
                        ),
                        offset: iced::Vector::new(0.0, 4.0),
                        blur_radius: 12.0,
                    },
                    ..base
                },
                button::Status::Pressed => button::Style {
                    background: Some(
                        crate::view::components::theme::Palette::with_alpha(palette.teal, 0.6)
                            .into(),
                    ),
                    ..base
                },
                _ => base,
            }
        })
        .into()
}

pub fn secondary<'a, Message: 'a + Clone>(
    content: impl Into<Element<'a, Message>>,
    on_press: Message,
) -> Element<'a, Message> {
    base_button(content, on_press)
        .style(|theme: &Theme, status| {
            let palette = crate::view::components::theme::get_palette(theme);
            let base = button::Style {
                background: Some(palette.surface0.into()),
                text_color: palette.text,
                border: Border {
                    radius: 8.0.into(),
                    color: palette.surface1,
                    width: 1.0,
                },
                ..Default::default()
            };
            match status {
                button::Status::Hovered => button::Style {
                    background: Some(palette.surface1.into()),
                    border: Border {
                        radius: 8.0.into(),
                        color: palette.surface2,
                        width: 1.0,
                    },
                    ..base
                },
                _ => base,
            }
        })
        .into()
}

pub fn ghost<'a, Message: 'a + Clone>(
    content: impl Into<Element<'a, Message>>,
    on_press: Message,
) -> Element<'a, Message> {
    base_button(content, on_press)
        .style(|theme: &Theme, status| {
            let palette = crate::view::components::theme::get_palette(theme);
            let base = button::Style {
                background: None,
                text_color: palette.text,
                border: Border {
                    radius: 8.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            };
            match status {
                button::Status::Hovered => button::Style {
                    background: Some(palette.surface0.into()),
                    ..base
                },
                _ => base,
            }
        })
        .into()
}

pub fn destructive<'a, Message: 'a + Clone>(
    content: impl Into<Element<'a, Message>>,
    on_press: Message,
) -> Element<'a, Message> {
    base_button(content, on_press)
        .style(|theme: &Theme, status| {
            let palette = crate::view::components::theme::get_palette(theme);
            let base = button::Style {
                background: Some(palette.red.into()),
                text_color: palette.crust,
                border: Border {
                    radius: 8.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            };
            match status {
                button::Status::Hovered => button::Style {
                    background: Some(
                        crate::view::components::theme::Palette::with_alpha(palette.red, 0.8)
                            .into(),
                    ),
                    ..base
                },
                _ => base,
            }
        })
        .into()
}

pub fn small_secondary<'a, Message: 'a + Clone>(
    content: impl Into<Element<'a, Message>>,
    on_press: Message,
) -> Element<'a, Message> {
    small_base_button(content, on_press)
        .style(|theme: &Theme, status| {
            let palette = crate::view::components::theme::get_palette(theme);
            let base = button::Style {
                background: Some(palette.surface0.into()),
                text_color: palette.text,
                border: Border {
                    radius: 6.0.into(),
                    color: palette.surface1,
                    width: 1.0,
                },
                ..Default::default()
            };
            match status {
                button::Status::Hovered => button::Style {
                    background: Some(palette.surface1.into()),
                    ..base
                },
                _ => base,
            }
        })
        .into()
}

pub fn small_destructive<'a, Message: 'a + Clone>(
    content: impl Into<Element<'a, Message>>,
    on_press: Message,
) -> Element<'a, Message> {
    small_base_button(content, on_press)
        .style(|theme: &Theme, status| {
            let palette = crate::view::components::theme::get_palette(theme);
            let base = button::Style {
                background: Some(
                    crate::view::components::theme::Palette::with_alpha(palette.red, 0.15).into(),
                ),
                text_color: palette.red,
                border: Border {
                    radius: 6.0.into(),
                    color: palette.red,
                    width: 1.0,
                },
                ..Default::default()
            };
            match status {
                button::Status::Hovered => button::Style {
                    background: Some(palette.red.into()),
                    text_color: palette.crust,
                    ..base
                },
                _ => base,
            }
        })
        .into()
}

pub fn small_primary<'a, Message: 'a + Clone>(
    content: impl Into<Element<'a, Message>>,
    on_press: Message,
) -> Element<'a, Message> {
    small_base_button(content, on_press)
        .style(|theme: &Theme, status| {
            let palette = crate::view::components::theme::get_palette(theme);
            let base = button::Style {
                background: Some(palette.blue.into()),
                text_color: palette.crust,
                border: Border {
                    radius: 6.0.into(),
                    ..Default::default()
                },
                shadow: Shadow {
                    color: crate::view::components::theme::Palette::with_alpha(palette.blue, 0.3),
                    offset: iced::Vector::new(0.0, 2.0),
                    blur_radius: 6.0,
                },
                snap: false,
            };
            match status {
                button::Status::Hovered => button::Style {
                    background: Some(
                        crate::view::components::theme::Palette::with_alpha(palette.blue, 0.8)
                            .into(),
                    ),
                    ..base
                },
                _ => base,
            }
        })
        .into()
}
