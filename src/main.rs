use crate::{
    calender::CalenderMessage,
    config::UserSettings,
    content_tools::{correct_arrow_movement, perform_ctrl_backspace, perform_ctrl_delete},
    dictionary::DICTIONARY,
    highlighter::{HighlightSettings, SpellHighlighter},
    history_stack::{HistoryStack, edit_action_to_history_event},
    logbox::LOGBOX,
    search_table::{SearchTable, SearchTableMessage},
    statistics::{BoundedDateStats, Stats},
    text_store::{DayStore, GlobalStore, MonthStore},
};
use calender::Calender;
use chrono::{DateTime, Datelike, Days, Duration, Local, Months, NaiveDate};
use iced::{
    Alignment::Center,
    Event, Font,
    Length::{self, FillPortion},
    Subscription,
    event::listen_with,
    keyboard::{self},
    widget::{
        self, Column, column, row,
        scrollable::{Direction, Scrollbar},
        text::Wrapping,
        text_editor::{self, Action, Content},
    },
};
use iced_aw::ContextMenu;
use keybinds::Keybinds;

mod calender;
mod config;
mod content_tools;
mod dictionary;
mod filetools;
mod highlighter;
mod history_stack;
mod logbox;
mod misc_tools;
mod search_table;
mod statistics;
mod text_store;

struct App {
    window_title: String,
    content: text_editor::Content,
    edited_active_day: bool,
    search_content: text_editor::Content,
    search_text: String,
    active_date_time: DateTime<Local>,
    calender: Calender,
    search_table: SearchTable,
    keybinds: Keybinds<KeyboardAction>,
    day_store: DayStore,
    month_store: MonthStore,
    global_store: GlobalStore,
    log_history_stack: HistoryStack,
    search_history_stack: HistoryStack,
    current_tab: Tab,
    current_editor: Option<Editor>,
    cursor_line_idx: usize,
    cursor_char_idx: usize,
    selected_misspelled_word: Option<String>,
    spell_suggestions: Vec<String>,
    last_edit_time: DateTime<Local>,
    settings: UserSettings,
}

#[derive(Debug, PartialEq)]
enum Editor {
    Log,
    Search,
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
    AcceptSpellcheck(usize),
    AddToDictionary(String),
    Render,
    ClearSearch,
    ToggleSearchCase,
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

        self.day_store = self
            .month_store
            .get_day_store(self.active_date_time.day0() as usize);

        self.calender
            .set_edited_days(self.month_store.edited_days());
    }

    fn write_store_to_disk(&self) {
        self.month_store.save_month();
    }

    fn sync_global_store(&mut self) {
        self.global_store.push_month_store(self.month_store.clone());
    }

    fn write_all(&mut self) {
        self.write_active_entry_to_store();
        self.write_store_to_disk();

        self.sync_global_store();
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
            self.write_all();

            self.month_store.load_month(new_datetime);
        }

        self.active_date_time = new_datetime;

        self.update_window_title();
        self.calender.update_calender_dates(self.active_date_time);
        self.load_active_entry();

        self.calender
            .set_edited_days(self.month_store.edited_days());

        self.last_edit_time = Local::now();

        self.log_history_stack.clear();
    }

    fn update_spellcheck(&mut self) {
        // TODO: allow direct right clicking on misspelled words without selection requirements
        // TODO: compute suggestions on another thread for better performance?

        // computing spellcheck suggestions is extremely expensive, so we only do so when the selection size has
        // changed
        let recompute_spell_suggestions = if let Some(selection) = self.content.selection() {
            self.selected_misspelled_word.replace(selection.clone()) != Some(selection)
        } else {
            self.spell_suggestions.clear();
            self.selected_misspelled_word = None;
            false
        };

        if let Some(selection) = self.content.selection()
            && !selection.contains(char::is_whitespace)
            && recompute_spell_suggestions
        {
            self.spell_suggestions.clear();

            let dictionary = DICTIONARY.read().expect("couldn't get dicitonary read");
            if !dictionary.check(&selection) {
                dictionary.suggest(&selection, &mut self.spell_suggestions);
                self.selected_misspelled_word = Some(selection.clone());
            } else {
                self.selected_misspelled_word = None;
            }
        }
    }

    fn recompute_search(&mut self) {
        self.search_table.clear();
        self.search_text.clear();

        let mut search_text = self.search_content.text();
        search_text.pop();

        if self.settings.ignore_search_case {
            search_text = search_text.to_lowercase();
        }

        if search_text.is_empty() || search_text == " " {
            return;
        }

        for month_store in self.global_store.month_stores().rev() {
            for day_store in month_store.days().rev() {
                let original_content_text = day_store.get_day_text();

                let content_text = if self.settings.ignore_search_case {
                    original_content_text.to_lowercase()
                } else {
                    original_content_text.clone()
                };

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
                        + original_content_text
                            .get(start_idx..subtext_idx)
                            .expect("couldn't get start content_text"))
                    .replace("\n", " ");

                    let bolded_text = original_content_text
                        .get(subtext_idx..(subtext_idx + search_text.chars().count()))
                        .expect("couldn't get bolded content_text")
                        .to_string();

                    let end_text = (original_content_text
                        .get((subtext_idx + search_text.chars().count())..end_idx)
                        .expect("couldn't get end content_text")
                        .to_string()
                        + " ...")
                        .replace("\n", " ");

                    let date = misc_tools::string_to_datetime(&day_store.date());

                    self.search_text = bolded_text.clone();

                    self.search_table
                        .insert_element(start_text, bolded_text, end_text, date);
                }
            }
        }
    }

    fn active_content_and_history_stack(&mut self) -> Option<(&mut Content, &mut HistoryStack)> {
        if let Some(editor) = &self.current_editor {
            match editor {
                Editor::Log => Some((&mut self.content, &mut self.log_history_stack)),
                Editor::Search => Some((&mut self.search_content, &mut self.search_history_stack)),
            }
        } else {
            None
        }
    }
}

