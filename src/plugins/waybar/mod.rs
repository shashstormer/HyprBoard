use self::parser::Node;
use self::schema::{ModuleSchema, OptionType};
use crate::core::waybar_action::{EditorTab, ReorderDirection, WaybarAction};
use crate::core::{AppMessage, Plugin, PluginMsg};
use crate::view::components::schema_renderer::{self, OptionDef};
use crate::view::components::{
    button as btn, color_picker, modal, text_input as ti, theme::AppTheme,
};
use iced::{
    Element, Length, Task,
    widget::{button, column, container, row, scrollable, stack, text},
};
use serde_json::Value;
use std::collections::HashMap;
use std::path::PathBuf;

pub mod css_parser;
pub mod parser;
pub mod presets_view;
pub mod schema;
mod tests;

use crate::core::presets::{Preset, PresetManager};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WaybarMode {
    Layout,
    Edit,
    Presets,
}

pub struct WaybarPlugin {
    id: usize,
    config_path: PathBuf,
    ast_root: Option<Node>,
    config_cache: Value,
    active_module: Option<String>,
    inputs: HashMap<String, String>,
    search_query: String,
    style_path: PathBuf,
    style_cache: String,
    preset_manager: PresetManager,
    presets_list: Vec<Preset>,
    active_preset: Option<String>,
    color_modal_open: bool,
    color_modal_target: Option<String>,
    color_modal_value: String,

    mode: WaybarMode,
    selected_item: Option<(String, usize)>,
    available_modules_cache: Vec<String>,
    delete_modal_open: bool,
    delete_target: Option<String>,
    delete_input: String,

    current_tab: EditorTab,
    json_content: iced::widget::text_editor::Content,

    style_content: iced::widget::text_editor::Content,

    style_bg_color: String,
    style_text_color: String,
    style_font_size: String,
    style_padding: String,
    debug_output: String,
    toasts: Vec<crate::view::components::toast::Toast>,
    create_custom_modal_open: bool,
    create_custom_name: String,
    json_error_modal_open: bool,
    json_error_message: String,
    pending_json_save: Option<String>,

    custom_option_key_input: String,
    custom_option_val_input: String,
    delete_option_modal_open: bool,
    delete_option_target: Option<String>,
}

impl WaybarPlugin {
    pub fn new(id: usize) -> Self {
        let home = std::env::var("HOME").unwrap_or("/tmp".into());
        let config_dir = PathBuf::from(home).join(".config/waybar");

        let config_path = config_dir.join("config.jsonc");
        let config_path = if !config_path.exists() {
            config_dir.join("config")
        } else {
            config_path
        };

        let style_path = config_dir.join("style.css");
        let style_cache = std::fs::read_to_string(&style_path).unwrap_or_default();

        let content = std::fs::read_to_string(&config_path).unwrap_or_else(|_| "{}".to_string());

        let ast_root = parser::parse(&content).ok();
        let config_cache = if let Some(root) = &ast_root {
            parser::to_json_value(root)
        } else {
            serde_json::Value::Null
        };

        let preset_manager = PresetManager::new("waybar");
        let active_preset = preset_manager.get_active();
        let presets_list = preset_manager.list();

        let mut plugin = WaybarPlugin {
            id,
            ast_root,
            config_path,
            config_cache,
            active_module: None,
            inputs: HashMap::new(),
            search_query: String::new(),
            style_path,
            style_cache,
            preset_manager,
            presets_list,
            active_preset,
            color_modal_open: false,
            color_modal_target: None,
            color_modal_value: String::new(),
            mode: WaybarMode::Layout,
            selected_item: None,
            available_modules_cache: Vec::new(),
            delete_modal_open: false,
            delete_target: None,
            delete_input: String::new(),

            current_tab: EditorTab::Settings,
            json_content: iced::widget::text_editor::Content::new(),
            style_content: iced::widget::text_editor::Content::new(),
            style_bg_color: String::new(),
            style_text_color: String::new(),
            style_font_size: String::new(),
            style_padding: String::new(),
            debug_output: String::new(),
            toasts: Vec::new(),
            create_custom_modal_open: false,
            create_custom_name: String::new(),
            json_error_modal_open: false,
            json_error_message: String::new(),
            pending_json_save: None,
            custom_option_key_input: String::new(),
            custom_option_val_input: String::new(),
            delete_option_modal_open: false,
            delete_option_target: None,
        };
        plugin.recalc_available_modules();
        plugin
    }

    fn get_list_items(&self, list_name: &str) -> Vec<String> {
        if list_name == "available" {
            return self.available_modules_cache.clone();
        }

        let root = match &self.config_cache {
            Value::Array(arr) => arr.get(0).unwrap_or(&Value::Null),
            obj => obj,
        };

        if let Some(list) = root.get(list_name).and_then(|v| v.as_array()) {
            list.iter()
                .map(|v| v.as_str().unwrap_or("").to_string())
                .collect()
        } else {
            Vec::new()
        }
    }

    fn recalc_available_modules(&mut self) {
        let all_schema_modules: Vec<String> = schema::get_schema()
            .into_iter()
            .map(|s| s.module_type)
            .filter(|m| m != "general")
            .collect();

        let used: Vec<String> = ["modules-left", "modules-center", "modules-right"]
            .iter()
            .flat_map(|l| self.get_list_items(l))
            .collect();

        self.available_modules_cache = all_schema_modules
            .into_iter()
            .filter(|m| !used.contains(m))
            .collect();
    }

    fn get_css_selector(&self, module_name: &str) -> String {
        if module_name.starts_with("custom/") {
            let name = module_name.strip_prefix("custom/").unwrap();
            format!("#custom-{}", name)
        } else {
            format!("#{}", module_name)
        }
    }

    fn get_modules(&self) -> Vec<String> {
        let root = match &self.config_cache {
            Value::Array(arr) => arr.get(0).unwrap_or(&Value::Null),
            obj => obj,
        };

        let mut modules: Vec<String> = if let Some(obj) = root.as_object() {
            obj.keys().cloned().collect()
        } else {
            Vec::new()
        };

        for list in ["modules-left", "modules-center", "modules-right"] {
            modules.extend(self.get_list_items(list));
        }

        let reserved = [
            "layer",
            "position",
            "height",
            "width",
            "spacing",
            "margin-top",
            "margin-bottom",
            "margin-left",
            "margin-right",
            "name",
            "mode",
            "modules-left",
            "modules-center",
            "modules-right",
            "include",
            "reload_style_on_change",
            "general",
        ];

        modules.sort();
        modules.dedup();

        modules
            .into_iter()
            .filter(|m| !reserved.contains(&m.as_str()))
            .collect()
    }

    fn get_schema_for_module(&self, module_name: &str) -> Option<ModuleSchema> {
        let schemas = schema::get_schema();
        if let Some(s) = schemas.iter().find(|s| s.module_type == module_name) {
            return Some(s.clone());
        }
        if module_name.starts_with("custom/") {
            return schemas.iter().find(|s| s.module_type == "custom").cloned();
        }
        None
    }

