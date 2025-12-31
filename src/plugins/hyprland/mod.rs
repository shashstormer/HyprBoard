use self::helpers::migration::HyprlandVersion;
use self::helpers::schema::OptionType;
use self::helpers::types::{EnvVar, ExecCommand, Gesture, Keybind, Monitor, WindowRule};
use crate::core::SearchResult;
use crate::core::presets::{Preset, PresetManager};
use crate::core::{AppMessage, Plugin, PluginMsg};
use crate::view::components::theme::AppTheme;
use crate::view::components::{
    button as btn, card, color_picker, modal, setting_row, text_input as ti, toggle,
};
use iced::{
    Color, Element, Length, Task, Theme,
    widget::{
        column, combo_box, container, pick_list, row, scrollable, slider as native_slider, stack,
        text,
    },
};
use std::collections::HashMap;

pub mod helpers;
pub mod view;
use helpers::{config_loader, schema};

pub struct HyprlandPlugin {
    hyprland_version: Option<HyprlandVersion>,
    config: config_loader::ConfigLoader,
    preset_manager: PresetManager,
    active_tab_id: String,

    monitors: Vec<Monitor>,
    window_rules: Vec<WindowRule>,
    layer_rules: Vec<helpers::types::LayerRule>,
    exec_cmds: Vec<ExecCommand>,
    env_vars: Vec<EnvVar>,
    keybinds: Vec<Keybind>,
    gestures: Vec<Gesture>,
    presets_list: Vec<Preset>,
    active_preset: Option<String>,

    modal_type: Option<String>,
    modal_inputs: HashMap<String, String>,
    editing_raw: Option<String>,

    color_modal_open: bool,
    color_modal_target: Option<String>,
    color_modal_value: String,

    highlighted_id: Option<String>,
    dispatcher_combo: combo_box::State<String>,

    keybind_filter: String,
    exec_filter: String,
    env_filter: String,
    rule_filter: String,
    layer_filter: String,
    gesture_filter: String,
    settings_filter: String,
    capturing_bind: bool,
}

impl HyprlandPlugin {
    pub fn new() -> Self {
        let mut loader = config_loader::ConfigLoader::new();
        let _ = loader.load();
        let preset_manager = PresetManager::new("hyprland");
        let active_preset = preset_manager.get_active();

        let hyprland_version = HyprlandVersion::detect();
        if let Some(ref v) = hyprland_version {
            println!("Detected Hyprland version: {}", v.to_string());
        }

        let mut modal_type = None;

        if let Some(ref v) = hyprland_version {
            if v.supports_new_window_rules() {
                if let Some(ref conf) = loader.get_hypr_conf() {
                    if helpers::migration::ConfigMigrator::needs_migration(conf) {
                        modal_type = Some("upgrade_migration".to_string());
                    }
                }
            }
        }

        if modal_type.is_none() {
            if let Some(name) = &active_preset {
                if let Ok(files) = preset_manager.load(name) {
                    if let Some(preset_content) = files.get("hyprland.conf") {
                        if let Ok(current_content) = std::fs::read_to_string(
                            std::path::PathBuf::from(
                                std::env::var("HOME").unwrap_or("/tmp".into()),
                            )
                            .join(".config/hypr/hyprland.conf"),
                        ) {
                            if preset_content.trim() != current_content.trim() {
                                modal_type = Some("preset_conflict".to_string());
                            }
                        }
                    }
                }
            }
        }

        let mut plugin = Self {
            hyprland_version,
            config: loader,
            preset_manager,
            active_tab_id: "general".to_string(),
            monitors: Vec::new(),
            window_rules: Vec::new(),
            layer_rules: Vec::new(),
            exec_cmds: Vec::new(),
            env_vars: Vec::new(),
            keybinds: Vec::new(),
            gestures: Vec::new(),
            presets_list: Vec::new(),
            active_preset,
            modal_type,
            modal_inputs: HashMap::new(),
            editing_raw: None,
            highlighted_id: None,
            dispatcher_combo: combo_box::State::new(
                crate::plugins::hyprland::helpers::dispatchers::DISPATCHERS
                    .iter()
                    .map(|s| s.to_string())
                    .collect(),
            ),
            keybind_filter: String::new(),
            exec_filter: String::new(),
            env_filter: String::new(),
            rule_filter: String::new(),
            layer_filter: String::new(),
            gesture_filter: String::new(),
            settings_filter: String::new(),
            capturing_bind: false,
            color_modal_open: false,
            color_modal_target: None,
            color_modal_value: String::new(),
        };
        plugin.refresh_data();
        plugin
    }

    fn refresh_data(&mut self) {
        self.monitors = self.config.get_monitors();
        self.window_rules = self.config.get_window_rules();
        self.layer_rules = self.config.get_layer_rules();
        self.exec_cmds = self.config.get_exec();
        self.env_vars = self.config.get_env();
        self.keybinds = self.config.get_binds();
        self.gestures = self.config.get_gestures();
        self.presets_list = self.preset_manager.list();
    }

    fn input_val(&self, key: &str) -> String {
        self.modal_inputs.get(key).cloned().unwrap_or_default()
    }

    fn parse_rule_props(&self, s: &str) -> Vec<(String, String)> {
        let mut props = Vec::new();
        for part in s.split(',').map(|p| p.trim()) {
            if part.starts_with("match:") {
                let rest = &part[6..];
                if let Some(space_idx) = rest.find(' ') {
                    props.push((
                        format!("match:{}", &rest[..space_idx]),
                        rest[space_idx + 1..].trim().to_string(),
                    ));
                } else {
                    props.push((part.to_string(), String::new()));
                }
            } else if !part.is_empty() {
                if let Some((k, v)) = part.split_once(':') {
                    props.push((format!("match:{}", k), v.to_string()));
                }
            }
        }
        props
    }

    fn parse_rule_effects(&self, s: &str) -> Vec<(String, String)> {
        let mut effects = Vec::new();
        for part in s.split(',').map(|p| p.trim()) {
            if !part.is_empty() && !part.starts_with("match:") {
                let parts: Vec<&str> = part.splitn(2, ' ').collect();
                let name = parts[0].to_string();
                let val = parts.get(1).map(|s| s.to_string()).unwrap_or_default();
                effects.push((name, val));
            }
        }
        effects
    }
}

impl Plugin for HyprlandPlugin {
    fn name(&self) -> String {
        "Hyprland".to_string()
    }

