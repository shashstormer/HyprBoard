use iced::widget::slider;
use std::ops::RangeInclusive;

pub fn range<'a, T, Message>(
    range: RangeInclusive<T>,
    value: T,
    on_change: impl Fn(T) -> Message + 'a,
) -> slider::Slider<'a, T, Message>
where
    T: Copy + From<u8> + std::cmp::PartialOrd + 'a + num_traits::cast::FromPrimitive,
    Message: Clone + 'a,
    f64: From<T>,
{
    slider(range, value, on_change).style(|theme: &iced::Theme, status| {
        let palette = crate::view::components::theme::get_palette(theme);
        let rail = slider::Rail {
            backgrounds: (palette.blue.into(), palette.surface0.into()),
            width: 4.0,
            border: iced::Border {
                radius: 2.0.into(),
                ..Default::default()
            },
        };
        let handle = slider::Handle {
            shape: slider::HandleShape::Circle { radius: 8.0 },
            background: palette.blue.into(),
            border_width: 0.0,
            border_color: iced::Color::TRANSPARENT,
        };

        match status {
            slider::Status::Hovered | slider::Status::Dragged => slider::Style {
                rail: slider::Rail {
                    backgrounds: (palette.blue.into(), palette.surface1.into()),
                    ..rail
                },
                handle: slider::Handle {
                    background: palette.blue.into(),
                    ..handle
                },
            },
            slider::Status::Active => slider::Style { rail, handle },
        }
    })
}