    fn render_module_preview<'a>(
        &'a self,
        module_name: &str,
        theme: &'a AppTheme,
    ) -> Element<'a, AppMessage> {
        let palette = theme.palette();
        let schema = self.get_schema_for_module(module_name);

        let root = match &self.config_cache {
            Value::Array(arr) => arr.get(0).unwrap_or(&Value::Null),
            o => o,
        };

        let get_val = |key: &str| -> String {
            if let Some(mod_val) = root.get(module_name) {
                if let Some(v) = mod_val.get(key) {
                    return v.as_str().unwrap_or("").to_string();
                }
            }
            if let Some(s) = &schema {
                if let Some(opt) = s.options.iter().find(|o| o.name == key) {
                    return opt.default.clone();
                }
            }
            "".to_string()
        };

        let format = get_val("format");
        let mut display_text = if format.is_empty() {
            module_name.to_string()
        } else {
            format
        };

        let icon = schema.as_ref().map(|s| s.icon).unwrap_or(' ');

        lazy_static::lazy_static! {
            static ref RE: regex::Regex = regex::Regex::new(r"\{([a-zA-Z0-9_-]+)(?::.*?)?\}").unwrap();
            static ref REPLACEMENTS_MAP: HashMap<&'static str, &'static str> = HashMap::from([
                 ("usage", "12"),
                 ("capacity", "75"),
                 ("temperatureC", "45"),
                 ("temperatureF", "113"),
                 ("ipaddr", "192.168.1.55"),
                 ("ifname", "wlan0"),
                 ("essid", "Home_WiFi"),
                 ("signalStrength", "80"),
                 ("volume", "65"),
                 ("icon", ""),
            ]);
        }

        display_text = RE
            .replace_all(&display_text, |caps: &regex::Captures| {
                let key = &caps[1];
                if key == "icon" {
                    return icon.to_string();
                }

                if display_text.contains("%H") || display_text.contains("%M") {
                    if caps
                        .get(0)
                        .map(|m| m.as_str().contains(":%"))
                        .unwrap_or(false)
                    {
                        return "14:30".to_string();
                    }
                }
                if display_text.contains("%Y") || display_text.contains("%d") {
                    if caps
                        .get(0)
                        .map(|m| m.as_str().contains(":%"))
                        .unwrap_or(false)
                    {
                        return "2023-10-27".to_string();
                    }
                }

                if let Some(val) = REPLACEMENTS_MAP.get(key) {
                    val.to_string()
                } else {
                    "?".to_string()
                }
            })
            .to_string();

        if display_text.contains("{icon}") {
            display_text = display_text.replace("{icon}", &icon.to_string());
        }

        container(text(display_text).size(13).style(move |_: &_| text::Style {
            color: Some(palette.text),
        }))
        .padding([4, 8])
        .style(move |_: &_| container::Style {
            background: Some(iced::Background::Color(palette.surface0)),
            border: iced::Border {
                radius: 4.0.into(),
                width: 1.0,
                color: palette.surface1,
            },
            ..Default::default()
        })
        .into()
    }

    fn save_config(&mut self) {
        if let Some(root) = &self.ast_root {
            let new_content = parser::to_string(root);
            if let Err(e) = std::fs::write(&self.config_path, &new_content) {
                eprintln!("[Waybar] Failed to save config: {}", e);
                self.toasts.push(crate::view::components::toast::Toast::new(
                    format!("Failed to save: {}", e),
                    crate::view::components::toast::ToastType::Error,
                ));
            } else {
                self.toasts.push(crate::view::components::toast::Toast::new(
                    "Configuration Saved".to_string(),
                    crate::view::components::toast::ToastType::Success,
                ));
            }
            self.config_cache = parser::to_json_value(root);
            self.recalc_available_modules();
        }
    }
}

impl Plugin for WaybarPlugin {
    fn name(&self) -> String {
        "Waybar".to_string()
    }
    fn icon(&self) -> char {
        '直'
    }

