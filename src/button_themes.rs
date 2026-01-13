use crate::journal_theme::LIGHT;
use iced::{
    Background, Border, Color, Shadow, Theme, Vector,
    widget::button::{self, Status},
};

/// the default button styling, using the default background color with no border or shadow. darkens the button when
/// hovered or pressed
pub fn standard_button_style(_theme: &Theme, status: Status) -> button::Style {
    let base_style = button::Style {
        background: Some(Background::Color(LIGHT.default_background)),
        text_color: LIGHT.default_text,
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
    };

    let mut darkened_style = base_style;
    let darkened_background = LIGHT.darken(LIGHT.default_background);
    darkened_style.background = Some(Background::Color(darkened_background));

    match status {
        Status::Active => base_style,
        Status::Hovered | Status::Pressed | Status::Disabled => darkened_style,
    }
}