impl App {
    fn title(&self) -> String {
        self.window_title.clone()
    }

    pub fn view(&'_ self) -> Column<'_, Message> {
        let (cursor_line_idx, cursor_char_idx) = self.content.cursor_position();
        let cursor_spellcheck_timed_out =
            Local::now().signed_duration_since(self.last_edit_time) > Duration::milliseconds(500);

        let back_button = widget::button(widget::Text::new("<---").align_x(Center))
            .on_press(Message::BackOneDay)
            .width(FillPortion(1))
            .height(100);
        let today_button = widget::button(widget::Text::new("Today").align_x(Center))
            .on_press(Message::JumpToToday)
            .width(FillPortion(1))
            .height(100);
        let forward_button = widget::button(widget::Text::new("--->").align_x(Center))
            .on_press(Message::ForwardOneDay)
            .width(FillPortion(1))
            .height(100);

        let daily_nav_bar = row![back_button, today_button, forward_button].width(7 * 36);

        let cal = Calender::view(&self.calender);
        let temp_calender_bar = row![cal];

        let search_tab_btn = widget::button(widget::Text::new("Search").size(12))
            .on_press(Message::TabSwitched(Tab::Search));
        let stats_tab_btn = widget::button(widget::Text::new("Stats").size(12))
            .on_press(Message::TabSwitched(Tab::Stats));
        let todo_tab_btn = widget::button(widget::Text::new("Todo").size(12))
            .on_press(Message::TabSwitched(Tab::Todo));

        let tab_bar = row![search_tab_btn, stats_tab_btn, todo_tab_btn];

        let tab_area = match self.current_tab {
            Tab::Search => {
                let searchbar = widget::text_editor(&self.search_content)
                    .placeholder("Search entries...")
                    .on_action(Message::EditSearch)
                    .size(13)
                    .font(Font::DEFAULT)
                    .wrapping(Wrapping::None);

                let clear_search_button = widget::button(widget::Text::new("<=").size(9).center())
                    .on_press(Message::ClearSearch)
                    .width(32)
                    .height(26);
                let match_case_button = widget::button(widget::Text::new("Aa").size(9).center())
                    .on_press(Message::ToggleSearchCase)
                    .width(32)
                    .height(26);

                let search_line = row![searchbar, clear_search_button, match_case_button];

                let table = SearchTable::view(&self.search_table);

                let search_results = column![table];
                column![search_line, search_results]
            }
            Tab::Stats => {
                let dwc = self.day_store.word_count().to_string();
                let dcc = self.day_store.char_count().to_string();

                let mwc = self.month_store.word_count().to_string();
                let mcc = self.month_store.char_count().to_string();

                let twc = self.global_store.word_count().to_string();
                let tcc = self.global_store.char_count().to_string();

                let maw = format!("{:.2}", self.month_store.average_words());
                let taw = format!("{:.2}", self.global_store.average_words());
                let mac = format!("{:.2}", self.month_store.average_chars());
                let tac = format!("{:.2}", self.global_store.average_chars());

                column![
                    widget::Text::new("Current Day"),
                    widget::Text::new("     Words:      ".to_string() + &dwc),
                    widget::Text::new("     Characters: ".to_string() + &dcc),
                    widget::Text::new("This Month"),
                    widget::Text::new("     Words:      ".to_string() + &mwc),
                    widget::Text::new("     Characters: ".to_string() + &mcc),
                    widget::Text::new("     Average Words: ".to_string() + &maw),
                    widget::Text::new("     Average Chars: ".to_string() + &mac),
                    widget::Text::new("Total"),
                    widget::Text::new("     Words:      ".to_string() + &twc),
                    widget::Text::new("     Characters: ".to_string() + &tcc),
                    widget::Text::new("     Average Words: ".to_string() + &taw),
                    widget::Text::new("     Average Chars: ".to_string() + &tac),
                ]
            }
            Tab::Todo => {
                column![widget::Text::new("Todo area")]
            }
        }
        .width(250);

        let tab_view = column![tab_bar, tab_area];

        let left_ui = column![daily_nav_bar, temp_calender_bar, tab_view];

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

        let log_text_input = widget::text_editor(&self.content)
            .placeholder("Type today's log...")
            .on_action(Message::Edit)
            .size(13)
            .font(Font::DEFAULT)
            .wrapping(Wrapping::WordOrGlyph)
            .height(Length::Shrink)
            .highlight_with::<SpellHighlighter>(
                HighlightSettings {
                    cursor_line_idx,
                    cursor_char_idx,
                    cursor_spellcheck_timed_out,
                    search_text: self.search_text.clone(),
                    ignore_search_case: self.settings.ignore_search_case,
                },
                highlighter::highlight_to_format,
            );

        let log_edit_area = widget::scrollable(log_text_input)
            .width(Length::Fill)
            .height(Length::Fill)
            .direction(Direction::Vertical(Scrollbar::new().spacing(0).margin(2)));

        let mut spellcheck_context_menu_contents: Vec<(String, Message)> = vec![];

        if self.selected_misspelled_word.is_some() {
            for (i, suggestion) in self.spell_suggestions.iter().enumerate() {
                spellcheck_context_menu_contents
                    .push((suggestion.to_string(), Message::AcceptSpellcheck(i)));
            }
        }

        let composite_editor = ContextMenu::new(log_edit_area, move || {
            let mut spellcheck_context_menu_buttons = vec![];

            for (button_text, button_message) in spellcheck_context_menu_contents.iter() {
                spellcheck_context_menu_buttons.push(
                    widget::button(widget::Text::new(button_text.clone()).size(12))
                        .on_press(button_message.clone())
                        .width(125)
                        .into(),
                );
            }

            let suggestion_count = spellcheck_context_menu_buttons.len();

            let suggestions_scroll = if suggestion_count < 6 {
                column(spellcheck_context_menu_buttons)
            } else {
                column![widget::scrollable(column(spellcheck_context_menu_buttons)).height(125)]
            };

            let mut suggestion_menu = if suggestion_count > 0 {
                column![
                    widget::Text::new("Did you mean:").size(12),
                    suggestions_scroll
                ]
            } else {
                column![]
            };

            if let Some(word) = &self.selected_misspelled_word {
                let contains_whitespace = word.chars().any(|chara| chara.is_whitespace());

                if !contains_whitespace {
                    suggestion_menu = suggestion_menu.push(
                        widget::button(
                            widget::Text::new("Add \"".to_string() + word + "\" to dictionary")
                                .size(13),
                        )
                        .on_press(Message::AddToDictionary(word.clone()))
                        .width(125),
                    )
                }
            }
            suggestion_menu.into()
        });

        let right_ui = column![right_top_bar, composite_editor];

        let top_ui = row![left_ui, right_ui];

        let logbox = widget::text(
            LOGBOX
                .read()
                .expect("couldn't get logbox read")
                .get_log_at_time(),
        )
        .size(14)
        .font(Font::DEFAULT)
        .height(Length::Shrink);

        let bottom_ui = row![logbox];

        let layout = column![top_ui, bottom_ui];

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
                self.current_editor = Some(Editor::Log);

                if self.content.selection().is_none() {
                    (self.cursor_line_idx, self.cursor_char_idx) = self.content.cursor_position();
                }

                match &action {
                    Action::SelectWord => {
                        let content_text = self.content.text();
                        let (cursor_line, cursor_char) = self.content.cursor_position();

                        let line = content_text
                            .lines()
                            .nth(cursor_line)
                            .expect("couldn't extract line");

                        self.cursor_char_idx = misc_tools::first_whitespace_left(line, cursor_char);
                        self.cursor_line_idx = cursor_line;
                    }
                    Action::Edit(edit) => {
                        self.edited_active_day = true;
                        self.last_edit_time = Local::now();

                        let history_event = edit_action_to_history_event(
                            &self.content,
                            edit.clone(),
                            self.cursor_line_idx,
                            self.cursor_char_idx,
                        );
                        self.log_history_stack.push_undo_action(history_event);
                    }
                    _ => {}
                }

                let old_cursor_position = self.content.cursor_position();

                self.content.perform(action.clone());

                if let Action::Move(motion) = action {
                    correct_arrow_movement(&mut self.content, old_cursor_position, motion);
                }

                self.update_spellcheck();
            }
            Message::EditSearch(action) => {
                if self.current_editor != Some(Editor::Search) {
                    self.current_editor = Some(Editor::Search);

                    self.write_active_entry_to_store();
                    self.sync_global_store();
                }

                if self.content.selection().is_none() {
                    (self.cursor_line_idx, self.cursor_char_idx) = self.content.cursor_position();
                }
                if let text_editor::Action::Edit(edit) = &action {
                    let history_event = edit_action_to_history_event(
                        &self.search_content,
                        edit.clone(),
                        self.cursor_line_idx,
                        self.cursor_char_idx,
                    );
                    self.search_history_stack.push_undo_action(history_event);
                }

                let old_cursor_position = self.search_content.cursor_position();

                self.search_content.perform(action.clone());

                if let Action::Move(motion) = action {
                    correct_arrow_movement(&mut self.search_content, old_cursor_position, motion);
                }

                self.recompute_search();
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
                            self.write_all();
                        }
                        KeyboardAction::BackspaceWord => {
                            self.edited_active_day = true;
                            self.last_edit_time = Local::now();

                            let stopping_chars = [
                                ' ', '.', '!', '?', ',', '-', '_', '\"', ';', ':', '(', ')', '{',
                                '}', '[', ']',
                            ];

                            let cursor_line_idx = self.cursor_line_idx;
                            let cursor_char_idx = self.cursor_char_idx;

                            if let Some((content, history_stack)) =
                                self.active_content_and_history_stack()
                            {
                                // revert the standard backspace that can't be caught
                                history_stack.revert(content);

                                history_stack.push_undo_action(perform_ctrl_backspace(
                                    content,
                                    &stopping_chars,
                                    cursor_line_idx,
                                    cursor_char_idx,
                                ));
                            }

                            if self.current_editor == Some(Editor::Search) {
                                self.recompute_search();
                            }
                        }
                        KeyboardAction::BackspaceSentence => {
                            self.edited_active_day = true;
                            self.last_edit_time = Local::now();

                            let stopping_chars = ['.', '!', '?', '\"', ';', ':'];

                            let cursor_line_idx = self.cursor_line_idx;
                            let cursor_char_idx = self.cursor_char_idx;

                            if let Some((content, history_stack)) =
                                self.active_content_and_history_stack()
                            {
                                // revert the standard backspace that can't be caught
                                history_stack.revert(content);

                                history_stack.push_undo_action(perform_ctrl_backspace(
                                    content,
                                    &stopping_chars,
                                    cursor_line_idx,
                                    cursor_char_idx,
                                ));
                            }

                            if self.current_editor == Some(Editor::Search) {
                                self.recompute_search();
                            }
                        }
                        KeyboardAction::Delete => {
                            // not sure why the text_editor action handler doesn't do this on its own
                            self.edited_active_day = true;
                            self.last_edit_time = Local::now();

                            let cursor_line_idx = self.cursor_line_idx;
                            let cursor_char_idx = self.cursor_char_idx;

                            if let Some((content, history_stack)) =
                                self.active_content_and_history_stack()
                            {
                                let history_event = edit_action_to_history_event(
                                    content,
                                    text_editor::Edit::Delete,
                                    cursor_line_idx,
                                    cursor_char_idx,
                                );
                                history_stack.push_undo_action(history_event);

                                content.perform(Action::Edit(text_editor::Edit::Delete));
                            }

                            if self.current_editor == Some(Editor::Search) {
                                self.recompute_search();
                            }
                        }
                        KeyboardAction::DeleteWord => {
                            self.edited_active_day = true;
                            self.last_edit_time = Local::now();

                            let stopping_chars = [
                                ' ', '.', '!', '?', ',', '-', '_', '\"', ';', ':', '(', ')', '{',
                                '}', '[', ']',
                            ];

                            let cursor_line_idx = self.cursor_line_idx;
                            let cursor_char_idx = self.cursor_char_idx;

                            if let Some((content, history_stack)) =
                                self.active_content_and_history_stack()
                            {
                                history_stack.push_undo_action(perform_ctrl_delete(
                                    content,
                                    &stopping_chars,
                                    cursor_line_idx,
                                    cursor_char_idx,
                                ));
                            }

                            if self.current_editor == Some(Editor::Search) {
                                self.recompute_search();
                            }
                        }
                        KeyboardAction::DeleteSentence => {
                            self.edited_active_day = true;
                            self.last_edit_time = Local::now();

                            let stopping_chars = ['.', '!', '?', '\"', ';', ':'];

                            let cursor_line_idx = self.cursor_line_idx;
                            let cursor_char_idx = self.cursor_char_idx;

                            if let Some((content, history_stack)) =
                                self.active_content_and_history_stack()
                            {
                                history_stack.push_undo_action(perform_ctrl_delete(
                                    content,
                                    &stopping_chars,
                                    cursor_line_idx,
                                    cursor_char_idx,
                                ));
                            }

                            if self.current_editor == Some(Editor::Search) {
                                self.recompute_search();
                            }
                        }
                        KeyboardAction::Undo => {
                            self.edited_active_day = true;
                            self.last_edit_time = Local::now();

                            if let Some((content, history_stack)) =
                                self.active_content_and_history_stack()
                            {
                                history_stack.perform_undo(content);
                            }

                            if self.current_editor == Some(Editor::Search) {
                                self.recompute_search();
                            }
                        }
                        KeyboardAction::Redo => {
                            self.edited_active_day = true;
                            self.last_edit_time = Local::now();

                            if let Some((content, history_stack)) =
                                self.active_content_and_history_stack()
                            {
                                history_stack.perform_redo(content);
                            }

                            if self.current_editor == Some(Editor::Search) {
                                self.recompute_search();
                            }
                        }
                        KeyboardAction::Debug => {
                            println!("debug!");
                        }
                    }
                }
            }
            Message::TableSearch(table_message) => {
                let SearchTableMessage::EntryClicked(table_time) = table_message;

                self.reload_date(table_time);
            }
            Message::TabSwitched(tab) => {
                self.current_tab = tab;
            }
            Message::AcceptSpellcheck(suggestion_idx) => {
                let selected_suggestion = self.spell_suggestions[suggestion_idx].clone();

                let equivalent_edit = text_editor::Edit::Paste(selected_suggestion.into());

                let history_event = edit_action_to_history_event(
                    &self.content,
                    equivalent_edit.clone(),
                    self.cursor_line_idx,
                    self.cursor_char_idx,
                );
                self.log_history_stack.push_undo_action(history_event);

                self.content.perform(Action::Edit(equivalent_edit));
            }
            Message::AddToDictionary(word) => {
                dictionary::add_word_to_personal_dictionary(&word);
            }
            Message::Render => {
                self.view();
            }
            Message::ClearSearch => {
                self.search_content = Content::new();

                self.update(Message::EditSearch(Action::Move(
                    text_editor::Motion::DocumentEnd,
                )));
            }
            Message::ToggleSearchCase => {
                self.settings.ignore_search_case = !self.settings.ignore_search_case;

                self.update(Message::EditSearch(Action::Move(
                    text_editor::Motion::DocumentEnd,
                )));
            }
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        let subscriptions = vec![
            listen_with(|event, _, _| match event {
                Event::Keyboard(event) => Some(Message::KeyEvent(event)),
                _ => None,
            }),
            // ensure view() gets called at a minimum of 10 FPS
            iced::time::every(std::time::Duration::from_millis(100))
                .map(|_instant| Message::Render),
        ];

        Subscription::batch(subscriptions)
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
            search_text: String::default(),
            calender: Calender::default(),
            search_table: SearchTable::default(),
            keybinds,
            day_store: DayStore::default(),
            month_store: MonthStore::default(),
            global_store: GlobalStore::default(),
            log_history_stack: HistoryStack::default(),
            search_history_stack: HistoryStack::default(),
            current_tab: Tab::Search,
            current_editor: None,
            cursor_line_idx: 0,
            cursor_char_idx: 0,
            selected_misspelled_word: None,
            spell_suggestions: vec![],
            last_edit_time: Local::now(),
            settings: UserSettings::default(),
        };

        df.month_store.load_month(Local::now());
        df.global_store.load_all();

        df.update(Message::JumpToToday);

        df
    }
}

fn main() -> iced::Result {
    iced::application(App::title, App::update, App::view)
        .subscription(App::subscription)
        .run()
}
