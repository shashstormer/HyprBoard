use super::color_utils::Hsv;
use super::text_input;
use iced::widget::canvas::{self, gradient};
use iced::widget::{button, column, container, row, slider, text};
use iced::{Background, Color, Element, Length, Point, Rectangle, Size, Theme, mouse};

pub fn color_picker<'a, Message>(
    value: &str,
    on_change: impl Fn(String) -> Message + 'a,
    on_click: Message,
) -> Element<'a, Message>
where
    Message: Clone + 'a,
{
    let color_preview = if let Some(c) = parse_color(value) {
        container(text("  "))
            .width(Length::Fixed(24.0))
            .height(Length::Fixed(24.0))
            .style(move |_: &Theme| container::Style {
                background: Some(iced::Background::Color(c)),
                border: iced::Border {
                    color: Color::from_rgb8(88, 91, 112),
                    width: 1.0,
                    radius: 4.0.into(),
                },
                ..Default::default()
            })
    } else {
        container(
            text("?")
                .size(12)
                .align_x(iced::alignment::Horizontal::Center)
                .align_y(iced::alignment::Vertical::Center),
        )
        .width(Length::Fixed(24.0))
        .height(Length::Fixed(24.0))
        .style(|_: &Theme| container::Style {
            background: Some(iced::Background::Color(Color::TRANSPARENT)),
            border: iced::Border {
                color: Color::from_rgb8(88, 91, 112),
                width: 1.0,
                radius: 4.0.into(),
            },
            ..Default::default()
        })
    };

    let preview_btn =
        button(color_preview)
            .on_press(on_click)
            .padding(0)
            .style(|_theme: &Theme, _status| button::Style {
                background: None,
                ..Default::default()
            });

    row![
        container(text_input::input("#RRGGBB", value, on_change)).width(Length::Fill),
        preview_btn
    ]
    .spacing(10)
    .align_y(iced::Alignment::Center)
    .into()
}

