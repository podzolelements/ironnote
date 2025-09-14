use crate::{
    calender::CalenderMessage,
    history_stack::{HistoryEvent, HistoryStack, edit_action_to_history_event},
    misc_tools::chars_all_same_in_string,
    search_table::{SearchTable, SearchTableMessage},
    text_store::{DayStore, MonthStore},
};
use calender::Calender;
use chrono::{DateTime, Datelike, Days, Local, Months, NaiveDate};
use iced::{
    Event, Font, Subscription,
    event::listen_with,
    keyboard::{self},
    widget::{
        self, Row, Space, Text, column, row,
        text::Wrapping,
        text_editor::{self, Action},
    },
};
use keybinds::Keybinds;

mod calender;
mod content_tools;
mod filetools;
mod history_stack;
mod misc_tools;
mod search_table;
mod text_store;

struct App {
    window_title: String,
    content: text_editor::Content,
    edited_active_day: bool,
    search_content: text_editor::Content,
    active_date_time: DateTime<Local>,
    calender: Calender,
    search_table: SearchTable,
    keybinds: Keybinds<KeyboardAction>,
    day_store: DayStore,
    month_store: MonthStore,
    version_stack: HistoryStack,
    current_tab: Tab,
    cursor_line_idx: usize,
    cursor_char_idx: usize,
}

enum KeyboardAction {
    Save,
    BackspaceWord,
    BackspaceSentence,
    Delete,
    DeleteWord,
    DeleteSentence,
    Undo,
    Redo,
    Debug,
}

#[derive(Debug, Clone)]
pub enum Tab {
    Search,
    Stats,
    Todo,
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
    TableSearch(SearchTableMessage),
    KeyEvent(keyboard::Event),
    TabSwitched(Tab),
}

impl App {
    fn load_active_entry(&mut self) {
        self.day_store = self
            .month_store
            .get_day_store(self.active_date_time.day0() as usize);

        self.content = text_editor::Content::with_text(&self.day_store.get_day_text());
    }

    fn write_active_entry_to_store(&mut self) {
        self.month_store
            .set_day_text(self.active_date_time.day0() as usize, self.content.text());
    }

    fn write_store_to_disk(&self) {
        self.month_store.save_month();
    }

    fn update_window_title(&mut self) {
        let formated_date = self.active_date_time.format("%A, %B %d, %Y").to_string();
        let new_title = "ironnote - ".to_string() + &formated_date;

        self.window_title = new_title;
    }

    fn reload_date(&mut self, new_datetime: DateTime<Local>) {
        let current_month = self.active_date_time.month();
        let current_year = self.active_date_time.year();
        let new_month = new_datetime.month();
        let new_year = new_datetime.year();

        if (current_month != new_month) || (current_year != new_year) {
            self.write_active_entry_to_store();
            self.write_store_to_disk();

            self.month_store.load_month(new_datetime);
        }

        self.active_date_time = new_datetime;

        self.update_window_title();
        self.calender.update_calender_dates(self.active_date_time);
        self.load_active_entry();

        self.version_stack.clear();
    }