    fn icon(&self) -> char {
        ''
    }

    fn update(&mut self, message: PluginMsg) -> Task<AppMessage> {
        match message {
            PluginMsg::UpdateConfig(path, value) => {
                self.config.set_option(&path, &value);
                let _ = self.config.save();
            }
            PluginMsg::SwitchInternalTab(tab_id) => {
                self.active_tab_id = tab_id;
            }
            PluginMsg::OpenModal(modal_id) => {
                self.modal_type = Some(modal_id.clone());
                self.modal_inputs.clear();
                self.editing_raw = None;

                if modal_id.contains("bind") || modal_id.contains("gesture") {
                    self.dispatcher_combo = iced::widget::combo_box::State::new(
                        crate::plugins::hyprland::helpers::dispatchers::DISPATCHERS
                            .iter()
                            .map(|s| s.to_string())
                            .collect(),
                    );
                }

                if modal_id.starts_with("edit_") {
                    if let Some((_, raw)) = modal_id.split_once(':') {
                        self.editing_raw = Some(raw.to_string());
                        if modal_id.starts_with("edit_window_rule") {
                            if let Some(rule) = self.window_rules.iter().find(|r| r.raw == raw) {
                                self.modal_inputs
                                    .insert("effect".to_string(), rule.effect_str());
                                self.modal_inputs
                                    .insert("match".to_string(), rule.match_str());
                                self.modal_inputs
                                    .insert("type".to_string(), rule.rule_type.clone());
                                self.modal_inputs.insert(
                                    "name".to_string(),
                                    rule.name.clone().unwrap_or_default(),
                                );
                            }
                        } else if modal_id.starts_with("edit_exec") {
                            if let Some(cmd) = self.exec_cmds.iter().find(|c| c.raw == raw) {
                                self.modal_inputs
                                    .insert("type".to_string(), cmd.exec_type.clone());
                                self.modal_inputs
                                    .insert("command".to_string(), cmd.command.clone());
                            }
                        } else if modal_id.starts_with("edit_env") {
                            if let Some(var) = self.env_vars.iter().find(|v| v.raw == raw) {
                                self.modal_inputs
                                    .insert("name".to_string(), var.name.clone());
                                self.modal_inputs
                                    .insert("value".to_string(), var.value.clone());
                            }
                        } else if modal_id.starts_with("edit_bind") {
                            if let Some(bind) = self.keybinds.iter().find(|b| b.raw == raw) {
                                self.modal_inputs
                                    .insert("mods".to_string(), bind.mods.clone());
                                self.modal_inputs
                                    .insert("key".to_string(), bind.key.clone());
                                self.modal_inputs
                                    .insert("dispatcher".to_string(), bind.dispatcher.clone());
                                self.modal_inputs
                                    .insert("params".to_string(), bind.params.clone());
                                self.modal_inputs
                                    .insert("type".to_string(), bind.bind_type.clone());
                            }
                        } else if modal_id.starts_with("edit_gesture") {
                            if let Some(g) = self.gestures.iter().find(|g| g.raw == raw) {
                                self.modal_inputs
                                    .insert("fingers".to_string(), g.fingers.to_string());
                                self.modal_inputs
                                    .insert("direction".to_string(), g.direction.clone());
                                self.modal_inputs
                                    .insert("action".to_string(), g.action.clone());
                                self.modal_inputs
                                    .insert("dispatcher".to_string(), g.dispatcher.clone());
                                self.modal_inputs
                                    .insert("params".to_string(), g.params.clone());
                                self.modal_inputs
                                    .insert("mod_key".to_string(), g.mod_key.clone());
                                self.modal_inputs
                                    .insert("scale".to_string(), g.scale.clone());
                            }
                        }
                    }
                }
            }
            PluginMsg::CloseModal => {
                self.modal_type = None;
                self.modal_inputs.clear();
                self.editing_raw = None;
            }
            PluginMsg::Edit(action, type_id, data) => match action.as_str() {
                "input" => {
                    if type_id == "keybind_filter" {
                        self.keybind_filter = data;
                    } else if type_id == "exec_filter" {
                        self.exec_filter = data;
                    } else if type_id == "env_filter" {
                        self.env_filter = data;
                    } else if type_id == "rule_filter" {
                        self.rule_filter = data;
                    } else if type_id == "layer_filter" {
                        self.layer_filter = data;
                    } else if type_id == "gesture_filter" {
                        self.gesture_filter = data;
                    } else if type_id == "settings_filter" {
                        self.settings_filter = data;
                    } else {
                        self.modal_inputs.insert(type_id, data);
                    }
                }
                "delete" => {
                    match type_id.as_str() {
                        "window_rule" => self.config.delete_window_rule(&data),
                        "layer_rule" => self.config.delete_layer_rule(&data),
                        "exec" => self.config.delete_exec(&data),
                        "env" => self.config.delete_env(&data),
                        "bind" => self.config.delete_bind(&data),
                        "gesture" => self.config.delete_gesture(&data),
                        _ => {}
                    }
                    let _ = self.config.save();
                    self.refresh_data();
                }
                "bind_detected" => {
                    self.modal_inputs.insert("mods".to_string(), type_id);
                    self.modal_inputs.insert("key".to_string(), data);
                    self.capturing_bind = false;
                }
                "preset_save" => {
                    let name = self.input_val("name");
                    if !name.is_empty() {
                        if let Ok(content) = std::fs::read_to_string(
                            std::path::PathBuf::from(
                                std::env::var("HOME").unwrap_or("/tmp".into()),
                            )
                            .join(".config/hypr/hyprland.conf"),
                        ) {
                            let mut files = HashMap::new();
                            files.insert("hyprland.conf".to_string(), content);
                            let _ = self.preset_manager.save(&name, &files);
                            self.refresh_data();
                            self.modal_type = None;
                        }
                    }
                }
                "preset_load" | "preset_select" => {
                    if let Ok(files) = self.preset_manager.load(&data) {
                        if let Some(content) = files.get("hyprland.conf") {
                            if let Ok(_) = std::fs::write(
                                std::path::PathBuf::from(
                                    std::env::var("HOME").unwrap_or("/tmp".into()),
                                )
                                .join(".config/hypr/hyprland.conf"),
                                content,
                            ) {
                                let _ = self.config.load();
                                self.active_preset = Some(data.clone());
                                let _ = self.preset_manager.set_active(Some(&data));
                                self.refresh_data();
                            }
                        }
                    }
                }
                "preset_detach" => {
                    self.active_preset = None;
                    let _ = self.preset_manager.set_active(None);
                    self.modal_type = None;
                }
                "run_migration" => {
                    let config_path = self.config.config_path.clone();
                    if let Some(conf) = self.config.get_hypr_conf_mut() {
                        let _ = helpers::migration::ConfigMigrator::backup_config(&config_path);
                        let result = helpers::migration::ConfigMigrator::migrate(conf);
                        println!(
                            "Migration complete: {} rules migrated, {} options renamed",
                            result.migrated_rules, result.renamed_options
                        );
                        let _ = self.config.save();
                        self.refresh_data();
                    }
                    self.modal_type = None;
                }
                "preset_overwrite" => {
                    if let Some(name) = &self.active_preset {
                        if let Ok(content) = std::fs::read_to_string(
                            std::path::PathBuf::from(
                                std::env::var("HOME").unwrap_or("/tmp".into()),
                            )
                            .join(".config/hypr/hyprland.conf"),
                        ) {
                            let mut files = HashMap::new();
                            files.insert("hyprland.conf".to_string(), content);
                            let _ = self.preset_manager.save(name, &files);
                            self.modal_type = None;
                        }
                    }
                }
                "preset_delete" => {
                    let _ = self.preset_manager.delete(&data);
                    if self.active_preset.as_ref() == Some(&data) {
                        self.active_preset = None;
                        let _ = self.preset_manager.set_active(None);
                    }
                    self.refresh_data();
                }
                "submit" => {
                    let use_new_syntax = self
                        .hyprland_version
                        .as_ref()
                        .map(|v| v.supports_new_window_rules())
                        .unwrap_or(false);

                    if type_id == "add_window_rule" {
                        let rule = WindowRule {
                            name: {
                                let n = self.input_val("name");
                                if n.is_empty() { None } else { Some(n) }
                            },
                            rule_type: if use_new_syntax {
                                "windowrule"
                            } else {
                                "windowrulev2"
                            }
                            .to_string(),
                            props: self.parse_rule_props(&self.input_val("match")),
                            effects: self.parse_rule_effects(&self.input_val("effect")),
                            raw: String::new(),
                            is_block: false,
                        };
                        self.config.add_window_rule(rule, use_new_syntax);
                    } else if type_id.starts_with("edit_window_rule") {
                        if let Some(old_raw) = &self.editing_raw {
                            let rule = WindowRule {
                                name: {
                                    let n = self.input_val("name");
                                    if n.is_empty() { None } else { Some(n) }
                                },
                                rule_type: if use_new_syntax {
                                    "windowrule"
                                } else {
                                    "windowrulev2"
                                }
                                .to_string(),
                                props: self.parse_rule_props(&self.input_val("match")),
                                effects: self.parse_rule_effects(&self.input_val("effect")),
                                raw: String::new(),
                                is_block: false,
                            };
                            self.config
                                .update_window_rule(old_raw, rule, use_new_syntax);
                        }
                    } else if type_id == "add_exec" {
                        let cmd = ExecCommand {
                            exec_type: self
                                .input_val("type")
                                .is_empty()
                                .then(|| "exec-once".to_string())
                                .unwrap_or(self.input_val("type")),
                            command: self.input_val("command"),
                            raw: String::new(),
                        };
                        self.config.add_exec(cmd);
                    } else if type_id.starts_with("edit_exec") {
                        if let Some(old_raw) = &self.editing_raw {
                            let cmd = ExecCommand {
                                exec_type: self.input_val("type"),
                                command: self.input_val("command"),
                                raw: String::new(),
                            };
                            self.config.update_exec(old_raw, cmd);
                        }
                    } else if type_id == "add_env" {
                        let var = EnvVar {
                            name: self.input_val("name"),
                            value: self.input_val("value"),
                            raw: String::new(),
                        };
                        self.config.add_env(var);
                    } else if type_id.starts_with("edit_env") {
                        if let Some(old_raw) = &self.editing_raw {
                            let var = EnvVar {
                                name: self.input_val("name"),
                                value: self.input_val("value"),
                                raw: String::new(),
                            };
                            self.config.update_env(old_raw, var);
                        }
                    } else if type_id == "add_bind" {
                        let raw_type = self.input_val("type");
                        let type_str = raw_type
                            .split_whitespace()
                            .next()
                            .unwrap_or("bind")
                            .to_string();
                        let bind = Keybind {
                            bind_type: if type_str.is_empty() {
                                "bind".to_string()
                            } else {
                                type_str
                            },
                            mods: self.input_val("mods"),
                            key: self.input_val("key"),
                            dispatcher: self.input_val("dispatcher"),
                            params: self.input_val("params"),
                            raw: String::new(),
                        };
                        self.config.add_bind(bind);
                    } else if type_id.starts_with("edit_bind") {
                        if let Some(old_raw) = &self.editing_raw {
                            let raw_type = self.input_val("type");
                            let type_str = raw_type
                                .split_whitespace()
                                .next()
                                .unwrap_or("bind")
                                .to_string();
                            let bind = Keybind {
                                bind_type: type_str,
                                mods: self.input_val("mods"),
                                key: self.input_val("key"),
                                dispatcher: self.input_val("dispatcher"),
                                params: self.input_val("params"),
                                raw: String::new(),
                            };
                            self.config.update_bind(old_raw, bind);
                        }
                    } else if type_id == "add_gesture" {
                        let g = Gesture {
                            fingers: self.input_val("fingers").parse().unwrap_or(3),
                            direction: self.input_val("direction"),
                            action: self.input_val("action"),
                            dispatcher: self.input_val("dispatcher"),
                            params: self.input_val("params"),
                            mod_key: self.input_val("mod_key"),
                            scale: self.input_val("scale"),
                            raw: String::new(),
                        };
                        self.config.add_gesture(g);
                    } else if type_id.starts_with("edit_gesture") {
                        if let Some(old_raw) = &self.editing_raw {
                            let g = Gesture {
                                fingers: self.input_val("fingers").parse().unwrap_or(3),
                                direction: self.input_val("direction"),
                                action: self.input_val("action"),
                                dispatcher: self.input_val("dispatcher"),
                                params: self.input_val("params"),
                                mod_key: self.input_val("mod_key"),
                                scale: self.input_val("scale"),
                                raw: String::new(),
                            };
                            self.config.update_gesture(old_raw, g);
                        }
                    } else if type_id == "save_preset" {
                        let content =
                            std::fs::read_to_string(&self.config.config_path).unwrap_or_default();
                        let mut files = HashMap::new();
                        files.insert("hyprland.conf".to_string(), content);
                        let _ = self.preset_manager.save(&self.input_val("name"), &files);
                        self.refresh_data();
                    }

                    if !type_id.contains("preset") {
                        let _ = self.config.save();
                        if let Some(name) = &self.active_preset {
                            if let Ok(content) = std::fs::read_to_string(
                                std::path::PathBuf::from(
                                    std::env::var("HOME").unwrap_or("/tmp".into()),
                                )
                                .join(".config/hypr/hyprland.conf"),
                            ) {
                                let mut files = HashMap::new();
                                files.insert("hyprland.conf".to_string(), content);
                                let _ = self.preset_manager.save(name, &files);
                            }
                        }
                    }
                    self.refresh_data();
                    self.modal_type = None;
                }
                "color_pick" => {
                    let current_val = self
                        .config
                        .get_option(&type_id)
                        .unwrap_or("rgba(0,0,0,1)".to_string());
                    self.color_modal_open = true;
                    self.color_modal_target = Some(type_id.clone());
                    self.color_modal_value = current_val;
                }
                "color_cancel" => {
                    self.color_modal_open = false;
                    self.color_modal_target = None;
                }
                "color_apply" => {
                    if let Some(target) = &self.color_modal_target {
                        self.config.set_option(target, &self.color_modal_value);
                        let _ = self.config.save();
                    }
                    self.color_modal_open = false;
                    self.color_modal_target = None;
                }
                "color_update" => {
                    self.color_modal_value = data;
                }
                _ => {}
            },
            PluginMsg::LoadPreset(name) => {
                if let Ok(files) = self.preset_manager.load(&name) {
                    if let Some(content) = files.get("hyprland.conf") {
                        if let Ok(_) = std::fs::write(
                            std::path::PathBuf::from(
                                std::env::var("HOME").unwrap_or("/tmp".into()),
                            )
                            .join(".config/hypr/hyprland.conf"),
                            content,
                        ) {
                            let _ = self.config.load();
                            self.active_preset = Some(name.clone());
                            let _ = self.preset_manager.set_active(Some(&name));
                            self.refresh_data();
                        }
                    }
                }
            }
            PluginMsg::JumpTo(res) => {
                self.active_tab_id = res.tab_id.clone();
                if res.id.contains(":section:") || res.id == res.tab_id {
                    self.highlighted_id = None;
                } else {
                    self.highlighted_id = Some(res.id.clone());
                }

                if res.tab_id == "keybinds" {
                    self.keybind_filter = res.id.clone();
                } else if res.tab_id == "exec" {
                    self.exec_filter = res.id.clone();
                } else if res.tab_id == "windowrules" {
                    self.rule_filter = res.id.clone();
                } else if res.tab_id == "env" {
                    self.env_filter = res.id.clone();
                } else if res.tab_id == "gestures" {
                    self.gesture_filter = res.id.clone();
                } else if res.tab_id != "monitors" && res.tab_id != "presets" {
                    if res.id == res.tab_id {
                        self.settings_filter.clear();
                    } else if res.id.contains(":section:") {
                        if let Some(section_name) = res.id.split(":section:").nth(1) {
                            if let Some(tab) =
                                schema::get_schema().iter().find(|t| t.id == res.tab_id)
                            {
                                if let Some(section) =
                                    tab.sections.iter().find(|s| s.name == section_name)
                                {
                                    self.settings_filter = section.title.clone();
                                }
                            }
                        }
                    } else {
                        if let Some((_, opt_name)) = res.id.split_once(':') {
                            self.settings_filter = opt_name.to_string();
                        }
                    }
                }

                let highlight = Task::perform(
                    async {
                        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                    },
                    |_| AppMessage::PluginMessage(0, PluginMsg::ClearHighlight),
                );

                return highlight;
            }
            PluginMsg::ClearHighlight => {
                self.highlighted_id = None;
            }
            PluginMsg::Toggle(flag) => {
                self.capturing_bind = flag;
            }
            _ => {}
        }

        Task::none()
    }

