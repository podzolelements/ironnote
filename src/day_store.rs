use crate::statistics::Stats;

#[derive(Debug, Default, Clone)]
pub struct DayStore {
    date: String,
    entry_text: String,
    modified: bool,
}

impl DayStore {
    pub fn new(date: &str) -> Self {
        Self {
            date: date.to_string(),
            entry_text: String::default(),
            modified: false,
        }
    }

    pub fn get_day_text(&self) -> String {
        self.entry_text.clone()
    }

    pub fn set_day_text(&mut self, new_text: String) {
        self.entry_text = new_text;
        self.modified = true;
    }

    pub fn date(&self) -> String {
        self.date.clone()
    }

    pub fn contains_entry(&self) -> bool {
        !(self.entry_text.is_empty() || self.entry_text == "\n")
    }

    pub fn modified(&self) -> bool {
        self.modified
    }
}

impl Stats for DayStore {
    fn word_count(&self) -> usize {
        self.entry_text.split_whitespace().count()
    }

    fn char_count(&self) -> usize {
        self.entry_text.chars().count()
    }
}