    fn ctrl_backspace(&mut self, stopping_chars: &[char]) {
        if self.content.cursor_position() == (0, 0) {
            return;
        }

        // revert the standard backspace that can't be caught
        self.version_stack.revert(&mut self.content);

        let mut removed_chars = String::new();
        let (cursor_line_start, cursor_char_start) = self.content.cursor_position();

        if let Some(selection) = self.content.selection() {
            let selection_bounds = content_tools::get_selection_bounds(
                &self.content,
                self.cursor_line_idx,
                self.cursor_char_idx,
            );
            let (adjusted_cursor_line, adjusted_cursor_char) = content_tools::locate_cursor_start(
                &self.content,
                self.cursor_line_idx,
                self.cursor_char_idx,
            );
            self.version_stack.push_undo_action(HistoryEvent {
                selection: Some(selection_bounds),
                text_removed: Some(selection),
                text_added: None,
                cursor_line_idx: adjusted_cursor_line,
                cursor_char_idx: adjusted_cursor_char,
            });

            self.content
                .perform(Action::Edit(text_editor::Edit::Backspace));
            return;
        }

        // on edge of newline
        if cursor_char_start == 0 {
            let (cursor_line, cursor_char) = self.content.cursor_position();

            let (new_cursor_line, new_cursor_char) =
                content_tools::decrement_cursor_position(&self.content, cursor_line, cursor_char);

            self.version_stack.push_undo_action(HistoryEvent {
                selection: None,
                text_removed: Some("\n".to_string()),
                text_added: None,
                cursor_line_idx: new_cursor_line,
                cursor_char_idx: new_cursor_char,
            });

            self.content
                .perform(Action::Edit(text_editor::Edit::Backspace));
            return;
        }

        let content_text = self.content.text();
        let char_line = content_text
            .lines()
            .nth(cursor_line_start)
            .expect("couldn't extract line");

        let mut backspace_head = cursor_char_start - 1;

        let first_char_removed = char_line
            .chars()
            .nth(backspace_head)
            .expect("couldn't extract char from line");

        let mut removing_seqence_of_stops = false;

        loop {
            let char_to_remove = char_line
                .chars()
                .nth(backspace_head)
                .expect("couldn't extract char from line");

            removed_chars.push(char_to_remove);
            self.content
                .perform(Action::Edit(text_editor::Edit::Backspace));

            if backspace_head > 0 {
                backspace_head -= 1;
            } else {
                break;
            }

            let next_char_to_remove = char_line
                .chars()
                .nth(backspace_head)
                .expect("couldn't extract char from line");

            if stopping_chars.contains(&first_char_removed)
                && first_char_removed == next_char_to_remove
                && chars_all_same_in_string(&removed_chars)
                && (removed_chars.chars().count() == 1 || removing_seqence_of_stops)
            {
                removing_seqence_of_stops = true;
                continue;
            } else if removing_seqence_of_stops {
                break;
            }

            if stopping_chars.contains(&next_char_to_remove) {
                break;
            }
        }

        removed_chars = removed_chars.chars().rev().collect();

        let cursor_line_end = cursor_line_start + 1 - removed_chars.lines().count();
        let cursor_char_end = cursor_char_start - removed_chars.chars().count();

        self.version_stack.push_undo_action(HistoryEvent {
            selection: None,
            text_removed: Some(removed_chars),
            text_added: None,
            cursor_line_idx: cursor_line_end,
            cursor_char_idx: cursor_char_end,
        });
    }

    fn ctrl_delete(&mut self, stopping_chars: &[char]) {
        let content_text = self.content.text();

        let (cursor_line_start, cursor_char_start) = self.content.cursor_position();

        let line_count = content_text.lines().count();
        let line = content_text
            .lines()
            .nth(cursor_line_start)
            .expect("couldn't extract line");

        let char_count = line.chars().count();

        if let Some(selection) = self.content.selection() {
            let selection_bounds = content_tools::get_selection_bounds(
                &self.content,
                self.cursor_line_idx,
                self.cursor_char_idx,
            );
            let (adjusted_cursor_line, adjusted_cursor_char) = content_tools::locate_cursor_start(
                &self.content,
                self.cursor_line_idx,
                self.cursor_char_idx,
            );
            self.version_stack.push_undo_action(HistoryEvent {
                selection: Some(selection_bounds),
                text_removed: Some(selection),
                text_added: None,
                cursor_line_idx: adjusted_cursor_line,
                cursor_char_idx: adjusted_cursor_char,
            });

            self.content
                .perform(Action::Edit(text_editor::Edit::Backspace));
            return;
        }

        if line_count == (cursor_line_start + 1) && char_count == cursor_char_start {
            // nothing to delete, end of text
        } else if char_count == cursor_char_start {
            // deletes following newline
            self.version_stack.push_undo_action(HistoryEvent {
                selection: None,
                text_removed: Some('\n'.to_string()),
                text_added: None,
                cursor_line_idx: cursor_line_start,
                cursor_char_idx: cursor_char_start,
            });
            self.content
                .perform(Action::Edit(text_editor::Edit::Delete));
        } else {
            // standard ctrl+delete
            let mut removed_chars = String::new();
            let first_char_removed = line
                .chars()
                .nth(cursor_char_start)
                .expect("couldn't extract char from line");

            let mut delete_head = cursor_char_start;

            let mut removing_sequence_of_stops = false;

            loop {
                let char_to_remove = line
                    .chars()
                    .nth(delete_head)
                    .expect("couldn't extract char from line");

                removed_chars.push(char_to_remove);
                self.content
                    .perform(Action::Edit(text_editor::Edit::Delete));

                if (delete_head + 1) < char_count {
                    delete_head += 1;
                } else {
                    break;
                }

                let next_char_to_remove = line
                    .chars()
                    .nth(delete_head)
                    .expect("couldn't extract char from line");

                if stopping_chars.contains(&first_char_removed)
                    && first_char_removed == next_char_to_remove
                    && chars_all_same_in_string(&removed_chars)
                    && (removed_chars.chars().count() == 1 || removing_sequence_of_stops)
                {
                    removing_sequence_of_stops = true;
                    continue;
                } else if removing_sequence_of_stops {
                    break;
                }

                if stopping_chars.contains(&next_char_to_remove) {
                    break;
                }
            }

            self.version_stack.push_undo_action(HistoryEvent {
                selection: None,
                text_removed: Some(removed_chars),
                text_added: None,
                cursor_line_idx: cursor_line_start,
                cursor_char_idx: cursor_char_start,
            });
        }
    }
}