pub fn view_modal<'a, Message>(
    current_value: &str,
    on_change: impl Fn(String) -> Message + 'a + Clone,
    on_cancel: Message,
    on_submit: Message,
) -> Element<'a, Message>
where
    Message: Clone + 'a,
{
    let (current_color, format) =
        parse_color_fmt(current_value).unwrap_or((Color::BLACK, ColorFormat::Hex));

    let hsv = Hsv::from_color(current_color);
    let r_byte = (current_color.r * 255.0) as u8;
    let g_byte = (current_color.g * 255.0) as u8;
    let b_byte = (current_color.b * 255.0) as u8;

    let format_val = format.clone();
    let on_change_clone = on_change.clone();
    let dispatch_color = move |c: Color| {
        let s = format_color(c, &format_val);
        on_change_clone(s)
    };

    let dispatch_1 = dispatch_color.clone();
    let update_hsv = move |h, s, v| {
        let new_c = Hsv::new(h, s, v).to_color();
        dispatch_1(Color {
            a: current_color.a,
            ..new_c
        })
    };

    let dispatch_2 = dispatch_color.clone();
    let update_rgba = move |r, g, b, a| dispatch_2(Color::from_rgba8(r, g, b, (a * 255.0) as f32));

    let update_hsv_1 = update_hsv.clone();
    let update_hsv_2 = update_hsv.clone();

    let sat_val_box = canvas::Canvas::new(SatValBox {
        hsv,
        on_change: Box::new(move |new_hsv| update_hsv_1(new_hsv.h, new_hsv.s, new_hsv.v)),
    })
    .width(Length::Fill)
    .height(Length::Fixed(150.0));

    let hue_strip = canvas::Canvas::new(HueStrip {
        hue: hsv.h,
        on_change: Box::new(move |new_h| update_hsv_2(new_h, hsv.s, hsv.v)),
    })
    .width(Length::Fill)
    .height(Length::Fixed(12.0));

    let a_slider = slider(0.0..=1.0, current_color.a, move |v| {
        let mut c = current_color;
        c.a = v;
        dispatch_color(c)
    })
    .step(0.01);

    let update_rgba_1 = update_rgba.clone();
    let r_slider = slider(0.0..=255.0, current_color.r * 255.0, move |v| {
        update_rgba_1(v as u8, g_byte, b_byte, current_color.a)
    })
    .step(1.0);

    let update_rgba_2 = update_rgba.clone();
    let g_slider = slider(0.0..=255.0, current_color.g * 255.0, move |v| {
        update_rgba_2(r_byte, v as u8, b_byte, current_color.a)
    })
    .step(1.0);

    let update_rgba_3 = update_rgba.clone();
    let b_slider = slider(0.0..=255.0, current_color.b * 255.0, move |v| {
        update_rgba_3(r_byte, g_byte, v as u8, current_color.a)
    })
    .step(1.0);

    column![
        text("Color Picker").size(20),
        container(sat_val_box)
            .width(Length::Fill)
            .style(|_: &Theme| container::Style {
                border: iced::Border {
                    color: Color::from_rgb8(100, 100, 100),
                    width: 1.0,
                    radius: 4.0.into()
                },
                ..Default::default()
            }),
        container(hue_strip)
            .width(Length::Fill)
            .style(|_: &Theme| container::Style {
                border: iced::Border {
                    color: Color::from_rgb8(100, 100, 100),
                    width: 1.0,
                    radius: 4.0.into()
                },
                ..Default::default()
            }),
        row![text("Alpha").size(12).width(40), a_slider]
            .align_y(iced::Alignment::Center)
            .spacing(5),
        text(format!(
            "R: {}  G: {}  B: {}  A: {:.2}",
            r_byte, g_byte, b_byte, current_color.a
        ))
        .size(14),
        row![text("R").width(20), r_slider,]
            .spacing(5)
            .align_y(iced::Alignment::Center),
        row![text("G").width(20), g_slider,]
            .spacing(5)
            .align_y(iced::Alignment::Center),
        row![text("B").width(20), b_slider,]
            .spacing(5)
            .align_y(iced::Alignment::Center),
        row![
            text("Value: "),
            container(text_input::input("Value", current_value, on_change.clone()))
                .width(Length::Fill),
            container("")
                .width(Length::Fixed(40.0))
                .height(Length::Fixed(20.0))
                .style(move |_| container::Style {
                    background: Some(Background::Color(current_color)),
                    border: iced::Border {
                        color: Color::BLACK,
                        width: 1.0,
                        radius: 2.0.into()
                    },
                    ..Default::default()
                }),
        ]
        .spacing(10)
        .align_y(iced::Alignment::Center),
        row![
            crate::view::components::button::destructive(text("Cancel"), on_cancel),
            crate::view::components::button::primary(text("Apply"), on_submit)
        ]
        .spacing(20)
    ]
    .spacing(10)
    .width(320)
    .padding(10)
    .into()
}

#[derive(Clone, Debug, PartialEq)]
enum ColorFormat {
    Hex,
    HexAlpha,
    HyprlandHex,
    Rgb,
    Rgba,
}

fn parse_color_fmt(s: &str) -> Option<(Color, ColorFormat)> {
    let s = s.trim();
    if s.starts_with("#") {
        let hex = &s[1..];
        if hex.len() == 6 {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            return Some((Color::from_rgb8(r, g, b), ColorFormat::Hex));
        } else if hex.len() == 8 {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            let a = u8::from_str_radix(&hex[6..8], 16).ok()?;
            return Some((
                Color::from_rgba8(r, g, b, a as f32 / 255.0),
                ColorFormat::HexAlpha,
            ));
        }
    } else if s.starts_with("0x") {
        if s.len() == 10 {
            let hex = &s[2..];
            let a = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let r = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let g = u8::from_str_radix(&hex[4..6], 16).ok()?;
            let b = u8::from_str_radix(&hex[6..8], 16).ok()?;
            return Some((
                Color::from_rgba8(r, g, b, a as f32 / 255.0),
                ColorFormat::HyprlandHex,
            ));
        }
    } else if s.starts_with("rgba(") && s.ends_with(")") {
        let content = &s[5..s.len() - 1];
        let parts: Vec<&str> = content.split(',').map(|p| p.trim()).collect();
        if parts.len() >= 3 {
            let r = parts[0].parse::<f32>().ok()?;
            let g = parts[1].parse::<f32>().ok()?;
            let b = parts[2].parse::<f32>().ok()?;
            let a = if parts.len() > 3 {
                parts[3].parse::<f32>().unwrap_or(1.0)
            } else {
                1.0
            };
            return Some((
                Color::from_rgba8(r as u8, g as u8, b as u8, a),
                ColorFormat::Rgba,
            ));
        }
    } else if s.starts_with("rgb(") && s.ends_with(")") {
        let content = &s[4..s.len() - 1];
        let parts: Vec<&str> = content.split(',').map(|p| p.trim()).collect();
        if parts.len() >= 3 {
            let r = parts[0].parse::<u8>().ok()?;
            let g = parts[1].parse::<u8>().ok()?;
            let b = parts[2].parse::<u8>().ok()?;
            return Some((Color::from_rgb8(r, g, b), ColorFormat::Rgb));
        }
    }

    if s.len() == 6 {
        let r = u8::from_str_radix(&s[0..2], 16).ok()?;
        let g = u8::from_str_radix(&s[2..4], 16).ok()?;
        let b = u8::from_str_radix(&s[4..6], 16).ok()?;
        return Some((Color::from_rgb8(r, g, b), ColorFormat::Hex));
    }

    None
}

