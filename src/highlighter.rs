use crate::dictionary::{self, DICTIONARY};
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

#[derive(Debug, Clone, PartialEq)]
pub struct HighlightSettings {
    pub(crate) cursor_line_idx: usize,
    pub(crate) cursor_char_idx: usize,
    pub(crate) cursor_spellcheck_timed_out: bool,
}

#[derive(Debug)]
pub struct SpellHighlighter {
    current_line: usize,
    settings: HighlightSettings,
}

pub enum SpellHighlightColor {
    Red,
}

impl Highlighter for SpellHighlighter {
    type Settings = HighlightSettings;
    type Highlight = SpellHighlightColor;
    type Iterator<'a> = std::vec::IntoIter<(Range<usize>, Self::Highlight)>;

    fn new(settings: &Self::Settings) -> Self {
        SpellHighlighter {
            current_line: 0,
            settings: settings.clone(),
        }
    }

    fn update(&mut self, new_settings: &Self::Settings) {
        self.settings = new_settings.clone();
    }

    fn change_line(&mut self, line: usize) {
        self.current_line = line;
    }

    fn highlight_line(&mut self, line: &str) -> Self::Iterator<'_> {
        let mut highlights = Vec::new();

        let dictionary = DICTIONARY.read().expect("couldn't get dictionary read");

        let cursor_line = self.settings.cursor_line_idx;
        let cursor_char = self.settings.cursor_char_idx;

        for (word, start, end) in dictionary::extract_words(line) {
            if !(dictionary.check(word)
                || (!self.settings.cursor_spellcheck_timed_out
                    && (cursor_line == self.current_line
                        && start <= cursor_char
                        && cursor_char <= end)))
            {
                highlights.push((start..end, SpellHighlightColor::Red));
            }
        }

        highlights.into_iter()
    }

    fn current_line(&self) -> usize {
        self.current_line
    }
}
