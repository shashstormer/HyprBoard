#![allow(dead_code)]
mod config;
mod core;
mod plugins;
mod utils;
mod view;

use crate::view::components::button::ghost;
use crate::view::components::modal;
use crate::view::components::theme::AppTheme;
use core::{AppMessage, Plugin, PluginMsg, SearchResult};
use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;
use iced::Length;
use iced::widget::{column, container, row, scrollable, stack, text, text_input};
use iced::{Border, Color, Element, Shadow, Subscription, Task, Theme, event, keyboard};

struct HyprBoard {
    active_tab_index: usize,
    plugins: Vec<Box<dyn Plugin>>,
    is_search_open: bool,
    search_query: String,
    search_results: Vec<(usize, SearchResult)>,
    search_input_id: iced::widget::Id,
    active_theme: AppTheme,
}

impl HyprBoard {
    fn new() -> (Self, Task<AppMessage>) {
        let config = config::Config::load();

        let mut app = Self {
            active_tab_index: 0,
            plugins: Vec::new(),
            is_search_open: false,
            search_query: String::new(),
            search_results: Vec::new(),
            search_input_id: iced::widget::Id::unique(),
            active_theme: config.theme,
        };

        for plugin_name in config.plugins {
            match plugin_name.as_str() {
                "hyprland" => app.register(plugins::hyprland::HyprlandPlugin::new()),
                "waybar" => {
                    let id = app.plugins.len();
                    app.register(plugins::waybar::WaybarPlugin::new(id))
                }
                "hyprlock" => {
                    let id = app.plugins.len();
                    app.register(plugins::hyprlock::HyprlockPlugin::new(id))
                }
                "themes" => {
                    let id = app.plugins.len();
                    app.register(plugins::themes::ThemesPlugin::new(id))
                }
                _ => println!("Unknown plugin: {}", plugin_name),
            }
        }

        (app, Task::none())
    }

    fn register(&mut self, plugin: impl Plugin + 'static) {
        self.plugins.push(Box::new(plugin));
    }

    fn update(&mut self, message: AppMessage) -> Task<AppMessage> {
        match message {
            AppMessage::SwitchTab(index) => {
                self.active_tab_index = index;
                Task::none()
            }
            AppMessage::PluginMessage(index, msg) => {
                if let Some(plugin) = self.plugins.get_mut(index) {
                    return plugin.update(msg);
                }
                Task::none()
            }
            AppMessage::None => Task::none(),
            AppMessage::ToggleSearch => {
                self.is_search_open = !self.is_search_open;
                if self.is_search_open {
                    self.search_query.clear();
                    self.search_results.clear();
                    return iced::widget::operation::focus(self.search_input_id.clone());
                }
                Task::none()
            }
            AppMessage::CloseSearch => {
                self.is_search_open = false;
                Task::none()
            }
            AppMessage::ApplyBundle(bundle) => {
                let mut tasks = Vec::new();
                for (_idx, plugin) in self.plugins.iter_mut().enumerate() {
                    let name = plugin.name().to_lowercase();
                    if let Some(preset_name) = bundle.items.get(&name) {
                        tasks.push(plugin.update(PluginMsg::LoadPreset(preset_name.clone())));
                    }
                }
                Task::batch(tasks)
            }
            AppMessage::Search(query) => {
                self.search_query = query.clone();
                if query.is_empty() {
                    self.search_results.clear();
                } else {
                    let matcher = SkimMatcherV2::default();
                    let mut scored_results: Vec<(i64, usize, SearchResult)> = self
                        .plugins
                        .iter()
                        .enumerate()
                        .flat_map(|(idx, p)| {
                            p.searchable_items().into_iter().map(move |r| (idx, r))
                        })
                        .filter_map(|(idx, r)| {
                            let score = matcher
                                .fuzzy_match(&r.title, &query)
                                .or_else(|| matcher.fuzzy_match(&r.id, &query));
                            score.map(|s| (s, idx, r))
                        })
                        .collect();

                    scored_results.sort_by(|a, b| b.0.cmp(&a.0));

                    self.search_results = scored_results
                        .into_iter()
                        .take(20)
                        .map(|(_, idx, r)| (idx, r))
                        .collect();
                }
                Task::none()
            }
            AppMessage::JumpToResult(idx, res) => {
                self.is_search_open = false; // Close modal on jump
                self.active_tab_index = idx;
                self.search_query.clear();
                self.search_results.clear();
                if let Some(plugin) = self.plugins.get_mut(idx) {
                    return plugin.update(PluginMsg::JumpTo(res));
                }
                Task::none()
            }

            AppMessage::SetTheme(theme) => {
                self.active_theme = theme;
                // Save theme persistence
                let mut config = config::Config::load();
                config.theme = theme;
                config.save();
                Task::none()
            }
            AppMessage::GlobalKeyPress(key, modifiers) => {
                if !self.is_search_open {
                    if let Some(plugin) = self.plugins.get_mut(self.active_tab_index) {
                        return plugin.update(PluginMsg::KeyPress(key, modifiers));
                    }
                }
                Task::none()
            }
        }
    }

