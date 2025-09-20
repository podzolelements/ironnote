use crate::{dictionary::DICTIONARY, misc_tools::extract_words};
use iced::{Color, Font, widget::text::Highlighter};
use iced_core::text::highlighter::Format;
use std::ops::Range;

/// converts the custom highlighting scheme into and iced font format
pub fn highlight_to_format(highlight: &SpellHighlightColor, _theme: &iced::Theme) -> Format<Font> {
    let color = match highlight {
        SpellHighlightColor::Red => Some(Color::new(1.0, 0.0, 0.0, 1.0)),
    };

    Format { color, font: None }
}

#[derive(Debug)]
pub struct SpellHighlighter {
    current_line: usize,
}

pub enum SpellHighlightColor {
    Red,
}

impl Highlighter for SpellHighlighter {
    type Settings = ();
    type Highlight = SpellHighlightColor;
    type Iterator<'a> = std::vec::IntoIter<(Range<usize>, Self::Highlight)>;

    fn new(_new_settings: &Self::Settings) -> Self {
        SpellHighlighter { current_line: 0 }
    }

    fn update(&mut self, _new_settings: &Self::Settings) {}

    fn change_line(&mut self, line: usize) {
        self.current_line = line;
    }

    fn highlight_line(&mut self, line: &str) -> Self::Iterator<'_> {
        let mut highlights = Vec::new();

        let dictionary = DICTIONARY.read().expect("e");

        for (word, start, end) in extract_words(line) {
            if !dictionary.check(word) {
                highlights.push((start..end, SpellHighlightColor::Red));
            }
        }

        highlights.into_iter()
    }

    fn current_line(&self) -> usize {
        self.current_line
    }
}
