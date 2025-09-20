use crate::filetools::setup_savedata_dirs;
use chrono::{DateTime, Datelike, Days, Local};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;

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
}

#[derive(Serialize, Deserialize, Debug)]
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

    pub fn set_day_text(&mut self, day: usize, text: String) {
        self.days[day].set_day_text(text);
        self.days[day].modified = true;
    }

    pub fn days(&self) -> &Vec<DayStore> {
        &self.days
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

            if new_entry.entry_text.is_empty() || new_entry.entry_text == "\n" {
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
            println!("saved {}", self.month);
        } else {
            // if there previously were entries that got deleted on the current save, resulting in the month store
            // becoming empty, delete the file
            if save_path.exists() {
                fs::remove_file(save_path).expect("couldn't remove existing json");
            }
        }
    }
}
