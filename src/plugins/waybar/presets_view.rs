use crate::core::presets::Preset;
use crate::core::waybar_action::WaybarAction;
use crate::core::{AppMessage, PluginMsg};
use crate::view::components::{button as btn, card, text_input as ti};
use iced::{
    Color, Element, Length,
    widget::{column, container, pick_list, row, scrollable, text},
};

pub fn view<'a>(
    presets: &'a [Preset],
    active_preset: Option<&'a String>,
    preset_name_input: &'a str,
) -> Element<'a, AppMessage> {
    let preset_names: Vec<String> = presets.iter().map(|p| p.name.clone()).collect();
    let selected = active_preset.and_then(|name| preset_names.iter().find(|n| *n == name).cloned());

    let selector = row![
        text("Active Preset:").size(14),
        pick_list(preset_names.clone(), selected, |name| {
            AppMessage::PluginMessage(1, PluginMsg::Waybar(WaybarAction::PresetLoad(name)))
        })
        .placeholder("-- Select Preset --"),
    ]
    .spacing(10)
    .align_y(iced::Alignment::Center);

    let save_section = row![
        container(ti::input("Preset Name", preset_name_input, |v| {
            AppMessage::PluginMessage(1, PluginMsg::Waybar(WaybarAction::PresetInput(v)))
        }))
        .width(Length::Fixed(200.0)),
        btn::small_primary(
            text("💾 Save"),
            AppMessage::PluginMessage(1, PluginMsg::Waybar(WaybarAction::PresetSave))
        )
    ]
    .spacing(10)
    .align_y(iced::Alignment::Center);

    let list = column(
        presets
            .iter()
            .map(|preset| {
                let is_active = active_preset.map(|a| a == &preset.name).unwrap_or(false);
                let load_msg = AppMessage::PluginMessage(
                    1,
                    PluginMsg::Waybar(WaybarAction::PresetLoad(preset.name.clone())),
                );
                let delete_msg = AppMessage::PluginMessage(
                    1,
                    PluginMsg::Waybar(WaybarAction::PresetDelete(preset.name.clone())),
                );

                let bg = if is_active {
                    Color::from_rgb8(30, 35, 50)
                } else {
                    Color::from_rgb8(24, 24, 37)
                };

                container(card::card(
                    row![
                        column![
                            row![
                                text(preset.name.clone()).size(16).font(iced::font::Font {
                                    weight: iced::font::Weight::Bold,
                                    ..Default::default()
                                }),
                                if is_active {
                                    container(text("Active").size(10).style(|_| {
                                        iced::widget::text::Style {
                                            color: Some(Color::from_rgb8(166, 227, 161)),
                                        }
                                    }))
                                    .style(|_| container::Style {
                                        background: Some(iced::Background::Color(
                                            Color::from_rgb8(30, 50, 40),
                                        )),
                                        border: iced::Border {
                                            radius: 4.0.into(),
                                            ..Default::default()
                                        },
                                        ..Default::default()
                                    })
                                    .padding([2, 8])
                                    .into()
                                } else {
                                    Element::from(text(""))
                                }
                            ]
                            .spacing(10)
                            .align_y(iced::Alignment::Center),
                            text(preset.description.clone()).size(12).style(|_| {
                                iced::widget::text::Style {
                                    color: Some(Color::from_rgb8(127, 132, 156)),
                                }
                            }),
                        ]
                        .spacing(5)
                        .width(Length::Fill),
                        row![
                            if !is_active {
                                btn::small_primary(text("Load"), load_msg)
                            } else {
                                btn::small_secondary(text("Loaded"), AppMessage::None)
                            },
                            btn::small_destructive(text("Delete"), delete_msg),
                        ]
                        .spacing(8)
                    ]
                    .align_y(iced::Alignment::Center)
                    .spacing(10),
                ))
                .style(move |_| container::Style {
                    background: Some(iced::Background::Color(bg)),
                    border: iced::Border {
                        radius: 8.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .into()
            })
            .collect::<Vec<_>>(),
    )
    .spacing(12);

    let empty_state = if presets.is_empty() {
        Some(
            container(
                text("No presets saved yet. Click 'Save Current Config' to create your first preset.")
                    .size(14)
                    .style(|_| iced::widget::text::Style { color: Some(Color::from_rgb8(127, 132, 156)) })
            )
            .padding(20)
        )
    } else {
        None
    };

    column![
        text("Waybar Presets").size(18).font(iced::font::Font {
            weight: iced::font::Weight::Bold,
            ..Default::default()
        }),
        text("Presets include both config and style.")
            .size(12)
            .style(|_| iced::widget::text::Style {
                color: Some(Color::from_rgb8(160, 160, 160))
            }),
        row![selector, save_section]
            .spacing(20)
            .align_y(iced::Alignment::Center),
        if let Some(empty) = empty_state {
            Element::from(empty)
        } else {
            scrollable(list).height(Length::Fill).into()
        }
    ]
    .spacing(16)
    .into()
}
