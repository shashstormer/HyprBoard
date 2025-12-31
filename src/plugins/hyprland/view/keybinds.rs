use crate::core::{AppMessage, PluginMsg};
use crate::plugins::hyprland::helpers::types::Keybind;
use crate::view::components::{button as btn, card, text_input as ti};
use iced::widget::Id;
use iced::{
    Color, Element, Length,
    widget::{column, container, row, scrollable, text},
};
use std::collections::HashMap;

fn header_cell(label: &'static str, width: u16) -> Element<'static, AppMessage> {
    container(
        text(label)
            .size(12)
            .font(iced::font::Font {
                weight: iced::font::Weight::Bold,
                ..Default::default()
            })
            .style(|theme: &iced::Theme| {
                let palette = crate::view::components::theme::get_palette(theme);
                iced::widget::text::Style {
                    color: Some(palette.subtext0),
                }
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
            .style(|theme: &iced::Theme| {
                let palette = crate::view::components::theme::get_palette(theme);
                container::Style {
                    background: Some(palette.surface0.into()),
                    border: iced::Border {
                        radius: 4.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                }
            })
            .padding([2, 6]),
    )
    .width(Length::Fixed(width as f32))
    .padding(4)
    .into()
}

pub fn view<'a>(
    binds: &'a [Keybind],
    highlighted_id: Option<String>,
    filter: &'a str,
) -> Element<'a, AppMessage> {
    let add_btn = btn::small_primary(
        text("+ Add Keybind"),
        AppMessage::PluginMessage(0, PluginMsg::OpenModal("add_bind".to_string())),
    );

    let filtered_binds: Vec<&Keybind> = if filter.is_empty() {
        binds.iter().collect()
    } else {
        let f = filter.to_lowercase();
        binds
            .iter()
            .filter(|b| {
                b.dispatcher.to_lowercase().contains(&f)
                    || b.params.to_lowercase().contains(&f)
                    || b.key.to_lowercase().contains(&f)
                    || b.mods.to_lowercase().contains(&f)
                    || b.raw.to_lowercase().contains(&f)
            })
            .collect()
    };

    let mut duplicates: std::collections::HashSet<(String, String, String)> =
        std::collections::HashSet::new();
    let mut seen: std::collections::HashSet<(String, String, String)> =
        std::collections::HashSet::new();

    for bind in binds {
        let key = (bind.bind_type.clone(), bind.mods.clone(), bind.key.clone());
        if !seen.insert(key.clone()) {
            duplicates.insert(key);
        }
    }

    let mut groups: HashMap<String, Vec<&Keybind>> = HashMap::new();
    for bind in filtered_binds {
        groups.entry(bind.bind_type.clone()).or_default().push(bind);
    }

    let mut sorted_keys: Vec<_> = groups.keys().cloned().collect();
    sorted_keys.sort();

    let groups_list = column(
        sorted_keys
            .into_iter()
            .map(|key| {
                let group_binds = groups.get(&key).unwrap();
                let count = group_binds.len();

                let header = container(row![
                    header_cell("Type", 70),
                    header_cell("Mods", 90),
                    header_cell("Key", 70),
                    header_cell("Dispatcher", 110),
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
                .style(|theme: &iced::Theme| {
                    let palette = crate::view::components::theme::get_palette(theme);
                    container::Style {
                        background: Some(palette.surface0.into()),
                        border: iced::Border {
                            radius: 8.0.into(),
                            ..Default::default()
                        },
                        ..Default::default()
                    }
                });

                fn cell_fill(content: String) -> Element<'static, AppMessage> {
                    container(text(content).size(13))
                        .width(Length::Fill)
                        .padding(8)
                        .into()
                }

                let rows = column(
                    group_binds
                        .iter()
                        .enumerate()
                        .map(|(i, bind)| {
                            let edit_msg = AppMessage::PluginMessage(
                                0,
                                PluginMsg::OpenModal(format!("edit_bind:{}", bind.raw)),
                            );
                            let delete_msg = AppMessage::PluginMessage(
                                0,
                                PluginMsg::Edit(
                                    "delete".to_string(),
                                    "bind".to_string(),
                                    bind.raw.clone(),
                                ),
                            );

                            let conflict_key =
                                (bind.bind_type.clone(), bind.mods.clone(), bind.key.clone());

                            let is_conflict = duplicates.contains(&conflict_key);

                            let is_highlighted = highlighted_id
                                .as_ref()
                                .map(|h| *h == bind.raw)
                                .unwrap_or(false);
                            let id = bind.raw.clone();

                            container(row![
                                code_cell(bind.bind_type.clone(), 70),
                                cell(bind.mods.clone(), 90),
                                code_cell(bind.key.clone(), 70),
                                code_cell(bind.key.clone(), 70),
                                cell(bind.dispatcher.clone(), 110),
                                cell_fill(if bind.params.is_empty() {
                                    "-".to_string()
                                } else {
                                    bind.params.clone()
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
                            .style(move |theme: &iced::Theme| {
                                let palette = crate::view::components::theme::get_palette(theme);
                                let bg = if is_conflict {
                                    crate::view::components::theme::Palette::with_alpha(
                                        palette.red,
                                        0.1,
                                    )
                                } else if is_highlighted {
                                    crate::view::components::theme::Palette::with_alpha(
                                        palette.yellow,
                                        0.2,
                                    )
                                } else if i % 2 == 0 {
                                    palette.crust
                                } else {
                                    palette.base
                                };

                                container::Style {
                                    background: Some(iced::Background::Color(bg)),
                                    ..Default::default()
                                }
                            })
                            .into()
                        })
                        .collect::<Vec<_>>(),
                );

                let table = card::card(column![header, rows]);

                column![
                    text(format!("{} ({})", key, count))
                        .size(14)
                        .font(iced::font::Font {
                            weight: iced::font::Weight::Bold,
                            ..Default::default()
                        })
                        .style(|_| iced::widget::text::Style {
                            color: Some(Color::from_rgb8(148, 156, 187))
                        }),
                    table
                ]
                .spacing(8)
                .into()
            })
            .collect::<Vec<_>>(),
    )
    .spacing(24);

    column![
        row![
            text(format!("Keybinds ({})", binds.len()))
                .size(18)
                .font(iced::font::Font {
                    weight: iced::font::Weight::Bold,
                    ..Default::default()
                }),
            ti::input("Filter by dispatcher, key, params...", filter, |s| {
                AppMessage::PluginMessage(
                    0,
                    PluginMsg::Edit("input".into(), "keybind_filter".into(), s),
                )
            }),
            add_btn
        ]
        .spacing(20)
        .align_y(iced::Alignment::Center),
        scrollable(groups_list)
            .id(Id::new("keybinds_scroll"))
            .height(Length::Fill)
    ]
    .spacing(16)
    .into()
}