    fn subscription(&self) -> iced::Subscription<AppMessage> {
        if self.capturing_bind && self.modal_type.clone().unwrap_or_default().contains("bind") {
            iced::event::listen().map(|e| match e {
                iced::Event::Keyboard(iced::keyboard::Event::KeyPressed {
                    key, modifiers, ..
                }) => {
                    let mods = modifiers_to_string(modifiers);
                    let key_str = key_to_string(key.clone());

                    if !key_str.is_empty() && key_str != "Unidentified" {
                        AppMessage::PluginMessage(
                            0,
                            PluginMsg::Edit("bind_detected".into(), mods, key_str),
                        )
                    } else {
                        AppMessage::None
                    }
                }
                _ => AppMessage::None,
            })
        } else if self.modal_type.is_some() {
            iced::event::listen().map(|e| {
                if let iced::Event::Keyboard(iced::keyboard::Event::KeyPressed { key, .. }) = e {
                    if key == iced::keyboard::Key::Named(iced::keyboard::key::Named::Escape) {
                        return AppMessage::PluginMessage(0, PluginMsg::CloseModal);
                    }
                }
                AppMessage::None
            })
        } else {
            iced::Subscription::none()
        }
    }

    fn view<'a>(&'a self, _theme: &'a AppTheme) -> Element<'a, AppMessage> {
        let schema = schema::get_schema();

        let mut tabs_list = schema
            .iter()
            .map(|t| (t.id.clone(), t.title.clone(), t.icon))
            .collect::<Vec<_>>();
        tabs_list.push(("monitors".to_string(), "Monitors".to_string(), '🖥'));
        tabs_list.push(("windowrules".to_string(), "Rules".to_string(), ''));
        tabs_list.push(("exec".to_string(), "Startup".to_string(), '🚀'));
        tabs_list.push(("env".to_string(), "Env Vars".to_string(), ''));
        tabs_list.push(("keybinds".to_string(), "Keybinds".to_string(), '⌨'));
        tabs_list.push(("gestures".to_string(), "Gestures".to_string(), '👆'));
        tabs_list.push(("presets".to_string(), "Presets".to_string(), '💾'));

        let tabs = row(tabs_list.iter().map(|(id, title, icon)| {
            let is_active = self.active_tab_id == *id;
            let label = format!("{} {}", icon, title);
            let msg = AppMessage::PluginMessage(0, PluginMsg::SwitchInternalTab(id.clone()));

            if is_active {
                btn::secondary(text(label), msg)
            } else {
                btn::ghost(text(label), msg)
            }
        }))
        .spacing(5);

        let tabs = scrollable(tabs).direction(iced::widget::scrollable::Direction::Horizontal(
            iced::widget::scrollable::Scrollbar::default(),
        ));

        let content: Element<AppMessage> = match self.active_tab_id.as_str() {
            "monitors" => view::monitors::view(&self.monitors),
            "windowrules" => view::window_rules::view(
                &self.window_rules,
                self.highlighted_id.clone(),
                &self.rule_filter,
            ),
            "layerrules" => view::layer_rules::view(&self.layer_rules, &self.layer_filter),
            "exec" => view::exec::view(
                &self.exec_cmds,
                self.highlighted_id.clone(),
                &self.exec_filter,
            ),
            "env" => view::env::view(
                &self.env_vars,
                self.highlighted_id.clone(),
                &self.env_filter,
            ),
            "keybinds" => view::keybinds::view(
                &self.keybinds,
                self.highlighted_id.clone(),
                &self.keybind_filter,
            ),
            "gestures" => view::gestures::view(
                &self.gestures,
                self.highlighted_id.clone(),
                &self.gesture_filter,
            ),
            "presets" => view::presets::view(&self.presets_list, self.active_preset.as_ref()),
            _ => {
                if let Some(tab) = schema.iter().find(|t| t.id == self.active_tab_id) {
                    let filter = self.settings_filter.to_lowercase();
                    let sections = column(
                        tab.sections
                            .iter()
                            .filter_map(|section| {
                                let matches_section =
                                    section.title.to_lowercase().contains(&filter);
                                let matching_options: Vec<_> = section
                                    .options
                                    .iter()
                                    .filter(|opt| {
                                        matches_section
                                            || opt.name.to_lowercase().contains(&filter)
                                            || opt.description.to_lowercase().contains(&filter)
                                    })
                                    .collect();

                                if matching_options.is_empty() {
                                    return None;
                                }

                                let section_name = section.name.clone();
                                Some(
                                    card::card(column![
                                        text(section.title.clone()).size(16).font(
                                            iced::font::Font {
                                                weight: iced::font::Weight::Bold,
                                                ..Default::default()
                                            }
                                        ),
                                        column(matching_options.into_iter().map(|opt| {
                                            let path = format!("{}:{}", section_name, opt.name);
                                            let current_val = self
                                                .config
                                                .get_option(&path)
                                                .unwrap_or(opt.default.clone());

                                            let control: Element<AppMessage> = match &opt
                                                .option_type
                                            {
                                                OptionType::Bool => {
                                                    let is_checked = current_val == "true"
                                                        || current_val == "yes"
                                                        || current_val == "1";
                                                    let path_clone = path.clone();
                                                    let new_val = if is_checked {
                                                        "false".to_string()
                                                    } else {
                                                        "true".to_string()
                                                    };
                                                    toggle::toggle(
                                                        is_checked,
                                                        AppMessage::PluginMessage(
                                                            0,
                                                            PluginMsg::UpdateConfig(
                                                                path_clone, new_val,
                                                            ),
                                                        ),
                                                    )
                                                }
                                                OptionType::Int | OptionType::Float => {
                                                    if let (Some(min), Some(max)) =
                                                        (opt.min, opt.max)
                                                    {
                                                        let val: f64 =
                                                            current_val.parse().unwrap_or(
                                                                opt.default.parse().unwrap_or(0.0),
                                                            );
                                                        let path_clone = path.clone();
                                                        let step = opt.step.unwrap_or(
                                                            if matches!(
                                                                opt.option_type,
                                                                OptionType::Float
                                                            ) {
                                                                0.1
                                                            } else {
                                                                1.0
                                                            },
                                                        );
                                                        row![
                                                            native_slider(
                                                                min..=max,
                                                                val,
                                                                move |v| {
                                                                    let formatted = if step >= 1.0 {
                                                                        format!("{}", v as i64)
                                                                    } else {
                                                                        format!("{:.2}", v)
                                                                    };
                                                                    AppMessage::PluginMessage(
                                                                        0,
                                                                        PluginMsg::UpdateConfig(
                                                                            path_clone.clone(),
                                                                            formatted,
                                                                        ),
                                                                    )
                                                                }
                                                            )
                                                            .step(step)
                                                            .width(Length::Fixed(150.0)),
                                                            text(format!(" {}", current_val))
                                                                .size(14)
                                                        ]
                                                        .spacing(10)
                                                        .into()
                                                    } else {
                                                        let path_clone = path.clone();
                                                        ti::input(
                                                            &opt.default,
                                                            &current_val,
                                                            move |v| {
                                                                AppMessage::PluginMessage(
                                                                    0,
                                                                    PluginMsg::UpdateConfig(
                                                                        path_clone.clone(),
                                                                        v,
                                                                    ),
                                                                )
                                                            },
                                                        )
                                                        .into()
                                                    }
                                                }
                                                OptionType::Enum => {
                                                    if let Some(choices) = &opt.choices {
                                                        let path_clone = path.clone();
                                                        let choices_vec: Vec<String> =
                                                            choices.clone();
                                                        let selected = choices_vec
                                                            .iter()
                                                            .find(|c| **c == current_val)
                                                            .cloned();
                                                        pick_list(choices_vec, selected, move |v| {
                                                            AppMessage::PluginMessage(
                                                                0,
                                                                PluginMsg::UpdateConfig(
                                                                    path_clone.clone(),
                                                                    v,
                                                                ),
                                                            )
                                                        })
                                                        .into()
                                                    } else {
                                                        text(current_val.clone()).into()
                                                    }
                                                }
                                                OptionType::Color | OptionType::Gradient => {
                                                    let path_picker = path.clone();
                                                    let path_btn = path.clone();
                                                    color_picker::color_picker(
                                                        &current_val,
                                                        move |v| {
                                                            AppMessage::PluginMessage(
                                                                0,
                                                                PluginMsg::UpdateConfig(
                                                                    path_picker.clone(),
                                                                    v,
                                                                ),
                                                            )
                                                        },
                                                        AppMessage::PluginMessage(
                                                            0,
                                                            PluginMsg::Edit(
                                                                "color_pick".into(),
                                                                path_btn.clone(),
                                                                "".into(),
                                                            ),
                                                        ),
                                                    )
                                                }
                                                OptionType::String | OptionType::Vec2 => {
                                                    let path_clone = path.clone();
                                                    ti::input(
                                                        &opt.default,
                                                        &current_val,
                                                        move |v| {
                                                            AppMessage::PluginMessage(
                                                                0,
                                                                PluginMsg::UpdateConfig(
                                                                    path_clone.clone(),
                                                                    v,
                                                                ),
                                                            )
                                                        },
                                                    )
                                                    .into()
                                                }
                                            };

                                            setting_row::setting_row(
                                                opt.description.clone(),
                                                opt.name.clone(),
                                                control,
                                            )
                                        }))
                                        .spacing(8)
                                    ])
                                    .into(),
                                )
                            })
                            .collect::<Vec<Element<AppMessage>>>(),
                    )
                    .spacing(20);

                    column![
                        ti::input("Filter settings...", &self.settings_filter, |s| {
                            AppMessage::PluginMessage(
                                0,
                                PluginMsg::Edit("input".into(), "settings_filter".into(), s),
                            )
                        }),
                        scrollable(sections)
                    ]
                    .spacing(10)
                    .into()
                } else {
                    text("Tab content not found").into()
                }
            }
        };

        let main_view = column![container(tabs).padding(10), content]
            .spacing(10)
            .padding(20);

        if self.color_modal_open {
            stack![
                container(main_view)
                    .width(Length::Fill)
                    .height(Length::Fill),
                modal::overlay(
                    color_picker::view_modal(
                        &self.color_modal_value,
                        move |s| AppMessage::PluginMessage(
                            0,
                            PluginMsg::Edit("color_update".into(), "".into(), s)
                        ),
                        AppMessage::PluginMessage(
                            0,
                            PluginMsg::Edit("color_cancel".into(), "".into(), "".into())
                        ),
                        AppMessage::PluginMessage(
                            0,
                            PluginMsg::Edit("color_apply".into(), "".into(), "".into())
                        )
                    ),
                    AppMessage::PluginMessage(
                        0,
                        PluginMsg::Edit("color_cancel".into(), "".into(), "".into())
                    ),
                    true
                )
            ]
            .into()
        } else if let Some(modal_id) = &self.modal_type {
            let modal_content = self.view_modal_content(modal_id);
            view::modal::modal(modal_content)
        } else {
            main_view.into()
        }
    }
    fn searchable_items(&self) -> Vec<SearchResult> {
        let mut results = Vec::new();

        for cmd in &self.exec_cmds {
            results.push(SearchResult {
                id: cmd.raw.clone(),
                title: format!("Exec: {}", cmd.command),
                description: cmd.exec_type.clone(),
                tab_id: "exec".to_string(),
            });
        }

        for rule in &self.window_rules {
            results.push(SearchResult {
                id: rule.raw.clone(),
                title: format!("Rule: {}", rule.match_str()),
                description: rule.effect_str(),
                tab_id: "rules".to_string(),
            });
        }

        for bind in &self.keybinds {
            results.push(SearchResult {
                id: bind.raw.clone(),
                title: format!("Bind: {} {}", bind.mods, bind.key),
                description: format!("{} {}", bind.dispatcher, bind.params),
                tab_id: "keybinds".to_string(),
            });
        }

        for env in &self.env_vars {
            results.push(SearchResult {
                id: env.raw.clone(),
                title: format!("Env: {}", env.name),
                description: env.value.clone(),
                tab_id: "env".to_string(),
            });
        }

        for gesture in &self.gestures {
            results.push(SearchResult {
                id: gesture.raw.clone(),
                title: format!("Gesture: {} fingers {}", gesture.fingers, gesture.direction),
                description: gesture.action.clone(),
                tab_id: "gestures".to_string(),
            });
        }

        for tab in schema::get_schema() {
            results.push(SearchResult {
                id: tab.id.clone(),
                title: format!("Menu: {}", tab.title),
                description: "Settings Menu".to_string(),
                tab_id: tab.id.clone(),
            });

            for section in tab.sections {
                let section_id = format!("{}:section:{}", tab.id, section.name);
                results.push(SearchResult {
                    id: section_id,
                    title: format!("Submenu: {} > {}", tab.title, section.title),
                    description: "Settings Section".to_string(),
                    tab_id: tab.id.clone(),
                });

                for option in section.options {
                    let path = format!("{}:{}", section.name, option.name);

                    let val = self
                        .config
                        .get_option(&path)
                        .unwrap_or(option.default.clone());

                    results.push(SearchResult {
                        id: path,
                        title: format!("Setting: {}", option.name),
                        description: format!("{} (Current: {})", option.description, val),
                        tab_id: tab.id.clone(),
                    });
                }
            }
        }

        results
    }
}