fn format_color(c: Color, fmt: &ColorFormat) -> String {
    let r = (c.r * 255.0) as u8;
    let g = (c.g * 255.0) as u8;
    let b = (c.b * 255.0) as u8;
    let a_byte = (c.a * 255.0) as u8;

    match fmt {
        ColorFormat::Hex => format!("#{:02x}{:02x}{:02x}", r, g, b),
        ColorFormat::HexAlpha => format!("#{:02x}{:02x}{:02x}{:02x}", r, g, b, a_byte),
        ColorFormat::HyprlandHex => format!("0x{:02x}{:02x}{:02x}{:02x}", a_byte, r, g, b),
        ColorFormat::Rgb => format!("rgb({}, {}, {})", r, g, b),
        ColorFormat::Rgba => format!("rgba({}, {}, {}, {:.2})", r, g, b, c.a),
    }
}

struct SatValBox<'a, Message> {
    hsv: Hsv,
    on_change: Box<dyn Fn(Hsv) -> Message + 'a>,
}

#[derive(Default)]
struct SatValState {
    is_dragging: bool,
}

impl<'a, Message> canvas::Program<Message> for SatValBox<'a, Message>
where
    Message: Clone,
{
    type State = SatValState;

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &iced::Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());

        let hue_color = Hsv::new(self.hsv.h, 1.0, 1.0).to_color();

        let horizon = canvas::Gradient::Linear(
            gradient::Linear::new(Point::ORIGIN, Point::new(bounds.width, 0.0))
                .add_stop(0.0, Color::WHITE)
                .add_stop(1.0, hue_color),
        );

        frame.fill_rectangle(Point::ORIGIN, bounds.size(), horizon);

        let vertical = canvas::Gradient::Linear(
            gradient::Linear::new(Point::ORIGIN, Point::new(0.0, bounds.height))
                .add_stop(0.0, Color::TRANSPARENT)
                .add_stop(1.0, Color::BLACK),
        );

        frame.fill_rectangle(Point::ORIGIN, bounds.size(), vertical);

        let cursor_x = self.hsv.s * bounds.width;
        let cursor_y = (1.0 - self.hsv.v) * bounds.height;
        let cursor_pos = Point::new(cursor_x, cursor_y);

        let circle = canvas::Path::circle(cursor_pos, 5.0);
        let stroke_color = if self.hsv.v > 0.5 {
            Color::BLACK
        } else {
            Color::WHITE
        };
        frame.stroke(
            &circle,
            canvas::Stroke::default()
                .with_color(stroke_color)
                .with_width(2.0),
        );

        vec![frame.into_geometry()]
    }

    fn update(
        &self,
        state: &mut Self::State,
        event: &iced::Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<canvas::Action<Message>> {
        let cursor_position = match cursor.position_in(bounds) {
            Some(p) => p,
            None => {
                if state.is_dragging {
                    if let Some(pos) = cursor.position() {
                        let rel_x = pos.x - bounds.x;
                        let rel_y = pos.y - bounds.y;
                        let s = (rel_x / bounds.width).clamp(0.0, 1.0);
                        let v = 1.0 - (rel_y / bounds.height).clamp(0.0, 1.0);
                        let new_hsv = Hsv::new(self.hsv.h, s, v);
                        return Some(canvas::Action::publish((self.on_change)(new_hsv)));
                    }
                }
                return None;
            }
        };

        match event {
            iced::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                state.is_dragging = true;
                let s = (cursor_position.x / bounds.width).clamp(0.0, 1.0);
                let v = 1.0 - (cursor_position.y / bounds.height).clamp(0.0, 1.0);
                return Some(canvas::Action::publish((self.on_change)(Hsv::new(
                    self.hsv.h, s, v,
                ))));
            }
            iced::Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                if state.is_dragging {
                    let s = (cursor_position.x / bounds.width).clamp(0.0, 1.0);
                    let v = 1.0 - (cursor_position.y / bounds.height).clamp(0.0, 1.0);
                    return Some(canvas::Action::publish((self.on_change)(Hsv::new(
                        self.hsv.h, s, v,
                    ))));
                }
            }
            iced::Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                state.is_dragging = false;
                return Some(canvas::Action::capture());
            }
            _ => {}
        }

        None
    }
}

