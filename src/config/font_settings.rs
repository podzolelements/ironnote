use iced::{Background, Border, Color, Font, Padding, border::Radius, widget::markdown};

use crate::ui::journal_theme::LIGHT;

// TODO: move to to user preferences and make serdeable
pub fn markdown_settings() -> markdown::Settings {
    let base_font_size = 13.into();

    markdown::Settings {
        text_size: base_font_size,
        h1_size: base_font_size * 2.0,
        h2_size: base_font_size * 1.5,
        h3_size: base_font_size * 1.17,
        h4_size: base_font_size,
        h5_size: base_font_size * 0.83,
        h6_size: base_font_size * 0.67,
        code_size: base_font_size,
        spacing: 5.0.into(),
        style: markdown::Style {
            font: Font::DEFAULT,
            inline_code_highlight: markdown::Highlight {
                background: Background::Color(LIGHT.darkened_background()),
                border: Border {
                    color: Color::TRANSPARENT,
                    width: 0.0,
                    radius: Radius::new(0.0),
                },
            },
            inline_code_padding: Padding::default(),
            inline_code_color: LIGHT.default_text,
            inline_code_font: Font::MONOSPACE,
            code_block_font: Font::MONOSPACE,
            link_color: LIGHT.link,
        },
    }
}
