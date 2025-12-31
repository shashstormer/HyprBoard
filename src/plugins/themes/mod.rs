use crate::core::presets::{Bundle, BundleManager, PresetManager};
use crate::core::{AppMessage, Plugin, PluginMsg};
use crate::view::components::{button as btn, card, text_input as ti, theme::AppTheme};
use iced::{
    Element, Length, Task,
    widget::{button, column, container, row, scrollable, text},
};
use std::collections::HashMap;

pub struct ThemesPlugin {
    id: usize,
    bundle_manager: BundleManager,
    bundles: Vec<Bundle>,
    input_value: String,
}

impl ThemesPlugin {
    pub fn new(id: usize) -> Self {
        let manager = BundleManager::new();
        let bundles = manager.list();
        Self {
            id,
            bundle_manager: manager,
            bundles,
            input_value: String::new(),
        }
    }
}

impl Plugin for ThemesPlugin {
    fn name(&self) -> String {
        "Themes".to_string()
    }

    fn icon(&self) -> char {
        '🎨'
    }

    fn update(&mut self, message: PluginMsg) -> Task<AppMessage> {
        match message {
            PluginMsg::InputChanged(val) => {
                self.input_value = val;
            }
            PluginMsg::Edit(action, _, data) => match action.as_str() {
                "bundle_create" => {
                    let name = self.input_value.trim();
                    if !name.is_empty() {
                        let mut items = HashMap::new();

                        if let Some(h) = PresetManager::new("hyprland").get_active() {
                            items.insert("hyprland".to_string(), h);
                        }
                        if let Some(w) = PresetManager::new("waybar").get_active() {
                            items.insert("waybar".to_string(), w);
                        }
                        if let Some(l) = PresetManager::new("hyprlock").get_active() {
                            items.insert("hyprlock".to_string(), l);
                        }

                        let bundle = Bundle {
                            name: name.to_string(),
                            items,
                        };

                        let _ = self.bundle_manager.save(&bundle);
                        self.bundles = self.bundle_manager.list();
                        self.input_value.clear();
                    }
                }
                "bundle_delete" => {
                    let _ = self.bundle_manager.delete(&data);
                    self.bundles = self.bundle_manager.list();
                }
                "bundle_apply" => {
                    if let Some(bundle) = self.bundles.iter().find(|b| b.name == data) {
                        return Task::done(AppMessage::ApplyBundle(bundle.clone()));
                    }
                }
                _ => {}
            },
            _ => {}
        }
        Task::none()
    }

    fn view<'a>(&'a self, theme: &'a AppTheme) -> Element<'a, AppMessage> {
        let id = self.id;
        let create_section = row![
            container(ti::input("New Bundle Name", &self.input_value, move |v| {
                AppMessage::PluginMessage(id, PluginMsg::InputChanged(v))
            }))
            .width(Length::Fixed(200.0)),
            btn::primary(
                text("Create Bundle from Active Presets"),
                AppMessage::PluginMessage(
                    id,
                    PluginMsg::Edit("bundle_create".into(), "".into(), "".into())
                )
            )
        ]
        .spacing(10)
        .align_y(iced::Alignment::Center);

        let list = column(
            self.bundles
                .iter()
                .map(|bundle| {
                    let bundle_name = bundle.name.clone();
                    let bundle_name2 = bundle.name.clone();
                    container(card::card(
                        row![
                            column![
                                text(bundle.name.clone()).size(16).font(iced::font::Font {
                                    weight: iced::font::Weight::Bold,
                                    ..Default::default()
                                }),
                                text(format!("{} presets", bundle.items.len()))
                                    .size(12)
                                    .style(|theme: &iced::Theme| {
                                        let palette =
                                            crate::view::components::theme::get_palette(theme);
                                        iced::widget::text::Style {
                                            color: Some(palette.overlay1),
                                        }
                                    }),
                            ]
                            .width(Length::Fill),
                            row![
                                btn::small_primary(
                                    text("Apply"),
                                    AppMessage::PluginMessage(
                                        id,
                                        PluginMsg::Edit(
                                            "bundle_apply".into(),
                                            "".into(),
                                            bundle_name
                                        )
                                    )
                                ),
                                btn::small_destructive(
                                    text("Delete"),
                                    AppMessage::PluginMessage(
                                        id,
                                        PluginMsg::Edit(
                                            "bundle_delete".into(),
                                            "".into(),
                                            bundle_name2
                                        )
                                    )
                                ),
                            ]
                            .spacing(8)
                        ]
                        .align_y(iced::Alignment::Center)
                        .spacing(10),
                    ))
                    .style(|theme: &iced::Theme| {
                        let palette = crate::view::components::theme::get_palette(theme);
                        container::Style {
                            background: Some(iced::Background::Color(palette.base)),
                            border: iced::Border {
                                radius: 8.0.into(),
                                ..Default::default()
                            },
                            ..Default::default()
                        }
                    })
                    .into()
                })
                .collect::<Vec<_>>(),
        )
        .spacing(10);

        let themes_list = row(AppTheme::all().iter().map(|t| {
            let is_active = t == theme;
            let theme_val = *t;

            let card_content = column![
                text(t.to_string()).size(16).font(iced::font::Font {
                    weight: iced::font::Weight::Bold,
                    ..Default::default()
                }),
                if is_active {
                    text("Active").size(12).style(|theme: &iced::Theme| {
                        let palette = crate::view::components::theme::get_palette(theme);
                        iced::widget::text::Style {
                            color: Some(palette.green),
                        }
                    })
                } else {
                    text("Click to Apply")
                        .size(12)
                        .style(|theme: &iced::Theme| {
                            let palette = crate::view::components::theme::get_palette(theme);
                            iced::widget::text::Style {
                                color: Some(palette.overlay2),
                            }
                        })
                }
            ]
            .spacing(5);

            button(container(card_content).padding(15))
                .on_press(AppMessage::SetTheme(theme_val))
                .style(move |th: &iced::Theme, status| {
                    let palette = crate::view::components::theme::get_palette(th);
                    let b = iced::widget::button::secondary(th, status);
                    let is_hovered = status == iced::widget::button::Status::Hovered;

                    let border_color = if is_active {
                        palette.blue
                    } else if is_hovered {
                        palette.surface2
                    } else {
                        palette.surface0
                    };

                    iced::widget::button::Style {
                        background: Some(iced::Background::Color(if is_active {
                            crate::view::components::theme::Palette::with_alpha(palette.blue, 0.1)
                        } else {
                            palette.base
                        })),
                        border: iced::Border {
                            color: border_color,
                            width: 2.0,
                            radius: 8.0.into(),
                        },
                        text_color: if is_active {
                            palette.text
                        } else {
                            palette.subtext0
                        },
                        ..b
                    }
                })
                .into()
        }))
        .spacing(15)
        .wrap();

        column![
            text("Appearance").size(24).font(iced::font::Font {
                weight: iced::font::Weight::Bold,
                ..Default::default()
            }),
            themes_list,
            text("Theme Bundles").size(24).font(iced::font::Font {
                weight: iced::font::Weight::Bold,
                ..Default::default()
            }),
            text("Bundles verify that your active presets across all plugins are saved together.")
                .size(14)
                .style(|theme: &iced::Theme| {
                    let palette = crate::view::components::theme::get_palette(theme);
                    iced::widget::text::Style {
                        color: Some(palette.subtext0),
                    }
                }),
            create_section,
            scrollable(list).height(Length::Fill)
        ]
        .spacing(20)
        .padding(20)
        .into()
    }
}
