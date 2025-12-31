use crate::core::presets::{Preset, PresetManager};
use crate::core::{AppMessage, Plugin, PluginMsg, SearchResult};
use crate::utils::hyprlang::{HyprConf, HyprLang};
use crate::view::components::{
    button as btn, color_picker, modal, text_input as ti, theme::AppTheme,
};
use iced::Color;
use iced::{
    Element, Length, Task,
    widget::{column, container, row, scrollable, stack, text},
};
use std::collections::HashMap;
use std::path::PathBuf;

pub mod schema;
use crate::view::components::schema_renderer::{
    self, OptionDef, OptionType as SchemaOptionType, Section as SchemaSection,
};
use schema::OptionType;

pub struct HyprlockPlugin {
    id: usize,
    config_path: PathBuf,
    hypr_lang: Option<HyprLang>,
    config: Option<HyprConf>,
    active_tab_id: String,
    active_section_idx: Option<usize>,

    preset_manager: PresetManager,
    presets_list: Vec<Preset>,
    active_preset: Option<String>,

    input_state: HashMap<String, String>,

    color_modal_open: bool,
    color_modal_target: Option<String>,
    color_modal_value: String,
}

impl HyprlockPlugin {
    pub fn new(id: usize) -> Self {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
        let path = PathBuf::from(&home).join(".config/hypr/hyprlock.conf");

        let preset_manager = PresetManager::new("hyprlock");
        let active_preset = preset_manager.get_active();
        let presets_list = preset_manager.list();

        let mut plugin = Self {
            id,
            config_path: path.clone(),
            hypr_lang: None,
            config: None,
            active_tab_id: "general".to_string(),
            active_section_idx: None,
            preset_manager,
            presets_list,
            active_preset,
            input_state: HashMap::new(),
            color_modal_open: false,
            color_modal_target: None,
            color_modal_value: String::new(),
        };
        plugin.load_config();
        plugin
    }

    fn load_config(&mut self) {
        let hypr = HyprLang::new(self.config_path.to_string_lossy().to_string());
        if let Ok(conf) = hypr.load() {
            self.config = Some(conf);
            self.hypr_lang = Some(hypr);
        } else {
            self.config = Some(HyprConf::new());
            self.hypr_lang = Some(hypr);
        }
    }

    fn save_config(&self) {
        if let (Some(lang), Some(conf)) = (&self.hypr_lang, &self.config) {
            let _ = lang.save(conf);
        }
    }

    fn get_value(&self, path: &str, default: &str) -> String {
        if let Some(conf) = &self.config {
            conf.get(path).unwrap_or(default.to_string())
        } else {
            default.to_string()
        }
    }
}

impl Plugin for HyprlockPlugin {
    fn name(&self) -> String {
        "Hyprlock".to_string()
    }

    fn icon(&self) -> char {
        '🔒'
    }