    fn update(&mut self, message: PluginMsg) -> Task<AppMessage> {
        self.toasts.retain(|t| !t.is_expired());
        match message {
            PluginMsg::SwitchInternalTab(module_name) => {
                match module_name.as_str() {
                    "layout" | "group" | "order" => self.mode = WaybarMode::Layout,
                    "settings" => {
                        self.mode = WaybarMode::Edit;
                        self.active_module = None;
                    }
                    "presets" => self.mode = WaybarMode::Presets,
                    _ => {
                        self.active_module = Some(module_name);
                        self.mode = WaybarMode::Edit;
                    }
                }
                self.inputs.clear();
            }
            PluginMsg::UpdateConfig(path, value) => {
                let path = path.replace(".", ":");
                if path == "internal:search" {
                    self.search_query = value;
                    return Task::none();
                }
                if path == "internal:view_mode" {
                    self.inputs.insert("view_mode".into(), value);
                    return Task::none();
                }

                if let Some(stripped) = path.strip_prefix("style:") {
                    if let Some((selector, prop)) = stripped.split_once(':') {
                        let mut parser = css_parser::CssParser::new(&self.style_cache);
                        parser.set_property(selector, prop, &value);
                        let new_content = parser.to_string();

                        let _ = std::fs::write(&self.style_path, &new_content);
                        self.style_cache = new_content;
                        self.style_content =
                            iced::widget::text_editor::Content::with_text(&self.style_cache);

                        if let Some(m) = &self.active_module {
                            if selector == self.get_css_selector(m) {
                                match prop {
                                    "background-color" | "background" => {
                                        self.style_bg_color = value
                                    }
                                    "color" => self.style_text_color = value,
                                    "font-size" => self.style_font_size = value,
                                    "padding" => self.style_padding = value,
                                    _ => {}
                                }
                            }
                        }
                    }
                    return Task::none();
                }

                if let Some((mod_name, key)) = path.split_once(':') {
                    let val_to_set = if value == "true" {
                        serde_json::Value::Bool(true)
                    } else if value == "false" {
                        serde_json::Value::Bool(false)
                    } else if let Ok(i) = value.parse::<i64>() {
                        serde_json::Value::Number(i.into())
                    } else if let Ok(f) = value.parse::<f64>() {
                        if let Some(n) = serde_json::Number::from_f64(f) {
                            serde_json::Value::Number(n)
                        } else {
                            Value::String(value.clone())
                        }
                    } else {
                        Value::String(value.clone())
                    };

                    if let Some(root) = &mut self.ast_root {
                        let is_array = matches!(root, Node::List(_));

                        let path_vec = if mod_name == "general" {
                            if is_array {
                                vec!["0".to_string(), key.to_string()]
                            } else {
                                vec![key.to_string()]
                            }
                        } else {
                            if let Node::List(bars) = root {
                                let mut bar_idx = 0;
                                for (i, (bar_node, _)) in bars.children.iter().enumerate() {
                                    if let Node::Dict(dict) = bar_node {
                                        if dict.children.iter().any(|(k, _, _)| k.value == mod_name)
                                        {
                                            bar_idx = i;
                                            break;
                                        }
                                    }
                                }
                                vec![bar_idx.to_string(), mod_name.to_string(), key.to_string()]
                            } else {
                                vec![mod_name.to_string(), key.to_string()]
                            }
                        };

                        let path_refs: Vec<&str> = path_vec.iter().map(|s| s.as_str()).collect();
                        parser::set_value(root, &path_refs, val_to_set.clone());
                        self.save_config();

                        if let Some(name) = &self.active_preset {
                            if let (Ok(config), Ok(style)) = (
                                std::fs::read_to_string(&self.config_path),
                                std::fs::read_to_string(&self.style_path),
                            ) {
                                let mut files = HashMap::new();
                                files.insert("config.jsonc".to_string(), config);
                                files.insert("style.css".to_string(), style);
                                let _ = self.preset_manager.save(name, &files);
                            }
                        }
                    }
                }
            }
            PluginMsg::Waybar(action) => match action {
                WaybarAction::PresetModalOpen => {
                    self.active_module = Some("presets".to_string());
                    self.inputs
                        .insert("preset_name".to_string(), "".to_string());
                }
                WaybarAction::PresetInput(data) => {
                    self.inputs.insert("preset_name".to_string(), data);
                }
                WaybarAction::PresetSave => {
                    let name = self.inputs.get("preset_name").cloned().unwrap_or_default();
                    if !name.is_empty() {
                        if let (Ok(config), Ok(style)) = (
                            std::fs::read_to_string(&self.config_path),
                            std::fs::read_to_string(&self.style_path),
                        ) {
                            let mut files = HashMap::new();
                            files.insert("config.jsonc".to_string(), config);
                            files.insert("style.css".to_string(), style);
                            let _ = self.preset_manager.save(&name, &files);
                            self.presets_list = self.preset_manager.list();
                            self.inputs.remove("preset_name");
                        }
                    }
                }
                WaybarAction::PresetLoad(name) => {
                    return self.update(PluginMsg::LoadPreset(name));
                }
                WaybarAction::PresetDelete(name) => {
                    let _ = self.preset_manager.delete(&name);
                    if self.active_preset.as_ref() == Some(&name) {
                        self.active_preset = None;
                        let _ = self.preset_manager.set_active(None);
                    }
                    self.presets_list = self.preset_manager.list();
                }
                WaybarAction::Reorder { item, direction } => {
                    let lists = ["modules-left", "modules-center", "modules-right"];
                    if let Some(list_name) = lists
                        .iter()
                        .find(|l| self.get_list_items(l).contains(&item))
                    {
                        let current_items = self.get_list_items(list_name);
                        if let Some(idx) = current_items.iter().position(|x| x == &item) {
                            let new_idx = match direction {
                                ReorderDirection::Up => {
                                    if idx > 0 {
                                        idx - 1
                                    } else {
                                        idx
                                    }
                                }
                                ReorderDirection::Down => {
                                    if idx < current_items.len() - 1 {
                                        idx + 1
                                    } else {
                                        idx
                                    }
                                }
                            };

                            if idx != new_idx {
                                if let Some(root) = &mut self.ast_root {
                                    let list_path = if matches!(root, Node::List(_)) {
                                        vec!["0", *list_name]
                                    } else {
                                        vec![*list_name]
                                    };
                                    if let Some(node) =
                                        parser::remove_from_list_by_value(root, &list_path, &item)
                                    {
                                        let _ = parser::insert_into_list(
                                            root, &list_path, new_idx, node,
                                        );
                                        self.save_config();
                                    }
                                }
                            }
                        }
                    }
                }
                WaybarAction::Move { item, target_list } => {
                    let lists = ["modules-left", "modules-center", "modules-right"];
                    if let Some(from_list) = lists
                        .iter()
                        .find(|l| self.get_list_items(l).contains(&item))
                    {
                        let target_len = self.get_list_items(&target_list).len();
                        if let Some(root) = &mut self.ast_root {
                            let from_path = if matches!(root, Node::List(_)) {
                                vec!["0", *from_list]
                            } else {
                                vec![*from_list]
                            };
                            if let Some(node) =
                                parser::remove_from_list_by_value(root, &from_path, &item)
                            {
                                let to_path = if matches!(root, Node::List(_)) {
                                    vec!["0", target_list.as_str()]
                                } else {
                                    vec![target_list.as_str()]
                                };
                                let _ = parser::insert_into_list(root, &to_path, target_len, node);
                                self.save_config();
                            }
                        }
                    }
                }
                WaybarAction::Remove { item } => {
                    let lists = ["modules-left", "modules-center", "modules-right"];
                    if let Some(from_list) = lists
                        .iter()
                        .find(|l| self.get_list_items(l).contains(&item))
                    {
                        if let Some(root) = &mut self.ast_root {
                            let from_path = if matches!(root, Node::List(_)) {
                                vec!["0", *from_list]
                            } else {
                                vec![*from_list]
                            };
                            let _ = parser::remove_from_list_by_value(root, &from_path, &item);
                            self.save_config();
                        }
                    }
                }
                WaybarAction::Add { item, target_list } => {
                    let target_len = self.get_list_items(&target_list).len();
                    if let Some(root) = &mut self.ast_root {
                        let to_path = if matches!(root, Node::List(_)) {
                            vec!["0", target_list.as_str()]
                        } else {
                            vec![target_list.as_str()]
                        };
                        let val = serde_json::Value::String(item.clone());
                        let node = parser::create_node_from_value(&val, "    ", 1);
                        let _ = parser::insert_into_list(root, &to_path, target_len, node);
                        self.save_config();
                    }
                }
                WaybarAction::ColorPick { target } => {
                    let current_val = if target.starts_with("style:") {
                        if let Some(stripped) = target.strip_prefix("style:") {
                            if let Some((selector, prop)) = stripped.split_once(':') {
                                let parser = css_parser::CssParser::new(&self.style_cache);
                                parser
                                    .get_property(selector, prop)
                                    .unwrap_or("rgba(0,0,0,1)".to_string())
                            } else {
                                "rgba(0,0,0,1)".to_string()
                            }
                        } else {
                            "rgba(0,0,0,1)".to_string()
                        }
                    } else {
                        let parts: Vec<&str> = target.split(':').collect();
                        if parts.len() == 2 {
                            let m = parts[0];
                            let opt = parts[1];
                            let root = match &self.config_cache {
                                Value::Array(arr) => arr.get(0).unwrap_or(&Value::Null),
                                o => o,
                            };
                            let val_node = if m == "general" {
                                root.get(opt)
                            } else {
                                root.get(m).and_then(|v| v.get(opt))
                            };
                            val_node
                                .map(|v| v.as_str().unwrap_or("").to_string())
                                .unwrap_or("rgba(0,0,0,1)".to_string())
                        } else {
                            "rgba(0,0,0,1)".to_string()
                        }
                    };
                    self.color_modal_open = true;
                    self.color_modal_target = Some(target);
                    self.color_modal_value = current_val;
                }
                WaybarAction::ColorCancel => {
                    self.color_modal_open = false;
                    self.color_modal_target = None;
                }
                WaybarAction::ColorApply => {
                    if let Some(target) = &self.color_modal_target {
                        return self.update(PluginMsg::UpdateConfig(
                            target.clone(),
                            self.color_modal_value.clone(),
                        ));
                    }
                    self.color_modal_open = false;
                    self.color_modal_target = None;
                }
                WaybarAction::ColorUpdate(val) => {
                    self.color_modal_value = val;
                }
                WaybarAction::DeleteInit(target) => {
                    self.delete_modal_open = true;
                    self.delete_target = Some(target);
                    self.delete_input = String::new();
                }
                WaybarAction::DeleteInput(val) => {
                    self.delete_input = val;
                }
                WaybarAction::DeleteConfirm => {
                    if let Some(target) = &self.delete_target {
                        if &self.delete_input == target {
                            let lists = ["modules-left", "modules-center", "modules-right"];
                            if let Some(root) = &mut self.ast_root {
                                for list in lists {
                                    let list_path = if matches!(root, Node::List(_)) {
                                        vec!["0", list]
                                    } else {
                                        vec![list]
                                    };
                                    let _ =
                                        parser::remove_from_list_by_value(root, &list_path, target);
                                }

                                let path_vec = if matches!(root, Node::List(_)) {
                                    vec!["0", target.as_str()]
                                } else {
                                    vec![target.as_str()]
                                };
                                let path_refs = path_vec;
                                let _ = parser::remove_key(root, &path_refs);
                            }
                        }
                    }
                    self.delete_modal_open = false;
                    self.delete_target = None;
                    self.delete_input = String::new();
                    self.save_config();
                }
                WaybarAction::DeleteCancel => {
                    self.delete_modal_open = false;
                    self.delete_target = None;
                    self.delete_input = String::new();
                }
                WaybarAction::CreateCustomInit => {
                    self.create_custom_modal_open = true;
                    self.create_custom_name = String::new();
                }
                WaybarAction::CreateCustomInput(val) => {
                    self.create_custom_name = val;
                }
                WaybarAction::CreateCustomConfirm(name) => {
                    let name = if name.starts_with("custom/") {
                        name.to_string()
                    } else {
                        format!("custom/{}", name)
                    };

                    let exists = if let Some(_root) = &self.ast_root {
                        let val = match self.config_cache {
                            Value::Array(ref arr) => arr.get(0),
                            ref o => Some(o),
                        };
                        val.and_then(|v| v.get(&name)).is_some()
                    } else {
                        false
                    };

                    if !exists {
                        if let Some(mut root) = self.ast_root.take() {
                            let path = if matches!(root, Node::List(_)) {
                                vec!["0", name.as_str()]
                            } else {
                                vec![name.as_str()]
                            };

                            let default_config = serde_json::json!({
                                "exec": "echo 'New Component'",
                                "format": "{}",
                                "interval": 30
                            });

                            parser::set_value(&mut root, &path, default_config);
                            self.ast_root = Some(root);
                            self.save_config();
                            self.recalc_available_modules();

                            self.mode = WaybarMode::Edit;
                            self.active_module = Some(name.clone());
                        }
                    }

                    self.create_custom_modal_open = false;
                    self.create_custom_name = String::new();
                }
                WaybarAction::CreateCustomCancel => {
                    self.create_custom_modal_open = false;
                    self.create_custom_name = String::new();
                }
                WaybarAction::SwitchTab(tab) => {
                    if self.current_tab == tab {
                        return Task::none();
                    }

                    if self.current_tab == EditorTab::Json {
                        let text = self.json_content.text();
                        match serde_json::from_str::<Value>(&text) {
                            Ok(val) => {
                                let mut success = false;
                                if let Some(m) = &self.active_module {
                                    if let Some(root_node) = &mut self.ast_root {
                                        let is_array = matches!(root_node, Node::List(_));

                                        if m == "general" {
                                            let path_prefix =
                                                if is_array { vec!["0"] } else { vec![] };

                                            if let Some(obj) = val.as_object() {
                                                let existing_keys = if let Some(old_obj) =
                                                    match &self.config_cache {
                                                        Value::Array(arr) => {
                                                            arr.get(0).unwrap_or(&Value::Null)
                                                        }
                                                        o => o,
                                                    }
                                                    .as_object()
                                                {
                                                    old_obj.keys().cloned().collect::<Vec<_>>()
                                                } else {
                                                    Vec::new()
                                                };

                                                for k in existing_keys {
                                                    if !obj.contains_key(&k) {
                                                        let mut path_vec = path_prefix.clone();
                                                        path_vec.push(&k);
                                                        let _ = parser::remove_key(
                                                            root_node, &path_vec,
                                                        );
                                                    }
                                                }

                                                for (k, v) in obj {
                                                    let path = if is_array {
                                                        vec!["0", k.as_str()]
                                                    } else {
                                                        vec![k.as_str()]
                                                    };
                                                    parser::set_value(root_node, &path, v.clone());
                                                }
                                                success = true;
                                            }
                                        } else {
                                            let path = if is_array {
                                                vec!["0", m.as_str()]
                                            } else {
                                                vec![m.as_str()]
                                            };
                                            parser::set_value(root_node, &path, val);
                                            success = true;
                                        }
                                    }
                                }

                                if success {
                                    self.save_config();
                                }
                            }
                            Err(e) => {
                                self.json_error_message = format!("Invalid JSON: {}", e);
                                self.json_error_modal_open = true;
                                return Task::none();
                            }
                        }
                    } else if self.current_tab == EditorTab::Style {
                        self.style_cache = self.style_content.text();
                        let _ = std::fs::write(&self.style_path, &self.style_cache);
                    }

                    self.current_tab = tab;

                    match self.current_tab {
                        EditorTab::Json => {
                            if let Some(m) = &self.active_module {
                                let root = match &self.config_cache {
                                    Value::Array(arr) => arr.get(0).unwrap_or(&Value::Null),
                                    o => o,
                                };

                                if m == "general" {
                                    let general_keys = [
                                        "layer",
                                        "position",
                                        "height",
                                        "width",
                                        "spacing",
                                        "margin-top",
                                        "margin-bottom",
                                        "margin-left",
                                        "margin-right",
                                        "name",
                                        "mode",
                                        "include",
                                        "reload_style_on_change",
                                        "gtk-layer-shell",
                                    ];
                                    let mut map = serde_json::Map::new();
                                    if let Some(obj) = root.as_object() {
                                        for k in general_keys {
                                            if let Some(v) = obj.get(k) {
                                                map.insert(k.to_string(), v.clone());
                                            }
                                        }

                                        for (k, v) in obj {
                                            if !general_keys.contains(&k.as_str()) {
                                                if ![
                                                    "modules-left",
                                                    "modules-center",
                                                    "modules-right",
                                                ]
                                                .contains(&k.as_str())
                                                {
                                                    map.insert(k.to_string(), v.clone());
                                                }
                                            }
                                        }
                                    }
                                    self.json_content =
                                        iced::widget::text_editor::Content::with_text(
                                            &serde_json::to_string_pretty(&Value::Object(map))
                                                .unwrap_or_default(),
                                        );
                                } else {
                                    if let Some(val) = root.get(m) {
                                        self.json_content =
                                            iced::widget::text_editor::Content::with_text(
                                                &serde_json::to_string_pretty(val)
                                                    .unwrap_or_default(),
                                            );
                                    } else {
                                        self.json_content =
                                            iced::widget::text_editor::Content::with_text("{}");
                                    }
                                }
                            }
                        }
                        EditorTab::Style => {
                            self.style_content =
                                iced::widget::text_editor::Content::with_text(&self.style_cache);
                            if let Some(m) = &self.active_module {
                                let selector = self.get_css_selector(m);
                                let parser = css_parser::CssParser::new(&self.style_cache);
                                self.style_bg_color = parser
                                    .get_property(&selector, "background-color")
                                    .or_else(|| parser.get_property(&selector, "background"))
                                    .unwrap_or_default();
                                self.style_text_color =
                                    parser.get_property(&selector, "color").unwrap_or_default();
                                self.style_font_size = parser
                                    .get_property(&selector, "font-size")
                                    .unwrap_or_default();
                                self.style_padding = parser
                                    .get_property(&selector, "padding")
                                    .unwrap_or_default();
                            }
                        }
                        EditorTab::Settings => {}
                    }
                }
                WaybarAction::UpdateJson(action) => {
                    self.json_content.perform(action);
                }
                WaybarAction::UpdateStyle(action) => {
                    self.style_content.perform(action);
                }
                WaybarAction::ShowToast(msg, type_) => {
                    self.toasts
                        .push(crate::view::components::toast::Toast::new(msg, type_));
                }
                WaybarAction::JsonErrorModalClose => {
                    self.json_error_modal_open = false;
                    self.json_error_message = String::new();
                }
                WaybarAction::JsonErrorModalConfirm => {
                    if let Some(_content) = self.pending_json_save.take() {}
                    self.json_error_modal_open = false;
                }
                WaybarAction::DebugRun(cmd) => {
                    self.debug_output = format!("Running: {}\n", cmd);
                    let id = self.id;
                    return Task::perform(
                        async move {
                            match std::process::Command::new("sh")
                                .arg("-c")
                                .arg(&cmd)
                                .output()
                            {
                                Ok(output) => {
                                    let stdout = String::from_utf8_lossy(&output.stdout);
                                    let stderr = String::from_utf8_lossy(&output.stderr);
                                    format!(
                                        "--- STDOUT ---\n{}\n--- STDERR ---\n{}",
                                        stdout, stderr
                                    )
                                }
                                Err(e) => format!("Error: {}", e),
                            }
                        },
                        move |out| {
                            AppMessage::PluginMessage(
                                id,
                                PluginMsg::Waybar(WaybarAction::DebugOutput(out)),
                            )
                        },
                    );
                }
                WaybarAction::DebugOutput(out) => {
                    self.debug_output.push_str(&out);
                }
                WaybarAction::DebugStop => {}
                WaybarAction::CustomOptionInputKey(val) => {
                    self.custom_option_key_input = val;
                }
                WaybarAction::CustomOptionInputValue(val) => {
                    self.custom_option_val_input = val;
                }
                WaybarAction::CustomOptionAdd => {
                    let key = self.custom_option_key_input.trim().to_string();
                    let val_str = self.custom_option_val_input.clone();

                    if !key.is_empty() {
                        let exists = if let Some(m) = &self.active_module {
                            let root = match &self.config_cache {
                                Value::Array(arr) => arr.get(0).unwrap_or(&Value::Null),
                                o => o,
                            };
                            if let Some(mod_val) = root.get(m) {
                                mod_val.get(&key).is_some()
                            } else if m == "general" {
                                root.get(&key).is_some()
                            } else {
                                false
                            }
                        } else {
                            false
                        };

                        if !exists {
                            let val_to_set = if val_str == "true" {
                                serde_json::Value::Bool(true)
                            } else if val_str == "false" {
                                serde_json::Value::Bool(false)
                            } else if let Ok(i) = val_str.parse::<i64>() {
                                serde_json::Value::Number(i.into())
                            } else if let Ok(f) = val_str.parse::<f64>() {
                                if let Some(n) = serde_json::Number::from_f64(f) {
                                    serde_json::Value::Number(n)
                                } else {
                                    Value::String(val_str)
                                }
                            } else {
                                Value::String(val_str)
                            };

                            if let Some(mod_name) = &self.active_module {
                                let mod_name = mod_name.clone();
                                if let Some(root) = &mut self.ast_root {
                                    let is_array = matches!(root, Node::List(_));

                                    let path_vec = if mod_name == "general" {
                                        if is_array {
                                            vec!["0".to_string(), key.clone()]
                                        } else {
                                            vec![key.clone()]
                                        }
                                    } else {
                                        if let Node::List(bars) = root {
                                            let mut bar_idx = 0;
                                            for (i, (bar_node, _)) in
                                                bars.children.iter().enumerate()
                                            {
                                                if let Node::Dict(dict) = bar_node {
                                                    if dict
                                                        .children
                                                        .iter()
                                                        .any(|(k, _, _)| k.value == mod_name)
                                                    {
                                                        bar_idx = i;
                                                        break;
                                                    }
                                                }
                                            }
                                            vec![bar_idx.to_string(), mod_name, key.clone()]
                                        } else {
                                            vec![mod_name, key.clone()]
                                        }
                                    };

                                    let path_refs: Vec<&str> =
                                        path_vec.iter().map(|s| s.as_str()).collect();
                                    parser::set_value(root, &path_refs, val_to_set);
                                    self.save_config();

                                    self.custom_option_key_input.clear();
                                    self.custom_option_val_input.clear();
                                }
                            }
                        }
                    }
                }
                WaybarAction::CustomOptionDeleteInit(key) => {
                    self.delete_option_target = Some(key);
                    self.delete_option_modal_open = true;
                }
                WaybarAction::CustomOptionDeleteConfirm => {
                    if let Some(target_key) = &self.delete_option_target {
                        if let Some(mod_name) = &self.active_module {
                            let mod_name = mod_name.clone();
                            if let Some(root) = &mut self.ast_root {
                                let is_array = matches!(root, Node::List(_));

                                let path_vec = if mod_name == "general" {
                                    if is_array {
                                        vec!["0".to_string(), target_key.clone()]
                                    } else {
                                        vec![target_key.clone()]
                                    }
                                } else {
                                    if let Node::List(bars) = root {
                                        let mut bar_idx = 0;
                                        for (i, (bar_node, _)) in bars.children.iter().enumerate() {
                                            if let Node::Dict(dict) = bar_node {
                                                if dict
                                                    .children
                                                    .iter()
                                                    .any(|(k, _, _)| k.value == mod_name)
                                                {
                                                    bar_idx = i;
                                                    break;
                                                }
                                            }
                                        }
                                        vec![bar_idx.to_string(), mod_name, target_key.clone()]
                                    } else {
                                        vec![mod_name, target_key.clone()]
                                    }
                                };

                                let path_refs: Vec<&str> =
                                    path_vec.iter().map(|s| s.as_str()).collect();
                                let _ = parser::remove_key(root, &path_refs);
                                self.save_config();
                            }
                        }
                    }
                    self.delete_option_modal_open = false;
                    self.delete_option_target = None;
                }
                WaybarAction::CustomOptionDeleteCancel => {
                    self.delete_option_modal_open = false;
                    self.delete_option_target = None;
                }
            },
            PluginMsg::JumpTo(res) => {
                self.active_module = Some(res.id);
                self.inputs.clear();
            }
            PluginMsg::LoadPreset(name) => {
                if let Ok(files) = self.preset_manager.load(&name) {
                    let mut changed = false;
                    if let Some(config) = files.get("config.jsonc").or_else(|| files.get("config"))
                    {
                        if std::fs::write(&self.config_path, config).is_ok() {
                            if let Ok(root) = parser::parse(config) {
                                self.ast_root = Some(root.clone());
                                self.config_cache = parser::to_json_value(&root);
                                changed = true;
                            }
                        }
                    }
                    if let Some(style) = files.get("style.css").or_else(|| files.get("style")) {
                        if std::fs::write(&self.style_path, style).is_ok() {
                            self.style_cache = style.clone();
                            changed = true;
                        }
                    }

                    if changed {
                        self.active_preset = Some(name.clone());
                        let _ = self.preset_manager.set_active(Some(&name));
                    }
                }
            }
            PluginMsg::Select(list, idx) => {
                self.selected_item = Some((list, idx));
            }
            PluginMsg::KeyPress(key, _mods) => {
                println!("[Waybar] KeyPress: {:?}", key);
                if let Some((list, idx)) = &self.selected_item {
                    let list = list.clone();
                    let idx = *idx;

                    match key {
                        iced::keyboard::Key::Named(iced::keyboard::key::Named::ArrowUp) => {
                            if idx > 0 {
                                if list != "available" {
                                    if let Some(item) = self.get_list_items(&list).get(idx).cloned()
                                    {
                                        let _ =
                                            self.update(PluginMsg::Waybar(WaybarAction::Reorder {
                                                item,
                                                direction: ReorderDirection::Up,
                                            }));
                                    }
                                }
                                self.selected_item = Some((list, idx - 1));
                            }
                        }

                        iced::keyboard::Key::Named(iced::keyboard::key::Named::ArrowDown) => {
                            let len = self.get_list_items(&list).len();
                            if idx < len - 1 {
                                if list != "available" {
                                    if let Some(item) = self.get_list_items(&list).get(idx).cloned()
                                    {
                                        let _ =
                                            self.update(PluginMsg::Waybar(WaybarAction::Reorder {
                                                item,
                                                direction: ReorderDirection::Down,
                                            }));
                                    }
                                }
                                self.selected_item = Some((list, idx + 1));
                            }
                        }

                        iced::keyboard::Key::Named(iced::keyboard::key::Named::ArrowLeft) => {
                            let target_list = match list.as_str() {
                                "modules-right" => "modules-center",
                                "modules-center" => "modules-left",
                                "modules-left" => "modules-right",
                                "available" => "modules-right",
                                _ => "modules-left",
                            };
                            match self.get_list_items(&list).get(idx).cloned() {
                                Some(item) => {
                                    if list != "available" && target_list != "available" {
                                        let target_len = self.get_list_items(target_list).len();
                                        let _ =
                                            self.update(PluginMsg::Waybar(WaybarAction::Move {
                                                item,
                                                target_list: target_list.into(),
                                            }));
                                        self.selected_item =
                                            Some((target_list.to_string(), target_len));
                                    } else {
                                        let target_len = self.get_list_items(target_list).len();
                                        let new_idx = if target_len > 0 {
                                            idx.min(target_len - 1)
                                        } else {
                                            0
                                        };
                                        self.selected_item =
                                            Some((target_list.to_string(), new_idx));
                                    }
                                }
                                None => {}
                            }
                        }

                        iced::keyboard::Key::Named(iced::keyboard::key::Named::ArrowRight) => {
                            let target_list = match list.as_str() {
                                "modules-left" => "modules-center",
                                "modules-center" => "modules-right",
                                "modules-right" => "available",
                                "available" => "modules-left",
                                _ => "modules-right",
                            };
                            match self.get_list_items(&list).get(idx).cloned() {
                                Some(item) => {
                                    if list != "available" && target_list != "available" {
                                        let target_len = self.get_list_items(target_list).len();
                                        let _ =
                                            self.update(PluginMsg::Waybar(WaybarAction::Move {
                                                item,
                                                target_list: target_list.into(),
                                            }));
                                        self.selected_item =
                                            Some((target_list.to_string(), target_len));
                                    } else {
                                        let target_len = self.get_list_items(target_list).len();
                                        let new_idx = if target_len > 0 {
                                            idx.min(target_len - 1)
                                        } else {
                                            0
                                        };
                                        self.selected_item =
                                            Some((target_list.to_string(), new_idx));
                                    }
                                }
                                None => {}
                            }
                        }

                        iced::keyboard::Key::Character(c)
                            if c.as_str() == "x" || c.as_str() == "X" =>
                        {
                            if let Some(item) = self.get_list_items(&list).get(idx).cloned() {
                                if list == "available" {
                                    let _ = self.update(PluginMsg::Waybar(WaybarAction::Add {
                                        item: item.clone(),
                                        target_list: "modules-right".into(),
                                    }));
                                } else {
                                    let _ = self
                                        .update(PluginMsg::Waybar(WaybarAction::Remove { item }));
                                    self.selected_item = None;
                                }
                            }
                        }

                        iced::keyboard::Key::Named(iced::keyboard::key::Named::Delete)
                        | iced::keyboard::Key::Named(iced::keyboard::key::Named::Backspace) => {
                            if list != "available" {
                                if let Some(item) = self.get_list_items(&list).get(idx).cloned() {
                                    let _ = self
                                        .update(PluginMsg::Waybar(WaybarAction::Remove { item }));
                                    self.selected_item = None;
                                }
                            }
                        }

                        iced::keyboard::Key::Named(iced::keyboard::key::Named::Tab) => {
                            let lists = [
                                "modules-left",
                                "modules-center",
                                "modules-right",
                                "available",
                            ];

                            let mut all_items = Vec::new();
                            for l in &lists {
                                for (i, _) in self.get_list_items(l).iter().enumerate() {
                                    all_items.push((l.to_string(), i));
                                }
                            }

                            if let Some(pos) =
                                all_items.iter().position(|(l, i)| l == &list && i == &idx)
                            {
                                let next = (pos + 1) % all_items.len();
                                if let Some((nl, ni)) = all_items.get(next) {
                                    self.selected_item = Some((nl.clone(), *ni));
                                }
                            }
                        }
                        _ => {}
                    }
                } else {
                    if key == iced::keyboard::Key::Named(iced::keyboard::key::Named::Tab) {
                        let lists = [
                            "modules-left",
                            "modules-center",
                            "modules-right",
                            "available",
                        ];
                        for l in &lists {
                            if !self.get_list_items(l).is_empty() {
                                self.selected_item = Some((l.to_string(), 0));
                                break;
                            }
                        }
                    }
                }
            }
            _ => {}
        }
        Task::none()
    }