impl HyprlandPlugin {
    fn view_modal_content(&self, modal_id: &str) -> Element<'_, AppMessage> {
        if modal_id == "preset_conflict" {
            return column![
                text("Configuration Changed")
                    .size(22)
                    .font(iced::font::Font {
                        weight: iced::font::Weight::Bold,
                        ..Default::default()
                    })
                    .style(|_: &Theme| iced::widget::text::Style {
                        color: Some(Color::from_rgb8(205, 214, 244))
                    }),
                text(format!(
                    "Your current configuration differs from the active preset '{}'.",
                    self.active_preset.clone().unwrap_or_default()
                ))
                .size(16)
                .style(|_: &Theme| iced::widget::text::Style {
                    color: Some(Color::from_rgb8(166, 173, 200))
                }),
                row![
                    btn::primary(
                        text("Update Preset"),
                        AppMessage::PluginMessage(
                            0,
                            PluginMsg::Edit(
                                "preset_overwrite".into(),
                                "conflict".into(),
                                "".into()
                            )
                        )
                    ),
                    btn::secondary(
                        text("Create New Preset"),
                        AppMessage::PluginMessage(0, PluginMsg::OpenModal("save_preset".into()))
                    ),
                    btn::destructive(
                        text("Detach Preset"),
                        AppMessage::PluginMessage(
                            0,
                            PluginMsg::Edit("preset_detach".into(), "conflict".into(), "".into())
                        )
                    ),
                ]
                .spacing(12)
            ]
            .spacing(24)
            .into();
        }

