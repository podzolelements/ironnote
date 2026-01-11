use crate::{
    dictionary::WORD_REGEX,
    word_count::{WordCount, WordCounts},
};
use chrono::NaiveDate;

#[derive(Debug, Default, Clone)]
pub struct DayStore {
    date: NaiveDate,
    entry_text: String,
    modified: bool,
    word_counts: WordCounts,
}

impl DayStore {
    pub fn new(date: NaiveDate) -> Self {
        Self {
            date,
            entry_text: String::default(),
            modified: false,
            word_counts: WordCounts::default(),
        }
    }

    /// creates a new day store with text already present
    pub fn with_day_text(date: NaiveDate, inital_text: String) -> Self {
        Self {
            date,
            entry_text: inital_text,
            modified: false,
            word_counts: WordCounts::default(),
        }
    }

    pub fn get_day_text(&self) -> String {
        self.entry_text.clone()
    }

    pub fn set_day_text(&mut self, new_text: String) {
        self.entry_text = new_text;
        self.modified = true;

        self.word_counts.set_sync(false);
    }

    pub fn date(&self) -> NaiveDate {
        self.date
    }

    pub fn contains_entry(&self) -> bool {
        !(self.entry_text.is_empty() || self.entry_text == "\n")
    }

    pub fn modified(&self) -> bool {
        self.modified
    }
}

impl WordCount for DayStore {
    fn reload_current_counts(&mut self) {
        self.word_counts.clear_current();

        let words: Vec<String> = WORD_REGEX
            .find_iter(&self.entry_text)
            .map(|word| word.as_str().to_lowercase())
            .collect();

        for word in words {
            self.word_counts.insert_or_add(&word, 1);
        }

        let char_count = self.entry_text.chars().count();
        self.word_counts.set_total_char_count(char_count);
    }

    fn is_word_count_in_sync(&mut self) -> bool {
        self.word_counts.in_sync()
    }

    fn word_diff(&self) -> Vec<(String, i32)> {
        self.word_counts.word_diff()
    }

    fn sync_current_to_upstream(&mut self) {
        self.word_counts.sync_current_to_upstream()
    }

    fn get_word_count(&self, word: &str) -> usize {
        self.word_counts.get_word_count(word)
    }

    fn total_word_count(&self) -> usize {
        self.word_counts.total_word_count()
    }

    fn total_char_count(&self) -> usize {
        self.word_counts.total_char_count()
    }
}
