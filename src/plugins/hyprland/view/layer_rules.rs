use crate::core::{AppMessage, PluginMsg};
use crate::plugins::hyprland::helpers::types::LayerRule;
use crate::view::components::{button as btn, card, text_input as ti};
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

fn code_cell_fill(content: String) -> Element<'static, AppMessage> {
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
    .width(Length::Fill)
    .padding(4)
    .into()
}

pub fn view<'a>(rules: &[LayerRule], filter: &'a str) -> Element<'a, AppMessage> {
    let add_btn = btn::small_primary(
        text("+ Add Layer Rule"),
        AppMessage::PluginMessage(0, PluginMsg::OpenModal("add_layer_rule".to_string())),
    );

    let search_bar = container(ti::input("Search layer rules...", filter, |s| {
        AppMessage::PluginMessage(0, PluginMsg::Edit("input".into(), "layer_filter".into(), s))
    }))
    .width(Length::Fixed(250.0));

    let header = container(row![
        header_cell("Namespace", 200),
        container(
            text("Effects")
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

    let rows = column(
        rules
            .iter()
            .filter(|rule| {
                if filter.is_empty() {
                    return true;
                }
                let f = filter.to_lowercase();
                let ns = rule
                    .props
                    .iter()
                    .find(|(k, _)| k == "match:namespace")
                    .map(|(_, v)| v.as_str())
                    .unwrap_or("");
                ns.to_lowercase().contains(&f) || rule.raw.to_lowercase().contains(&f)
            })
            .enumerate()
            .map(|(i, rule)| {
                let delete_msg = AppMessage::PluginMessage(
                    0,
                    PluginMsg::Edit(
                        "delete".to_string(),
                        "layer_rule".to_string(),
                        rule.raw.clone(),
                    ),
                );

                let bg = if i % 2 == 0 {
                    Color::from_rgb8(24, 24, 37)
                } else {
                    Color::from_rgb8(30, 30, 46)
                };

                let namespace = rule
                    .props
                    .iter()
                    .find(|(k, _)| k == "match:namespace")
                    .map(|(_, v)| v.clone())
                    .unwrap_or_else(|| "-".to_string());

                let effects = rule
                    .effects
                    .iter()
                    .map(|(k, v)| {
                        if v.is_empty() {
                            k.clone()
                        } else {
                            format!("{} {}", k, v)
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(", ");

                container(row![
                    cell(namespace, 200),
                    code_cell_fill(effects),
                    container(row![btn::small_destructive(text("Del"), delete_msg),].spacing(4))
                        .width(Length::Fixed(120.0))
                        .padding(4)
                ])
                .style(move |_| container::Style {
                    background: Some(iced::Background::Color(bg)),
                    ..Default::default()
                })
                .into()
            })
            .collect::<Vec<_>>(),
    );

    let table = card::card(column![header, rows]);

    column![
        row![
            text(format!("Layer Rules ({})", rules.len()))
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
