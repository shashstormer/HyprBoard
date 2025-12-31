use crate::core::{AppMessage, PluginMsg};
use crate::plugins::hyprland::helpers::types::EnvVar;
use crate::view::components::{badge, button as btn, card, text_input as ti};
use iced::widget::Id;
use iced::{
    Element, Length,
    widget::{column, container, row, scrollable, text},
};

pub fn view<'a>(
    vars: &[EnvVar],
    highlighted_id: Option<String>,
    filter: &'a str,
) -> Element<'a, AppMessage> {
    let add_btn = btn::small_primary(
        text("+ Add Environment Variable"),
        AppMessage::PluginMessage(0, PluginMsg::OpenModal("add_env".to_string())),
    );

    let search_bar = container(ti::input("Search variables...", filter, |s| {
        AppMessage::PluginMessage(0, PluginMsg::Edit("input".into(), "env_filter".into(), s))
    }))
    .width(Length::Fixed(250.0));

    let mut gtk_vars = Vec::new();
    let mut qt_vars = Vec::new();
    let mut xdg_vars = Vec::new();
    let mut xcursor_vars = Vec::new();
    let mut nvidia_vars = Vec::new();
    let mut aq_vars = Vec::new();
    let mut hyprland_vars = Vec::new();
    let mut other_vars = Vec::new();

    for var in vars {
        if !filter.is_empty() {
            let f = filter.to_lowercase();
            if !var.name.to_lowercase().contains(&f) && !var.value.to_lowercase().contains(&f) {
                continue;
            }
        }

        let name = var.name.to_uppercase();
        if name.starts_with("GTK") || name.starts_with("GDK") {
            gtk_vars.push(var);
        } else if name.starts_with("QT") {
            qt_vars.push(var);
        } else if name.starts_with("XDG") {
            xdg_vars.push(var);
        } else if name.starts_with("XCURSOR") {
            xcursor_vars.push(var);
        } else if name.contains("NVIDIA")
            || name.starts_with("__GL")
            || name == "GBM_BACKEND"
            || name == "LIBVA_DRIVER_NAME"
        {
            nvidia_vars.push(var);
        } else if name.starts_with("AQ_") {
            aq_vars.push(var);
        } else if name.starts_with("HYPRLAND") {
            hyprland_vars.push(var);
        } else {
            other_vars.push(var);
        }
    }

    let render_group = |title: &'static str,
                        group: Vec<&EnvVar>,
                        highlighted_id: Option<String>|
     -> Option<Element<'_, AppMessage>> {
        if group.is_empty() {
            return None;
        }

        let items = column(
            group
                .into_iter()
                .map(|var| {
                    let edit_msg = AppMessage::PluginMessage(
                        0,
                        PluginMsg::OpenModal(format!("edit_env:{}", var.raw)),
                    );
                    let delete_msg = AppMessage::PluginMessage(
                        0,
                        PluginMsg::Edit("delete".to_string(), "env".to_string(), var.raw.clone()),
                    );

                    let id = var.raw.clone();
                    let is_highlighted = highlighted_id.as_ref().map(|h| *h == id).unwrap_or(false);

                    container(card::card(
                        row![
                            column![
                                text(var.name.clone()).size(14).font(iced::font::Font {
                                    weight: iced::font::Weight::Bold,
                                    ..Default::default()
                                }),
                                badge::badge(&var.value, badge::Style::Neutral),
                            ]
                            .spacing(8)
                            .width(Length::Fill),
                            row![
                                btn::small_secondary(text("Edit"), edit_msg),
                                btn::small_destructive(text("Delete"), delete_msg),
                            ]
                            .spacing(8)
                        ]
                        .align_y(iced::Alignment::Center)
                        .spacing(10),
                    ))
                    .id(Id::from(id))
                    .style(move |_| {
                        if is_highlighted {
                            container::Style {
                                background: Some(iced::Color::from_rgba(1.0, 1.0, 0.0, 0.2).into()),
                                ..Default::default()
                            }
                        } else {
                            container::Style::default()
                        }
                    })
                    .into()
                })
                .collect::<Vec<_>>(),
        )
        .spacing(8);

        Some(
            column![
                text(title)
                    .size(14)
                    .font(iced::font::Font {
                        weight: iced::font::Weight::Bold,
                        ..Default::default()
                    })
                    .style(|_| iced::widget::text::Style {
                        color: Some(iced::Color::from_rgb8(148, 156, 187))
                    }),
                items
            ]
            .spacing(8)
            .into(),
        )
    };

    let groups = vec![
        render_group("GTK/GDK", gtk_vars, highlighted_id.clone()),
        render_group("QT", qt_vars, highlighted_id.clone()),
        render_group("XDG", xdg_vars, highlighted_id.clone()),
        render_group("Cursor", xcursor_vars, highlighted_id.clone()),
        render_group("NVIDIA/Hardware", nvidia_vars, highlighted_id.clone()),
        render_group("Aquamarine", aq_vars, highlighted_id.clone()),
        render_group("Hyprland", hyprland_vars, highlighted_id.clone()),
        render_group("Other", other_vars, highlighted_id.clone()),
    ];

    let content = column(groups.into_iter().flatten()).spacing(20);

    column![
        row![
            text(format!("Environment Variables ({})", vars.len()))
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
        scrollable(content).height(Length::Fill)
    ]
    .spacing(20)
    .into()
}
