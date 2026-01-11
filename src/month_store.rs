use crate::{
    day_store::DayStore,
    user_preferences::preferences,
    word_count::{TimedWordCount, WordCount, WordCounts},
};
use chrono::{Datelike, Days, NaiveDate};
use serde_json::Value;
use std::fs;

#[derive(Debug, Clone)]
pub struct MonthStore {
    days: Vec<DayStore>,
    first_of_month: NaiveDate,
    word_counts: WordCounts,
}

impl MonthStore {
    /// creates a new month store from the given date
    pub fn new(first_of_month: NaiveDate) -> Self {
        let days = Self::generate_day_stores(first_of_month);

        Self {
            days,
            first_of_month,
            word_counts: WordCounts::default(),
        }
    }

    pub fn day(&self, day: usize) -> &DayStore {
        &self.days[day]
    }

    pub fn day_mut(&mut self, day: usize) -> &mut DayStore {
        &mut self.days[day]
    }

    pub fn first_of_month(&self) -> NaiveDate {
        self.first_of_month
    }

    pub fn get_yyyy_mm(&self) -> String {
        self.first_of_month.format("%Y-%m").to_string()
    }

    pub fn edited_days(&self) -> [bool; 31] {
        let mut edited_days = [false; 31];
        for (i, day_store) in self.days.iter().enumerate().take(31) {
            edited_days[i] = day_store.contains_entry();
        }

        edited_days
    }

    pub fn edited_day_count(&self) -> usize {
        self.edited_days().iter().filter(|day| **day).count()
    }

    pub fn days(&self) -> impl DoubleEndedIterator<Item = &DayStore> {
        self.days.iter()
    }

    /// creates the collection of properly initialized day stores for the month based on the given date
    fn generate_day_stores(first_of_month: NaiveDate) -> Vec<DayStore> {
        let mut day_stores = Vec::new();

        let mut iterative_date = first_of_month;

        for _i in 0..first_of_month.num_days_in_month() {
            let new_day_store = DayStore::new(iterative_date);
            day_stores.push(new_day_store);

            iterative_date = iterative_date
                .checked_add_days(Days::new(1))
                .expect("couldn't add day");
        }

        day_stores
    }

    /// attempts to load the month store of the given date from disk. if a valid month store is found matching the
    /// date, it is loaded, otherwise an empty month store is generated
    pub fn load_month(&mut self, first_of_month: NaiveDate) {
        self.first_of_month = first_of_month;

        let filename = self.get_yyyy_mm() + ".json";

        let mut save_file_path = preferences().paths.savedata_dir();
        save_file_path.push(filename);

        self.days.clear();

        match fs::exists(&save_file_path) {
            Err(_) => {
                panic!("couldn't determine if file exists");
            }
            Ok(file_exists) => {
                if !file_exists {
                    self.days = Self::generate_day_stores(self.first_of_month);

                    return;
                }
            }
        }

        let month_json =
            fs::read_to_string(&save_file_path).expect("couldn't read json into string");

        let json_data: serde_json::Map<String, Value> =
            if let Ok(data) = serde_json::from_str(&month_json) {
                data
            } else {
                serde_json::Map::new()
            };

        let mut iterative_date = self.first_of_month;

        for _i in 0..(self.first_of_month.num_days_in_month()) {
            let new_date = iterative_date.to_string();

            let entry_text = if let Some(entry_value) = json_data.get(&new_date) {
                let entry: String =
                    serde_json::from_value(entry_value.clone()).expect("invalid entry format");
                entry
            } else {
                "".to_string()
            };

            let new_day_store = DayStore::with_day_text(iterative_date, entry_text);
            self.days.push(new_day_store);

            iterative_date = iterative_date
                .checked_add_days(Days::new(1))
                .expect("couldn't add day");
        }
    }

    /// writes the month store to the disk with the filename "YYYY-MM.json"
    pub fn save_month(&self) {
        let filename = self.get_yyyy_mm() + ".json";

        let mut save_file_path = preferences().paths.savedata_dir();
        save_file_path.push(filename);

        let month_json = if let Ok(existing_savedata) = fs::read_to_string(&save_file_path) {
            existing_savedata
        } else {
            "{}".to_string()
        };

        let mut json_data: serde_json::Map<String, Value> =
            if let Ok(data) = serde_json::from_str(&month_json) {
                data
            } else {
                serde_json::Map::new()
            };

        for i in 0..(self.first_of_month.num_days_in_month() as usize) {
            let new_entry = self.days[i].clone();

            if !new_entry.modified() {
                continue;
            }

            if !new_entry.contains_entry() {
                json_data.remove_entry(&new_entry.date().to_string());
            } else {
                json_data.insert(
                    new_entry.date().to_string(),
                    serde_json::to_value(new_entry.get_day_text()).expect("unable to serialize"),
                );
            }
        }

        let new_json =
            serde_json::to_string_pretty(&json_data).expect("couldn't serialize on save");

        if new_json != "{}" {
            fs::write(&save_file_path, new_json).expect("couldn't save new json");
        } else {
            // if there previously were entries that got deleted on the current save, resulting in the month store
            // becoming empty, delete the file
            if save_file_path.exists() {
                fs::remove_file(save_file_path).expect("couldn't remove existing json");
            }
        }
    }
}

impl WordCount for MonthStore {
    fn reload_current_counts(&mut self) {
        if self.is_word_count_in_sync() {
            return;
        }

        let mut day_diffs = vec![];

        for day in &mut self.days {
            let diff = day.update_word_count();

            if !diff.is_empty() {
                day_diffs.push(diff);
            }
        }

        for diff in day_diffs {
            for (word, diff_count) in diff {
                self.word_counts.insert_or_add(&word, diff_count);
            }
        }

        let char_count = self
            .days
            .iter()
            .map(|day_store| day_store.total_char_count())
            .sum();
        self.word_counts.set_total_char_count(char_count);
    }

    fn is_word_count_in_sync(&mut self) -> bool {
        let current_sync = self
            .days
            .iter_mut()
            .all(|day_store| day_store.is_word_count_in_sync());

        self.word_counts.set_sync(current_sync);

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

impl TimedWordCount for MonthStore {
    fn average_words(&self) -> f64 {
        let average_words = (self.total_word_count() as f64) / (self.edited_day_count() as f64);

        if average_words.is_finite() {
            average_words
        } else {
            0.0
        }
    }

    fn average_chars(&self) -> f64 {
        let average_chars = (self.total_char_count() as f64) / (self.edited_day_count() as f64);

        if average_chars.is_finite() {
            average_chars
        } else {
            0.0
        }
    }
}