    fn view(&self) -> Element<'_, AppMessage> {
        let sidebar_items = self.plugins.iter().enumerate().map(|(index, plugin)| {
            let is_active = index == self.active_tab_index;

            let icon = text(plugin.icon().to_string())
                .font(iced::font::Font::with_name("Symbols Nerd Font Mono"))
                .size(20);

            let label = text(plugin.name()).size(14);

            let content = row![icon, label]
                .spacing(12)
                .align_y(iced::Alignment::Center);

            let btn_content = container(content)
                .width(Length::Fill)
                .padding([10, 12])
                .style(move |theme: &Theme| {
                    let palette = view::components::theme::get_palette(theme);
                    container::Style {
                        background: if is_active {
                            Some(palette.surface0.into())
                        } else {
                            None
                        },
                        border: iced::Border {
                            radius: 8.0.into(),
                            ..Default::default()
                        },
                        text_color: if is_active {
                            Some(palette.blue)
                        } else {
                            Some(palette.subtext0)
                        },
                        ..Default::default()
                    }
                });

            button(btn_content)
                .on_press(AppMessage::SwitchTab(index))
                .style(|_, _| button::Style::default())
                .padding(0)
                .into()
        });

        let sidebar = container(column(sidebar_items).spacing(8).padding(16))
            .width(220)
            .height(Length::Fill)
            .style(|theme: &Theme| {
                let palette = view::components::theme::get_palette(theme);
                container::Style {
                    background: Some(palette.crust.into()),
                    border: Border {
                        color: palette.mantle,
                        width: 1.0,
                        radius: 0.0.into(),
                    },
                    ..Default::default()
                }
            });

        let content = if let Some(plugin) = self.plugins.get(self.active_tab_index) {
            plugin.view(&self.active_theme)
        } else {
            container(text("No Plugins Loaded")).into()
        };

        let header = container(
            row![
                text("Dashboard")
                    .size(26)
                    .font(iced::font::Font {
                        weight: iced::font::Weight::Bold,
                        ..Default::default()
                    })
                    .style(|theme: &Theme| {
                        let palette = view::components::theme::get_palette(theme);
                        text::Style {
                            color: Some(palette.text),
                        }
                    }),
                iced::widget::Space::new().width(Length::Fill),
                ghost(
                    row![
                        text("").font(iced::font::Font::with_name("Symbols Nerd Font Mono")),
                        text("Search (Ctrl+K)").size(14)
                    ]
                    .spacing(8),
                    AppMessage::ToggleSearch
                ),
                iced::widget::Space::new().width(Length::Fixed(20.0)),
                container(text("v0.1.0").size(11).style(|theme: &Theme| {
                    let palette = view::components::theme::get_palette(theme);
                    text::Style {
                        color: Some(palette.overlay1),
                    }
                }))
                .padding([6, 12])
                .style(|theme: &Theme| {
                    let palette = view::components::theme::get_palette(theme);
                    container::Style {
                        background: Some(iced::Background::Color(palette.surface0)),
                        border: Border {
                            radius: 16.0.into(),
                            ..Default::default()
                        },
                        ..Default::default()
                    }
                })
            ]
            .align_y(iced::Alignment::Center),
        )
        .padding([16, 24])
        .width(Length::Fill)
        .style(|theme: &Theme| {
            let palette = view::components::theme::get_palette(theme);
            container::Style {
                background: Some(iced::Background::Color(palette.base)),
                border: Border {
                    color: palette.surface0,
                    width: 0.0,
                    radius: 0.0.into(),
                },
                shadow: Shadow {
                    color: view::components::theme::Palette::with_alpha(palette.crust, 0.2),
                    offset: iced::Vector::new(0.0, 2.0),
                    blur_radius: 8.0,
                },
                ..Default::default()
            }
        });

        let content_area = container(content)
            .padding(24)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(|theme: &Theme| {
                let palette = view::components::theme::get_palette(theme);
                container::Style {
                    background: Some(palette.base.into()),
                    ..Default::default()
                }
            });

        let dashboard = row![sidebar, column![header, content_area].width(Length::Fill)];

