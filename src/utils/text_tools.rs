use iced::{
    advanced::graphics::text::cosmic_text::{Attrs, Buffer, FontSystem, Metrics, Shaping},
    widget::text::LineHeight,
};
use std::sync::{LazyLock, Mutex};

// The FontSystem is absurdly expensive to create, so this one gets reused on each call
static FONT_SYSTEM: LazyLock<Mutex<FontSystem>> = LazyLock::new(|| Mutex::new(FontSystem::new()));

/// Calculates the width of a string in pixels. It's not exactly right, but it is close enough with a little padding.
/// Based on https://discourse.iced.rs/t/measuring-text-size/686/2
pub fn string_width(text: &str, font_size: f32) -> f32 {
    let line_height = LineHeight::default();
    let line_height_absolute = line_height.to_absolute(font_size.into()).0;

    let metrics = Metrics {
        font_size,
        line_height: line_height_absolute,
    };

    let mut text_buffer = Buffer::new_empty(metrics);
    let attrs = Attrs::new();

    text_buffer.set_text(
        &mut FONT_SYSTEM.lock().unwrap_or_else(|err| err.into_inner()),
        text,
        &attrs,
        Shaping::Advanced,
        None,
    );

    text_buffer
        .layout_runs()
        .fold(0.0, |width, run| run.line_w.max(width))
}
