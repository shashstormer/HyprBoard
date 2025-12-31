use crate::core::{AppMessage, PluginMsg};
use crate::plugins::hyprland::helpers::types::ExecCommand;
use crate::view::components::{badge, button as btn, card, text_input as ti};
use iced::widget::Id;
use iced::{
    Element, Length,
    widget::{column, container, row, scrollable, text},
};

pub fn view<'a>(
    cmds: &[ExecCommand],
    highlighted_id: Option<String>,
    filter: &'a str,
) -> Element<'a, AppMessage> {
    let add_btn = btn::small_primary(
        text("+ Add Command"),
        AppMessage::PluginMessage(0, PluginMsg::OpenModal("add_exec".to_string())),
    );

    let search_bar = container(ti::input("Search commands...", filter, |s| {
        AppMessage::PluginMessage(0, PluginMsg::Edit("input".into(), "exec_filter".into(), s))
    }))
    .width(Length::Fixed(250.0));

    let list = column(
        cmds.iter()
            .filter(|cmd| {
                if filter.is_empty() {
                    return true;
                }
                let f = filter.to_lowercase();
                cmd.command.to_lowercase().contains(&f) || cmd.exec_type.to_lowercase().contains(&f)
            })
            .map(|cmd| {
                let edit_msg = AppMessage::PluginMessage(
                    0,
                    PluginMsg::OpenModal(format!("edit_exec:{}", cmd.raw)),
                );
                let delete_msg = AppMessage::PluginMessage(
                    0,
                    PluginMsg::Edit("delete".to_string(), "exec".to_string(), cmd.raw.clone()),
                );

                let is_highlighted = highlighted_id
                    .as_ref()
                    .map(|h| *h == cmd.raw)
                    .unwrap_or(false);

                container(card::card(
                    row![
                        column![
                            badge::badge(
                                &cmd.exec_type,
                                if cmd.exec_type == "exec-once" {
                                    badge::Style::Success
                                } else {
                                    badge::Style::Info
                                }
                            ),
                            text(cmd.command.clone()).size(14).font(iced::font::Font {
                                weight: iced::font::Weight::Bold,
                                ..Default::default()
                            }),
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
                .id(Id::from(cmd.raw.clone()))
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
    .spacing(12);

    column![
        row![
            text(format!("Startup Commands ({})", cmds.len()))
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
        scrollable(list).height(Length::Fill)
    ]
    .spacing(20)
    .into()
}
