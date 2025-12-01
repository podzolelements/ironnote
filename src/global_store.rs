use crate::{
    day_store::DayStore,
    filetools,
    misc_tools::{self, string_to_datetime},
    month_store::MonthStore,
    word_count::{TimedWordCount, WordCount, WordCounts},
};
use chrono::{DateTime, Datelike, Days, Local, Months, NaiveDate};
use regex::Regex;
use std::sync::LazyLock;

#[derive(Debug)]
pub struct GlobalStore {
    entries: Vec<MonthStore>,
    date_time: DateTime<Local>,
    word_counts: WordCounts,
}

impl Default for GlobalStore {
    fn default() -> Self {
        let mut global_store = Self {
            entries: Vec::default(),
            date_time: DateTime::default(),
            word_counts: WordCounts::default(),
        };

        global_store.set_current_store_date(Local::now());

        global_store
    }
}

impl GlobalStore {
    /// changes the active date_time, adding the month if it doesn't exist
    pub fn set_current_store_date(&mut self, new_date_time: DateTime<Local>) {
        self.date_time = new_date_time;

        if !self.entries.iter().any(|month_store| {
            month_store.get_yyyy_mm() == self.date_time.format("%Y-%m").to_string()
        }) {
            self.push_month_store(MonthStore::new(self.date_time));
        }
    }

    /// mutable reference to the current month store based on the active date_time
    pub fn month_mut(&mut self) -> &mut MonthStore {
        let month_index = self
            .entries
            .iter()
            .position(|month_store| {
                month_store.get_yyyy_mm() == self.date_time.format("%Y-%m").to_string()
            })
            .expect("month doesn't exist");

        &mut self.entries[month_index]
    }

    /// mutable reference to the current day store based on the active date_time
    pub fn day_mut(&mut self) -> &mut DayStore {
        let day_index = self.date_time.day0() as usize;

        self.month_mut().day_mut(day_index)
    }

    /// reference to the current month store based on the active date_time
    pub fn month(&self) -> &MonthStore {
        let month_index = self
            .entries
            .iter()
            .position(|month_store| {
                month_store.get_yyyy_mm() == self.date_time.format("%Y-%m").to_string()
            })
            .expect("month doesn't exist");

        &self.entries[month_index]
    }

    /// reference to the current day store based on the active date_time
    pub fn day(&self) -> &DayStore {
        let day_index = self.date_time.day0() as usize;

        self.month().day(day_index)
    }

    /// returns the active date_time of the global store
    pub fn date_time(&self) -> DateTime<Local> {
        self.date_time
    }

    /// loads all entries from disk, overwriting any existing data in the store
    pub fn load_all(&mut self) {
        static FILENAME_REGEX: LazyLock<Regex> =
            LazyLock::new(|| Regex::new(r"\d\d\d\d-\d\d\.json").expect("couldn't create regex"));

        let filepath = filetools::savedata_path();

        if let Ok(files_in_savedir) = filepath.read_dir() {
            for file in files_in_savedir.flatten() {
                if !file.path().is_file() {
                    continue;
                }

                let filename = file
                    .file_name()
                    .into_string()
                    .expect("couldn't convert filename to string");

                if !FILENAME_REGEX.is_match(&filename) {
                    continue;
                }

                let file_date = filename[0..7].to_string() + "-01";
                let date_time = misc_tools::string_to_datetime(&file_date);

                let mut month_store = MonthStore::new(date_time);
                month_store.load_month(date_time);

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
            .map(|month_store| {
                string_to_datetime(&(month_store.get_yyyy_mm() + "-01")).date_naive()
            })
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
            let month_datetime = string_to_datetime(&month_date.to_string());
            self.add_month_to_store(MonthStore::new(month_datetime));
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
    pub fn get_day(&self, datetime: DateTime<Local>) -> Option<DayStore> {
        let year_month = datetime.format("%Y-%m").to_string();
        let day = datetime.day0() as usize;

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

    /// returns the datetime of the first edited day in the store, if it exists
    pub fn first_edited_day(&self) -> Option<DateTime<Local>> {
        for month in self.month_stores() {
            for day in month.days() {
                if day.contains_entry() {
                    return Some(string_to_datetime(&day.date()));
                }
            }
        }

        None
    }

    /// returns the datetime of the last edited day in the store, if it exists
    pub fn last_edited_day(&self) -> Option<DateTime<Local>> {
        for month in self.month_stores().rev() {
            for day in month.days().rev() {
                if day.contains_entry() {
                    return Some(string_to_datetime(&day.date()));
                }
            }
        }

        None
    }

    /// returns the previously edited day relative to the given datetime, if it exists
    pub fn get_previous_edited_day(
        &self,
        active_entry: DateTime<Local>,
    ) -> Option<DateTime<Local>> {
        let earliest_entry = self.first_edited_day()?;

        if active_entry.date_naive() <= earliest_entry.date_naive() {
            return None;
        }

        let mut test_datetime = active_entry
            .checked_sub_days(Days::new(1))
            .expect("couldn't subtract day");

        while test_datetime.date_naive() >= earliest_entry.date_naive() {
            if !self
                .get_day(test_datetime)
                .is_some_and(|ds| ds.contains_entry())
            {
                test_datetime = test_datetime
                    .checked_sub_days(Days::new(1))
                    .expect("couldn't subtract day");
                continue;
            }

            return Some(test_datetime);
        }

        None
    }

    /// returns the next edited day relative to the given datetime, if it exists
    pub fn get_next_edited_day(&self, active_entry: DateTime<Local>) -> Option<DateTime<Local>> {
        let latest_entry = self.last_edited_day()?;

        if active_entry.date_naive() >= latest_entry.date_naive() {
            return None;
        }

        let mut test_datetime = active_entry
            .checked_add_days(Days::new(1))
            .expect("couldn't add day");

        while test_datetime.date_naive() <= latest_entry.date_naive() {
            if !self
                .get_day(test_datetime)
                .is_some_and(|ds| ds.contains_entry())
            {
                test_datetime = test_datetime
                    .checked_add_days(Days::new(1))
                    .expect("couldn't add day");

                continue;
            }

            return Some(test_datetime);
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
        (self.total_word_count() as f64) / (self.edited_day_count() as f64)
    }

    fn average_chars(&self) -> f64 {
        (self.total_char_count() as f64) / (self.edited_day_count() as f64)
    }
}
