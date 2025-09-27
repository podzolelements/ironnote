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

        if self.current_line() != 0 {
            self.change_line(0);
        }
    }

    fn change_line(&mut self, line: usize) {
        self.current_line = line;
    }

    fn highlight_line(&mut self, line: &str) -> Self::Iterator<'_> {
        let mut highlights = Vec::new();

        let dictionary = DICTIONARY.read().expect("couldn't get dictionary read");

        let cursor_line = self.settings.cursor_line_idx;
        let cursor_char = self.settings.cursor_char_idx;
        let timed_out = self.settings.cursor_spellcheck_timed_out;

        for (word, start, end) in dictionary::extract_words(line) {
            // disable highlighting for the word at the cursor if the edit timeout hasn't triggered yet
            if !timed_out
                && cursor_line == self.current_line
                && cursor_char != 0
                && start <= cursor_char
                && cursor_char <= end
            {
                continue;
            }

            if !dictionary.check(word) {
                highlights.push((start..end, SpellHighlightColor::Red));
            }
        }

        self.current_line += 1;

        highlights.into_iter()
    }

    fn current_line(&self) -> usize {
        self.current_line
    }
}
