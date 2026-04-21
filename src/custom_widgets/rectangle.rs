use iced::{Background, Border, Color, Element, Shadow, Theme, Vector, widget::button};

/// Constructs a rectangle widget of the given dimensions and color
pub fn build_rectangle<'a, M: 'a + Clone>(width: f32, height: f32, color: Color) -> Element<'a, M> {
    let button_style = move |_theme: &Theme, _status: button::Status| -> button::Style {
        button::Style {
            background: Some(Background::Color(color)),
            text_color: Color::TRANSPARENT,
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 0.0.into(),
            },
            shadow: Shadow {
                color: Color::TRANSPARENT,
                offset: Vector::ZERO,
                blur_radius: 0.0,
            },
            snap: true,
        }
    };

    button("")
        .width(width)
        .height(height)
        .style(button_style)
        .into()
}
