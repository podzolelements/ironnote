use chrono::{DateTime, Datelike, Days, Local, Months, NaiveDate};
use core::panic;
use iced::{
    Event, Font, Subscription,
    event::listen_with,
    keyboard::{self},
    widget::{
        self, Row, Space, button, column, row,
        text::Wrapping,
        text_editor::{self, Action},
    },
};
use keybinds::Keybinds;
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
    keybinds: Keybinds<KeyboardAction>,
}

enum KeyboardAction {
    Save,
    BackspaceWord,
    BackspaceSentence,
    Delete,
    DeleteWord,
    DeleteSentence,
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
    Calender(CalenderMessage),
    KeyEvent(keyboard::Event),
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

    fn reload_date(&mut self, active_datetime: DateTime<Local>) {
        self.update_window_title();
        self.calender.update_calender_dates(active_datetime);
        self.load_active_entry();
    }

    fn ctrl_backspace(&mut self, stopping_chars: &[char]) {
        let (line_idx, char_idx) = self.content.cursor_position();

        let content_text = self.content.text();
        let char_line = content_text
            .lines()
            .nth(line_idx)
            .expect("couldn't extract character line");

        if char_idx == 0 {
            return;
        }

        let mut backspace_head = char_idx - 1;
        let mut should_backspace_next_char = true;

        while should_backspace_next_char {
            self.content
                .perform(Action::Edit(text_editor::Edit::Backspace));

            if backspace_head > 0 {
                backspace_head -= 1;
            } else {
                should_backspace_next_char = false;
            }

            let next_char_to_backspace = char_line
                .chars()
                .nth(backspace_head)
                .expect("couldn't get char from line");

            if stopping_chars.contains(&next_char_to_backspace) {
                should_backspace_next_char = false;
            }

            // if there is a consecutive sequence of the same Ctrl+Backspace
            // stopping character, keep going until the last one is hit
            if backspace_head > 0 {
                let test_delete_head = backspace_head - 1;
                let next_next_char = char_line
                    .chars()
                    .nth(test_delete_head)
                    .expect("couldn't get char from line");

                if next_next_char == next_char_to_backspace
                    && (stopping_chars.contains(&next_next_char))
                {
                    should_backspace_next_char = true;
                }
            }
        }
    }