struct HueStrip<'a, Message> {
    hue: f32,
    on_change: Box<dyn Fn(f32) -> Message + 'a>,
}

#[derive(Default)]
struct HueState {
    is_dragging: bool,
}

impl<'a, Message> canvas::Program<Message> for HueStrip<'a, Message>
where
    Message: Clone,
{
    type State = HueState;

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &iced::Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());

        let gradient = canvas::Gradient::Linear(
            gradient::Linear::new(Point::ORIGIN, Point::new(bounds.width, 0.0))
                .add_stop(0.0, Color::from_rgb(1.0, 0.0, 0.0))
                .add_stop(0.17, Color::from_rgb(1.0, 1.0, 0.0))
                .add_stop(0.33, Color::from_rgb(0.0, 1.0, 0.0))
                .add_stop(0.50, Color::from_rgb(0.0, 1.0, 1.0))
                .add_stop(0.67, Color::from_rgb(0.0, 0.0, 1.0))
                .add_stop(0.83, Color::from_rgb(1.0, 0.0, 1.0))
                .add_stop(1.0, Color::from_rgb(1.0, 0.0, 0.0)),
        );

        frame.fill_rectangle(Point::ORIGIN, bounds.size(), gradient);

        let cursor_x = (self.hue / 360.0) * bounds.width;
        let line = canvas::Path::rectangle(
            Point::new(cursor_x - 2.0, 0.0),
            Size::new(4.0, bounds.height),
        );
        frame.fill(&line, Color::WHITE);
        frame.stroke(
            &line,
            canvas::Stroke::default()
                .with_color(Color::BLACK)
                .with_width(1.0),
        );

        vec![frame.into_geometry()]
    }

    fn update(
        &self,
        state: &mut Self::State,
        event: &iced::Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<canvas::Action<Message>> {
        let cursor_position = match cursor.position_in(bounds) {
            Some(p) => p,
            None => {
                if state.is_dragging {
                    if let Some(pos) = cursor.position() {
                        let rel_x = pos.x - bounds.x;
                        let h = (rel_x / bounds.width).clamp(0.0, 1.0) * 360.0;
                        return Some(canvas::Action::publish((self.on_change)(h)));
                    }
                }
                return None;
            }
        };

        match event {
            iced::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                state.is_dragging = true;
                let h = (cursor_position.x / bounds.width).clamp(0.0, 1.0) * 360.0;
                return Some(canvas::Action::publish((self.on_change)(h)));
            }
            iced::Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                if state.is_dragging {
                    let h = (cursor_position.x / bounds.width).clamp(0.0, 1.0) * 360.0;
                    return Some(canvas::Action::publish((self.on_change)(h)));
                }
            }
            iced::Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                state.is_dragging = false;
                return Some(canvas::Action::capture());
            }
            _ => {}
        }
        None
    }
}

pub fn parse_color(hex: &str) -> Option<Color> {
    parse_color_fmt(hex).map(|(c, _)| c)
}
