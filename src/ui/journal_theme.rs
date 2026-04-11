use iced::{Color, color};

pub struct JournalTheme {
    pub(crate) default_background: Color,
    pub(crate) default_text: Color,
    pub(crate) dimmed_text: Color,

    /// how much to subtract from another color to make a darkened effect
    pub(crate) darkening_delta: Color,

    pub(crate) selection: Color,
    pub(crate) selection_text: Color,

    pub(crate) char_count_floor: Color,
    pub(crate) char_count_ceiling: Color,
}

pub const LIGHT: JournalTheme = JournalTheme {
    default_background: color!(0xffffff, 1.0),
    default_text: color!(0x000000, 1.0),
    dimmed_text: color!(0x949494, 1.0),

    darkening_delta: color!(0x333333, 0.0),

    selection: color!(0x179bdd, 1.0),
    selection_text: color!(0xffffff, 1.0),

    char_count_floor: color!(0xb0ffce, 0.8),
    char_count_ceiling: color!(0x00762d, 0.8),
};

impl JournalTheme {
    /// applies the darkening_delta by subtracting it from the given color, returning the result
    pub fn darken(&self, color_to_darken: Color) -> Color {
        let dark_r = (color_to_darken.r - self.darkening_delta.r).max(0.0);
        let dark_g = (color_to_darken.g - self.darkening_delta.g).max(0.0);
        let dark_b = (color_to_darken.b - self.darkening_delta.b).max(0.0);
        let dark_a = (color_to_darken.a - self.darkening_delta.a).max(0.0);

        Color::from_linear_rgba(dark_r, dark_g, dark_b, dark_a)
    }
}
