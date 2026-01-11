use crate::{
    day_store::DayStore,
    misc_tools::{self},
    month_store::MonthStore,
    user_preferences::preferences,
    word_count::{TimedWordCount, WordCount, WordCounts},
};
use chrono::{Datelike, Days, Local, Months, NaiveDate};
use regex::Regex;
use std::sync::LazyLock;

#[derive(Debug)]
pub struct GlobalStore {
    entries: Vec<MonthStore>,
    current_date: NaiveDate,
    word_counts: WordCounts,
}

impl Default for GlobalStore {
    fn default() -> Self {
        let mut global_store = Self {
            entries: Vec::default(),
            current_date: NaiveDate::default(),
            word_counts: WordCounts::default(),
        };

        global_store.set_current_store_date(Local::now().date_naive());

        global_store
    }
}

impl GlobalStore {
    /// changes the current date, adding the month if it doesn't exist
    pub fn set_current_store_date(&mut self, new_date: NaiveDate) {
        self.current_date = new_date;

        if !self.entries.iter().any(|month_store| {
            month_store.get_yyyy_mm() == self.current_date.format("%Y-%m").to_string()
        }) {
            let first_of_month = self
                .current_date
                .with_day(1)
                .expect("invalid first of month");

            self.push_month_store(MonthStore::new(first_of_month));
        }
    }

    /// mutable access to the current month store based on the current date in the GlobalStore
    pub fn month_mut(&mut self) -> &mut MonthStore {
        let month_index = self
            .entries
            .iter()
            .position(|month_store| {
                month_store.get_yyyy_mm() == self.current_date.format("%Y-%m").to_string()
            })
            .expect("month doesn't exist");

        &mut self.entries[month_index]
    }

    /// mutable access to the current day store based on the current date in the global store
    pub fn day_mut(&mut self) -> &mut DayStore {
        let day_index = self.current_date.day0() as usize;

        self.month_mut().day_mut(day_index)
    }

    /// reference to the current month store based on the current date in the global store
    pub fn month(&self) -> &MonthStore {
        let month_index = self
            .entries
            .iter()
            .position(|month_store| {
                month_store.get_yyyy_mm() == self.current_date.format("%Y-%m").to_string()
            })
            .expect("month doesn't exist");

        &self.entries[month_index]
    }

    /// reference to the current day store based on the current date in the global store
    pub fn day(&self) -> &DayStore {
        let day_index = self.current_date.day0() as usize;

        self.month().day(day_index)
    }

    /// returns the current date of the global store
    pub fn current_date(&self) -> NaiveDate {
        self.current_date
    }

    /// loads all entries from disk, overwriting any existing data in the store
    pub fn load_all(&mut self) {
        static FILENAME_REGEX: LazyLock<Regex> =
            LazyLock::new(|| Regex::new(r"\d\d\d\d-\d\d\.json").expect("couldn't create regex"));

        let savedata_dir = preferences().paths.savedata_dir();

        if let Ok(savedata_entries) = savedata_dir.read_dir() {
            for dir_entry in savedata_entries.flatten() {
                if !dir_entry.path().is_file() {
                    continue;
                }

                let filename = dir_entry
                    .file_name()
                    .into_string()
                    .expect("couldn't convert filename to string");

                if !FILENAME_REGEX.is_match(&filename) {
                    continue;
                }

                let file_date_first = filename[0..7].to_string() + "-01";
                let first_of_month = misc_tools::yyyy_mm_dd_string_to_date(&file_date_first);

                let mut month_store = MonthStore::new(first_of_month);
                month_store.load_month(first_of_month);

                self.add_month_to_store(month_store);
            }
        }

        self.add_empty_months();
    }

    /// writes the store to disk
    pub fn save_all(&self) {
        for month in &self.entries {
            month.save_month();
        }
    }

    /// since adding months can be discontinuous in time, the missing ones should be added to ensure time continuity
    fn add_empty_months(&mut self) {
        self.sort_month_stores();

        let current_months: Vec<NaiveDate> = self
            .entries
            .iter()
            .map(|month_store| month_store.first_of_month())
            .collect();

        if current_months.len() < 2 {
            return;
        }

        let start_month = current_months.first().expect("couldn't get start month");
        let mut current_month = *start_month;
        let end_month = current_months.last().expect("couldn't get end month");

        let mut missing_months = vec![];

        while current_month < *end_month {
            if !current_months.contains(&current_month) {
                missing_months.push(current_month);
            }

            current_month = current_month
                .checked_add_months(Months::new(1))
                .expect("couldn't add month");
        }

        for month_date in missing_months {
            let first_of_month = month_date.with_day(1).expect("invalid first of month");
            self.add_month_to_store(MonthStore::new(first_of_month));
        }

        self.sort_month_stores();
    }

    /// adds a month into the store. if the new month is dated the same as an existing entry, the existing one is
    /// overwritten by the new one
    fn add_month_to_store(&mut self, new_month_store: MonthStore) {
        self.entries.retain(|month_store| {
            month_store.get_yyyy_mm() != new_month_store.get_yyyy_mm().clone()
        });

        self.entries.push(new_month_store);
    }

