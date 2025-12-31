use iced::widget::{container, pick_list, row, text};
use iced::{Element, Length, Theme};

pub fn dropdown<'a, T, L, Message>(
    label: &'a str,
    options: L,
    selected: Option<T>,
    on_select: impl Fn(T) -> Message + 'a,
) -> Element<'a, Message>
where
    T: ToString + PartialEq + Clone + 'a,
    L: std::borrow::Borrow<[T]> + 'a,
    Message: Clone + 'a,
{
    let label_text = text(label).size(14).style(|theme: &Theme| {
        let palette = crate::view::components::theme::get_palette(theme);
        text::Style {
            color: Some(palette.text),
        }
    });

    let picker = pick_list(options, selected, on_select)
        .placeholder("Select...")
        .padding([6, 10])
        .text_size(13)
        .style(|theme: &Theme, _status| {
            let palette = crate::view::components::theme::get_palette(theme);
            pick_list::Style {
                background: palette.base.into(),
                text_color: palette.text,
                placeholder_color: palette.overlay1,
                border: iced::Border {
                    color: palette.surface1,
                    width: 1.0,
                    radius: 6.0.into(),
                },
                handle_color: palette.blue,
            }
        });

    row![
        label_text,
        container(picker)
            .width(Length::Fill)
            .align_x(iced::alignment::Horizontal::Right),
    ]
    .spacing(12)
    .align_y(iced::Alignment::Center)
    .padding([8, 0])
    .into()
}

pub fn dropdown_compact<'a, T, L, Message>(
    options: L,
    selected: Option<T>,
    on_select: impl Fn(T) -> Message + 'a,
) -> pick_list::PickList<'a, T, L, T, Message>
where
    T: ToString + PartialEq + Clone + 'a,
    L: std::borrow::Borrow<[T]> + 'a,
    Message: Clone + 'a,
{
    pick_list(options, selected, on_select)
        .placeholder("Select...")
        .padding([6, 10])
        .text_size(13)
        .style(|theme: &Theme, _status| {
            let palette = crate::view::components::theme::get_palette(theme);
            pick_list::Style {
                background: palette.base.into(),
                text_color: palette.text,
                placeholder_color: palette.overlay1,
                border: iced::Border {
                    color: palette.surface1,
                    width: 1.0,
                    radius: 6.0.into(),
                },
                handle_color: palette.blue,
            }
        })
}