        if modal_id == "upgrade_migration" {
            let version_str = self
                .hyprland_version
                .as_ref()
                .map(|v| v.to_string())
                .unwrap_or_else(|| "0.53.0+".to_string());
            return column![
                text("Hyprland Upgrade Detected")
                    .size(22)
                    .font(iced::font::Font {
                        weight: iced::font::Weight::Bold,
                        ..Default::default()
                    })
                    .style(|_: &Theme| iced::widget::text::Style {
                        color: Some(Color::from_rgb8(205, 214, 244))
                    }),
                text(format!(
                    "You are running Hyprland {}. Your config uses legacy window/layer rule syntax.",
                    version_str
                ))
                .size(16)
                .style(|_: &Theme| iced::widget::text::Style {
                    color: Some(Color::from_rgb8(166, 173, 200))
                }),
                text("Would you like to migrate to the new window rule syntax?")
                    .size(14)
                    .style(|_: &Theme| iced::widget::text::Style {
                        color: Some(Color::from_rgb8(127, 132, 156))
                    }),
                row![
                    btn::primary(
                        text("Migrate Config"),
                        AppMessage::PluginMessage(
                            0,
                            PluginMsg::Edit("run_migration".into(), "".into(), "".into())
                        )
                    ),
                    btn::ghost(
                        text("Skip (Keep Legacy)"),
                        AppMessage::PluginMessage(0, PluginMsg::CloseModal)
                    ),
                ]
                .spacing(12)
            ]
            .spacing(20)
            .into();
        }