impl App {
    fn title(&self) -> String {
        self.window_title.clone()
    }

    pub fn view(&'_ self) -> Row<'_, Message> {
        let back_button = widget::button("<--")
            .on_press(Message::BackOneDay)
            .height(100);
        let today_button = widget::button("Today")
            .on_press(Message::JumpToToday)
            .height(100);
        let forward_button = widget::button("-->")
            .on_press(Message::ForwardOneDay)
            .height(100);

        let hspace = Space::new(5, 5);
        let hspace2 = Space::new(5, 5);

        let buttonbar = row![back_button, hspace, today_button, hspace2, forward_button];

        let cal = Calender::view(&self.calender);
        let temp_calender_bar = row![cal];

        let search_tab_btn = widget::button(Text::new("Search").size(12))
            .on_press(Message::TabSwitched(Tab::Search));
        let stats_tab_btn =
            widget::button(Text::new("Stats").size(12)).on_press(Message::TabSwitched(Tab::Stats));
        let todo_tab_btn =
            widget::button(Text::new("Todo").size(12)).on_press(Message::TabSwitched(Tab::Todo));

        let tab_bar = row![search_tab_btn, stats_tab_btn, todo_tab_btn];

        let tab_area = match self.current_tab {
            Tab::Search => {
                let seachbar = widget::text_editor(&self.search_content)
                    .placeholder("Search entries...")
                    .on_action(Message::EditSearch)
                    .size(13)
                    .font(Font::DEFAULT)
                    .wrapping(Wrapping::None);

                let table = SearchTable::view(&self.search_table);

                let search_results = column![table];
                column![seachbar, search_results]
            }
            Tab::Stats => {
                column![Text::new("Stats area")]
            }
            Tab::Todo => {
                column![Text::new("Todo area")]
            }
        }
        .width(250);

        let tab_view = column![tab_bar, tab_area];

        let left_ui = column![buttonbar, temp_calender_bar, tab_view];

        let right_top_bar = row![
            widget::button("test button 0")
                .on_press(Message::TempTopBarMessage)
                .height(100),
            widget::button("test button 1")
                .on_press(Message::TempTopBarMessage)
                .height(100),
            widget::button("test button 2")
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
                    self.write_active_entry_to_store();
                    self.edited_active_day = false;
                }
                let new_datetime = self
                    .active_date_time
                    .checked_sub_days(Days::new(1))
                    .expect("failed to go to previous day");
                self.reload_date(new_datetime);
            }
            Message::ForwardOneDay => {
                if self.edited_active_day {
                    self.write_active_entry_to_store();
                    self.edited_active_day = false;
                }
                let new_datetime = self
                    .active_date_time
                    .checked_add_days(Days::new(1))
                    .expect("failed to go to next day");

                self.reload_date(new_datetime);
            }
            Message::JumpToToday => {
                let new_datetime = Local::now();
                self.reload_date(new_datetime);
            }
            Message::UpdateCalender => {
                println!("cal");
            }
            Message::Edit(action) => {
                if self.content.selection().is_none() {
                    (self.cursor_line_idx, self.cursor_char_idx) = self.content.cursor_position();
                }
                if let text_editor::Action::Edit(edit) = &action {
                    self.edited_active_day = true;
                    self.version_stack.clear_redo_stack();

                    let history_event = edit_action_to_history_event(
                        &self.content,
                        edit.clone(),
                        self.cursor_line_idx,
                        self.cursor_char_idx,
                    );
                    self.version_stack.push_undo_action(history_event);
                }

                self.content.perform(action);
            }
            Message::EditSearch(action) => {
                self.search_content.perform(action);

                self.search_table.clear();

                let mut search_text = self.search_content.text();
                search_text.pop();

                for day_store in self.month_store.days() {
                    let search_text = search_text.clone();
                    let content_text = day_store.get_day_text();

                    if search_text.is_empty() || search_text == " " {
                        continue;
                    }

                    if let Some(subtext_idx) = content_text.find(&search_text) {
                        let start_idx = if ((subtext_idx as i32) - 30) < 0 {
                            0
                        } else {
                            subtext_idx - 30
                        };
                        let end_idx = if subtext_idx + 50 > content_text.chars().count() {
                            content_text.chars().count()
                        } else {
                            subtext_idx + 50
                        };

                        let start_text = (day_store.date()
                            + " ... "
                            + content_text
                                .get(start_idx..subtext_idx)
                                .expect("couldn't get start content_text"))
                        .replace("\n", "");

                        let end_text = (content_text
                            .get((subtext_idx + search_text.chars().count())..end_idx)
                            .expect("couldn't get end content_text")
                            .to_string()
                            + " ...")
                            .replace("\n", "");

                        self.search_table
                            .insert_element(start_text, search_text, end_text);
                    }
                }
            }
            Message::TempTopBarMessage => {
                println!("topbar");
            }
            Message::Calender(calmes) => match calmes {
                CalenderMessage::DayButton(new_day, month) => {
                    if self.edited_active_day {
                        self.write_active_entry_to_store();
                        self.edited_active_day = false;
                    }

                    let new_datetime = match month {
                        calender::Month::Last => {
                            let days_in_last_month = if self.active_date_time.month() == 1 {
                                31
                            } else {
                                let nd = NaiveDate::from_ymd_opt(
                                    self.active_date_time.year(),
                                    self.active_date_time.month() - 1,
                                    1,
                                )
                                .expect("bad date");

                                nd.num_days_in_month() as u32
                            };

                            let days_to_go_back =
                                (days_in_last_month - new_day) + self.active_date_time.day();

                            self.active_date_time
                                .checked_sub_days(Days::new(days_to_go_back as u64))
                                .expect("couldn't go into the past")
                        }
                        calender::Month::Current => {
                            let delta_day = (new_day as i32) - (self.active_date_time.day() as i32);

                            let mag_delta_day = delta_day.unsigned_abs() as u64;

                            if delta_day == 0 {
                                return;
                            }
                            if delta_day < 0 {
                                self.active_date_time
                                    .checked_sub_days(Days::new(mag_delta_day))
                                    .expect("couldn't jump into the past")
                            } else {
                                self.active_date_time
                                    .checked_add_days(Days::new(mag_delta_day))
                                    .expect("couldn't jump into the future")
                            }
                        }
                        calender::Month::Next => {
                            let days_to_go_forward = (self.active_date_time.num_days_in_month()
                                as u64
                                - self.active_date_time.day() as u64)
                                + new_day as u64;

                            self.active_date_time
                                .checked_add_days(Days::new(days_to_go_forward))
                                .expect("couldn't go into the future")
                        }
                    };

                    self.reload_date(new_datetime);
                }
                CalenderMessage::BackMonth => {
                    let new_datetime = self
                        .active_date_time
                        .checked_sub_months(Months::new(1))
                        .expect("couldn't go back a month");

                    self.reload_date(new_datetime);
                }
                CalenderMessage::ForwardMonth => {
                    let new_datetime = self
                        .active_date_time
                        .checked_add_months(Months::new(1))
                        .expect("couldn't go forward a month");

                    self.reload_date(new_datetime);
                }
                CalenderMessage::BackYear => {
                    let new_datetime = self
                        .active_date_time
                        .checked_sub_months(Months::new(12))
                        .expect("couldn't go back a year");

                    self.reload_date(new_datetime);
                }
                CalenderMessage::ForwardYear => {
                    let new_datetime = self
                        .active_date_time
                        .checked_add_months(Months::new(12))
                        .expect("couldn't go forward a year");

                    self.reload_date(new_datetime);
                }
            },
            Message::KeyEvent(event) => {
                if let Some(action) = self.keybinds.dispatch(event) {
                    match action {
                        KeyboardAction::Save => {
                            self.write_active_entry_to_store();
                            self.write_store_to_disk();
                        }
                        KeyboardAction::BackspaceWord => {
                            self.edited_active_day = true;
                            let stopping_chars = [
                                ' ', '.', '!', '?', ',', '-', '_', '\"', ';', ':', '(', ')', '{',
                                '}', '[', ']',
                            ];

                            self.ctrl_backspace(&stopping_chars);
                        }
                        KeyboardAction::BackspaceSentence => {
                            self.edited_active_day = true;
                            let stopping_chars = ['.', '!', '?', '\"', ';', ':'];

                            self.ctrl_backspace(&stopping_chars);
                        }
                        KeyboardAction::Delete => {
                            // not sure why the text_editor action handler doesn't do this on its own
                            self.edited_active_day = true;
                            self.version_stack.clear_redo_stack();

                            let history_event = edit_action_to_history_event(
                                &self.content,
                                text_editor::Edit::Delete,
                                self.cursor_line_idx,
                                self.cursor_char_idx,
                            );
                            self.version_stack.push_undo_action(history_event);

                            self.content
                                .perform(Action::Edit(text_editor::Edit::Delete));
                        }
                        KeyboardAction::DeleteWord => {
                            self.edited_active_day = true;
                            let stopping_chars = [
                                ' ', '.', '!', '?', ',', '-', '_', '\"', ';', ':', '(', ')', '{',
                                '}', '[', ']',
                            ];
                            self.ctrl_delete(&stopping_chars);
                        }
                        KeyboardAction::DeleteSentence => {
                            self.edited_active_day = true;
                            let stopping_chars = ['.', '!', '?', '\"', ';', ':'];
                            self.ctrl_delete(&stopping_chars);
                        }
                        KeyboardAction::Undo => {
                            self.edited_active_day = true;
                            self.version_stack.perform_undo(&mut self.content);
                        }
                        KeyboardAction::Redo => {
                            self.edited_active_day = true;
                            self.version_stack.perform_redo(&mut self.content);
                        }
                        KeyboardAction::Debug => {
                            content_tools::select_text(&mut self.content, 3, 3, 5);
                        }
                    }
                }
            }
            Message::TableSearch(table_message) => {
                let SearchTableMessage::EntryClicked(table_id) = table_message;
                println!("search table {}", table_id);
            }
            Message::TabSwitched(tab) => {
                self.current_tab = tab;
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
            .bind("Ctrl+z", KeyboardAction::Undo)
            .expect("couldn't bind Ctrl+z");
        keybinds
            .bind("Ctrl+Z", KeyboardAction::Redo)
            .expect("couldn't bind Ctrl+Z");
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
        keybinds
            .bind("Ctrl+d", KeyboardAction::Debug)
            .expect("couldn't bind Ctrl+d");

        let mut df = Self {
            window_title: String::default(),
            active_date_time: Local::now(),
            edited_active_day: false,
            content: text_editor::Content::default(),
            search_content: text_editor::Content::default(),
            calender: Calender::default(),
            search_table: SearchTable::default(),
            keybinds,
            day_store: DayStore::default(),
            month_store: MonthStore::default(),
            version_stack: HistoryStack::default(),
            current_tab: Tab::Search,
            cursor_line_idx: 0,
            cursor_char_idx: 0,
        };

        df.month_store.load_month(Local::now());
        df.update(Message::JumpToToday);

        df
    }
}

fn main() -> iced::Result {
    iced::application(App::title, App::update, App::view)
        .subscription(App::subscription)
        .run()
}