        modal::overlay(
            if self.is_search_open {
                let modal_content =
                    container(column![
                        row![
                            text("")
                                .font(iced::font::Font::with_name("Symbols Nerd Font Mono"))
                                .size(20)
                                .style(|_| text::Style {
                                    color: Some(Color::from_rgba8(255, 255, 255, 0.5))
                                }),
                            text_input("Search...", &self.search_query)
                                .id(self.search_input_id.clone())
                                .on_input(AppMessage::Search)
                                .padding(0)
                                .size(20)
                                .style(|theme: &Theme, status| {
                                    let mut s = iced::widget::text_input::default(theme, status);
                                    s.background = iced::Background::Color(Color::TRANSPARENT);
                                    s.border.width = 0.0;
                                    s.border.radius = 0.0.into();
                                    s.value = Color::WHITE;
                                    s.placeholder = Color::from_rgba8(255, 255, 255, 0.3);
                                    s.selection = Color::from_rgb8(137, 180, 250);
                                    s
                                })
                        ]
                        .spacing(12)
                        .align_y(iced::Alignment::Center)
                        .padding(20),
                        container("")
                            .width(Length::Fill)
                            .height(1)
                            .style(|_| container::Style {
                                background: Some(iced::Background::Color(Color::from_rgba8(
                                    255, 255, 255, 0.1
                                ))),
                                ..Default::default()
                            }),
                        scrollable(
                            column(
                                self.search_results
                                    .iter()
                                    .map(|(idx, res)| {
                                        let msg = AppMessage::JumpToResult(*idx, res.clone());

                                        button(
                                            row![
                                                column![
                                                    text(res.title.clone())
                                                        .size(16)
                                                        .font(iced::font::Font {
                                                            weight: iced::font::Weight::Bold,
                                                            ..Default::default()
                                                        })
                                                        .style(|theme: &Theme| {
                                                            let palette = view::components::theme::get_palette(theme);
                                                            text::Style {
                                                                color: Some(palette.text),
                                                            }
                                                        }),
                                                    text(res.description.clone()).size(13).style(
                                                        |theme: &Theme| {
                                                            let palette = view::components::theme::get_palette(theme);
                                                            text::Style {
                                                                color: Some(palette.subtext0),
                                                            }
                                                        }
                                                    )
                                                ]
                                                .spacing(4),
                                                iced::widget::Space::new().width(Length::Fill),
                                                container(
                                                    text(if res.id.contains(":section:") {
                                                        "Section"
                                                    } else {
                                                        "Setting"
                                                    })
                                                    .size(10)
                                                )
                                                .padding([4, 8])
                                                .style(|theme: &Theme| {
                                                    let palette = view::components::theme::get_palette(theme);
                                                    container::Style {
                                                        background: Some(iced::Background::Color(
                                                            view::components::theme::Palette::with_alpha(palette.surface2, 0.1)
                                                        )),
                                                        border: Border {
                                                            radius: 12.0.into(),
                                                            ..Default::default()
                                                        },
                                                        text_color: Some(palette.subtext0),
                                                        ..Default::default()
                                                    }
                                                })
                                            ]
                                            .align_y(iced::Alignment::Center)
                                            .padding(12),
                                        )
                                        .on_press(msg)
                                        .padding(0)
                                        .width(Length::Fill)
                                        .style(|theme: &Theme, status| {
                                            let palette = view::components::theme::get_palette(theme);
                                            let base_style = iced::widget::button::secondary(theme, status);
                                            match status {
                                                iced::widget::button::Status::Hovered
                                                | iced::widget::button::Status::Pressed => {
                                                    iced::widget::button::Style {
                                                        background: Some(iced::Background::Color(
                                                            view::components::theme::Palette::with_alpha(palette.surface2, 0.2)
                                                        )),
                                                        text_color: palette.text,
                                                        ..base_style
                                                    }
                                                }
                                                _ => iced::widget::button::Style {
                                                    background: None,
                                                    ..base_style
                                                },
                                            }
                                        })
                                        .into()
                                    })
                                    .collect::<Vec<_>>()
                            )
                            .spacing(0)
                        )
                        .height(Length::Fixed(400.0))
                    ])
                    .width(Length::Fixed(600.0))
                    .style(view::components::modal::container_style);

                stack![
                    dashboard,
                    container(modal_content)
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .center_x(Length::Fill)
                        .center_y(Length::Fill)
                        .padding(20)
                ]
                .into()
            } else {
                dashboard.into()
            },
            AppMessage::CloseSearch,
            self.is_search_open,
        )
    }

    fn subscription(&self) -> Subscription<AppMessage> {
        let sub = if let Some(plugin) = self.plugins.get(self.active_tab_index) {
            plugin.subscription()
        } else {
            Subscription::none()
        };

        let hotkeys = event::listen_with(|e, status, _window| {
            if let event::Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. }) = e {
                if status == event::Status::Ignored {
                    if key == keyboard::Key::Named(keyboard::key::Named::Escape) {
                        return Some(AppMessage::CloseSearch);
                    }
                    if key == keyboard::Key::Character("k".into()) && modifiers.control() {
                        return Some(AppMessage::ToggleSearch);
                    }
                    return Some(AppMessage::GlobalKeyPress(key, modifiers));
                }
            }
            None
        });

        Subscription::batch(vec![sub, hotkeys])
    }
}

pub fn main() -> iced::Result {
    iced::application(HyprBoard::new, HyprBoard::update, HyprBoard::view)
        .subscription(HyprBoard::subscription)
        .title(|_: &HyprBoard| "HyprBoard".to_string())
        .theme(|app: &HyprBoard| app.active_theme.to_iced_theme())
        .run()
}

use iced::widget::button;
