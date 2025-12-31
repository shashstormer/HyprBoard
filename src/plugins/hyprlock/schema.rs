#[derive(Debug, Clone, PartialEq)]
pub enum OptionType {
    Bool,
    Int,
    Float,
    String,
    Color,
    Gradient,
    Vec2,
    Monitor,
    File,
}

#[derive(Debug, Clone)]
pub struct HyprlockOption {
    pub name: String,
    pub option_type: OptionType,
    pub default: String,
    pub description: String,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub step: Option<f64>,
    pub choices: Option<Vec<String>>,
}

#[derive(Debug, Clone)]
pub struct HyprlockSection {
    pub name: String,
    pub title: String,
    pub icon: char,
    pub options: Vec<HyprlockOption>,
    pub is_list: bool,
}

pub fn get_schema() -> Vec<HyprlockSection> {
    vec![
        HyprlockSection {
            name: "general".to_string(),
            title: "General".to_string(),
            icon: '⚙',
            is_list: false,
            options: vec![
                opt("no_fade_in", OptionType::Bool, "false", "Disable fade in"),
                opt("no_fade_out", OptionType::Bool, "false", "Disable fade out"),
                opt(
                    "hide_cursor",
                    OptionType::Bool,
                    "false",
                    "Hide cursor when locked",
                ),
                opt(
                    "grace",
                    OptionType::Int,
                    "0",
                    "seconds to wait before locking",
                ),
                opt(
                    "disable_loading_bar",
                    OptionType::Bool,
                    "false",
                    "Disable loading bar",
                ),
                opt(
                    "ignore_empty_input",
                    OptionType::Bool,
                    "false",
                    "Ignore empty input",
                ),
                opt(
                    "immediate_render",
                    OptionType::Bool,
                    "false",
                    "Render immediately",
                ),
                opt("text_trim", OptionType::Bool, "true", "Trim text"),
                opt(
                    "fractional_scaling",
                    OptionType::Int,
                    "2",
                    "Fractional scaling (0, 1, 2)",
                ),
            ],
        },
        HyprlockSection {
            name: "background".to_string(),
            title: "Backgrounds".to_string(),
            icon: '🖼',
            is_list: true,
            options: vec![
                opt("monitor", OptionType::Monitor, "", "Monitor to apply to"),
                opt(
                    "path",
                    OptionType::File,
                    "",
                    "Path to image or 'screenshot'",
                ),
                opt(
                    "color",
                    OptionType::Color,
                    "rgba(25, 20, 20, 1.0)",
                    "Background color if no image",
                ),
                opt(
                    "blur_passes",
                    OptionType::Int,
                    "0",
                    "Blur passes (0 to disable)",
                ),
                opt("blur_size", OptionType::Int, "7", "Blur size"),
                opt("noise", OptionType::Float, "0.0117", "Noise amount"),
                opt("contrast", OptionType::Float, "0.8916", "Contrast"),
                opt("brightness", OptionType::Float, "0.8172", "Brightness"),
                opt("vibrancy", OptionType::Float, "0.1696", "Vibrancy"),
                opt(
                    "vibrancy_darkness",
                    OptionType::Float,
                    "0.0",
                    "Vibrancy darkness",
                ),
            ],
        },
        HyprlockSection {
            name: "input-field".to_string(),
            title: "Input Fields".to_string(),
            icon: '🔒',
            is_list: true,
            options: vec![
                opt("monitor", OptionType::Monitor, "", "Monitor"),
                opt("size", OptionType::Vec2, "200, 50", "Size (width, height)"),
                opt(
                    "outline_thickness",
                    OptionType::Int,
                    "3",
                    "Outline thickness",
                ),
                opt("dots_size", OptionType::Float, "0.33", "Dots size"),
                opt("dots_spacing", OptionType::Float, "0.15", "Dots spacing"),
                opt("dots_center", OptionType::Bool, "false", "Center dots"),
                opt(
                    "outer_color",
                    OptionType::Color,
                    "rgb(151, 151, 151)",
                    "Outer ring color",
                ),
                opt(
                    "inner_color",
                    OptionType::Color,
                    "rgb(200, 200, 200)",
                    "Inner circle color",
                ),
                opt(
                    "font_color",
                    OptionType::Color,
                    "rgb(10, 10, 10)",
                    "Font color",
                ),
                opt("fade_on_empty", OptionType::Bool, "true", "Fade on empty"),
                opt(
                    "placeholder_text",
                    OptionType::String,
                    "Input Password...",
                    "Placeholder text",
                ),
                opt("hide_input", OptionType::Bool, "false", "Hide input"),
                opt("position", OptionType::Vec2, "0, -20", "Position (x, y)"),
                opt("halign", OptionType::String, "center", "Horizontal Align"),
                opt("valign", OptionType::String, "center", "Vertical Align"),
            ],
        },
        HyprlockSection {
            name: "label".to_string(),
            title: "Labels".to_string(),
            icon: '🏷',
            is_list: true,
            options: vec![
                opt("monitor", OptionType::Monitor, "", "Monitor"),
                opt("text", OptionType::String, "$TIME", "Text (supports vars)"),
                opt(
                    "color",
                    OptionType::Color,
                    "rgb(255, 255, 255)",
                    "Text color",
                ),
                opt("font_size", OptionType::Int, "25", "Font size"),
                opt("font_family", OptionType::String, "Sans", "Font family"),
                opt("position", OptionType::Vec2, "0, 80", "Position (x, y)"),
                opt("halign", OptionType::String, "center", "Horizontal Align"),
                opt("valign", OptionType::String, "center", "Vertical Align"),
            ],
        },
    ]
}

fn opt(name: &str, typ: OptionType, default: &str, desc: &str) -> HyprlockOption {
    HyprlockOption {
        name: name.to_string(),
        option_type: typ,
        default: default.to_string(),
        description: desc.to_string(),
        min: None,
        max: None,
        step: None,
        choices: None,
    }
}
