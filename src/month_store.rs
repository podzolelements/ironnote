use crate::{
    day_store::DayStore,
    filetools::setup_savedata_dirs,
    logbox::LOGBOX,
    statistics::{BoundedDateStats, Stats},
};
use chrono::{DateTime, Datelike, Days, Local, NaiveDate};
use serde_json::Value;
use std::fs;

#[derive(Debug, Clone)]
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
    /// creates a new month store from the given naive_date
    pub fn new(naive_date: NaiveDate) -> Self {
        let days_in_month = naive_date.num_days_in_month();

        let days = vec![DayStore::default(); days_in_month as usize];
        let month = naive_date.format("%Y-%m").to_string();

        Self {
            days,
            month,
            days_in_month,
        }
    }

    pub fn day(&self, day: usize) -> &DayStore {
        &self.days[day]
    }

    pub fn day_mut(&mut self, day: usize) -> &mut DayStore {
        &mut self.days[day]
    }

    pub fn get_yyyy_mm(&self) -> String {
        self.month.clone()
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

                        let new_day_store = DayStore::new(new_date);
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

            let mut new_day_store = DayStore::new(new_date);
            new_day_store.set_day_text(entry_text);
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

            if !new_entry.modified() {
                continue;
            }

            if !new_entry.contains_entry() {
                data.remove_entry(&new_entry.date());
            } else {
                data.insert(
                    new_entry.date().clone(),
                    serde_json::to_value(new_entry.get_day_text()).unwrap(),
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
