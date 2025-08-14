#![allow(unused)]
use std::{fs, path::PathBuf};

use chrono::{DateTime, Datelike, Local};
use iced::{
    Font, Subscription,
    keyboard::{self, Key, Modifiers},
    widget::{Row, Space, button, column, row, text::Wrapping, text_editor},
};

struct App {
    text: String,
    content: text_editor::Content,
    search_content: text_editor::Content,
    active_date_time: DateTime<Local>,
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
}

impl App {
    pub fn view(&self) -> Row<Message> {
        let back_button = button("<--").on_press(Message::BackOneDay).height(100);
        let today_button = button("Today").on_press(Message::JumpToToday).height(100);
        let forward_button = button("-->").on_press(Message::ForwardOneDay).height(100);

        let hspace = Space::new(5, 5);
        let hspace2 = Space::new(5, 5);

        let buttonbar = row![back_button, hspace, today_button, hspace2, forward_button];


        let temp_calender_bar = row![
            column![
                button("12").on_press(Message::UpdateCalender),
                button("12").on_press(Message::UpdateCalender),
                button("12").on_press(Message::UpdateCalender),
                button("12").on_press(Message::UpdateCalender),
                button("12").on_press(Message::UpdateCalender),
                button("12").on_press(Message::UpdateCalender),
            ],
            column![
                button("12").on_press(Message::UpdateCalender),
                button("12").on_press(Message::UpdateCalender),
                button("12").on_press(Message::UpdateCalender),
                button("12").on_press(Message::UpdateCalender),
                button("12").on_press(Message::UpdateCalender),
                button("12").on_press(Message::UpdateCalender),
            ],
            column![
                button("12").on_press(Message::UpdateCalender),
                button("12").on_press(Message::UpdateCalender),
                button("12").on_press(Message::UpdateCalender),
                button("12").on_press(Message::UpdateCalender),
                button("12").on_press(Message::UpdateCalender),
                button("12").on_press(Message::UpdateCalender),
            ],
            column![
                button("12").on_press(Message::UpdateCalender),
                button("12").on_press(Message::UpdateCalender),
                button("12").on_press(Message::UpdateCalender),
                button("12").on_press(Message::UpdateCalender),
                button("12").on_press(Message::UpdateCalender),
                button("12").on_press(Message::UpdateCalender),
            ],
            column![
                button("12").on_press(Message::UpdateCalender),
                button("12").on_press(Message::UpdateCalender),
                button("12").on_press(Message::UpdateCalender),
                button("12").on_press(Message::UpdateCalender),
                button("12").on_press(Message::UpdateCalender),
                button("12").on_press(Message::UpdateCalender),
            ],
            column![
                button("12").on_press(Message::UpdateCalender),
                button("12").on_press(Message::UpdateCalender),
                button("12").on_press(Message::UpdateCalender),
                button("12").on_press(Message::UpdateCalender),
                button("12").on_press(Message::UpdateCalender),
                button("12").on_press(Message::UpdateCalender),
            ],
            column![
                button("12").on_press(Message::UpdateCalender),
                button("12").on_press(Message::UpdateCalender),
                button("12").on_press(Message::UpdateCalender),
                button("12").on_press(Message::UpdateCalender),
                button("12").on_press(Message::UpdateCalender),
                button("12").on_press(Message::UpdateCalender),
            ],
        ];

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
            }
            Message::ForwardOneDay => {
            }
            Message::JumpToToday => {
                self.active_date_time = Local::now();
            }
            Message::UpdateCalender => {
                println!("cal");
            }
            Message::Edit(action) => {
                self.content.perform(action);
            }
            Message::EditSearch(action) => {
                self.search_content.perform(action);
            }
            Message::TempTopBarMessage => {
                println!("topbar");
            }
            Message::Save => {
                let year = self.active_date_time.year();
                let month = self.active_date_time.month();

                let mut filename = String::default();
                filename += &year.to_string();
                filename += "-";
                filename.push_str(&format!("{:02}", month));
                filename += ".txt";

                let mut save_path = PathBuf::new();

                let home = dirs::home_dir().expect("Couldn't open home dir!");
                save_path.push(home);
                save_path.push(".ironnote");
                save_path.push("data");
                save_path.push(filename);

                if let Some(parent) = save_path.parent() {
                    if let Err(e) = fs::create_dir_all(parent) {
                        eprintln!("Failed to create parent dirs: {}", e);
                        return;
                    }
                }

                match fs::write(&save_path, self.content.text()) {
                    Err(e) => eprintln!("Failed to write file: {}", e),
                    Ok(_) => println!("saved!"),
                }
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
            active_date_time: Local::now(),
            text: String::default(),
            content: text_editor::Content::default(),
            search_content: text_editor::Content::default(),
        }
    }
}

fn main() -> iced::Result {
    iced::application("ironnote", App::update, App::view)
        .subscription(App::subscription)
        .run()
}