        let title = if modal_id.contains("add") {
            "Add Item"
        } else {
            "Edit Item"
        };
        let header = text(title)
            .size(22)
            .font(iced::font::Font {
                weight: iced::font::Weight::Bold,
                ..Default::default()
            })
            .style(|_: &Theme| iced::widget::text::Style {
                color: Some(Color::from_rgb8(205, 214, 244)),
            });

        let label_style = |_: &Theme| iced::widget::text::Style {
            color: Some(Color::from_rgb8(166, 173, 200)),
        };

        let fields: Element<AppMessage> = if modal_id.starts_with("add_window_rule")
            || modal_id.starts_with("edit_window_rule")
        {
            let effects = vec![
                "float",
                "tile",
                "fullscreen",
                "maximize",
                "nofocus",
                "pin",
                "opacity 0.9",
                "noborder",
                "noshadow",
                "noblur",
                "center",
            ];
            let current_effect = self.input_val("effect");
            let selected_effect = effects
                .iter()
                .find(|t| **t == current_effect)
                .map(|s| s.to_string());

            column![
                text("Rule Type").size(13).style(label_style),
                ti::input("windowrule", &self.input_val("type"), |s| {
                    AppMessage::PluginMessage(0, PluginMsg::Edit("input".into(), "type".into(), s))
                }),
                text("Effect").size(13).style(label_style),
                pick_list(
                    effects.iter().map(|s| s.to_string()).collect::<Vec<_>>(),
                    selected_effect,
                    |s| AppMessage::PluginMessage(
                        0,
                        PluginMsg::Edit("input".into(), "effect".into(), s)
                    )
                ),
                text("Match").size(13).style(label_style),
                ti::input("float", &self.input_val("match"), |s| {
                    AppMessage::PluginMessage(0, PluginMsg::Edit("input".into(), "match".into(), s))
                }),
            ]
            .spacing(12)
            .into()
        } else if modal_id.starts_with("add_exec") || modal_id.starts_with("edit_exec") {
            let exec_types = vec!["exec-once", "exec"];
            let current_type = self.input_val("type");
            let selected = if current_type.is_empty() {
                Some("exec-once".to_string())
            } else {
                exec_types
                    .iter()
                    .find(|t| **t == current_type)
                    .map(|s| s.to_string())
            };

            column![
                text("Type").size(13).style(label_style),
                pick_list(
                    exec_types.iter().map(|s| s.to_string()).collect::<Vec<_>>(),
                    selected,
                    |s| AppMessage::PluginMessage(
                        0,
                        PluginMsg::Edit("input".into(), "type".into(), s)
                    )
                ),
                text("Command").size(13).style(label_style),
                ti::input("command", &self.input_val("command"), |s| {
                    AppMessage::PluginMessage(
                        0,
                        PluginMsg::Edit("input".into(), "command".into(), s),
                    )
                }),
            ]
            .spacing(12)
            .into()
        } else if modal_id.starts_with("add_env") || modal_id.starts_with("edit_env") {
            column![
                text("Variable Name").size(13).style(label_style),
                ti::input("GTK_THEME", &self.input_val("name"), |s| {
                    AppMessage::PluginMessage(0, PluginMsg::Edit("input".into(), "name".into(), s))
                }),
                text("Value").size(13).style(label_style),
                ti::input("Adwaita", &self.input_val("value"), |s| {
                    AppMessage::PluginMessage(0, PluginMsg::Edit("input".into(), "value".into(), s))
                }),
            ]
            .spacing(12)
            .into()
        } else if modal_id.starts_with("add_bind") || modal_id.starts_with("edit_bind") {
            let bind_types = vec![
                "bind - Standard",
                "binde - Repeat",
                "bindm - Mouse",
                "bindl - Works on Lock Screen",
                "bindr - Trigger on Release",
                "bindel - Repeat + Lock Screen",
            ];
            let current_type = self.input_val("type");
            let selected = if current_type.is_empty() {
                Some("bind - Standard".to_string())
            } else {
                bind_types
                    .iter()
                    .find(|t| t.split_whitespace().next().unwrap_or("") == current_type)
                    .map(|s| s.to_string())
            };

            scrollable(
                column![
                    text("Bind Type").size(13).style(label_style),
                    pick_list(
                        bind_types.iter().map(|s| s.to_string()).collect::<Vec<_>>(),
                        selected,
                        |s| AppMessage::PluginMessage(
                            0,
                            PluginMsg::Edit("input".into(), "type".into(), s)
                        )
                    ),
                    row![
                        text("Modifiers & Key").size(13).style(label_style),
                        iced::widget::Space::new().width(iced::Length::Fill),
                        if self.capturing_bind {
                            btn::small_destructive(
                                text("Stop Recording"),
                                AppMessage::PluginMessage(0, PluginMsg::Toggle(false)),
                            )
                        } else {
                            btn::small_secondary(
                                text("Record Combo"),
                                AppMessage::PluginMessage(0, PluginMsg::Toggle(true)),
                            )
                        }
                    ]
                    .align_y(iced::Alignment::Center)
                    .width(iced::Length::Fill),
                    row![
                        ti::input("SUPER", &self.input_val("mods"), |s| {
                            AppMessage::PluginMessage(
                                0,
                                PluginMsg::Edit("input".into(), "mods".into(), s),
                            )
                        }),
                        ti::input("Q", &self.input_val("key"), |s| AppMessage::PluginMessage(
                            0,
                            PluginMsg::Edit("input".into(), "key".into(), s)
                        )),
                    ]
                    .spacing(8),
                    text("Dispatcher").size(13).style(label_style),
                    {
                        let current = self.input_val("dispatcher");
                        let selected = if current.is_empty() {
                            None
                        } else {
                            Some(&current)
                        };

                        combo_box(&self.dispatcher_combo, "Select Dispatcher", selected, |s| {
                            AppMessage::PluginMessage(
                                0,
                                PluginMsg::Edit("input".into(), "dispatcher".into(), s),
                            )
                        })
                        .on_input(|s| {
                            AppMessage::PluginMessage(
                                0,
                                PluginMsg::Edit("input".into(), "dispatcher".into(), s),
                            )
                        })
                        .width(iced::Length::Fill)
                    },
                    text("Parameters").size(13).style(label_style),
                    ti::input("kitty", &self.input_val("params"), |s| {
                        AppMessage::PluginMessage(
                            0,
                            PluginMsg::Edit("input".into(), "params".into(), s),
                        )
                    }),
                ]
                .spacing(12),
            )
            .height(iced::Length::Fixed(300.0))
            .into()
        } else if modal_id.starts_with("add_gesture") || modal_id.starts_with("edit_gesture") {
            scrollable(
                column![
                    text("Fingers").size(13).style(label_style),
                    ti::input("3", &self.input_val("fingers"), |s| {
                        AppMessage::PluginMessage(
                            0,
                            PluginMsg::Edit("input".into(), "fingers".into(), s),
                        )
                    }),
                    text("Direction").size(13).style(label_style),
                    ti::input("left", &self.input_val("direction"), |s| {
                        AppMessage::PluginMessage(
                            0,
                            PluginMsg::Edit("input".into(), "direction".into(), s),
                        )
                    }),
                    text("Action").size(13).style(label_style),
                    ti::input("workspace", &self.input_val("action"), |s| {
                        AppMessage::PluginMessage(
                            0,
                            PluginMsg::Edit("input".into(), "action".into(), s),
                        )
                    }),
                    text("Dispatcher").size(13).style(label_style),
                    {
                        let current = self.input_val("dispatcher");
                        let selected = if current.is_empty() {
                            None
                        } else {
                            Some(&current)
                        };

                        combo_box(&self.dispatcher_combo, "Select Dispatcher", selected, |s| {
                            AppMessage::PluginMessage(
                                0,
                                PluginMsg::Edit("input".into(), "dispatcher".into(), s),
                            )
                        })
                        .on_input(|s| {
                            AppMessage::PluginMessage(
                                0,
                                PluginMsg::Edit("input".into(), "dispatcher".into(), s),
                            )
                        })
                        .width(iced::Length::Fill)
                    },
                    text("Parameters").size(13).style(label_style),
                    ti::input("params", &self.input_val("params"), |s| {
                        AppMessage::PluginMessage(
                            0,
                            PluginMsg::Edit("input".into(), "params".into(), s),
                        )
                    }),
                    text("Mod Key (optional)").size(13).style(label_style),
                    ti::input("", &self.input_val("mod_key"), |s| {
                        AppMessage::PluginMessage(
                            0,
                            PluginMsg::Edit("input".into(), "mod_key".into(), s),
                        )
                    }),
                    text("Scale (optional)").size(13).style(label_style),
                    ti::input("", &self.input_val("scale"), |s| AppMessage::PluginMessage(
                        0,
                        PluginMsg::Edit("input".into(), "scale".into(), s)
                    )),
                ]
                .spacing(12),
            )
            .height(iced::Length::Fixed(300.0))
            .into()
        } else if modal_id.starts_with("save_preset") {
            column![
                text("Preset Name").size(13).style(label_style),
                ti::input("My Preset", &self.input_val("name"), |s| {
                    AppMessage::PluginMessage(0, PluginMsg::Edit("input".into(), "name".into(), s))
                }),
            ]
            .spacing(12)
            .into()
        } else {
            text("Unknown modal type").into()
        };

