use iced::{Color, Theme};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum AppTheme {
    #[default]
    CatppuccinMocha,
    Nord,
    Drifter,
}

impl AppTheme {
    pub fn palette(&self) -> Palette {
        match self {
            AppTheme::CatppuccinMocha => Palette::catppuccin_mocha(),
            AppTheme::Nord => Palette::nord(),
            AppTheme::Drifter => Palette::drifter(),
        }
    }

    pub fn all() -> &'static [AppTheme] {
        &[AppTheme::CatppuccinMocha, AppTheme::Nord, AppTheme::Drifter]
    }

    pub fn to_iced_theme(&self) -> Theme {
        let p = self.palette();
        Theme::Custom(std::sync::Arc::new(iced::theme::Custom::new(
            self.to_string(),
            iced::theme::Palette {
                background: p.base,
                text: p.text,
                primary: p.blue,
                success: p.green,
                warning: p.yellow,
                danger: p.red,
            },
        )))
    }
}

impl std::fmt::Display for AppTheme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppTheme::CatppuccinMocha => write!(f, "Catppuccin Mocha"),
            AppTheme::Nord => write!(f, "Nord"),
            AppTheme::Drifter => write!(f, "Drifter"),
        }
    }
}

pub struct Palette {
    pub crust: Color,
    pub mantle: Color,
    pub base: Color,
    pub surface0: Color,
    pub surface1: Color,
    pub surface2: Color,
    pub overlay0: Color,
    pub overlay1: Color,
    pub overlay2: Color,
    pub text: Color,
    pub subtext0: Color,
    pub subtext1: Color,
    pub lavender: Color,
    pub blue: Color,
    pub sapphire: Color,
    pub sky: Color,
    pub teal: Color,
    pub green: Color,
    pub yellow: Color,
    pub peach: Color,
    pub maroon: Color,
    pub red: Color,
    pub mauve: Color,
    pub pink: Color,
    pub flamingo: Color,
    pub rosewater: Color,
}

impl Palette {
    pub fn catppuccin_mocha() -> Self {
        Self {
            crust: Color::from_rgb(0.067, 0.067, 0.102),
            mantle: Color::from_rgb(0.094, 0.094, 0.145),
            base: Color::from_rgb(0.121, 0.121, 0.180),
            surface0: Color::from_rgb(0.180, 0.180, 0.247),
            surface1: Color::from_rgb(0.227, 0.227, 0.302),
            surface2: Color::from_rgb(0.275, 0.275, 0.357),
            overlay0: Color::from_rgb(0.420, 0.420, 0.510),
            overlay1: Color::from_rgb(0.478, 0.478, 0.561),
            overlay2: Color::from_rgb(0.576, 0.576, 0.647),
            text: Color::from_rgb(0.804, 0.839, 0.957),
            subtext1: Color::from_rgb(0.725, 0.757, 0.863),
            subtext0: Color::from_rgb(0.651, 0.678, 0.784),
            lavender: Color::from_rgb(0.706, 0.745, 0.996),
            blue: Color::from_rgb(0.537, 0.706, 0.980),
            sapphire: Color::from_rgb(0.455, 0.773, 0.906),
            sky: Color::from_rgb(0.537, 0.859, 0.933),
            teal: Color::from_rgb(0.580, 0.886, 0.843),
            green: Color::from_rgb(0.651, 0.890, 0.631),
            yellow: Color::from_rgb(0.976, 0.902, 0.600),
            peach: Color::from_rgb(0.980, 0.702, 0.529),
            maroon: Color::from_rgb(0.922, 0.518, 0.588),
            red: Color::from_rgb(0.953, 0.545, 0.659),
            mauve: Color::from_rgb(0.796, 0.651, 0.969),
            pink: Color::from_rgb(0.961, 0.761, 0.906),
            flamingo: Color::from_rgb(0.949, 0.757, 0.788),
            rosewater: Color::from_rgb(0.961, 0.827, 0.804),
        }
    }

