#[derive(Debug, Clone)]
pub enum WaybarAction {
    PresetModalOpen,
    PresetInput(String),
    PresetSave,
    PresetLoad(String),
    PresetDelete(String),
    Reorder {
        item: String,
        direction: ReorderDirection,
    },
    Move {
        item: String,
        target_list: String,
    },
    Remove {
        item: String,
    },
    Add {
        item: String,
        target_list: String,
    },
    DeleteInit(String),
    DeleteInput(String),
    DeleteConfirm,
    DeleteCancel,
    ColorPick {
        target: String,
    },
    ColorCancel,
    ColorApply,
    ColorUpdate(String),
    CreateCustomInit,
    CreateCustomInput(String),
    CreateCustomConfirm(String),
    CreateCustomCancel,
    SwitchTab(EditorTab),
    UpdateJson(iced::widget::text_editor::Action),
    UpdateStyle(iced::widget::text_editor::Action),
    ShowToast(String, crate::view::components::toast::ToastType),
    DebugRun(String),
    DebugOutput(String),
    DebugStop,
    JsonErrorModalClose,
    JsonErrorModalConfirm,
    CustomOptionDeleteInit(String),
    CustomOptionDeleteConfirm,
    CustomOptionDeleteCancel,
    CustomOptionAdd,
    CustomOptionInputKey(String),
    CustomOptionInputValue(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorTab {
    Settings,
    Json,
    Style,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReorderDirection {
    Up,
    Down,
}