    fn ctrl_delete(&mut self, stopping_chars: &[char]) {
        let (line_idx, char_idx) = self.content.cursor_position();

        let content_text = self.content.text();
        let Some(char_line) = content_text.lines().nth(line_idx) else {
            println!("triggering None on char_line");
            return;
        };

        if char_line.chars().count() == 0 {
            self.content
                .perform(Action::Edit(text_editor::Edit::Delete));
            return;
        }

        let mut delete_head = char_idx;
        let mut should_delete_next_char = true;

        while should_delete_next_char {
            self.content
                .perform(Action::Edit(text_editor::Edit::Delete));

            if delete_head < (char_line.chars().count() - 1) {
                delete_head += 1;
            } else {
                should_delete_next_char = false;
                continue;
            }

            let next_char_to_delete = char_line
                .chars()
                .nth(delete_head)
                .expect("couldn't get char from line");

            if stopping_chars.contains(&next_char_to_delete) {
                should_delete_next_char = false;
            }

            if delete_head < (char_line.chars().count() - 1) {
                let test_delete_head = delete_head + 1;
                let next_next_char = char_line
                    .chars()
                    .nth(test_delete_head)
                    .expect("couldn't get char from line");

                if next_next_char == next_char_to_delete
                    && (stopping_chars.contains(&next_next_char))
                {
                    should_delete_next_char = true;
                }
            }
        }
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

        let seachbar = widget::text_editor(&self.search_content)
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

        let input = widget::text_editor(&self.content)
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
                self.reload_date(self.active_date_time);
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

                self.reload_date(self.active_date_time);
            }
            Message::JumpToToday => {
                self.active_date_time = Local::now();
                self.reload_date(self.active_date_time);
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
            Message::Calender(calmes) => match calmes {
                CalenderMessage::DayButton(new_day, month) => {
                    if self.edited_active_day {
                        self.save_active_entry();
                        self.edited_active_day = false;
                    }

                    match month {
                        calender::Month::LastMonth => {
                            let days_in_last_month: u32;

                            if self.active_date_time.month() == 1 {
                                days_in_last_month = 31;
                            } else {
                                let nd = NaiveDate::from_ymd_opt(
                                    self.active_date_time.year(),
                                    self.active_date_time.month() - 1,
                                    1,
                                )
                                .expect("bad date");

                                days_in_last_month = nd.num_days_in_month() as u32;
                            }

                            let days_to_go_back =
                                (days_in_last_month - new_day) + self.active_date_time.day();

                            self.active_date_time = self
                                .active_date_time
                                .checked_sub_days(Days::new(days_to_go_back as u64))
                                .expect("couldn't go into the past");
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
                            let days_to_go_forward = (self.active_date_time.num_days_in_month()
                                as u64
                                - self.active_date_time.day() as u64)
                                + new_day as u64;

                            self.active_date_time = self
                                .active_date_time
                                .checked_add_days(Days::new(days_to_go_forward))
                                .expect("couldn't go into the future");
                        }
                    }

                    self.reload_date(self.active_date_time);
                }
                CalenderMessage::BackMonth => {
                    self.active_date_time = self
                        .active_date_time
                        .checked_sub_months(Months::new(1))
                        .expect("couldn't go back a month");

                    self.reload_date(self.active_date_time);
                }
                CalenderMessage::ForwardMonth => {
                    self.active_date_time = self
                        .active_date_time
                        .checked_add_months(Months::new(1))
                        .expect("couldn't go forward a month");

                    self.reload_date(self.active_date_time);
                }
                CalenderMessage::BackYear => {
                    self.active_date_time = self
                        .active_date_time
                        .checked_sub_months(Months::new(12))
                        .expect("couldn't go back a year");

                    self.reload_date(self.active_date_time);
                }
                CalenderMessage::ForwardYear => {
                    self.active_date_time = self
                        .active_date_time
                        .checked_add_months(Months::new(12))
                        .expect("couldn't go forward a year");

                    self.reload_date(self.active_date_time);
                }
            },
            Message::KeyEvent(event) => {
                if let Some(action) = self.keybinds.dispatch(event) {
                    match action {
                        KeyboardAction::Save => {
                            self.save_active_entry();
                        }
                        KeyboardAction::BackspaceWord => {
                            let stopping_chars = [
                                ' ', '.', '!', '?', ',', '-', '_', '\"', ';', ':', '(', ')', '{',
                                '}', '[', ']',
                            ];
                            self.ctrl_backspace(&stopping_chars);
                        }
                        KeyboardAction::BackspaceSentence => {
                            let stopping_chars = ['.', '!', '?', ',', '\"', ';', ':'];
                            self.ctrl_backspace(&stopping_chars);
                        }
                        KeyboardAction::Delete => {
                            // not sure why the text_editor action handler doesn't do this on its own
                            self.content
                                .perform(Action::Edit(text_editor::Edit::Delete));
                        }
                        KeyboardAction::DeleteWord => {
                            let stopping_chars = [
                                ' ', '.', '!', '?', ',', '-', '_', '\"', ';', ':', '(', ')', '{',
                                '}', '[', ']',
                            ];
                            self.ctrl_delete(&stopping_chars);
                        }
                        KeyboardAction::DeleteSentence => {
                            let stopping_chars = ['.', '!', '?', ',', '\"', ';', ':'];
                            self.ctrl_delete(&stopping_chars);
                        }
                    }
                }
            }
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        listen_with(|event, _, _| match event {
            Event::Keyboard(event) => Some(Message::KeyEvent(event)),
            _ => None,
        })
    }
}

impl Default for App {
    fn default() -> Self {
        let mut keybinds = Keybinds::default();

        keybinds
            .bind("Ctrl+s", KeyboardAction::Save)
            .expect("couldn't bind Ctrl+s");
        keybinds
            .bind("Ctrl+Backspace", KeyboardAction::BackspaceWord)
            .expect("couldn't bind Ctrl+Backspace");
        keybinds
            .bind("Ctrl+Shift+Backspace", KeyboardAction::BackspaceSentence)
            .expect("couldn't bind Ctrl+Shift+Backspace");
        // text_editor delete key doesn't seem to get handled right, so we need to manually implement it
        keybinds
            .bind("Delete", KeyboardAction::Delete)
            .expect("couldn't bind Delete");
        keybinds
            .bind("Ctrl+Delete", KeyboardAction::DeleteWord)
            .expect("couldn't bind Ctrl+Delete");
        keybinds
            .bind("Ctrl+Shift+Delete", KeyboardAction::DeleteSentence)
            .expect("couldn't bind Ctrl+Shift+Delete");

        let mut df = Self {
            window_title: String::default(),
            active_date_time: Local::now(),
            edited_active_day: false,
            content: text_editor::Content::default(),
            search_content: text_editor::Content::default(),
            calender: Calender::default(),
            keybinds: keybinds,
        };

        df.update(Message::JumpToToday);

        df
    }
}

fn main() -> iced::Result {
    iced::application(App::title, App::update, App::view)
        .subscription(App::subscription)
        .run()
}
