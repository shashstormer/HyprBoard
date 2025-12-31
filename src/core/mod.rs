use iced::{Element, Task};
pub mod presets;
pub mod waybar_action;

#[derive(Debug, Clone)]
pub enum AppMessage {
    SwitchTab(usize),
    PluginMessage(usize, PluginMsg),
    Search(String),
    JumpToResult(usize, SearchResult),
    ToggleSearch,
    CloseSearch,
    GlobalKeyPress(iced::keyboard::Key, iced::keyboard::Modifiers),
    ApplyBundle(presets::Bundle),
    SetTheme(AppTheme),
    None,
}

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub id: String,
    pub title: String,
    pub description: String,
    pub tab_id: String,
}

#[derive(Debug, Clone)]
pub enum PluginMsg {
    InputChanged(String),
    Toggle(bool),
    Action,
    SwitchInternalTab(String),
    OpenModal(String),
    CloseModal,
    Edit(String, String, String),
    UpdateConfig(String, String),
    Save,
    JumpTo(SearchResult),
    ClearHighlight,
    LoadPreset(String),
    Waybar(waybar_action::WaybarAction),

    Select(String, usize),
    KeyPress(iced::keyboard::Key, iced::keyboard::Modifiers),
    None,
}

use crate::view::components::theme::AppTheme;

pub trait Plugin {
    fn name(&self) -> String;

    fn icon(&self) -> char {
        '?'
    }

    fn update(&mut self, message: PluginMsg) -> Task<AppMessage>;

    fn view<'a>(&'a self, theme: &'a AppTheme) -> Element<'a, AppMessage>;

    fn searchable_items(&self) -> Vec<SearchResult> {
        Vec::new()
    }

    fn subscription(&self) -> iced::Subscription<AppMessage> {
        iced::Subscription::none()
    }
}