    pub fn nord() -> Self {
        Self {
            crust: Color::from_rgb(0.15, 0.16, 0.20),
            mantle: Color::from_rgb(0.18, 0.20, 0.25),
            base: Color::from_rgb(0.20, 0.22, 0.27),
            surface0: Color::from_rgb(0.23, 0.26, 0.32),
            surface1: Color::from_rgb(0.26, 0.29, 0.36),
            surface2: Color::from_rgb(0.30, 0.34, 0.42),
            overlay0: Color::from_rgb(0.35, 0.39, 0.48),
            overlay1: Color::from_rgb(0.45, 0.50, 0.60),
            overlay2: Color::from_rgb(0.55, 0.61, 0.72),
            text: Color::from_rgb(0.92, 0.94, 0.96),
            subtext1: Color::from_rgb(0.90, 0.91, 0.93),
            subtext0: Color::from_rgb(0.85, 0.87, 0.91),
            lavender: Color::from_rgb(0.71, 0.56, 0.68),
            blue: Color::from_rgb(0.53, 0.75, 0.82),
            sapphire: Color::from_rgb(0.55, 0.73, 0.80),
            sky: Color::from_rgb(0.55, 0.78, 0.90),
            teal: Color::from_rgb(0.56, 0.74, 0.73),
            green: Color::from_rgb(0.64, 0.75, 0.55),
            yellow: Color::from_rgb(0.92, 0.80, 0.55),
            peach: Color::from_rgb(0.82, 0.57, 0.47),
            maroon: Color::from_rgb(0.75, 0.38, 0.42),
            red: Color::from_rgb(0.75, 0.36, 0.38),
            mauve: Color::from_rgb(0.70, 0.53, 0.66),
            pink: Color::from_rgb(0.72, 0.55, 0.67),
            flamingo: Color::from_rgb(0.80, 0.50, 0.50),
            rosewater: Color::from_rgb(0.92, 0.92, 0.92),
        }
    }

    pub fn drifter() -> Self {
        Self {
            crust: Color::from_rgb(0.05, 0.05, 0.08),
            mantle: Color::from_rgb(0.08, 0.08, 0.12),
            base: Color::from_rgb(0.11, 0.11, 0.16),
            surface0: Color::from_rgb(0.15, 0.15, 0.22),
            surface1: Color::from_rgb(0.20, 0.20, 0.28),
            surface2: Color::from_rgb(0.25, 0.25, 0.35),
            overlay0: Color::from_rgb(0.35, 0.35, 0.45),
            overlay1: Color::from_rgb(0.45, 0.45, 0.55),
            overlay2: Color::from_rgb(0.55, 0.55, 0.65),
            text: Color::from_rgb(0.90, 0.90, 0.95),
            subtext1: Color::from_rgb(0.80, 0.80, 0.85),
            subtext0: Color::from_rgb(0.70, 0.70, 0.75),
            lavender: Color::from_rgb(0.85, 0.70, 0.95),
            blue: Color::from_rgb(0.40, 0.60, 0.90),
            sapphire: Color::from_rgb(0.35, 0.55, 0.85),
            sky: Color::from_rgb(0.50, 0.80, 0.95),
            teal: Color::from_rgb(0.40, 0.85, 0.80),
            green: Color::from_rgb(0.50, 0.85, 0.50),
            yellow: Color::from_rgb(0.95, 0.90, 0.60),
            peach: Color::from_rgb(0.95, 0.70, 0.50),
            maroon: Color::from_rgb(0.80, 0.40, 0.50),
            red: Color::from_rgb(0.90, 0.40, 0.40),
            mauve: Color::from_rgb(0.75, 0.60, 0.90),
            pink: Color::from_rgb(0.90, 0.60, 0.85),
            flamingo: Color::from_rgb(0.90, 0.65, 0.70),
            rosewater: Color::from_rgb(0.95, 0.85, 0.85),
        }
    }

    pub fn with_alpha(color: Color, alpha: f32) -> Color {
        Color { a: alpha, ..color }
    }
}

pub fn get_palette(theme: &Theme) -> Palette {
    match theme {
        Theme::Custom(custom) => match custom.to_string().as_str() {
            "Nord" => Palette::nord(),
            "Drifter" => Palette::drifter(),
            _ => Palette::catppuccin_mocha(),
        },
        _ => Palette::catppuccin_mocha(),
    }
}