    pub fn push_month_store(&mut self, new_month_store: MonthStore) {
        self.add_month_to_store(new_month_store);
        self.add_empty_months();
        self.sort_month_stores();
    }

    fn sort_month_stores(&mut self) {
        self.entries
            .sort_by_key(|month_store| month_store.get_yyyy_mm());
    }

    pub fn month_stores(&self) -> impl DoubleEndedIterator<Item = &MonthStore> {
        self.entries.iter()
    }

    /// retrieves the day store at the given date, if it exists
    pub fn get_day(&self, date: NaiveDate) -> Option<DayStore> {
        let year_month = date.format("%Y-%m").to_string();
        let day = date.day0() as usize;

        for month_store in self.month_stores() {
            if month_store.get_yyyy_mm() == year_month {
                return Some(month_store.day(day).clone());
            }
        }

        None
    }

    pub fn edited_day_count(&self) -> usize {
        self.month_stores().map(|ms| ms.edited_day_count()).sum()
    }

    /// returns the date of the first edited day in the store, if it exists
    pub fn first_edited_day(&self) -> Option<NaiveDate> {
        for month in self.month_stores() {
            for day in month.days() {
                if day.contains_entry() {
                    return Some(day.date());
                }
            }
        }

        None
    }

    /// returns the date of the last edited day in the store, if it exists
    pub fn last_edited_day(&self) -> Option<NaiveDate> {
        for month in self.month_stores().rev() {
            for day in month.days().rev() {
                if day.contains_entry() {
                    return Some(day.date());
                }
            }
        }

        None
    }

    /// returns the previously edited day relative to the given date, if it exists
    pub fn get_previous_edited_day(&self, active_entry: NaiveDate) -> Option<NaiveDate> {
        let earliest_entry = self.first_edited_day()?;

        if active_entry <= earliest_entry {
            return None;
        }

        let mut test_date = active_entry
            .checked_sub_days(Days::new(1))
            .expect("couldn't subtract day");

        while test_date >= earliest_entry {
            if !self
                .get_day(test_date)
                .is_some_and(|ds| ds.contains_entry())
            {
                test_date = test_date
                    .checked_sub_days(Days::new(1))
                    .expect("couldn't subtract day");
                continue;
            }

            return Some(test_date);
        }

        None
    }

    /// returns the next edited day relative to the given date, if it exists
    pub fn get_next_edited_day(&self, active_entry: NaiveDate) -> Option<NaiveDate> {
        let latest_entry = self.last_edited_day()?;

        if active_entry >= latest_entry {
            return None;
        }

        let mut test_date = active_entry
            .checked_add_days(Days::new(1))
            .expect("couldn't add day");

        while test_date <= latest_entry {
            if !self
                .get_day(test_date)
                .is_some_and(|ds| ds.contains_entry())
            {
                test_date = test_date
                    .checked_add_days(Days::new(1))
                    .expect("couldn't add day");

                continue;
            }

            return Some(test_date);
        }

        None
    }

    /// gets the number of the longest streak of consecutively edited days
    pub fn longest_streak(&self) -> u32 {
        let mut longest_found_streak = 0;
        let mut current_search_streak = 0;

        for month in self.month_stores() {
            for day in month.days() {
                if day.contains_entry() {
                    current_search_streak += 1;
                } else {
                    if current_search_streak > longest_found_streak {
                        longest_found_streak = current_search_streak;
                    }

                    current_search_streak = 0;
                }
            }
        }

        longest_found_streak
    }

    /// gets the number of consecutive edited days that connect to the last (most recent) edited day in the global store
    pub fn current_streak(&self) -> u32 {
        let mut current_streak = 0;
        let mut found_most_recent_day = false;

        for month in self.month_stores().rev() {
            for day in month.days().rev() {
                if !day.contains_entry() && !found_most_recent_day {
                    continue;
                }
                if !day.contains_entry() && found_most_recent_day {
                    return current_streak;
                }

                if day.contains_entry() {
                    found_most_recent_day = true;
                    current_streak += 1;
                }
            }
        }

        current_streak
    }
}

impl WordCount for GlobalStore {
    fn reload_current_counts(&mut self) {
        if self.is_word_count_in_sync() {
            return;
        }

        let mut month_diffs = vec![];

        for month in &mut self.entries {
            let diff = month.update_word_count();

            if !diff.is_empty() {
                month_diffs.push(diff);
            }
        }

        for diff in month_diffs {
            for (word, diff_count) in diff {
                self.word_counts.insert_or_add(&word, diff_count);
            }
        }

        let char_count = self
            .entries
            .iter()
            .map(|month_store| month_store.total_char_count())
            .sum();
        self.word_counts.set_total_char_count(char_count);
    }

    fn is_word_count_in_sync(&mut self) -> bool {
        let current_sync = self
            .entries
            .iter_mut()
            .all(|month_store| month_store.is_word_count_in_sync());

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

impl TimedWordCount for GlobalStore {
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
