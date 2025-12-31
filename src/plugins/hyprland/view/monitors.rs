use crate::core::AppMessage;
use crate::plugins::hyprland::helpers::types::Monitor;
use crate::view::components::{badge, card, setting_row};
use iced::{
    Element,
    widget::{column, row, text},
};

pub fn view(monitors: &[Monitor]) -> Element<'_, AppMessage> {
    let content = column(
        monitors
            .iter()
            .map(|mon| {
                let status = if mon.disabled { "Disabled" } else { "Active" };
                let badge_style = if mon.disabled {
                    badge::Style::Neutral
                } else {
                    badge::Style::Success
                };

                card::card(
                    column![
                        row![
                            text(mon.name.clone()).size(16).font(iced::font::Font {
                                weight: iced::font::Weight::Bold,
                                ..Default::default()
                            }),
                            badge::badge(status, badge_style),
                        ]
                        .spacing(10)
                        .align_y(iced::Alignment::Center),
                        if !mon.disabled {
                            column![
                                setting_row::setting_row(
                                    "Resolution",
                                    "Width x Height @ Refresh",
                                    Element::from(text(mon.resolution.clone()))
                                ),
                                setting_row::setting_row(
                                    "Position",
                                    "X x Y",
                                    Element::from(text(mon.position.clone()))
                                ),
                                setting_row::setting_row(
                                    "Scale",
                                    "UI Scale Factor",
                                    Element::from(text(mon.scale.clone()))
                                ),
                            ]
                            .spacing(5)
                        } else {
                            column![text("Monitor is disabled in config")]
                        }
                    ]
                    .spacing(10),
                )
                .into()
            })
            .collect::<Vec<_>>(),
    )
    .spacing(10);

    content.into()
}
