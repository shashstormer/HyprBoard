use iced::widget::{column, scrollable, text};
use iced::{Element, Length, Theme};
use std::collections::HashMap;

use super::button as btn;
use super::dropdown as dd;
use super::setting_row::setting_row;
use super::slider as sl;
use super::text_input as ti;
use super::toggle::toggle;
use crate::core::{AppMessage, PluginMsg};

#[derive(Debug, Clone, PartialEq)]
pub enum OptionType {
    Bool,
    Int,
    Float,
    String,
    Color,
    Gradient,
    Vec2,
    Enum,
    Monitor,
    File,
}

#[derive(Debug, Clone)]
pub struct OptionDef {
    pub name: String,
    pub option_type: OptionType,
    pub default: String,
    pub description: String,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub step: Option<f64>,
    pub choices: Option<Vec<String>>,
}

impl OptionDef {
    pub fn new(name: &str, option_type: OptionType, default: &str, description: &str) -> Self {
        Self {
            name: name.to_string(),
            option_type,
            default: default.to_string(),
            description: description.to_string(),
            min: None,
            max: None,
            step: None,
            choices: None,
        }
    }

    pub fn with_range(mut self, min: f64, max: f64, step: f64) -> Self {
        self.min = Some(min);
        self.max = Some(max);
        self.step = Some(step);
        self
    }

    pub fn with_choices(mut self, choices: Vec<String>) -> Self {
        self.choices = Some(choices);
        self
    }
}

#[derive(Debug, Clone)]
pub struct Section {
    pub name: String,
    pub title: String,
    pub options: Vec<OptionDef>,
}

pub fn render_option(
    opt: &OptionDef,
    current_value: &str,
    path: String,
    plugin_id: usize,
) -> Element<'static, AppMessage> {
    let opt_name = opt.name.clone();
    let opt_default = opt.default.clone();
    let opt_desc = opt.description.clone();

    let control: Element<'static, AppMessage> = match &opt.option_type {
        OptionType::Bool => {
            let checked = current_value == "true" || current_value == "yes";
            let p = path.clone();
            toggle(
                checked,
                AppMessage::PluginMessage(
                    plugin_id,
                    PluginMsg::UpdateConfig(
                        p.clone(),
                        if checked {
                            "false".to_string()
                        } else {
                            "true".to_string()
                        },
                    ),
                ),
            )
        }
        OptionType::Int | OptionType::Float => {
            if let (Some(min), Some(max)) = (opt.min, opt.max) {
                let value = current_value.parse::<f32>().unwrap_or(min as f32);
                let p = path.clone();
                sl::range(min as f32..=max as f32, value, move |v: f32| {
                    AppMessage::PluginMessage(
                        plugin_id,
                        PluginMsg::UpdateConfig(p.clone(), v.to_string()),
                    )
                })
                .step(opt.step.unwrap_or(1.0) as f32)
                .width(Length::Fixed(150.0))
                .into()
            } else {
                let p = path.clone();
                let val = current_value.to_string();
                ti::input(&opt_default, &val, move |v| {
                    AppMessage::PluginMessage(plugin_id, PluginMsg::UpdateConfig(p.clone(), v))
                })
                .padding([6, 12])
                .width(Length::Fixed(120.0))
                .into()
            }
        }
        OptionType::Enum => {
            if let Some(choices) = &opt.choices {
                let selected = choices
                    .iter()
                    .find(|c| c.as_str() == current_value)
                    .cloned();
                let choices_clone = choices.clone();
                let p = path.clone();
                dd::dropdown_compact(choices_clone, selected, move |v: String| {
                    AppMessage::PluginMessage(plugin_id, PluginMsg::UpdateConfig(p.clone(), v))
                })
                .into()
            } else {
                text("No choices").into()
            }
        }
        OptionType::Color => {
            let p = path.clone();
            let val_display = current_value.to_string();
            btn::secondary(
                text(val_display).size(13),
                AppMessage::PluginMessage(plugin_id, PluginMsg::OpenModal(p.clone())),
            )
        }
        OptionType::File => {
            let p = path.clone();
            btn::secondary(
                text("Browse...").size(13),
                AppMessage::PluginMessage(plugin_id, PluginMsg::OpenModal(p.clone())),
            )
        }
        _ => {
            let p = path.clone();
            let val = current_value.to_string();
            ti::input(&opt_default, &val, move |v| {
                AppMessage::PluginMessage(plugin_id, PluginMsg::UpdateConfig(p.clone(), v))
            })
            .padding([6, 12])
            .width(Length::Fill)
            .into()
        }
    };

    setting_row(opt_desc, opt_name, control)
}

pub fn render_section(
    section: &Section,
    values: &HashMap<String, String>,
    path_prefix: &str,
    plugin_id: usize,
) -> Element<'static, AppMessage> {
    let title = section.title.clone();
    let header = text(title)
        .size(16)
        .font(iced::font::Font {
            weight: iced::font::Weight::Bold,
            ..Default::default()
        })
        .style(|theme: &Theme| {
            let palette = crate::view::components::theme::get_palette(theme);
            text::Style {
                color: Some(palette.blue),
            }
        });

    let options: Vec<Element<'static, AppMessage>> = section
        .options
        .iter()
        .map(|opt| {
            let full_path = format!("{}.{}", path_prefix, opt.name);
            let val = values
                .get(&full_path)
                .cloned()
                .unwrap_or(opt.default.clone());
            render_option(opt, &val, full_path, plugin_id)
        })
        .collect();

    column![header, column(options).spacing(4)]
        .spacing(8)
        .into()
}

pub fn render_sections(
    sections: &[Section],
    values: &HashMap<String, String>,
    path_prefix: &str,
    plugin_id: usize,
) -> Element<'static, AppMessage> {
    let section_elements: Vec<Element<'static, AppMessage>> = sections
        .iter()
        .map(|section| {
            let section_path = format!("{}.{}", path_prefix, section.name);
            render_section(section, values, &section_path, plugin_id)
        })
        .collect();

    scrollable(column(section_elements).spacing(16).padding(16)).into()
}
