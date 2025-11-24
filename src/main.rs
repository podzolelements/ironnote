use crate::{
    calender::CalenderMessage,
    config::UserSettings,
    content_tools::{correct_arrow_movement, perform_ctrl_backspace, perform_ctrl_delete},
    context_menu::context_menu,
    dictionary::DICTIONARY,
    global_store::GlobalStore,
    highlighter::{HighlightSettings, SpellHighlighter},
    history_stack::{HistoryStack, edit_action_to_history_event},
    logbox::LOGBOX,
    menu_bar::{MenuBar, menu_bar},
    menu_bar_builder::{EditMessage, FileMessage, MenuMessage, build_menu_bar},
    misc_tools::point_on_edge_of_text,
    search_table::{SearchTable, SearchTableMessage},
    word_count::{TimedWordCount, WordCount},
};
use calender::Calender;
use chrono::{DateTime, Datelike, Days, Duration, Local, Months, NaiveDate};
use copypasta::{ClipboardContext, ClipboardProvider};
use iced::{
    Alignment::Center,
    Event, Font,
    Length::{self, FillPortion},
    Point, Size, Subscription, Task,
    event::listen_with,
    keyboard::{self},
    widget::{
        self, Column, column, mouse_area, row,
        scrollable::{Direction, Id, RelativeOffset, Scrollbar, snap_to},
        text::Wrapping,
        text_editor::{self, Action, Content},
        vertical_space,
    },
};
use iced_core::window;
use keybinds::Keybinds;

mod calender;
mod config;
mod content_tools;
mod context_menu;
mod day_store;
mod dictionary;
mod filetools;
mod global_store;
mod highlighter;
mod history_stack;
mod logbox;
mod menu_bar;
mod menu_bar_builder;
mod misc_tools;
mod month_store;
mod search_table;
mod word_count;

struct App {
    window_title: String,
    content: text_editor::Content,
    search_content: text_editor::Content,
    search_text: String,
    calender: Calender,
    search_table: SearchTable,
    keybinds: Keybinds<KeyboardAction>,
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
    show_context_menu: bool,
    mouse_position: Point,
    captured_mouse_position: Point,
    window_size: Size,
    window_mouse_position: Point,
    captured_window_mouse_position: Point,
    clipboard: ClipboardContext,
    menu_bar: MenuBar<Message>,
}

#[derive(Debug, PartialEq)]
enum Editor {
    Log,
    Search,
}

#[derive(Debug, Clone)]
/// these actions are not bound to their shortcuts via the keybinds structure, since the text_editor takes care of
/// handling them. these are called when the action needs to be performed manually without the shortcuts
pub enum UnboundKey {
    Cut,
    Copy,
    Paste,
}

#[derive(Debug, Clone)]
pub enum KeyboardAction {
    Save,
    BackspaceWord,
    BackspaceSentence,
    Delete,
    DeleteWord,
    DeleteSentence,
    Undo,
    Redo,
    Debug,
    JumpToContentStart,
    JumpToContentEnd,
    Unbound(UnboundKey),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Tab {
    Search,
    Stats,
    Todo,
}

#[derive(Debug, Default, Clone)]
pub enum Message {
    #[default]
    EmptyMessage,
    BackOneDay,
    ForwardOneDay,
    JumpToToday,
    Edit(text_editor::Action),
    EditSearch(text_editor::Action),
    TempTopBarMessage,
    Calender(CalenderMessage),
    TableSearch(SearchTableMessage),
    KeyEvent(keyboard::Event),
    ManualKeyEvent(KeyboardAction),
    WindowEvent(window::Event),
    TabSwitched(Tab),
    AcceptSpellcheck(usize),
    AddToDictionary(String),
    Render,
    ClearSearch,
    ToggleSearchCase,
    MouseMoved(Point),
    WindowMouseMoved(Point),
    RightClickEditArea,
    ExitContextMenu,
    MenuBar(MenuMessage),
}

impl App {
    /// retrieves the text from the store and overwrites the content with it
    fn load_active_entry(&mut self) {
        self.content = text_editor::Content::with_text(&self.global_store.day().get_day_text());
    }

    /// write the current text into the store
    fn write_active_entry_to_store(&mut self) {
        let mut current_text = self.content.text();
        // remove the trailing newline that is created by the content
        current_text.pop();

        self.global_store.day_mut().set_day_text(current_text);

        self.calender
            .set_edited_days(self.global_store.month().edited_days());

        self.global_store.update_word_count();
    }