        column![
            header,
            fields,
            row![
                btn::secondary(
                    text("Cancel"),
                    AppMessage::PluginMessage(0, PluginMsg::CloseModal)
                ),
                btn::primary(
                    text("Save"),
                    AppMessage::PluginMessage(
                        0,
                        PluginMsg::Edit(
                            if modal_id == "save_preset" {
                                "preset_save".into()
                            } else {
                                "submit".into()
                            },
                            modal_id.to_string(),
                            "".into()
                        )
                    )
                ),
            ]
            .spacing(12)
        ]
        .spacing(24)
        .into()
    }
}

fn modifiers_to_string(modifiers: iced::keyboard::Modifiers) -> String {
    let mut parts = Vec::new();
    if modifiers.contains(iced::keyboard::Modifiers::LOGO) {
        parts.push("SUPER");
    }
    if modifiers.contains(iced::keyboard::Modifiers::CTRL) {
        parts.push("CTRL");
    }
    if modifiers.contains(iced::keyboard::Modifiers::SHIFT) {
        parts.push("SHIFT");
    }
    if modifiers.contains(iced::keyboard::Modifiers::ALT) {
        parts.push("ALT");
    }
    parts.join(" ")
}

fn key_to_string(key: iced::keyboard::Key) -> String {
    match key {
        iced::keyboard::Key::Character(c) => c.to_uppercase().to_string(),
        iced::keyboard::Key::Named(n) => format!("{:?}", n).to_uppercase(),
        _ => String::new(),
    }
}