    fn update(&mut self, message: PluginMsg) -> Task<AppMessage> {
        match message {
            PluginMsg::SwitchInternalTab(tab) => {
                self.active_tab_id = tab;
            }
            PluginMsg::UpdateConfig(path, value) => {
                let path = path.replace(".", ":");
                if let Some(conf) = &mut self.config {
                    conf.set(&path, &value);
                    self.save_config();

                    if let Some(name) = &self.active_preset {
                        if let Ok(content) = std::fs::read_to_string(&self.config_path) {
                            let mut files = HashMap::new();
                            files.insert("hyprlock.conf".to_string(), content);
                            let _ = self.preset_manager.save(name, &files);
                        }
                    }
                }
            }
            PluginMsg::Edit(action, target, data) => {
                if action == "file_pick" {
                    let path = rfd::FileDialog::new().pick_file();
                    if let Some(p) = path {
                        return Task::done(AppMessage::PluginMessage(
                            self.id,
                            PluginMsg::UpdateConfig(target, p.to_string_lossy().to_string()),
                        ));
                    }
                } else if action == "color_pick" {
                    let current_val = self.get_value(&target, "rgba(0,0,0,1)");
                    self.color_modal_open = true;
                    self.color_modal_target = Some(target);
                    self.color_modal_value = current_val;
                } else if action == "color_cancel" {
                    self.color_modal_open = false;
                    self.color_modal_target = None;
                } else if action == "color_apply" {
                    if let Some(target) = &self.color_modal_target {
                        return Task::done(AppMessage::PluginMessage(
                            self.id,
                            PluginMsg::UpdateConfig(target.clone(), self.color_modal_value.clone()),
                        ));
                    }
                    self.color_modal_open = false;
                    self.color_modal_target = None;
                } else if action == "color_update" {
                    self.color_modal_value = data;
                } else if action == "preset_save" {
                    let name = self
                        .input_state
                        .get("preset_name")
                        .cloned()
                        .unwrap_or_default();
                    if !name.is_empty() {
                        if let Ok(content) = std::fs::read_to_string(&self.config_path) {
                            let mut files = HashMap::new();
                            files.insert("hyprlock.conf".to_string(), content);
                            let _ = self.preset_manager.save(&name, &files);
                            self.presets_list = self.preset_manager.list();
                            self.input_state.remove("preset_name");
                        }
                    }
                } else if action == "preset_load" {
                    if let Ok(files) = self.preset_manager.load(&data) {
                        if let Some(content) = files.get("hyprlock.conf") {
                            if std::fs::write(&self.config_path, content).is_ok() {
                                self.load_config();
                                self.active_preset = Some(data.clone());
                                let _ = self.preset_manager.set_active(Some(&data));
                            }
                        }
                    }
                } else if action == "preset_delete" {
                    let _ = self.preset_manager.delete(&data);
                    if self.active_preset.as_ref() == Some(&data) {
                        self.active_preset = None;
                        let _ = self.preset_manager.set_active(None);
                    }
                    self.presets_list = self.preset_manager.list();
                } else if action == "preset_input" {
                    self.input_state.insert("preset_name".to_string(), data);
                }
            }
            PluginMsg::LoadPreset(name) => {
                if let Ok(files) = self.preset_manager.load(&name) {
                    if let Some(content) = files.get("hyprlock.conf") {
                        if std::fs::write(&self.config_path, content).is_ok() {
                            self.load_config();
                            self.active_preset = Some(name.clone());
                            let _ = self.preset_manager.set_active(Some(&name));
                        }
                    }
                }
            }
            PluginMsg::None => {}
            _ => {}
        }
        Task::none()
    }

    fn subscription(&self) -> iced::Subscription<AppMessage> {
        iced::Subscription::none()
    }