    /// writes current entry to store and saves the store to disk
    fn save_all(&mut self) {
        self.write_active_entry_to_store();
        self.global_store.save_all();
    }

    /// reloads the window's title based on the current active date
    fn update_window_title(&mut self) {
        let formated_date = self
            .global_store
            .date_time()
            .format("%A, %B %d, %Y")
            .to_string();
        let new_title = "ironnote - ".to_string() + &formated_date;

        self.window_title = new_title;
    }

    /// writes the current entry into the store and changes the date of the current entry
    fn reload_date(&mut self, new_datetime: DateTime<Local>) {
        self.write_active_entry_to_store();

        self.global_store.set_current_store_date(new_datetime);

        self.update_window_title();
        self.calender
            .update_calender_dates(self.global_store.date_time());
        self.load_active_entry();

        self.calender
            .set_edited_days(self.global_store.month().edited_days());

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
            let mut spell_suggestions = vec![];

            let dictionary = DICTIONARY.read().expect("couldn't get dicitonary read");
            if !dictionary.check(&selection) {
                dictionary.suggest(&selection, &mut spell_suggestions);
                self.selected_misspelled_word = Some(selection.clone());

                self.global_store.update_word_count();

                let mut sorted_suggestions: Vec<_> = spell_suggestions
                    .iter()
                    .map(|word| {
                        let word_count = self.global_store.get_word_count(&word.to_lowercase());

                        (word_count, word)
                    })
                    .collect();

                sorted_suggestions.sort_by_key(|(word_count, _word)| *word_count);

                self.spell_suggestions = sorted_suggestions
                    .iter()
                    .map(|(_count, word)| word.to_string())
                    .rev()
                    .collect();
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

const LOG_EDIT_AREA_ID: &str = "log_edit_area";

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

        let calender = Calender::view(&self.calender).map(Message::Calender);

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

                let table = SearchTable::view(&self.search_table).map(Message::TableSearch);

                let search_results = column![table];
                column![search_line, search_results]
            }
            Tab::Stats => {
                let dwc = self.global_store.day().total_word_count().to_string();
                let dcc = self.global_store.day().total_char_count().to_string();

                let mwc = self.global_store.month().total_word_count().to_string();
                let mcc = self.global_store.month().total_char_count().to_string();

                let twc = self.global_store.total_word_count().to_string();
                let tcc = self.global_store.total_char_count().to_string();

                let maw = format!("{:.2}", self.global_store.month().average_words());
                let taw = format!("{:.2}", self.global_store.average_words());
                let mac = format!("{:.2}", self.global_store.month().average_chars());
                let tac = format!("{:.2}", self.global_store.average_chars());

                let longest_streak = format!("{}", self.global_store.longest_streak());
                let current_streak = format!("{}", self.global_store.current_streak());

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
                    widget::Text::new(
                        "     Current Streak: ".to_string() + &current_streak + " days"
                    ),
                    widget::Text::new(
                        "     Longest Streak: ".to_string() + &longest_streak + " days"
                    ),
                ]
            }
            Tab::Todo => {
                column![widget::Text::new("Todo area")]
            }
        }
        .width(250);

        let tab_view = column![tab_bar, tab_area];

        let left_ui = column![daily_nav_bar, calender, tab_view];

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
            .direction(Direction::Vertical(Scrollbar::new().spacing(0).margin(2)))
            .id(Id::new(LOG_EDIT_AREA_ID));

        let mouse_log_edit_area = mouse_area(log_edit_area)
            .on_right_release(Message::RightClickEditArea)
            .on_move(Message::MouseMoved);

        let mut spellcheck_context_menu_contents: Vec<(String, Message)> = vec![];

        if self.selected_misspelled_word.is_some() {
            for (i, suggestion) in self.spell_suggestions.iter().enumerate() {
                spellcheck_context_menu_contents
                    .push((suggestion.to_string(), Message::AcceptSpellcheck(i)));
            }
        }

        const MENU_SIZE: u16 = 13;
        const MENU_WIDTH: u16 = 125;

        let mut spellcheck_context_menu_buttons = column![];

        for (button_text, button_message) in spellcheck_context_menu_contents.iter() {
            spellcheck_context_menu_buttons = spellcheck_context_menu_buttons.push(
                widget::button(widget::Text::new(button_text.clone()).size(MENU_SIZE))
                    .on_press(button_message.clone())
                    .width(MENU_WIDTH),
            );
        }

