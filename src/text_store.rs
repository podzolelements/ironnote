use crate::{
    filetools::{self, setup_savedata_dirs},
    logbox::LOGBOX,
    misc_tools,
    statistics::{BoundedDateStats, Stats},
};
use chrono::{DateTime, Datelike, Days, Local};
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{fs, sync::LazyLock};

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct DayStore {
    date: String,
    entry_text: String,
    modified: bool,
}

impl DayStore {
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
}

impl Stats for DayStore {
    fn word_count(&self) -> usize {
        self.entry_text.split_whitespace().count()
    }

    fn char_count(&self) -> usize {
        self.entry_text.chars().count()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MonthStore {
    days: Vec<DayStore>,
    month: String,
    days_in_month: u8,
}

impl Default for MonthStore {
    fn default() -> Self {
        let days = vec![DayStore::default(); 31];
        Self {
            days,
            month: Default::default(),
            days_in_month: Default::default(),
        }
    }
}

impl MonthStore {
    pub fn get_day_store(&self, day: usize) -> DayStore {
        self.days[day].clone()
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

    pub fn set_day_text(&mut self, day: usize, text: String) {
        self.days[day].set_day_text(text);
        self.days[day].modified = true;
    }

    pub fn days(&self) -> impl DoubleEndedIterator<Item = &DayStore> {
        self.days.iter()
    }

    pub fn load_month(&mut self, date: DateTime<Local>) {
        let date_rfc3339 = date.to_rfc3339();
        self.month = (date_rfc3339[0..7]).to_string();
        self.days_in_month = date.num_days_in_month();

        let filename = self.month.clone() + ".json";
        let save_path = setup_savedata_dirs(&filename);

        self.days.clear();

        match fs::exists(&save_path) {
            Err(_) => {
                panic!("couldn't determine if file exists");
            }
            Ok(file_exists) => {
                if !file_exists {
                    let mut iterative_date =
                        date.with_day(1).expect("couldn't go to start of month");

                    for _ in 0..self.days_in_month {
                        let new_date_3339 = iterative_date.to_rfc3339();
                        let new_date = &new_date_3339[0..10];

                        let new_day_store = DayStore {
                            date: new_date.to_string(),
                            entry_text: String::default(),
                            modified: false,
                        };
                        self.days.push(new_day_store);

                        iterative_date = iterative_date
                            .checked_add_days(Days::new(1))
                            .expect("couldn't add day");
                    }

                    return;
                }
            }
        }

        let month_json = fs::read_to_string(&save_path).expect("couldn't read json into string");

        let data: serde_json::Map<String, Value> =
            serde_json::from_str(&month_json).expect("couldn't deserialize");

        let mut iterative_date = date.with_day(1).expect("couldn't go to start of month");

        for _ in 0..self.days_in_month {
            let new_date_3339 = iterative_date.to_rfc3339();
            let new_date = &new_date_3339[0..10];

            let entry_text = if let Some(entry_value) = data.get(new_date) {
                let entry: String =
                    serde_json::from_value(entry_value.clone()).expect("invalid entry format");
                entry
            } else {
                "".to_string()
            };

            let new_day_store = DayStore {
                date: new_date.to_string(),
                entry_text,
                modified: false,
            };
            self.days.push(new_day_store);

            iterative_date = iterative_date
                .checked_add_days(Days::new(1))
                .expect("couldn't add day");
        }
    }

    pub fn save_month(&self) {
        let filename = self.month.clone() + ".json";
        let save_path = setup_savedata_dirs(&filename);

        let month_json = if let Ok(existing_savedata) = fs::read_to_string(&save_path) {
            existing_savedata
        } else {
            "{}".to_string()
        };

        let mut data: serde_json::Map<String, Value> =
            serde_json::from_str(&month_json).expect("couldn't deserialize");

        for i in 0..(self.days_in_month as usize) {
            let new_entry = self.days[i].clone();

            if !new_entry.modified {
                continue;
            }

            if !new_entry.contains_entry() {
                data.remove_entry(&new_entry.date);
            } else {
                data.insert(
                    new_entry.date.clone(),
                    serde_json::to_value(new_entry.entry_text).unwrap(),
                );
            }
        }

        let new_json = serde_json::to_string_pretty(&data).expect("couldn't serialize on save");

        if new_json != "{}" {
            fs::write(&save_path, new_json).expect("couldn't save new json");
        } else {
            // if there previously were entries that got deleted on the current save, resulting in the month store
            // becoming empty, delete the file
            if save_path.exists() {
                fs::remove_file(save_path).expect("couldn't remove existing json");
            }
        }

        LOGBOX
            .write()
            .expect("couldn't get logbox write")
            .log("Saved");
    }
}

impl Stats for MonthStore {
    fn word_count(&self) -> usize {
        self.days().map(|day_store| day_store.word_count()).sum()
    }

    fn char_count(&self) -> usize {
        self.days().map(|day_store| day_store.char_count()).sum()
    }
}

impl BoundedDateStats for MonthStore {
    fn average_words(&self) -> f64 {
        self.word_count() as f64 / self.edited_day_count() as f64
    }

    fn average_chars(&self) -> f64 {
        self.char_count() as f64 / self.edited_day_count() as f64
    }
}

#[derive(Debug, Default)]
pub struct GlobalStore {
    entries: Vec<MonthStore>,
}

impl GlobalStore {
    pub fn load_all(&mut self) {
        self.entries.clear();

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

                let mut month_store = MonthStore::default();
                month_store.load_month(date_time);

                self.entries.push(month_store);
            }
        }

        self.entries
            .sort_by_key(|month_store| month_store.month.clone());
    }

    pub fn push_month_store(&mut self, new_month_store: MonthStore) {
        self.entries
            .retain(|month_store| month_store.month != new_month_store.month.clone());

        self.entries.push(new_month_store);

        self.entries
            .sort_by_key(|month_store| month_store.month.clone());
    }

    pub fn month_stores(&self) -> impl DoubleEndedIterator<Item = &MonthStore> {
        self.entries.iter()
    }

    pub fn edited_day_count(&self) -> usize {
        self.month_stores().map(|ms| ms.edited_day_count()).sum()
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

impl Stats for GlobalStore {
    fn word_count(&self) -> usize {
        self.month_stores().map(|ms| ms.word_count()).sum()
    }

    fn char_count(&self) -> usize {
        self.month_stores().map(|ms| ms.char_count()).sum()
    }
}

impl BoundedDateStats for GlobalStore {
    fn average_words(&self) -> f64 {
        self.word_count() as f64 / self.edited_day_count() as f64
    }

    fn average_chars(&self) -> f64 {
        self.char_count() as f64 / self.edited_day_count() as f64
    }
}
