use crate::core::{AppMessage, PluginMsg};
use crate::plugins::hyprland::helpers::types::WindowRule;
use crate::view::components::{button as btn, card, text_input as ti};
use iced::widget::Id;
use iced::{
    Element, Length,
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
    rules: &[WindowRule],
    highlighted_id: Option<String>,
    filter: &'a str,
) -> Element<'a, AppMessage> {
    let add_btn = btn::small_primary(
        text("+ Add Rule"),
        AppMessage::PluginMessage(0, PluginMsg::OpenModal("add_window_rule".to_string())),
    );

    let search_bar = container(ti::input("Search rules...", filter, |s| {
        AppMessage::PluginMessage(0, PluginMsg::Edit("input".into(), "rule_filter".into(), s))
    }))
    .width(Length::Fixed(250.0));

    let header = container(row![
        header_cell("Name", 100),
        header_cell("Props (match:*)", 200),
        container(
            text("Effects")
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

    fn code_cell_fill(content: String) -> Element<'static, AppMessage> {
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
        .width(Length::Fill)
        .padding(4)
        .into()
    }

    let rows = column(
        rules
            .iter()
            .filter(|rule| {
                if filter.is_empty() {
                    return true;
                }
                let f = filter.to_lowercase();
                rule.match_str().to_lowercase().contains(&f)
                    || rule.effect_str().to_lowercase().contains(&f)
                    || rule
                        .name
                        .as_ref()
                        .map(|n| n.to_lowercase().contains(&f))
                        .unwrap_or(false)
            })
            .enumerate()
            .map(|(i, rule)| {
                let edit_msg = AppMessage::PluginMessage(
                    0,
                    PluginMsg::OpenModal(format!("edit_window_rule:{}", rule.raw)),
                );
                let delete_msg = AppMessage::PluginMessage(
                    0,
                    PluginMsg::Edit(
                        "delete".to_string(),
                        "window_rule".to_string(),
                        rule.raw.clone(),
                    ),
                );

                let id = rule.raw.clone();
                let is_highlighted = highlighted_id.as_ref().map(|h| *h == id).unwrap_or(false);

                let name_display = rule.name.clone().unwrap_or_else(|| "-".to_string());
                let props_display = rule.match_str();
                let effects_display = rule.effect_str();

                container(row![
                    cell(name_display, 100),
                    code_cell(props_display, 200),
                    code_cell_fill(effects_display),
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
                    let bg = if is_highlighted {
                        crate::view::components::theme::Palette::with_alpha(palette.yellow, 0.2)
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
        row![
            text(format!("Window Rules ({})", rules.len()))
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
