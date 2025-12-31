use super::button as btn;
use super::text_input;
use iced::widget::{container, row, text};
use iced::{Element, Length};

pub fn file_picker<'a, Message>(
    value: &str,
    placeholder: &str,
    on_change: impl Fn(String) -> Message + 'a,
    on_browse: Message,
) -> Element<'a, Message>
where
    Message: Clone + 'a,
{
    row![
        container(text_input::input(placeholder, value, on_change)).width(Length::Fill),
        btn::secondary(
            text("📂").font(iced::font::Font::with_name("Symbols Nerd Font Mono")),
            on_browse
        )
    ]
    .spacing(10)
    .align_y(iced::Alignment::Center)
    .into()
}
