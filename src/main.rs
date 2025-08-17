use chrono::{DateTime, Datelike, Days, Local};
use core::panic;
use iced::{
    Font, Subscription,
    keyboard::{self, Key, Modifiers},
    widget::{Row, Space, button, column, row, text::Wrapping, text_editor},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{fs, path::PathBuf};

mod calender;
use calender::Calender;

use crate::calender::CalenderMessage;

struct App {
    window_title: String,
    content: text_editor::Content,
    edited_active_day: bool,
    search_content: text_editor::Content,
    active_date_time: DateTime<Local>,
    calender: Calender,
}

#[derive(Serialize, Deserialize, Debug)]
struct Entry {
    text: String,
}

#[derive(Debug, Clone)]
pub enum Message {
    BackOneDay,
    ForwardOneDay,
    JumpToToday,
    UpdateCalender,
    Edit(text_editor::Action),
    EditSearch(text_editor::Action),
    TempTopBarMessage,
    Save,
    Calender(CalenderMessage),
}

impl App {
    /// returns (date_key, month_json, save_path)
    fn prepare_rw_action(&self) -> (String, String, PathBuf) {
        let date_rfc3339 = self.active_date_time.to_rfc3339();
        let date_key = &date_rfc3339[0..10];
        let year_month = &date_rfc3339[0..7];

        let filename = year_month.to_string() + ".json";

        let mut save_path = PathBuf::new();

        let home = dirs::home_dir().expect("Couldn't open home dir!");
        save_path.push(home);
        save_path.push(".ironnote");
        save_path.push("data");
        save_path.push(filename);

        let save_parent_dir = save_path.parent().expect("save path has no parent???");
        fs::create_dir_all(&save_parent_dir).expect("couldn't create parent dirs");

        match fs::exists(&save_path) {
            Err(_) => {
                panic!("couldn't determine if file exists");
            }
            Ok(file_exists) => {
                if !file_exists {
                    fs::write(&save_path, "{}").expect("couldn't create month file");
                }
            }
        }

        let month_json = fs::read_to_string(&save_path).expect("couldn't read json into string");

        (date_key.to_string(), month_json, save_path)
    }

    fn load_active_entry(&mut self) {
        let (date_key, month_json, _) = self.prepare_rw_action();

        let data: serde_json::Map<String, Value> =
            serde_json::from_str(&month_json).expect("couldn't deserialize");

        if let Some(entry_value) = data.get(&date_key) {
            let entry: Entry =
                serde_json::from_value(entry_value.clone()).expect("invalid entry format");
            self.content = text_editor::Content::with_text(&entry.text);
        } else {
            self.content = text_editor::Content::with_text("");
        }

        println!("loaded {}", date_key);
    }

    fn save_active_entry(&self) {
        let (date_key, month_json, save_path) = self.prepare_rw_action();

        let mut data: serde_json::Map<String, Value> =
            serde_json::from_str(&month_json).expect("couldn't deserialize");

        let new_entry = Entry {
            text: self.content.text(),
        };

        data.insert(
            date_key.to_string(),
            serde_json::to_value(new_entry).unwrap(),
        );

        let new_json = serde_json::to_string_pretty(&data).expect("couldn't serialize on save");
        fs::write(&save_path, new_json).expect("couldn't save new json");

        println!("saved {}", date_key);
    }

    fn update_window_title(&mut self) {
        let formated_date = self.active_date_time.format("%A, %B %d, %Y").to_string();
        let new_title = "ironnote - ".to_string() + &formated_date;

        self.window_title = new_title;
    }
}

impl App {
    fn title(&self) -> String {
        self.window_title.clone()
    }

    pub fn view(&'_ self) -> Row<'_, Message> {
        let back_button = button("<--").on_press(Message::BackOneDay).height(100);
        let today_button = button("Today").on_press(Message::JumpToToday).height(100);
        let forward_button = button("-->").on_press(Message::ForwardOneDay).height(100);

        let hspace = Space::new(5, 5);
        let hspace2 = Space::new(5, 5);

        let buttonbar = row![back_button, hspace, today_button, hspace2, forward_button];

        let cal = Calender::view(&self.calender);
        let temp_calender_bar = row![cal];

        let seachbar = text_editor(&self.search_content)
            .placeholder("Search entries...")
            .on_action(Message::EditSearch)
            .size(13)
            .font(Font::DEFAULT)
            .wrapping(Wrapping::None)
            .width(250);

        let left_ui = column![buttonbar, temp_calender_bar, seachbar,];

        let right_top_bar = row![
            button("test button 0")
                .on_press(Message::TempTopBarMessage)
                .height(100),
            button("test button 1")
                .on_press(Message::TempTopBarMessage)
                .height(100),
            button("test button 2")
                .on_press(Message::TempTopBarMessage)
                .height(100),
        ];

        let input = text_editor(&self.content)
            .placeholder("Type today's log...")
            .on_action(Message::Edit)
            .size(13)
            .font(Font::DEFAULT)
            .wrapping(Wrapping::Word)
            .height(iced::Length::Fill);

        let right_ui = column![right_top_bar, input];

        let layout = row![left_ui, right_ui];

        layout
    }

    pub fn update(&mut self, message: Message) {
        match message {
            Message::BackOneDay => {
                if self.edited_active_day {
                    self.save_active_entry();
                    self.edited_active_day = false;
                }
                self.active_date_time = self
                    .active_date_time
                    .checked_sub_days(Days::new(1))
                    .expect("failed to go to previous day");
                self.update_window_title();
                self.calender.update_calender_dates(self.active_date_time);
                self.load_active_entry();
            }
            Message::ForwardOneDay => {
                if self.edited_active_day {
                    self.save_active_entry();
                    self.edited_active_day = false;
                }
                self.active_date_time = self
                    .active_date_time
                    .checked_add_days(Days::new(1))
                    .expect("failed to go to next day");
                self.update_window_title();
                self.calender.update_calender_dates(self.active_date_time);
                self.load_active_entry();
            }
            Message::JumpToToday => {
                self.active_date_time = Local::now();
                self.update_window_title();
                self.calender.update_calender_dates(self.active_date_time);
                self.load_active_entry();
            }
            Message::UpdateCalender => {
                println!("cal");
            }
            Message::Edit(action) => {
                match &action {
                    text_editor::Action::Edit(_edit) => {
                        self.edited_active_day = true;
                    }
                    _ => {}
                }

                self.content.perform(action);
            }
            Message::EditSearch(action) => {
                self.search_content.perform(action);
            }
            Message::TempTopBarMessage => {
                println!("topbar");
            }
            Message::Save => {
                self.save_active_entry();
            }
            Message::Calender(calmes) => {
                if self.edited_active_day {
                    self.save_active_entry();
                    self.edited_active_day = false;
                }

                let CalenderMessage::DayButton(new_day, month) = calmes;

                match month {
                    calender::Month::LastMonth => {
                        todo!();
                    }
                    calender::Month::CurrentMonth => {
                        let delta_day = (new_day as i32) - (self.active_date_time.day() as i32);

                        let mag_delta_day = delta_day.abs() as u64;

                        if delta_day == 0 {
                            return;
                        }
                        if delta_day < 0 {
                            self.active_date_time = self
                                .active_date_time
                                .checked_sub_days(Days::new(mag_delta_day))
                                .expect("couldn't jump into the past");
                        } else {
                            self.active_date_time = self
                                .active_date_time
                                .checked_add_days(Days::new(mag_delta_day))
                                .expect("couldn't jump into the future");
                        }
                    }
                    calender::Month::NextMonth => {
                        todo!();
                    }
                }

                self.update_window_title();
                self.calender.update_calender_dates(self.active_date_time);
            }
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        fn handle_hotkey(key: Key, modifiers: Modifiers) -> Option<Message> {
            match (modifiers, key.as_ref()) {
                (Modifiers::CTRL, Key::Character("s")) => Some(Message::Save),
                _ => None,
            }
        }

        keyboard::on_key_press(handle_hotkey)
    }
}

impl Default for App {
    fn default() -> Self {
        Self {
            window_title: "ironnote".to_string(),
            active_date_time: Local::now(),
            edited_active_day: false,
            content: text_editor::Content::default(),
            search_content: text_editor::Content::default(),
            calender: Calender::default(),
        }
    }
}

fn main() -> iced::Result {
    iced::application(App::title, App::update, App::view)
        .subscription(App::subscription)
        .run()
}