    fn view<'a>(&'a self, theme: &'a AppTheme) -> Element<'a, AppMessage> {
        let palette = theme.palette();

        let mode_btn = |label: &str, active: bool, msg: PluginMsg| -> Element<'a, AppMessage> {
            if active {
                btn::primary(
                    text(label.to_string()),
                    AppMessage::PluginMessage(self.id, msg),
                )
            } else {
                btn::ghost(
                    text(label.to_string()),
                    AppMessage::PluginMessage(self.id, msg),
                )
            }
        };

        // Waybar Preview

        let preview_bar_better = container(
            row![
                row(self
                    .get_list_items("modules-left")
                    .into_iter()
                    .map(|m| { self.render_module_preview(&m, theme) }))
                .spacing(4),
                iced::widget::Space::new().width(Length::Fill),
                row(self
                    .get_list_items("modules-center")
                    .into_iter()
                    .map(|m| { self.render_module_preview(&m, theme) }))
                .spacing(4),
                iced::widget::Space::new().width(Length::Fill),
                row(self
                    .get_list_items("modules-right")
                    .into_iter()
                    .map(|m| { self.render_module_preview(&m, theme) }))
                .spacing(4)
            ]
            .width(Length::Fill),
        )
        .padding(10)
        .style(move |_: &_| container::Style {
            background: Some(iced::Background::Color(palette.mantle)),
            border: iced::Border {
                width: 0.0,
                radius: 0.0.into(),
                color: iced::Color::TRANSPARENT,
            },
            ..Default::default()
        });

