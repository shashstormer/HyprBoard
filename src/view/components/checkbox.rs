use iced::Element;
use iced::widget::checkbox;

pub fn toggle<'a, Message>(
    label: impl Into<String>,
    is_checked: bool,
    on_toggle: impl Fn(bool) -> Message + 'a,
) -> Element<'a, Message>
where
    Message: Clone + 'a,
{
    iced::widget::row![
        checkbox(is_checked).on_toggle(on_toggle),
        iced::widget::text(label.into())
    ]
    .spacing(10)
    .into()
}
