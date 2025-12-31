use crate::core::{AppMessage, PluginMsg};
use crate::view::components::modal;
use iced::{Element, Length, widget::container};

pub fn modal<'a>(content: Element<'a, AppMessage>) -> Element<'a, AppMessage> {
    let modal_card = container(content)
        .width(Length::Fixed(520.0))
        .padding(24)
        .style(modal::container_style);

    modal::overlay(
        modal_card.into(),
        AppMessage::PluginMessage(0, PluginMsg::CloseModal),
        true,
    )
}