        // Mode Switcher
        let mode_bar = row![
            mode_btn(
                "Layout (Arrow Keys)",
                matches!(self.mode, WaybarMode::Layout),
                PluginMsg::SwitchInternalTab("layout".into())
            ),
            mode_btn(
                "Editor",
                matches!(self.mode, WaybarMode::Edit),
                PluginMsg::SwitchInternalTab("settings".into())
            ),
            mode_btn(
                "Presets",
                matches!(self.mode, WaybarMode::Presets),
                PluginMsg::SwitchInternalTab("presets".into())
            ),
        ]
        .spacing(10)
        .padding(10);

        // Wrap mode_bar and preview
        let top_section = column![preview_bar_better, mode_bar].spacing(0);

        let main_content: Element<'_, AppMessage> = match &self.mode {
            WaybarMode::Layout => {
                let lists = vec!["modules-left", "modules-center", "modules-right"];
                let cols = row(lists.into_iter().map(|lname| {
                    let items = self.get_list_items(lname);

                    let header = text(lname)
                        .size(16)
                        .style(move |_: &iced::Theme| text::Style {
                            color: Some(palette.subtext1),
                        });

                    let item_list = column(items.iter().enumerate().map(|(idx, item)| {
                        let is_selected = self
                            .selected_item
                            .as_ref()
                            .map(|(l, i)| l == lname && *i == idx)
                            .unwrap_or(false);

                        let content = container(text(item.clone()).size(14))
                            .padding([8, 12])
                            .width(Length::Fill)
                            .style(move |_: &iced::Theme| container::Style {
                                background: Some(iced::Background::Color(if is_selected {
                                    palette.surface2
                                } else {
                                    palette.surface1
                                })),
                                border: iced::Border {
                                    radius: 6.0.into(),
                                    color: if is_selected {
                                        palette.mauve
                                    } else {
                                        palette.surface2
                                    },
                                    width: if is_selected { 2.0 } else { 1.0 },
                                },
                                text_color: Some(palette.text),
                                ..Default::default()
                            });

                        button(content)
                            .on_press(AppMessage::PluginMessage(
                                self.id,
                                PluginMsg::Select(lname.to_string(), idx),
                            ))
                            .width(Length::Fill)
                            .style(|_, _| button::Style::default())
                            .padding(0)
                            .into()
                    }))
                    .spacing(8);

                    let zone = container(
                        column![header, scrollable(item_list).height(Length::Fill)].spacing(10),
                    )
                    .padding(10)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .style(move |_: &iced::Theme| container::Style {
                        background: Some(iced::Background::Color(palette.base)),
                        border: iced::Border {
                            width: 2.0,
                            color: palette.surface0,
                            radius: 12.0.into(),
                        },
                        ..Default::default()
                    });

                    zone.into()
                }))
                .spacing(20)
                .width(Length::Fill);

                let avail = &self.available_modules_cache;
                let avail_col = column![
                    text("Available Modules")
                        .size(16)
                        .style(move |_: &_| text::Style {
                            color: Some(palette.blue)
                        }),
                    ti::input("Search...", &self.search_query, move |v| {
                        AppMessage::PluginMessage(
                            1,
                            PluginMsg::UpdateConfig("internal:search".into(), v),
                        )
                    }),
                    scrollable(
                        column(avail.iter().enumerate().map(|(idx, m)| {
                            let m_clone = m.clone();
                            let is_selected = self
                                .selected_item
                                .as_ref()
                                .map(|(l, i)| l == "available" && *i == idx)
                                .unwrap_or(false);

                            let text_btn = button(text(m.clone()).size(13).width(Length::Fill))
                                .on_press(AppMessage::PluginMessage(
                                    self.id,
                                    PluginMsg::Select("available".into(), idx),
                                ))
                                .style(move |_, _| button::Style {
                                    background: Some(iced::Background::Color(if is_selected {
                                        palette.surface2
                                    } else {
                                        iced::Color::TRANSPARENT
                                    })),
                                    text_color: palette.text,
                                    ..Default::default()
                                })
                                .padding([6, 8])
                                .width(Length::Fill);

                            row![
                                text_btn,
                                btn::small_secondary(
                                    text("+"),
                                    AppMessage::PluginMessage(
                                        self.id,
                                        PluginMsg::Waybar(WaybarAction::Add {
                                            item: m_clone.clone(),
                                            target_list: "modules-right".into()
                                        })
                                    )
                                )
                            ]
                            .spacing(5)
                            .into()
                        }))
                        .spacing(5)
                    ),
                    container(btn::primary(
                        "Create Custom Module",
                        AppMessage::PluginMessage(
                            self.id,
                            PluginMsg::Waybar(WaybarAction::CreateCustomInit)
                        )
                    ))
                    .width(Length::Fill)
                ]
                .spacing(10)
                .width(Length::Fixed(250.0))
                .padding(10);

                row![cols, avail_col].spacing(20).into()
            }
            WaybarMode::Edit => {
                let modules = self.get_modules();
                let display_modules = modules.clone();
                let query = self.search_query.to_lowercase();

                let sidebar_list = column(
                    display_modules
                        .into_iter()
                        .filter(|m| query.is_empty() || m.to_lowercase().contains(&query))
                        .map(|m| {
                            let is_active = self.active_module.as_ref() == Some(&m);
                            let msg = AppMessage::PluginMessage(
                                self.id,
                                PluginMsg::SwitchInternalTab(m.clone()),
                            );
                            if is_active {
                                btn::secondary(text(m), msg)
                            } else {
                                btn::ghost(text(m), msg)
                            }
                        }),
                )
                .spacing(5);

                let sidebar = column![
                    ti::input("Search...", &self.search_query, move |v| {
                        AppMessage::PluginMessage(
                            self.id,
                            PluginMsg::UpdateConfig("internal:search".to_string(), v),
                        )
                    }),
                    scrollable(sidebar_list)
                ]
                .spacing(10);

                let content: Element<'a, AppMessage> = if let Some(m) = &self.active_module {
                    let is_custom = m.starts_with("custom/");

                    let tab_btn = |label: &str, tab: EditorTab, id: usize, current: EditorTab| {
                        let active = current == tab;
                        if active {
                            btn::primary(
                                text(label.to_string()),
                                AppMessage::PluginMessage(
                                    id,
                                    PluginMsg::Waybar(WaybarAction::SwitchTab(tab)),
                                ),
                            )
                        } else {
                            btn::secondary(
                                text(label.to_string()),
                                AppMessage::PluginMessage(
                                    id,
                                    PluginMsg::Waybar(WaybarAction::SwitchTab(tab)),
                                ),
                            )
                        }
                    };

                    let tab_bar = row![
                        text(format!("Editing: {}", m)).size(16).width(Length::Fill),
                        row![
                            tab_btn("Settings", EditorTab::Settings, self.id, self.current_tab),
                            tab_btn("JSON", EditorTab::Json, self.id, self.current_tab),
                            tab_btn("Style", EditorTab::Style, self.id, self.current_tab),
                        ]
                        .spacing(5)
                    ]
                    .spacing(10)
                    .align_y(iced::Alignment::Center);

                    let main_edit_area: Element<AppMessage> = match self.current_tab {
                        EditorTab::Json => column![
                            text("Raw JSON Configuration").size(14).style(
                                move |_: &iced::Theme| text::Style {
                                    color: Some(palette.subtext1)
                                }
                            ),
                            iced::widget::text_editor(&self.json_content)
                                .on_action(|action| AppMessage::PluginMessage(
                                    self.id,
                                    PluginMsg::Waybar(WaybarAction::UpdateJson(action))
                                ))
                                .height(Length::Fill)
                        ]
                        .spacing(10)
                        .height(Length::Fill)
                        .into(),
                        EditorTab::Style => {
                            let selector = self.get_css_selector(m);

                            let color_preview_btn = |label: &str, val: &str, target: String| {
                                let color = color_picker::parse_color(val)
                                    .unwrap_or(iced::Color::TRANSPARENT);

                                let preview = container(text("  "))
                                    .width(Length::Fixed(30.0))
                                    .height(Length::Fixed(30.0))
                                    .style(move |_: &iced::Theme| container::Style {
                                        background: Some(iced::Background::Color(color)),
                                        border: iced::Border {
                                            color: palette.subtext0,
                                            width: 1.0,
                                            radius: 4.0.into(),
                                        },
                                        ..Default::default()
                                    });

                                column![
                                    text(label.to_string()).size(12).style(
                                        move |_: &iced::Theme| text::Style {
                                            color: Some(palette.subtext1)
                                        }
                                    ),
                                    button(preview)
                                        .on_press(AppMessage::PluginMessage(
                                            self.id,
                                            PluginMsg::Waybar(WaybarAction::ColorPick { target })
                                        ))
                                        .padding(0)
                                        .style(|_, _| button::Style::default())
                                ]
                                .spacing(5)
                            };

                            let sel_font = selector.clone();
                            let sel_pad = selector.clone();

                            let style_controls = row![
                                color_preview_btn(
                                    "Background",
                                    &self.style_bg_color,
                                    format!("style:{}:background-color", selector)
                                ),
                                color_preview_btn(
                                    "Text Color",
                                    &self.style_text_color,
                                    format!("style:{}:color", selector)
                                ),
                                column![
                                    text("Font Size").size(12).style(move |_: &iced::Theme| {
                                        text::Style {
                                            color: Some(palette.subtext1),
                                        }
                                    }),
                                    ti::input("e.g. 12px", &self.style_font_size, move |v| {
                                        AppMessage::PluginMessage(
                                            self.id,
                                            PluginMsg::UpdateConfig(
                                                format!("style:{}:font-size", sel_font),
                                                v,
                                            ),
                                        )
                                    })
                                    .width(Length::Fixed(100.0))
                                ]
                                .spacing(5),
                                column![
                                    text("Padding").size(12).style(move |_: &iced::Theme| {
                                        text::Style {
                                            color: Some(palette.subtext1),
                                        }
                                    }),
                                    ti::input("e.g. 5px", &self.style_padding, move |v| {
                                        AppMessage::PluginMessage(
                                            self.id,
                                            PluginMsg::UpdateConfig(
                                                format!("style:{}:padding", sel_pad),
                                                v,
                                            ),
                                        )
                                    })
                                    .width(Length::Fixed(100.0))
                                ]
                                .spacing(5),
                            ]
                            .spacing(20)
                            .align_y(iced::Alignment::Center);

                            column![
                                style_controls,
                                text("Global CSS Style")
                                    .size(14)
                                    .style(move |_: &iced::Theme| text::Style {
                                        color: Some(palette.subtext1)
                                    }),
                                iced::widget::text_editor(&self.style_content)
                                    .on_action(|action| AppMessage::PluginMessage(
                                        self.id,
                                        PluginMsg::Waybar(WaybarAction::UpdateStyle(action))
                                    ))
                                    .height(Length::Fill)
                            ]
                            .spacing(10)
                            .height(Length::Fill)
                            .into()
                        }
                        EditorTab::Settings => {
                            let schema_opt = self.get_schema_for_module(m);

                            let root = match &self.config_cache {
                                Value::Array(arr) => arr.get(0).unwrap_or(&Value::Null),
                                o => o,
                            };

                            // 1. Build Map of OptionDefs (Key -> Def)
                            let mut option_defs: HashMap<String, OptionDef> = HashMap::new();
                            if let Some(schema) = &schema_opt {
                                for o in &schema.options {
                                    option_defs.insert(
                                        o.name.clone(),
                                        OptionDef::new(
                                            &o.name,
                                            match o.option_type {
                                                OptionType::Bool => {
                                                    schema_renderer::OptionType::Bool
                                                }
                                                OptionType::Int => schema_renderer::OptionType::Int,
                                                OptionType::Float => {
                                                    schema_renderer::OptionType::String
                                                }
                                                OptionType::String => {
                                                    schema_renderer::OptionType::String
                                                }
                                                OptionType::Color => {
                                                    schema_renderer::OptionType::Color
                                                }
                                                OptionType::Enum => {
                                                    schema_renderer::OptionType::String
                                                }
                                                OptionType::Json => {
                                                    schema_renderer::OptionType::String
                                                }
                                            },
                                            &o.default,
                                            &o.description,
                                        )
                                        .with_choices(o.choices.clone().unwrap_or_default()),
                                    );
                                }
                            }

                            // 2. Identify all keys from config + schema
                            let mut all_keys: Vec<String> = option_defs.keys().cloned().collect();

                            if let Some(mod_val) = root.get(m) {
                                if let Some(obj) = mod_val.as_object() {
                                    for k in obj.keys() {
                                        if !option_defs.contains_key(k) {
                                            all_keys.push(k.clone());
                                            // Add Generic Def for this custom key
                                            option_defs.insert(
                                                k.clone(),
                                                OptionDef::new(
                                                    k,
                                                    schema_renderer::OptionType::String,
                                                    "",
                                                    "Custom Option",
                                                ),
                                            );
                                        }
                                    }
                                }
                            }
                            // Special handling for general module
                            if m == "general" {
                                if let Some(obj) = root.as_object() {
                                    for (k, v) in obj {
                                        if !v.is_object()
                                            && !v.is_array()
                                            && !option_defs.contains_key(k)
                                        {
                                            all_keys.push(k.clone());
                                            option_defs.insert(
                                                k.clone(),
                                                OptionDef::new(
                                                    k,
                                                    schema_renderer::OptionType::String,
                                                    "",
                                                    "General Option",
                                                ),
                                            );
                                        }
                                    }
                                }
                            }

                            all_keys.sort();
                            all_keys.dedup();

                            // 3. Render List
                            let list_content = column(all_keys.into_iter().map(|key| {
                                let def = option_defs.get(&key).unwrap().clone();

                                // Get Value
                                let val = if let Some(mod_val) = root.get(m) {
                                    mod_val
                                        .get(&key)
                                        .map(|v| {
                                            if v.is_string() {
                                                v.as_str().unwrap().to_string()
                                            } else {
                                                v.to_string()
                                            }
                                        })
                                        .unwrap_or_else(|| def.default.clone())
                                } else if m == "general" {
                                    root.get(&key)
                                        .map(|v| {
                                            if v.is_string() {
                                                v.as_str().unwrap().to_string()
                                            } else {
                                                v.to_string()
                                            }
                                        })
                                        .unwrap_or_else(|| def.default.clone())
                                } else {
                                    def.default.clone()
                                };

                                let element = schema_renderer::render_option(
                                    &def,
                                    &val,
                                    format!("{}:{}", m, key),
                                    self.id,
                                );

                                row![
                                    container(element).width(Length::Fill),
                                    btn::ghost(
                                        "🗑",
                                        AppMessage::PluginMessage(
                                            self.id,
                                            PluginMsg::Waybar(
                                                WaybarAction::CustomOptionDeleteInit(key.clone())
                                            )
                                        )
                                    )
                                ]
                                .spacing(10)
                                .align_y(iced::Alignment::Center)
                                .into()
                            }))
                            .spacing(5);

                            let add_section = column![
                                text("Add New Option")
                                    .size(14)
                                    .style(move |_: &iced::Theme| text::Style {
                                        color: Some(palette.subtext1)
                                    }),
                                row![
                                    ti::input("Key", &self.custom_option_key_input, move |v| {
                                        AppMessage::PluginMessage(
                                            self.id,
                                            PluginMsg::Waybar(WaybarAction::CustomOptionInputKey(
                                                v,
                                            )),
                                        )
                                    })
                                    .width(Length::FillPortion(1)),
                                    ti::input("Value", &self.custom_option_val_input, move |v| {
                                        AppMessage::PluginMessage(
                                            self.id,
                                            PluginMsg::Waybar(
                                                WaybarAction::CustomOptionInputValue(v),
                                            ),
                                        )
                                    })
                                    .width(Length::FillPortion(2)),
                                    btn::primary(
                                        "Add",
                                        AppMessage::PluginMessage(
                                            self.id,
                                            PluginMsg::Waybar(WaybarAction::CustomOptionAdd)
                                        )
                                    )
                                ]
                                .spacing(10)
                            ]
                            .spacing(10)
                            .padding(10);

                            let final_view = scrollable(
                                column![list_content, add_section].spacing(20).padding(10),
                            );

                            // Debug section if relevant
                            if is_custom {
                                // Fetch exec command safely
                                let exec_cmd = root
                                    .get(m)
                                    .and_then(|v| v.get("exec"))
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("")
                                    .to_string();

                                column![
                                    final_view,
                                    text("Debug / Test").size(16).style(move |_: &iced::Theme| {
                                        text::Style {
                                            color: Some(palette.mauve),
                                        }
                                    }),
                                    row![
                                        btn::primary(
                                            "Run Module",
                                            AppMessage::PluginMessage(
                                                self.id,
                                                PluginMsg::Waybar(WaybarAction::DebugRun(exec_cmd))
                                            )
                                        ),
                                        text("Output will appear below...").size(12)
                                    ]
                                    .spacing(10),
                                    container(scrollable(
                                        text(&self.debug_output)
                                            .font(iced::Font::MONOSPACE)
                                            .size(12)
                                    ))
                                    .padding(10)
                                    .style(move |_| container::Style {
                                        background: Some(iced::Background::Color(palette.base)),
                                        border: iced::Border {
                                            color: palette.surface2,
                                            width: 1.0,
                                            radius: 4.0.into()
                                        },
                                        ..Default::default()
                                    })
                                    .height(Length::Fixed(150.0))
                                    .width(Length::Fill)
                                ]
                                .spacing(20)
                                .into()
                            } else {
                                final_view.into()
                            }
                        }
                    };

                    column![tab_bar, main_edit_area].spacing(10).into()
                } else {
                    text("Select a module to edit settings").into()
                };

