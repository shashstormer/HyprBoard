use crate::core::{AppMessage, PluginMsg};
use crate::plugins::hyprland::helpers::types::Gesture;
use crate::view::components::{button as btn, card, text_input as ti};
use iced::widget::Id;
use iced::{
    Color, Element, Length,
    widget::{column, container, row, scrollable, text},
};

fn header_cell(label: &'static str, width: u16) -> Element<'static, AppMessage> {
    container(
        text(label)
            .size(12)
            .font(iced::font::Font {
                weight: iced::font::Weight::Bold,
                ..Default::default()
            })
            .style(|_| iced::widget::text::Style {
                color: Some(Color::from_rgb8(166, 173, 200)),
            }),
    )
    .width(Length::Fixed(width as f32))
    .padding(8)
    .into()
}

fn cell(content: String, width: u16) -> Element<'static, AppMessage> {
    container(text(content).size(13))
        .width(Length::Fixed(width as f32))
        .padding(8)
        .into()
}

fn code_cell(content: String, width: u16) -> Element<'static, AppMessage> {
    container(
        container(text(content).size(12).font(iced::font::Font::MONOSPACE))
            .style(|_| container::Style {
                background: Some(iced::Background::Color(Color::from_rgb8(30, 30, 46))),
                border: iced::Border {
                    radius: 4.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            })
            .padding([2, 6]),
    )
    .width(Length::Fixed(width as f32))
    .padding(4)
    .into()
}

pub fn view<'a>(
    gestures: &[Gesture],
    highlighted_id: Option<String>,
    filter: &'a str,
) -> Element<'a, AppMessage> {
    let add_btn = btn::small_primary(
        text("+ Add Gesture"),
        AppMessage::PluginMessage(0, PluginMsg::OpenModal("add_gesture".to_string())),
    );

    let search_bar = container(ti::input("Search gestures...", filter, |s| {
        AppMessage::PluginMessage(
            0,
            PluginMsg::Edit("input".into(), "gesture_filter".into(), s),
        )
    }))
    .width(Length::Fixed(250.0));

    let header = container(row![
        header_cell("Fingers", 70),
        header_cell("Direction", 90),
        header_cell("Mod/Scale", 100),
        header_cell("Action", 120),
        container(
            text("Params")
                .size(12)
                .font(iced::font::Font {
                    weight: iced::font::Weight::Bold,
                    ..Default::default()
                })
                .style(|_| iced::widget::text::Style {
                    color: Some(Color::from_rgb8(166, 173, 200))
                })
        )
        .width(Length::Fill)
        .padding(8),
        header_cell("Actions", 120),
    ])
    .style(|_| container::Style {
        background: Some(iced::Background::Color(Color::from_rgb8(30, 30, 46))),
        border: iced::Border {
            radius: 8.0.into(),
            ..Default::default()
        },
        ..Default::default()
    });

    fn cell_fill(content: String) -> Element<'static, AppMessage> {
        container(text(content).size(13))
            .width(Length::Fill)
            .padding(8)
            .into()
    }

    let rows = column(
        gestures
            .iter()
            .filter(|gesture| {
                if filter.is_empty() {
                    return true;
                }
                let f = filter.to_lowercase();
                gesture.action.to_lowercase().contains(&f)
                    || gesture.dispatcher.to_lowercase().contains(&f)
                    || gesture.params.to_lowercase().contains(&f)
                    || gesture.direction.to_lowercase().contains(&f)
            })
            .enumerate()
            .map(|(i, gesture)| {
                let edit_msg = AppMessage::PluginMessage(
                    0,
                    PluginMsg::OpenModal(format!("edit_gesture:{}", gesture.raw)),
                );
                let delete_msg = AppMessage::PluginMessage(
                    0,
                    PluginMsg::Edit(
                        "delete".to_string(),
                        "gesture".to_string(),
                        gesture.raw.clone(),
                    ),
                );

                let bg = if i % 2 == 0 {
                    Color::from_rgb8(24, 24, 37)
                } else {
                    Color::from_rgb8(30, 30, 46)
                };

                let mod_scale = {
                    let mut parts = Vec::new();
                    if !gesture.mod_key.is_empty() {
                        parts.push(format!("mod:{}", gesture.mod_key));
                    }
                    if !gesture.scale.is_empty() {
                        parts.push(format!("scale:{}", gesture.scale));
                    }
                    if parts.is_empty() {
                        "-".to_string()
                    } else {
                        parts.join(", ")
                    }
                };

                let action_display = if gesture.action == "dispatcher" {
                    format!("dispatcher:{}", gesture.dispatcher)
                } else {
                    gesture.action.clone()
                };

                let id = gesture.raw.clone();
                let is_highlighted = highlighted_id.as_ref().map(|h| *h == id).unwrap_or(false);

                container(row![
                    code_cell(gesture.fingers.to_string(), 70),
                    cell(gesture.direction.clone(), 90),
                    cell(mod_scale, 100),
                    cell(action_display, 120),
                    cell_fill(if gesture.params.is_empty() {
                        "-".to_string()
                    } else {
                        gesture.params.clone()
                    }),
                    container(
                        row![
                            btn::small_secondary(text("Edit"), edit_msg),
                            btn::small_destructive(text("Del"), delete_msg),
                        ]
                        .spacing(4)
                    )
                    .width(Length::Fixed(120.0))
                    .padding(4)
                ])
                .id(Id::from(id))
                .style(move |_| {
                    if is_highlighted {
                        container::Style {
                            background: Some(iced::Color::from_rgba(1.0, 1.0, 0.0, 0.2).into()),
                            ..Default::default()
                        }
                    } else {
                        container::Style {
                            background: Some(iced::Background::Color(bg)),
                            ..Default::default()
                        }
                    }
                })
                .into()
            })
            .collect::<Vec<_>>(),
    );

    let empty_state = if gestures.is_empty() {
        Some(
            container(
                text("No gestures configured. Add a gesture like \"3, swipe, workspace\" to enable touchpad swiping.")
                    .size(14)
                    .style(|_| iced::widget::text::Style { color: Some(Color::from_rgb8(127, 132, 156)) })
            )
            .padding(20)
        )
    } else {
        None
    };

    let table = card::card(column![
        header,
        if let Some(empty) = empty_state {
            Element::from(empty)
        } else {
            rows.into()
        }
    ]);

    column![
        row![
            text(format!("Gestures ({})", gestures.len()))
                .size(18)
                .font(iced::font::Font {
                    weight: iced::font::Weight::Bold,
                    ..Default::default()
                }),
            iced::widget::Space::new().width(Length::Fill),
            search_bar,
            add_btn
        ]
        .spacing(20)
        .align_y(iced::Alignment::Center),
        scrollable(table).height(Length::Fill)
    ]
    .spacing(16)
    .into()
}
