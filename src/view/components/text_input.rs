use iced::widget::text_input;
use iced::{Border, Theme};

pub fn input<'a, Message>(
    placeholder: &str,
    value: &str,
    on_input: impl Fn(String) -> Message + 'a,
) -> text_input::TextInput<'a, Message>
where
    Message: Clone + 'a,
{
    text_input(placeholder, value)
        .on_input(on_input)
        .padding([12, 16])
        .style(|theme: &Theme, status| {
            let palette = crate::view::components::theme::get_palette(theme);
            let base = text_input::Style {
                background: iced::Background::Color(palette.base),
                border: Border {
                    color: palette.surface1,
                    width: 1.5,
                    radius: 8.0.into(),
                },
                icon: palette.subtext0,
                placeholder: palette.overlay1,
                value: palette.text,
                selection: palette.blue,
            };
            match status {
                text_input::Status::Focused { is_hovered: _ } => text_input::Style {
                    border: Border {
                        color: palette.blue,
                        width: 2.0,
                        radius: 8.0.into(),
                    },
                    ..base
                },
                text_input::Status::Hovered => text_input::Style {
                    border: Border {
                        color: palette.surface2,
                        width: 1.5,
                        radius: 8.0.into(),
                    },
                    ..base
                },
                _ => base,
            }
        })
}