                row![scrollable(sidebar).width(Length::Fixed(200.0)), content]
                    .spacing(20)
                    .into()
            }
            WaybarMode::Presets => {
                let preset_name_val = self
                    .inputs
                    .get("preset_name")
                    .map(|s| s.as_str())
                    .unwrap_or("");
                presets_view::view(
                    &self.presets_list,
                    self.active_preset.as_ref(),
                    preset_name_val,
                )
            }
        };

        let content_stack = stack![column![top_section, main_content].spacing(10)];

        let modal_layer: Element<AppMessage> = if self.color_modal_open {
            stack![
                content_stack,
                modal::overlay(
                    color_picker::view_modal(
                        &self.color_modal_value,
                        move |s| AppMessage::PluginMessage(
                            self.id,
                            PluginMsg::Waybar(WaybarAction::ColorUpdate(s))
                        ),
                        AppMessage::PluginMessage(
                            self.id,
                            PluginMsg::Waybar(WaybarAction::ColorCancel)
                        ),
                        AppMessage::PluginMessage(
                            self.id,
                            PluginMsg::Waybar(WaybarAction::ColorApply)
                        )
                    ),
                    AppMessage::PluginMessage(
                        self.id,
                        PluginMsg::Waybar(WaybarAction::ColorCancel)
                    ),
                    true
                )
            ]
            .into()
        } else if self.delete_modal_open {
            stack![
                content_stack,
                modal::overlay(
                    container(
                        column![
                            text(format!(
                                "Delete module '{}'?",
                                self.delete_target.as_deref().unwrap_or("")
                            ))
                            .size(18)
                            .style(move |_: &_| text::Style {
                                color: Some(palette.text)
                            }),
                            text("Type the module name to confirm deletion:")
                                .size(14)
                                .style(move |_: &_| text::Style {
                                    color: Some(palette.subtext1)
                                }),
                            ti::input("Module Name", &self.delete_input, move |v| {
                                AppMessage::PluginMessage(
                                    self.id,
                                    PluginMsg::Waybar(WaybarAction::DeleteInput(v)),
                                )
                            }),
                            row![
                                btn::ghost(
                                    text("Cancel"),
                                    AppMessage::PluginMessage(
                                        self.id,
                                        PluginMsg::Waybar(WaybarAction::DeleteCancel)
                                    )
                                ),
                                if self.delete_target.as_ref() == Some(&self.delete_input) {
                                    btn::primary(
                                        text("Delete"),
                                        AppMessage::PluginMessage(
                                            self.id,
                                            PluginMsg::Waybar(WaybarAction::DeleteConfirm),
                                        ),
                                    )
                                } else {
                                    btn::ghost(
                                        text("Delete"),
                                        AppMessage::PluginMessage(
                                            self.id,
                                            PluginMsg::Waybar(WaybarAction::DeleteConfirm),
                                        ),
                                    ) // Disabled style would be better but ghost works
                                }
                            ]
                            .spacing(10)
                        ]
                        .spacing(15)
                    )
                    .padding(20)
                    .style(modal::container_style)
                    .width(Length::Fixed(400.0))
                    .into(),
                    AppMessage::PluginMessage(
                        self.id,
                        PluginMsg::Waybar(WaybarAction::DeleteCancel)
                    ),
                    true
                )
            ]
            .into()
        } else if self.delete_option_modal_open {
            stack![
                content_stack,
                modal::overlay(
                    container(
                        column![
                            text(format!(
                                "Delete option '{}'?",
                                self.delete_option_target.as_deref().unwrap_or("")
                            ))
                            .size(18)
                            .style(move |_: &_| text::Style {
                                color: Some(palette.text)
                            }),
                            text("Are you sure you want to remove this configuration option?")
                                .size(14)
                                .style(move |_: &_| text::Style {
                                    color: Some(palette.subtext1)
                                }),
                            row![
                                btn::ghost(
                                    text("Cancel"),
                                    AppMessage::PluginMessage(
                                        self.id,
                                        PluginMsg::Waybar(WaybarAction::CustomOptionDeleteCancel)
                                    )
                                ),
                                btn::primary(
                                    text("Delete"),
                                    AppMessage::PluginMessage(
                                        self.id,
                                        PluginMsg::Waybar(WaybarAction::CustomOptionDeleteConfirm),
                                    ),
                                )
                            ]
                            .spacing(10)
                        ]
                        .spacing(15)
                    )
                    .padding(20)
                    .style(modal::container_style)
                    .width(Length::Fixed(400.0))
                    .into(),
                    AppMessage::PluginMessage(
                        self.id,
                        PluginMsg::Waybar(WaybarAction::CustomOptionDeleteCancel)
                    ),
                    true
                )
            ]
            .into()
        } else if self.create_custom_modal_open {
            stack![
                content_stack,
                modal::overlay(
                    container(
                        column![
                            text("Create Custom Module")
                                .size(18)
                                .style(move |_: &_| text::Style {
                                    color: Some(palette.text)
                                }),
                            text("Enter a unique name for the new module (e.g. 'gpu', 'weather'):")
                                .size(14)
                                .style(move |_: &_| text::Style {
                                    color: Some(palette.subtext1)
                                }),
                            ti::input("Module Name", &self.create_custom_name, move |v| {
                                AppMessage::PluginMessage(
                                    self.id,
                                    PluginMsg::Waybar(WaybarAction::CreateCustomInput(v)),
                                )
                            }),
                            row![
                                btn::ghost(
                                    text("Cancel"),
                                    AppMessage::PluginMessage(
                                        self.id,
                                        PluginMsg::Waybar(WaybarAction::CreateCustomCancel)
                                    )
                                ),
                                btn::primary(
                                    text("Create"),
                                    AppMessage::PluginMessage(
                                        self.id,
                                        PluginMsg::Waybar(WaybarAction::CreateCustomConfirm(
                                            self.create_custom_name.clone()
                                        ))
                                    )
                                )
                            ]
                            .spacing(10)
                        ]
                        .spacing(15)
                    )
                    .padding(20)
                    .style(modal::container_style)
                    .width(Length::Fixed(400.0))
                    .into(),
                    AppMessage::PluginMessage(
                        self.id,
                        PluginMsg::Waybar(WaybarAction::CreateCustomCancel)
                    ),
                    true
                )
            ]
            .into()
        } else if self.json_error_modal_open {
            stack![
                content_stack,
                modal::overlay(
                    container(
                        column![
                            text("JSON Syntax Error")
                                .size(18)
                                .style(move |_: &_| text::Style {
                                    color: Some(palette.red)
                                }),
                            text(&self.json_error_message).size(14).style(move |_: &_| {
                                text::Style {
                                    color: Some(palette.text),
                                }
                            }),
                            row![btn::primary(
                                "OK",
                                AppMessage::PluginMessage(
                                    self.id,
                                    PluginMsg::Waybar(WaybarAction::JsonErrorModalClose)
                                )
                            )]
                            .spacing(10)
                        ]
                        .spacing(15)
                    )
                    .padding(20)
                    .style(modal::container_style)
                    .width(Length::Fixed(400.0))
                    .into(),
                    AppMessage::PluginMessage(
                        self.id,
                        PluginMsg::Waybar(WaybarAction::JsonErrorModalClose)
                    ),
                    true
                )
            ]
            .into()
        } else {
            content_stack.into()
        };

        let active_toasts = self
            .toasts
            .iter()
            .map(|t| crate::view::components::toast::view(t));
        let toast_list = column(active_toasts)
            .spacing(10)
            .align_x(iced::Alignment::End)
            .width(Length::Fill);

        // Align bottom-right
        let toast_overlay = container(toast_list)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(20)
            .align_y(iced::alignment::Vertical::Bottom)
            .align_x(iced::alignment::Horizontal::Right);

        stack![modal_layer, toast_overlay].into()
    }

    fn searchable_items(&self) -> Vec<crate::core::SearchResult> {
        self.get_modules()
            .into_iter()
            .map(|m| {
                let schema = self.get_schema_for_module(&m);
                let title = schema
                    .as_ref()
                    .map(|s| s.title.clone())
                    .unwrap_or_else(|| m.clone());
                let description = schema
                    .as_ref()
                    .map(|s| format!("{} settings", s.title))
                    .unwrap_or_else(|| format!("Configure {}", m));

                crate::core::SearchResult {
                    id: m,
                    title,
                    description,
                    tab_id: "waybar".to_string(),
                }
            })
            .collect()
    }
}
