use iced::widget::{container, row, text};
use iced::{Element, Theme, Color, Padding};
use std::time::{Duration, Instant};



#[derive(Clone, Debug, PartialEq)]
pub enum ToastType {
    Info,
    Success,
    Error,
}

#[derive(Clone, Debug)]
pub struct Toast {
    pub message: String,
    pub type_: ToastType,
    pub created_at: Instant,
    pub duration: Duration,
}

impl Toast {
    pub fn new(message: String, type_: ToastType) -> Self {
        Self {
            message,
            type_,
            created_at: Instant::now(),
            duration: Duration::from_secs(3),
        }
    }

    pub fn is_expired(&self) -> bool {
        self.created_at.elapsed() > self.duration
    }
}

pub fn view<'a, Message: Clone + 'a>(toast: &'a Toast) -> Element<'a, Message> {
    let p = crate::view::components::theme::Palette::catppuccin_mocha();
    let (bg_color, text_color, icon) = match toast.type_ {
        ToastType::Info => (p.base, p.text, "ℹ"),
        ToastType::Success => (p.green, p.base, "✓"),
        ToastType::Error => (p.red, p.base, "✕"),
    };

    container(row![
        text(icon).size(16).style(move |_| text::Style { color: Some(text_color) }),
        text(&toast.message).size(14).style(move |_| text::Style { color: Some(text_color) })
    ].spacing(10))
    .padding(Padding::new(10.0))
    .style(move |_: &Theme| container::Style {
        background: Some(iced::Background::Color(bg_color)),
        border: iced::Border {
            color: p.surface0,
            width: 1.0,
            radius: 8.0.into(),
        },
        shadow: iced::Shadow {
            color: Color::from_rgba8(0, 0, 0, 0.3),
            offset: iced::Vector::new(0.0, 4.0),
            blur_radius: 10.0,
        },
        ..Default::default()
    })
    .into()
}