    fn view<'a>(&'a self, _theme: &'a AppTheme) -> Element<'a, AppMessage> {
        let schema = schema::get_schema();

        let mut tab_buttons: Vec<Element<AppMessage>> = schema
            .iter()
            .map(|s| {
                let is_active = self.active_tab_id == s.name;
                let label = format!("{} {}", s.icon, s.title);
                let msg = AppMessage::PluginMessage(
                    self.id,
                    PluginMsg::SwitchInternalTab(s.name.clone()),
                );

                if is_active {
                    btn::secondary(text(label), msg)
                } else {
                    btn::ghost(text(label), msg)
                }
            })
            .collect();

        let presets_active = self.active_tab_id == "presets";
        tab_buttons.push(if presets_active {
            btn::secondary(
                text("💾 Presets"),
                AppMessage::PluginMessage(self.id, PluginMsg::SwitchInternalTab("presets".into())),
            )
        } else {
            btn::ghost(
                text("💾 Presets"),
                AppMessage::PluginMessage(self.id, PluginMsg::SwitchInternalTab("presets".into())),
            )
        });

        let tabs = row(tab_buttons).spacing(5);

        let content: Element<AppMessage> = if self.active_tab_id == "presets" {
            let id = self.id;
            let preset_name_val = self
                .input_state
                .get("preset_name")
                .cloned()
                .unwrap_or_default();

            let save_section = row![
                container(ti::input("Preset Name", &preset_name_val, move |v| {
                    AppMessage::PluginMessage(
                        id,
                        PluginMsg::Edit("preset_input".into(), "".into(), v),
                    )
                }))
                .width(Length::Fixed(200.0)),
                btn::primary(
                    text("💾 Save Current"),
                    AppMessage::PluginMessage(
                        id,
                        PluginMsg::Edit("preset_save".into(), "".into(), "".into())
                    )
                )
            ]
            .spacing(10)
            .align_y(iced::Alignment::Center);

            let list = column(
                self.presets_list
                    .iter()
                    .map(|preset| {
                        let is_active = self.active_preset.as_ref() == Some(&preset.name);
                        let name = preset.name.clone();
                        let name2 = preset.name.clone();

                        container(
                            row![
                                column![
                                    text(preset.name.clone()).size(16),
                                    text(preset.description.clone()).size(12).style(|_| {
                                        iced::widget::text::Style {
                                            color: Some(Color::from_rgb8(127, 132, 156)),
                                        }
                                    }),
                                ]
                                .width(Length::Fill),
                                row![
                                    if !is_active {
                                        btn::small_primary(
                                            text("Load"),
                                            AppMessage::PluginMessage(
                                                id,
                                                PluginMsg::Edit(
                                                    "preset_load".into(),
                                                    "".into(),
                                                    name,
                                                ),
                                            ),
                                        )
                                    } else {
                                        btn::small_secondary(
                                            text("Active"),
                                            AppMessage::PluginMessage(id, PluginMsg::None),
                                        )
                                    },
                                    btn::small_destructive(
                                        text("Delete"),
                                        AppMessage::PluginMessage(
                                            id,
                                            PluginMsg::Edit(
                                                "preset_delete".into(),
                                                "".into(),
                                                name2
                                            )
                                        )
                                    ),
                                ]
                                .spacing(8)
                            ]
                            .align_y(iced::Alignment::Center)
                            .spacing(10),
                        )
                        .style(move |_| container::Style {
                            background: Some(iced::Background::Color(if is_active {
                                Color::from_rgb8(30, 35, 50)
                            } else {
                                Color::from_rgb8(24, 24, 37)
                            })),
                            border: iced::Border {
                                radius: 8.0.into(),
                                ..Default::default()
                            },
                            ..Default::default()
                        })
                        .padding(12)
                        .into()
                    })
                    .collect::<Vec<_>>(),
            )
            .spacing(10);

            column![
                text("Hyprlock Presets").size(18),
                save_section,
                scrollable(list).height(Length::Fill)
            ]
            .spacing(15)
            .into()
        } else if let Some(section) = schema.iter().find(|s| s.name == self.active_tab_id) {
            let options: Vec<OptionDef> = section
                .options
                .iter()
                .map(|opt| {
                    let typ = match opt.option_type {
                        OptionType::Bool => SchemaOptionType::Bool,
                        OptionType::Int => SchemaOptionType::Int,
                        OptionType::Float => SchemaOptionType::Float,
                        OptionType::String => SchemaOptionType::String,
                        OptionType::Color => SchemaOptionType::Color,
                        OptionType::File => SchemaOptionType::File,
                        OptionType::Vec2 => SchemaOptionType::Vec2,
                        OptionType::Monitor => SchemaOptionType::Monitor,
                        OptionType::Gradient => SchemaOptionType::Gradient,
                    };

                    let mut def = OptionDef::new(&opt.name, typ, &opt.default, &opt.description);
                    if let (Some(min), Some(max)) = (opt.min, opt.max) {
                        def = def.with_range(min, max, opt.step.unwrap_or(1.0));
                    }
                    if let Some(choices) = &opt.choices {
                        def = def.with_choices(choices.clone());
                    }
                    def
                })
                .collect();

            let schema_section = SchemaSection {
                name: section.name.clone(),
                title: section.title.clone(),
                options,
            };

            let mut values = HashMap::new();
            for opt in &section.options {
                let path = if section.name == "general" {
                    format!("general:{}", opt.name)
                } else {
                    format!("{}:{}", section.name, opt.name)
                };
                values.insert(
                    format!("{}.{}", section.name, opt.name),
                    self.get_value(&path, &opt.default),
                );
            }

            schema_renderer::render_section(&schema_section, &values, &section.name, self.id).into()
        } else {
            text("Section not found").into()
        };

        let base = column![container(tabs).padding(10), content]
            .spacing(20)
            .padding(20);

        if self.color_modal_open {
            stack![
                base,
                modal::overlay(
                    color_picker::view_modal(
                        &self.color_modal_value,
                        move |s| AppMessage::PluginMessage(
                            self.id,
                            PluginMsg::Edit("color_update".into(), "".into(), s)
                        ),
                        AppMessage::PluginMessage(
                            self.id,
                            PluginMsg::Edit("color_cancel".into(), "".into(), "".into())
                        ),
                        AppMessage::PluginMessage(
                            self.id,
                            PluginMsg::Edit("color_apply".into(), "".into(), "".into())
                        )
                    )
                    .into(),
                    AppMessage::PluginMessage(
                        self.id,
                        PluginMsg::Edit("color_cancel".into(), "".into(), "".into())
                    ),
                    true
                )
            ]
            .into()
        } else {
            base.into()
        }
    }

    fn searchable_items(&self) -> Vec<SearchResult> {
        vec![]
    }
}