        let suggestion_count = spellcheck_context_menu_contents.len();

        let suggestions_scroll = if suggestion_count < 6 {
            spellcheck_context_menu_buttons
        } else {
            column![widget::scrollable(spellcheck_context_menu_buttons).height(MENU_WIDTH)]
        };

        let mut suggestion_menu = if suggestion_count > 0 {
            column![
                widget::Text::new("Did you mean:").size(MENU_SIZE),
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
                            .size(MENU_SIZE),
                    )
                    .on_press(Message::AddToDictionary(word.clone()))
                    .width(MENU_WIDTH),
                )
            }
        }

        let cut_message = self
            .content
            .selection()
            .map(|_selection| Message::ManualKeyEvent(KeyboardAction::Unbound(UnboundKey::Cut)));
        let copy_message = Message::ManualKeyEvent(KeyboardAction::Unbound(UnboundKey::Copy));
        let paste_message = Message::ManualKeyEvent(KeyboardAction::Unbound(UnboundKey::Paste));

        let edit_menu = column![
            widget::button(widget::text("Cut").size(MENU_SIZE))
                .on_press_maybe(cut_message)
                .width(MENU_WIDTH),
            widget::button(widget::text("Copy").size(MENU_SIZE))
                .on_press(copy_message)
                .width(MENU_WIDTH),
            widget::button(widget::text("Paste").size(MENU_SIZE))
                .on_press(paste_message)
                .width(MENU_WIDTH)
        ];

        let undo_message = if self.log_history_stack.undo_stack_height() > 0 {
            Some(Message::ManualKeyEvent(KeyboardAction::Undo))
        } else {
            None
        };
        let redo_message = if self.log_history_stack.redo_stack_height() > 0 {
            Some(Message::ManualKeyEvent(KeyboardAction::Redo))
        } else {
            None
        };

        let history_menu = column![
            widget::button(widget::text("Undo").size(MENU_SIZE))
                .on_press_maybe(undo_message)
                .width(MENU_WIDTH),
            widget::button(widget::text("Redo").size(MENU_SIZE))
                .on_press_maybe(redo_message)
                .width(MENU_WIDTH),
        ];

        let total_context_menu = column![
            suggestion_menu,
            vertical_space().height(3),
            edit_menu,
            vertical_space().height(3),
            history_menu
        ];

        let distance_to_window_edge =
            self.window_size.width - self.captured_window_mouse_position.x;

        let context_menu_position = if distance_to_window_edge < (MENU_WIDTH + 15) as f32 {
            Point::new(
                self.captured_mouse_position.x - (MENU_WIDTH as f32),
                self.captured_mouse_position.y,
            )
        } else {
            self.captured_mouse_position
        };

        let composite_editor = context_menu(
            mouse_log_edit_area,
            total_context_menu,
            self.show_context_menu,
            context_menu_position,
            Message::ExitContextMenu,
        );

        let right_ui = column![right_top_bar, composite_editor.into()];

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

        let layout_ui = column![top_ui, bottom_ui];

        let layout_menus = menu_bar(layout_ui.into(), &self.menu_bar);

        let layout = column![mouse_area(layout_menus).on_move(Message::WindowMouseMoved)];

        layout
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::EmptyMessage => {
                panic!("uninitialized message");
            }
            Message::BackOneDay => {
                let previous_day = self
                    .global_store
                    .date_time()
                    .checked_sub_days(Days::new(1))
                    .expect("failed to go to previous day");

                let new_datetime = if self.global_store.day().contains_entry() {
                    previous_day
                } else {
                    self.global_store
                        .get_previous_edited_day(self.global_store.date_time())
                        .unwrap_or(previous_day)
                };

                self.reload_date(new_datetime);

                snap_to(Id::new(LOG_EDIT_AREA_ID), RelativeOffset::START)
            }
            Message::ForwardOneDay => {
                let next_day = self
                    .global_store
                    .date_time()
                    .checked_add_days(Days::new(1))
                    .expect("failed to go to next day");

                let new_datetime = if self.global_store.day().contains_entry() {
                    next_day
                } else {
                    self.global_store
                        .get_next_edited_day(self.global_store.date_time())
                        .unwrap_or(next_day)
                };

                self.reload_date(new_datetime);

                snap_to(Id::new(LOG_EDIT_AREA_ID), RelativeOffset::START)
            }
            Message::JumpToToday => {
                let new_datetime = Local::now();
                self.reload_date(new_datetime);

                snap_to(Id::new(LOG_EDIT_AREA_ID), RelativeOffset::START)
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

                let text = self.content.text();
                let (cursor_y, cursor_x) = self.content.cursor_position();
                let cursor_location = point_on_edge_of_text(&text, cursor_x, cursor_y, 3, 400);

                match cursor_location {
                    Some(true) => snap_to(Id::new(LOG_EDIT_AREA_ID), RelativeOffset::START),
                    Some(false) => snap_to(Id::new(LOG_EDIT_AREA_ID), RelativeOffset::END),
                    None => Task::none(),
                }
            }
            Message::EditSearch(action) => {
                if self.current_editor != Some(Editor::Search) {
                    self.current_editor = Some(Editor::Search);

                    self.write_active_entry_to_store();
                }

                if self.content.selection().is_none() {
                    (self.cursor_line_idx, self.cursor_char_idx) = self.content.cursor_position();
                }

                if let text_editor::Action::Edit(edit) = &action {
                    // prevent newlines from being entered into the searchbar since it causes issues with the
                    // highlighted search results, among other things
                    match edit {
                        text_editor::Edit::Insert(inserted_char) => {
                            if *inserted_char == '\n' {
                                return Task::none();
                            }
                        }
                        text_editor::Edit::Paste(pasted_text) => {
                            let pasted_string = pasted_text.to_string();
                            if pasted_string.contains("\n") {
                                return Task::none();
                            }
                        }
                        text_editor::Edit::Enter => {
                            return Task::none();
                        }
                        _ => {}
                    }

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

                Task::none()
            }
            Message::TempTopBarMessage => {
                println!("topbar");

                Task::none()
            }
            Message::Calender(calender_message) => {
                match calender_message {
                    CalenderMessage::DayButton(new_day, month) => {
                        let new_datetime = match month {
                            calender::Month::Last => {
                                let days_in_last_month =
                                    if self.global_store.date_time().month() == 1 {
                                        31
                                    } else {
                                        let nd = NaiveDate::from_ymd_opt(
                                            self.global_store.date_time().year(),
                                            self.global_store.date_time().month() - 1,
                                            1,
                                        )
                                        .expect("bad date");

                                        nd.num_days_in_month() as u32
                                    };

                                let days_to_go_back = (days_in_last_month - new_day)
                                    + self.global_store.date_time().day();

                                self.global_store
                                    .date_time()
                                    .checked_sub_days(Days::new(days_to_go_back as u64))
                                    .expect("couldn't go into the past")
                            }
                            calender::Month::Current => {
                                let delta_day =
                                    (new_day as i32) - (self.global_store.date_time().day() as i32);

                                let mag_delta_day = delta_day.unsigned_abs() as u64;

                                if delta_day == 0 {
                                    return Task::none();
                                }
                                if delta_day < 0 {
                                    self.global_store
                                        .date_time()
                                        .checked_sub_days(Days::new(mag_delta_day))
                                        .expect("couldn't jump into the past")
                                } else {
                                    self.global_store
                                        .date_time()
                                        .checked_add_days(Days::new(mag_delta_day))
                                        .expect("couldn't jump into the future")
                                }
                            }
                            calender::Month::Next => {
                                let days_to_go_forward =
                                    (self.global_store.date_time().num_days_in_month() as u64
                                        - self.global_store.date_time().day() as u64)
                                        + new_day as u64;

                                self.global_store
                                    .date_time()
                                    .checked_add_days(Days::new(days_to_go_forward))
                                    .expect("couldn't go into the future")
                            }
                        };

                        self.reload_date(new_datetime);
                    }
                    CalenderMessage::BackMonth => {
                        let new_datetime = self
                            .global_store
                            .date_time()
                            .checked_sub_months(Months::new(1))
                            .expect("couldn't go back a month");

                        self.reload_date(new_datetime);
                    }
                    CalenderMessage::ForwardMonth => {
                        let new_datetime = self
                            .global_store
                            .date_time()
                            .checked_add_months(Months::new(1))
                            .expect("couldn't go forward a month");

                        self.reload_date(new_datetime);
                    }
                    CalenderMessage::BackYear => {
                        let new_datetime = self
                            .global_store
                            .date_time()
                            .checked_sub_months(Months::new(12))
                            .expect("couldn't go back a year");

                        self.reload_date(new_datetime);
                    }
                    CalenderMessage::ForwardYear => {
                        let new_datetime = self
                            .global_store
                            .date_time()
                            .checked_add_months(Months::new(12))
                            .expect("couldn't go forward a year");

                        self.reload_date(new_datetime);
                    }
                }

                Task::none()
            }
            Message::KeyEvent(event) => {
                if let Some(action) = self.keybinds.dispatch(event) {
                    let key_action = action.clone();

                    self.update(Message::ManualKeyEvent(key_action))
                } else {
                    Task::none()
                }
            }
            Message::ManualKeyEvent(event) => {
                self.show_context_menu = false;

                match event {
                    KeyboardAction::Save => {
                        self.save_all();
                    }
                    KeyboardAction::BackspaceWord => {
                        self.last_edit_time = Local::now();

                        let stopping_chars = [
                            ' ', '.', '!', '?', ',', '-', '_', '\"', ';', ':', '(', ')', '{', '}',
                            '[', ']',
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
                        self.last_edit_time = Local::now();

                        let stopping_chars = [
                            ' ', '.', '!', '?', ',', '-', '_', '\"', ';', ':', '(', ')', '{', '}',
                            '[', ']',
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
                    KeyboardAction::JumpToContentStart => {
                        if let Some((active_content, _)) = self.active_content_and_history_stack() {
                            active_content
                                .perform(Action::Move(text_editor::Motion::DocumentStart));

                            return snap_to(Id::new(LOG_EDIT_AREA_ID), RelativeOffset::START);
                        }
                    }
                    KeyboardAction::JumpToContentEnd => {
                        if let Some((active_content, _)) = self.active_content_and_history_stack() {
                            active_content.perform(Action::Move(text_editor::Motion::DocumentEnd));

                            return snap_to(Id::new(LOG_EDIT_AREA_ID), RelativeOffset::END);
                        }
                    }
                    KeyboardAction::Unbound(unbounded_action) => {
                        match unbounded_action {
                            UnboundKey::Cut => {
                                if let Some(selection) = self.content.selection() {
                                    let clipboard_repsonse = self.clipboard.set_contents(selection);

                                    if clipboard_repsonse.is_err() {
                                        LOGBOX
                                            .write()
                                            .expect("couldn't get logbox write")
                                            .log("Unable to write to clipboard");
                                    } else {
                                        return self.update(Message::Edit(Action::Edit(
                                            text_editor::Edit::Backspace,
                                        )));
                                    }
                                }
                            }
                            UnboundKey::Copy => {
                                let copied_text = if let Some(selection) = self.content.selection()
                                {
                                    selection
                                } else {
                                    // if there is no selection, copy the entire line the cursor is in
                                    let (line, _char) = self.content.cursor_position();

                                    self.content
                                        .line(line)
                                        .expect("couldn't extract line")
                                        .to_string()
                                };

                                let clipboard_repsonse = self.clipboard.set_contents(copied_text);

                                if clipboard_repsonse.is_err() {
                                    LOGBOX
                                        .write()
                                        .expect("couldn't get logbox write")
                                        .log("Unable to write to clipboard");
                                }
                            }
                            UnboundKey::Paste => {
                                let clipboard_contents = self.clipboard.get_contents();

                                if let Ok(clipboard_text) = clipboard_contents {
                                    return self.update(Message::Edit(Action::Edit(
                                        text_editor::Edit::Paste(clipboard_text.into()),
                                    )));
                                } else {
                                    LOGBOX
                                        .write()
                                        .expect("couldn't get logbox write")
                                        .log("Unable to read from clipboard");
                                }
                            }
                        }
                    }
                }

                Task::none()
            }
            Message::TableSearch(table_message) => {
                let SearchTableMessage::EntryClicked(table_time) = table_message;

                self.reload_date(table_time);

                snap_to(Id::new(LOG_EDIT_AREA_ID), RelativeOffset::START)
            }
            Message::TabSwitched(tab) => {
                self.write_active_entry_to_store();

                self.current_tab = tab;

                if self.current_tab == Tab::Stats {
                    self.global_store.update_word_count();
                }

                Task::none()
            }
            Message::AcceptSpellcheck(suggestion_idx) => {
                let exit_message = self.update(Message::ExitContextMenu);

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

                exit_message
            }
            Message::AddToDictionary(word) => {
                let exit_message = self.update(Message::ExitContextMenu);

                dictionary::add_word_to_personal_dictionary(&word);

                exit_message
            }
            Message::Render => {
                self.view();

                Task::none()
            }
            Message::ClearSearch => {
                self.search_content = Content::new();

                self.update(Message::EditSearch(Action::Move(
                    text_editor::Motion::DocumentEnd,
                )))
            }
            Message::ToggleSearchCase => {
                self.settings.ignore_search_case = !self.settings.ignore_search_case;

                self.update(Message::EditSearch(Action::Move(
                    text_editor::Motion::DocumentEnd,
                )))
            }
            Message::MouseMoved(new_position) => {
                self.mouse_position = new_position;

                Task::none()
            }
            Message::RightClickEditArea => {
                self.captured_mouse_position = self.mouse_position;
                self.captured_window_mouse_position = self.window_mouse_position;

                self.show_context_menu = true;

                Task::none()
            }
            Message::ExitContextMenu => {
                self.show_context_menu = false;

                Task::none()
            }
            Message::WindowEvent(event) => {
                match event {
                    window::Event::Opened {
                        position: _position,
                        size: inital_size,
                    } => {
                        self.window_size = inital_size;
                    }
                    window::Event::Resized(new_size) => {
                        self.window_size = new_size;
                    }
                    _ => {}
                }

                Task::none()
            }
            Message::WindowMouseMoved(new_point) => {
                self.window_mouse_position = new_point;

                Task::none()
            }
            Message::MenuBar(menu_message) => {
                self.menu_bar.set_active_dropdown(None);

                match menu_message {
                    MenuMessage::ClickedAway => {
                        self.menu_bar.set_active_dropdown(None);
                    }
                    MenuMessage::ClickedMenu(menu_index) => {
                        self.menu_bar.set_active_dropdown(Some(menu_index));
                    }
                    MenuMessage::File(file_message) => match file_message {
                        FileMessage::Save => {
                            return self.update(Message::ManualKeyEvent(KeyboardAction::Save));
                        }
                    },
                    MenuMessage::Edit(edit_message) => match edit_message {
                        EditMessage::Undo => {
                            return self.update(Message::ManualKeyEvent(KeyboardAction::Undo));
                        }
                        EditMessage::Redo => {
                            return self.update(Message::ManualKeyEvent(KeyboardAction::Redo));
                        }
                        EditMessage::Cut => {
                            return self.update(Message::ManualKeyEvent(KeyboardAction::Unbound(
                                UnboundKey::Cut,
                            )));
                        }
                        EditMessage::Copy => {
                            return self.update(Message::ManualKeyEvent(KeyboardAction::Unbound(
                                UnboundKey::Copy,
                            )));
                        }
                        EditMessage::Paste => {
                            return self.update(Message::ManualKeyEvent(KeyboardAction::Unbound(
                                UnboundKey::Paste,
                            )));
                        }
                    },
                }

                Task::none()
            }
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        let subscriptions = vec![
            listen_with(|event, _, _| match event {
                Event::Keyboard(key_event) => Some(Message::KeyEvent(key_event)),
                Event::Window(window_event) => Some(Message::WindowEvent(window_event)),
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
        keybinds
            .bind("Ctrl+Up", KeyboardAction::JumpToContentStart)
            .expect("couldn't bind Ctrl+Up");
        keybinds
            .bind("Ctrl+Down", KeyboardAction::JumpToContentEnd)
            .expect("couldn't bind Ctrl+Down");

        let clipboard = ClipboardContext::new().expect("couldn't get clipboard");

        let mut df = Self {
            window_title: String::default(),
            content: text_editor::Content::default(),
            search_content: text_editor::Content::default(),
            search_text: String::default(),
            calender: Calender::default(),
            search_table: SearchTable::default(),
            keybinds,
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
            show_context_menu: false,
            mouse_position: Point::default(),
            captured_mouse_position: Point::default(),
            window_size: Size::default(),
            window_mouse_position: Point::default(),
            captured_window_mouse_position: Point::default(),
            clipboard,
            menu_bar: build_menu_bar(),
        };

        df.global_store.load_all();
        df.global_store.update_word_count();
        df.content = Content::with_text(&df.global_store.day().get_day_text());

        let _ = df.update(Message::JumpToToday);

        df
    }
}

fn main() -> iced::Result {
    iced::application(App::title, App::update, App::view)
        .subscription(App::subscription)
        .run()
}
