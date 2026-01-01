use crate::calender::{self, Calender, CalenderMessage};
use crate::clipboard::{read_clipboard, write_clipboard};
use crate::config::UserSettings;
use crate::context_menu::context_menu;
use crate::dictionary::{self, DICTIONARY};
use crate::highlighter::{self, HighlightSettings, SpellHighlighter};
use crate::keyboard_manager::{KeyboardAction, UnboundKey};
use crate::logbox::LOGBOX;
use crate::menu_bar::{MenuBar, menu_bar};
use crate::menu_bar_builder::{
    EditMessage, FileMessage, MENU_BAR_HEIGHT, MenuMessage, Menus, build_menu_bar,
};
use crate::misc_tools::point_on_edge_of_text;
use crate::search_table::{SearchTable, SearchTableMessage};
use crate::tabview::{TabviewItem, tab_view};
use crate::template_tasks::TemplateTaskMessage;
use crate::upgraded_content::{ContentAction, CtrlEdit, UpgradedContent};
use crate::window_manager::{WindowType, Windowable};
use crate::word_count::{TimedWordCount, WordCount};
use crate::{SharedAppState, UpstreamAction, misc_tools};
use chrono::{DateTime, Datelike, Days, Local, Months, NaiveDate};
use iced::Length::Fill;
use iced::widget::operation::snap_to;
use iced::widget::scrollable::{AbsoluteOffset, RelativeOffset, Viewport};
use iced::widget::text_editor::Action;
use iced::widget::{Id, Space, Text, tooltip};
use iced::window;
use iced::{
    Alignment::Center,
    Element, Font,
    Length::{self, FillPortion},
    Point, Size, Task,
    widget::{
        self, column, mouse_area, row,
        scrollable::{Direction, Scrollbar},
        text::Wrapping,
        text_editor::{self},
    },
};
use std::time;
use strum::Display;

#[derive(Debug, Default, Clone, PartialEq, Display)]
pub enum Tab {
    #[default]
    Tasks,
    Search,
    Stats,
}

impl Tab {
    pub fn to_index(&self) -> usize {
        match self {
            Tab::Tasks => 0,
            Tab::Search => 1,
            Tab::Stats => 2,
        }
    }
}

#[derive(Debug, PartialEq)]
/// options for the currently active text_editor
pub enum ActiveContent {
    Editor,
    Search,
}

#[derive(Debug)]
pub struct Main {
    title: String,
    active_content: Option<ActiveContent>,
    search_content: UpgradedContent,
    search_text: String,
    calender: Calender,
    search_table: SearchTable,
    current_tab: Tab,
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
    menu_bar: MenuBar<MainMessage>,
    editor_scroll_offset: AbsoluteOffset,
}

#[derive(Debug, Default, Clone)]
pub enum MainMessage {
    #[default]
    EmptyMessage,
    KeyEvent(KeyboardAction),
    WindowEvent(window::Event),

    BackOneDay,
    ForwardOneDay,
    JumpToToday,
    Edit(text_editor::Action),
    EditSearch(text_editor::Action),
    TempTopBarMessage,
    Calender(CalenderMessage),
    TableSearch(SearchTableMessage),
    TabSwitched(Tab),
    AcceptSpellcheck(usize),
    AddToDictionary(String),
    ClearSearch,
    ToggleSearchCase,
    MouseMoved(Point),
    WindowMouseMoved(Point),
    RightClickEditArea,
    ExitContextMenu,
    MenuBar(MenuMessage),
    OpenFileImportWindow,
    OpenFileExportWindow,
    EditorScrolled(Viewport),
    AddTask,
    TaskAction(TemplateTaskMessage),
}

const LOG_EDIT_AREA_ID: &str = "log_edit_area";

impl Windowable<MainMessage> for Main {
    fn title(&self) -> String {
        self.title.clone()
    }

    fn view<'a>(&'a self, state: &'a SharedAppState) -> Element<'a, MainMessage> {
        const TOOLTIP_DELAY: time::Duration = time::Duration::from_millis(600);

        let cursor_line_idx = state.content.cursor_line();
        let cursor_char_idx = state.content.cursor_column();

        let cursor_spellcheck_timed_out = Local::now().signed_duration_since(self.last_edit_time)
            > chrono::Duration::milliseconds(500);

        let back_button = widget::button(widget::Text::new("<---").align_x(Center))
            .on_press(MainMessage::BackOneDay)
            .width(FillPortion(1))
            .height(100);
        let today_button = widget::button(widget::Text::new("Today").align_x(Center))
            .on_press(MainMessage::JumpToToday)
            .width(FillPortion(1))
            .height(100);
        let forward_button = widget::button(widget::Text::new("--->").align_x(Center))
            .on_press(MainMessage::ForwardOneDay)
            .width(FillPortion(1))
            .height(100);

        let daily_nav_bar = row![back_button, today_button, forward_button].width(7 * 36);

        let cal = Calender::view(&self.calender).map(MainMessage::Calender);
        let temp_calender_bar = row![cal];

        let tasks_tab_content = {
            let tasks = column![
                state
                    .all_tasks
                    .build_tasks(state.global_store.date_time().date_naive())
                    .map(MainMessage::TaskAction),
            ];

            let add_button_h_padding = Space::new().width(Length::Fill);

            let add_button = widget::Button::new(Text::new("+").align_x(Center).align_y(Center))
                .on_press(MainMessage::AddTask)
                .width(40)
                .height(40);

            let add_button_tooltip = tooltip(
                add_button,
                Text::new("Create New Task").size(15),
                tooltip::Position::Left,
            )
            .delay(TOOLTIP_DELAY);

            let add_button_layer = row![add_button_h_padding, add_button_tooltip];

            column![tasks, Space::new().height(Fill), add_button_layer]
        };

        let tasks_tab = TabviewItem {
            tab_name: Tab::Tasks.to_string(),
            tab_clicked: MainMessage::TabSwitched(Tab::Tasks),
            content: tasks_tab_content.into(),
        };

        let search_tab_content = {
            let searchbar = widget::text_editor(self.search_content.raw_content())
                .placeholder("Search entries...")
                .on_action(MainMessage::EditSearch)
                .size(13)
                .font(Font::DEFAULT)
                .wrapping(Wrapping::None);

            let clear_search_button = widget::button(widget::Text::new("<=").size(9).center())
                .on_press(MainMessage::ClearSearch)
                .width(32)
                .height(26);
            let match_case_button = widget::button(widget::Text::new("Aa").size(9).center())
                .on_press(MainMessage::ToggleSearchCase)
                .width(32)
                .height(26);

            let clear_search_tooltip = tooltip(
                clear_search_button,
                Text::new("Clear Search").size(13),
                tooltip::Position::Top,
            )
            .delay(TOOLTIP_DELAY);

            let match_case_tooltip_text = if self.settings.ignore_search_case {
                "Match Case"
            } else {
                "Ignore Case"
            };

            let match_case_tooltip = tooltip(
                match_case_button,
                Text::new(match_case_tooltip_text).size(13),
                tooltip::Position::Top,
            )
            .delay(TOOLTIP_DELAY);

            let search_line = row![searchbar, clear_search_tooltip, match_case_tooltip];

            let table = SearchTable::view(&self.search_table).map(MainMessage::TableSearch);

            let search_results = column![table];
            column![search_line, search_results]
        };

        let search_tab = TabviewItem {
            tab_name: Tab::Search.to_string(),
            tab_clicked: MainMessage::TabSwitched(Tab::Search),
            content: search_tab_content.into(),
        };

        let stats_tab_content = {
            let dwc = state.global_store.day().total_word_count().to_string();
            let dcc = state.global_store.day().total_char_count().to_string();

            let mwc = state.global_store.month().total_word_count().to_string();
            let mcc = state.global_store.month().total_char_count().to_string();

            let twc = state.global_store.total_word_count().to_string();
            let tcc = state.global_store.total_char_count().to_string();

            let maw = format!("{:.2}", state.global_store.month().average_words());
            let taw = format!("{:.2}", state.global_store.average_words());
            let mac = format!("{:.2}", state.global_store.month().average_chars());
            let tac = format!("{:.2}", state.global_store.average_chars());

            let longest_streak = format!("{}", state.global_store.longest_streak());
            let current_streak = format!("{}", state.global_store.current_streak());

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
                widget::Text::new("     Current Streak: ".to_string() + &current_streak + " days"),
                widget::Text::new("     Longest Streak: ".to_string() + &longest_streak + " days"),
            ]
        };

        let stats_tab: TabviewItem<'_, MainMessage> = TabviewItem {
            tab_name: Tab::Stats.to_string(),
            tab_clicked: MainMessage::TabSwitched(Tab::Stats),
            content: stats_tab_content.into(),
        };

        let tab_elements = vec![tasks_tab, search_tab, stats_tab];

        let tab_view = tab_view(
            tab_elements,
            self.current_tab.to_index(),
            Length::Fixed(250.0),
            Length::Fill,
        );

        let left_ui = column![daily_nav_bar, temp_calender_bar, tab_view];

        let right_top_bar = row![
            widget::button("test button 0")
                .on_press(MainMessage::TempTopBarMessage)
                .height(100),
            widget::button("test button 1")
                .on_press(MainMessage::TempTopBarMessage)
                .height(100),
            widget::button("test button 2")
                .on_press(MainMessage::TempTopBarMessage)
                .height(100),
        ];

        let log_text_input = widget::text_editor(state.content.raw_content())
            .placeholder("Type today's log...")
            .on_action(MainMessage::Edit)
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

        let mut spellcheck_context_menu_contents: Vec<(String, MainMessage)> = vec![];

        if self.selected_misspelled_word.is_some() {
            for (i, suggestion) in self.spell_suggestions.iter().enumerate() {
                spellcheck_context_menu_contents
                    .push((suggestion.to_string(), MainMessage::AcceptSpellcheck(i)));
            }
        }

        const MENU_SIZE: u32 = 13;
        const MENU_WIDTH: u32 = 125;

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
                    .on_press(MainMessage::AddToDictionary(word.clone()))
                    .width(MENU_WIDTH),
                )
            }
        }

        let cut_message = state
            .content
            .selection()
            .map(|_selection| MainMessage::KeyEvent(KeyboardAction::Unbound(UnboundKey::Cut)));

        let copy_message = state
            .content
            .selection()
            .map(|_selection| MainMessage::KeyEvent(KeyboardAction::Unbound(UnboundKey::Copy)));

        let paste_message = MainMessage::KeyEvent(KeyboardAction::Unbound(UnboundKey::Paste));

        let edit_menu = column![
            widget::button(widget::text("Cut").size(MENU_SIZE))
                .on_press_maybe(cut_message)
                .width(MENU_WIDTH),
            widget::button(widget::text("Copy").size(MENU_SIZE))
                .on_press_maybe(copy_message)
                .width(MENU_WIDTH),
            widget::button(widget::text("Paste").size(MENU_SIZE))
                .on_press(paste_message)
                .width(MENU_WIDTH)
        ];

        // TODO: extend context menu to all text_editors
        let undo_message = if state.content.undo_stack_height() > 0 {
            Some(MainMessage::KeyEvent(KeyboardAction::Undo))
        } else {
            None
        };
        let redo_message = if state.content.redo_stack_height() > 0 {
            Some(MainMessage::KeyEvent(KeyboardAction::Redo))
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
            Space::new().height(3),
            edit_menu,
            Space::new().height(3),
            history_menu
        ];

        let mut context_menu_position = self.captured_mouse_position;
        context_menu_position.y += self.editor_scroll_offset.y;

        let distance_to_window_edge =
            self.window_size.width - self.captured_window_mouse_position.x;

        if distance_to_window_edge < (MENU_WIDTH + 15) as f32 {
            context_menu_position.x -= MENU_WIDTH as f32;
        }

        let editor_with_menu = context_menu(
            log_text_input,
            total_context_menu,
            self.show_context_menu,
            context_menu_position,
            MainMessage::ExitContextMenu,
        );

        let scrollable_editor = widget::scrollable(editor_with_menu)
            .width(Length::Fill)
            .on_scroll(MainMessage::EditorScrolled)
            .height(Length::Fill)
            .direction(Direction::Vertical(Scrollbar::new().spacing(0).margin(2)))
            .id(Id::new(LOG_EDIT_AREA_ID));

        let mouse_editor_area = mouse_area(scrollable_editor)
            .on_right_release(MainMessage::RightClickEditArea)
            .on_move(MainMessage::MouseMoved);

        let right_ui = column![right_top_bar, mouse_editor_area];

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

        let cursor_position_box = widget::Text::new(format!(
            "Ln {}, Col {}",
            cursor_line_idx + 1,
            cursor_char_idx
        ))
        .size(14);

        let bottom_ui = row![logbox, Space::new().width(Fill), cursor_position_box];

        let layout_ui = column![top_ui, bottom_ui];

        let layout_menus = menu_bar(layout_ui.into(), &self.menu_bar, MENU_BAR_HEIGHT);

        let layout = column![mouse_area(layout_menus).on_move(MainMessage::WindowMouseMoved)];

        layout.into()
    }

    fn update(&mut self, state: &mut SharedAppState, message: MainMessage) -> Task<MainMessage> {
        match message {
            MainMessage::EmptyMessage => {
                panic!("uninit message");
            }
            MainMessage::BackOneDay => {
                let previous_day = state
                    .global_store
                    .date_time()
                    .checked_sub_days(Days::new(1))
                    .expect("failed to go to previous day");

                let new_datetime = if state.global_store.day().contains_entry() {
                    previous_day
                } else {
                    state
                        .global_store
                        .get_previous_edited_day(state.global_store.date_time())
                        .unwrap_or(previous_day)
                };

                self.reload_date(state, new_datetime);

                snap_to(Id::new(LOG_EDIT_AREA_ID), RelativeOffset::START)
            }
            MainMessage::ForwardOneDay => {
                let next_day = state
                    .global_store
                    .date_time()
                    .checked_add_days(Days::new(1))
                    .expect("failed to go to next day");

                let new_datetime = if state.global_store.day().contains_entry() {
                    next_day
                } else {
                    state
                        .global_store
                        .get_next_edited_day(state.global_store.date_time())
                        .unwrap_or(next_day)
                };

                self.reload_date(state, new_datetime);

                snap_to(Id::new(LOG_EDIT_AREA_ID), RelativeOffset::START)
            }
            MainMessage::JumpToToday => {
                let new_datetime = Local::now();
                self.reload_date(state, new_datetime);

                snap_to(Id::new(LOG_EDIT_AREA_ID), RelativeOffset::START)
            }
            MainMessage::Edit(editor_action) => {
                self.active_content = Some(ActiveContent::Editor);

                if let Action::Edit(_edit) = &editor_action {
                    self.last_edit_time = Local::now();
                }

                state
                    .content
                    .perform(ContentAction::Standard(editor_action.clone()));

                self.update_spellcheck(state);

                let editor_text = state.content.text();
                let cursor_y = state.content.cursor_line();
                let cursor_x = state.content.cursor_column();
                let cursor_location =
                    point_on_edge_of_text(&editor_text, cursor_x, cursor_y, 3, 400);

                match cursor_location {
                    Some(true) => snap_to(Id::new(LOG_EDIT_AREA_ID), RelativeOffset::START),
                    Some(false) => snap_to(Id::new(LOG_EDIT_AREA_ID), RelativeOffset::END),
                    None => Task::none(),
                }
            }
            MainMessage::EditSearch(search_action) => {
                if self.active_content != Some(ActiveContent::Search) {
                    self.active_content = Some(ActiveContent::Search);

                    self.write_active_entry_to_store(state);
                }

                if let text_editor::Action::Edit(edit) = &search_action {
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
                }

                self.search_content
                    .perform(ContentAction::Standard(search_action.clone()));

                self.recompute_search(state);

                Task::none()
            }
            MainMessage::TempTopBarMessage => {
                println!("topbar");

                Task::none()
            }
            MainMessage::Calender(calender_message) => {
                match calender_message {
                    CalenderMessage::DayButton(new_day, month) => {
                        let new_datetime = match month {
                            calender::Month::Last => {
                                let days_in_last_month =
                                    if state.global_store.date_time().month() == 1 {
                                        31
                                    } else {
                                        let nd = NaiveDate::from_ymd_opt(
                                            state.global_store.date_time().year(),
                                            state.global_store.date_time().month() - 1,
                                            1,
                                        )
                                        .expect("bad date");

                                        nd.num_days_in_month() as u32
                                    };

                                let days_to_go_back = (days_in_last_month - new_day)
                                    + state.global_store.date_time().day();

                                state
                                    .global_store
                                    .date_time()
                                    .checked_sub_days(Days::new(days_to_go_back as u64))
                                    .expect("couldn't go into the past")
                            }
                            calender::Month::Current => {
                                let delta_day = (new_day as i32)
                                    - (state.global_store.date_time().day() as i32);

                                let mag_delta_day = delta_day.unsigned_abs() as u64;

                                if delta_day == 0 {
                                    return Task::none();
                                }
                                if delta_day < 0 {
                                    state
                                        .global_store
                                        .date_time()
                                        .checked_sub_days(Days::new(mag_delta_day))
                                        .expect("couldn't jump into the past")
                                } else {
                                    state
                                        .global_store
                                        .date_time()
                                        .checked_add_days(Days::new(mag_delta_day))
                                        .expect("couldn't jump into the future")
                                }
                            }
                            calender::Month::Next => {
                                let days_to_go_forward =
                                    (state.global_store.date_time().num_days_in_month() as u64
                                        - state.global_store.date_time().day() as u64)
                                        + new_day as u64;

                                state
                                    .global_store
                                    .date_time()
                                    .checked_add_days(Days::new(days_to_go_forward))
                                    .expect("couldn't go into the future")
                            }
                        };

                        self.reload_date(state, new_datetime);
                    }
                    CalenderMessage::BackMonth => {
                        let new_datetime = state
                            .global_store
                            .date_time()
                            .checked_sub_months(Months::new(1))
                            .expect("couldn't go back a month");

                        self.reload_date(state, new_datetime);
                    }
                    CalenderMessage::ForwardMonth => {
                        let new_datetime = state
                            .global_store
                            .date_time()
                            .checked_add_months(Months::new(1))
                            .expect("couldn't go forward a month");

                        self.reload_date(state, new_datetime);
                    }
                    CalenderMessage::BackYear => {
                        let new_datetime = state
                            .global_store
                            .date_time()
                            .checked_sub_months(Months::new(12))
                            .expect("couldn't go back a year");

                        self.reload_date(state, new_datetime);
                    }
                    CalenderMessage::ForwardYear => {
                        let new_datetime = state
                            .global_store
                            .date_time()
                            .checked_add_months(Months::new(12))
                            .expect("couldn't go forward a year");

                        self.reload_date(state, new_datetime);
                    }
                }

                Task::none()
            }
            MainMessage::KeyEvent(event) => {
                self.show_context_menu = false;

                match event {
                    KeyboardAction::Save => {
                        self.save_all(state);
                    }
                    KeyboardAction::BackspaceWord => {
                        self.last_edit_time = Local::now();

                        self.content_perform(state, ContentAction::Ctrl(CtrlEdit::BackspaceWord));
                    }
                    KeyboardAction::BackspaceSentence => {
                        self.last_edit_time = Local::now();

                        self.content_perform(
                            state,
                            ContentAction::Ctrl(CtrlEdit::BackspaceSentence),
                        );
                    }
                    KeyboardAction::DeleteWord => {
                        self.last_edit_time = Local::now();

                        self.content_perform(state, ContentAction::Ctrl(CtrlEdit::DeleteWord));
                    }
                    KeyboardAction::DeleteSentence => {
                        self.last_edit_time = Local::now();

                        self.content_perform(state, ContentAction::Ctrl(CtrlEdit::DeleteSentence));
                    }
                    KeyboardAction::Undo => {
                        self.last_edit_time = Local::now();

                        self.content_perform(state, ContentAction::Undo);
                    }
                    KeyboardAction::Redo => {
                        self.last_edit_time = Local::now();

                        self.content_perform(state, ContentAction::Redo);
                    }
                    KeyboardAction::Debug => {
                        println!("debug!");
                    }
                    KeyboardAction::JumpToContentStart => {
                        self.content_perform(
                            state,
                            ContentAction::Standard(Action::Move(
                                text_editor::Motion::DocumentStart,
                            )),
                        );

                        if self.active_content == Some(ActiveContent::Editor) {
                            return snap_to(Id::new(LOG_EDIT_AREA_ID), RelativeOffset::START);
                        }
                    }
                    KeyboardAction::JumpToContentEnd => {
                        self.content_perform(
                            state,
                            ContentAction::Standard(Action::Move(text_editor::Motion::DocumentEnd)),
                        );

                        if self.active_content == Some(ActiveContent::Editor) {
                            return snap_to(Id::new(LOG_EDIT_AREA_ID), RelativeOffset::END);
                        }
                    }
                    KeyboardAction::Unbound(unbounded_action) => match unbounded_action {
                        UnboundKey::Cut => {
                            if let Some(selection) = state.content.selection() {
                                write_clipboard(selection);

                                return self.update(
                                    state,
                                    MainMessage::Edit(Action::Edit(text_editor::Edit::Backspace)),
                                );
                            }
                        }
                        UnboundKey::Copy => {
                            if let Some(selection) = state.content.selection() {
                                write_clipboard(selection);
                            };
                        }
                        UnboundKey::Paste => {
                            let clipboard_text = read_clipboard();

                            return self.update(
                                state,
                                MainMessage::Edit(Action::Edit(text_editor::Edit::Paste(
                                    clipboard_text.into(),
                                ))),
                            );
                        }
                    },
                }

                Task::none()
            }
            MainMessage::TableSearch(table_message) => {
                let SearchTableMessage::EntryClicked(table_time) = table_message;

                self.reload_date(state, table_time);

                snap_to(Id::new(LOG_EDIT_AREA_ID), RelativeOffset::START)
            }
            MainMessage::TabSwitched(tab) => {
                self.write_active_entry_to_store(state);

                self.current_tab = tab;

                if self.current_tab == Tab::Stats {
                    state.global_store.update_word_count();
                }

                Task::none()
            }
            MainMessage::AcceptSpellcheck(suggestion_idx) => {
                let exit_message = self.update(state, MainMessage::ExitContextMenu);

                let selected_suggestion = self.spell_suggestions[suggestion_idx].clone();

                let equivalent_edit = text_editor::Edit::Paste(selected_suggestion.into());

                self.content_perform(
                    state,
                    ContentAction::Standard(Action::Edit(equivalent_edit)),
                );

                exit_message
            }
            MainMessage::AddToDictionary(word) => {
                let exit_message = self.update(state, MainMessage::ExitContextMenu);

                dictionary::add_word_to_personal_dictionary(&word);

                exit_message
            }
            MainMessage::ClearSearch => {
                // TODO: auto focus
                self.search_content = UpgradedContent::default();

                self.update(
                    state,
                    MainMessage::EditSearch(Action::Move(text_editor::Motion::DocumentEnd)),
                )
            }
            MainMessage::ToggleSearchCase => {
                self.settings.ignore_search_case = !self.settings.ignore_search_case;

                self.update(
                    state,
                    MainMessage::EditSearch(Action::Move(text_editor::Motion::DocumentEnd)),
                )
            }
            MainMessage::MouseMoved(new_position) => {
                self.mouse_position = new_position;

                Task::none()
            }
            MainMessage::RightClickEditArea => {
                self.captured_mouse_position = self.mouse_position;
                self.captured_window_mouse_position = self.window_mouse_position;

                self.show_context_menu = true;

                Task::none()
            }
            MainMessage::ExitContextMenu => {
                self.show_context_menu = false;

                Task::none()
            }
            MainMessage::WindowEvent(event) => {
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
            MainMessage::WindowMouseMoved(new_point) => {
                self.window_mouse_position = new_point;

                if self.menu_bar.is_dropdown_visible()
                    && self.window_mouse_position.y < MENU_BAR_HEIGHT as f32
                    && self.window_mouse_position.x < Menus::total_bar_width() as f32
                    && let Some(menu) =
                        Menus::menu_from_position(self.window_mouse_position.x as u32)
                {
                    return self
                        .update(state, MainMessage::MenuBar(MenuMessage::ClickedMenu(menu)));
                }

                Task::none()
            }
            MainMessage::MenuBar(menu_message) => {
                self.menu_bar.set_active_dropdown(None);

                match menu_message {
                    MenuMessage::ClickedAway => {
                        self.menu_bar.set_active_dropdown(None);
                    }
                    MenuMessage::ClickedMenu(menu) => {
                        self.menu_bar.set_active_dropdown(Some(menu.menu_index()));
                    }
                    MenuMessage::File(file_message) => match file_message {
                        FileMessage::Save => {
                            return self.update(state, MainMessage::KeyEvent(KeyboardAction::Save));
                        }
                        FileMessage::Import => {
                            return self.update(state, MainMessage::OpenFileImportWindow);
                        }
                        FileMessage::Export => {
                            return self.update(state, MainMessage::OpenFileExportWindow);
                        }
                    },
                    MenuMessage::Edit(edit_message) => match edit_message {
                        EditMessage::Undo => {
                            return self.update(state, MainMessage::KeyEvent(KeyboardAction::Undo));
                        }
                        EditMessage::Redo => {
                            return self.update(state, MainMessage::KeyEvent(KeyboardAction::Redo));
                        }
                        EditMessage::Cut => {
                            return self.update(
                                state,
                                MainMessage::KeyEvent(KeyboardAction::Unbound(UnboundKey::Cut)),
                            );
                        }
                        EditMessage::Copy => {
                            return self.update(
                                state,
                                MainMessage::KeyEvent(KeyboardAction::Unbound(UnboundKey::Copy)),
                            );
                        }
                        EditMessage::Paste => {
                            return self.update(
                                state,
                                MainMessage::KeyEvent(KeyboardAction::Unbound(UnboundKey::Paste)),
                            );
                        }
                    },
                }

                Task::none()
            }
            MainMessage::OpenFileImportWindow => {
                state.upstream_action = Some(UpstreamAction::CreateWindow(WindowType::FileImport));

                Task::none()
            }
            MainMessage::OpenFileExportWindow => {
                state.upstream_action = Some(UpstreamAction::CreateWindow(WindowType::FileExport));

                Task::none()
            }
            MainMessage::EditorScrolled(viewport) => {
                self.editor_scroll_offset = viewport.absolute_offset();

                Task::none()
            }
            MainMessage::AddTask => {
                state.upstream_action = Some(UpstreamAction::CreateWindow(WindowType::TaskCreator));

                Task::none()
            }
            MainMessage::TaskAction(template_message) => {
                state.all_tasks.template_tasks.update(template_message);

                Task::none()
            }
        }
    }
}

impl Default for Main {
    fn default() -> Self {
        Self {
            title: String::default(),
            active_content: None,
            search_content: UpgradedContent::default(),
            search_text: String::default(),
            calender: Calender::default(),
            search_table: SearchTable::default(),
            current_tab: Tab::default(),
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
            menu_bar: build_menu_bar(),
            editor_scroll_offset: AbsoluteOffset::default(),
        }
    }
}

impl Main {
    /// retrieves the text from the store and overwrites the content with it
    fn load_active_entry(&mut self, state: &mut SharedAppState) {
        state.content = UpgradedContent::with_text(&state.global_store.day().get_day_text());
    }

    /// write the current text into the store
    fn write_active_entry_to_store(&mut self, state: &mut SharedAppState) {
        let current_text = state.content.text();

        state.global_store.day_mut().set_day_text(current_text);

        self.calender
            .set_edited_days(state.global_store.month().edited_days());

        state.global_store.update_word_count();
    }

    /// writes current entry to store, saves the store to disk, and saves task list to disk
    fn save_all(&mut self, state: &mut SharedAppState) {
        self.write_active_entry_to_store(state);
        state.global_store.save_all();

        state.all_tasks.save_all();
    }

    /// reloads the window's title based on the current active date
    fn update_window_title(&mut self, state: &mut SharedAppState) {
        let formated_date = state
            .global_store
            .date_time()
            .format("%A, %B %d, %Y")
            .to_string();
        let new_title = "ironnote - ".to_string() + &formated_date;

        self.title = new_title;
    }

    /// writes the current entry into the store and changes the date of the current entry
    fn reload_date(&mut self, state: &mut SharedAppState, new_datetime: DateTime<Local>) {
        self.write_active_entry_to_store(state);

        state.global_store.set_current_store_date(new_datetime);

        self.update_window_title(state);
        self.calender
            .update_calender_dates(state.global_store.date_time());
        self.load_active_entry(state);

        self.calender
            .set_edited_days(state.global_store.month().edited_days());

        state
            .all_tasks
            .template_tasks
            .generate_template_entries(state.global_store.date_time().date_naive());

        self.last_edit_time = Local::now();

        self.content_perform(state, ContentAction::ClearHistoryStack);
    }

    fn update_spellcheck(&mut self, state: &mut SharedAppState) {
        // TODO: allow direct right clicking on misspelled words without selection requirements
        // TODO: compute suggestions on another thread for better performance?

        // computing spellcheck suggestions is extremely expensive, so we only do so when the selection size has
        // changed
        let recompute_spell_suggestions = if let Some(selection) = state.content.selection() {
            self.selected_misspelled_word.replace(selection.clone()) != Some(selection)
        } else {
            self.spell_suggestions.clear();
            self.selected_misspelled_word = None;
            false
        };

        if let Some(selection) = state.content.selection()
            && !selection.contains(char::is_whitespace)
            && recompute_spell_suggestions
        {
            let mut spell_suggestions = vec![];

            let dictionary = DICTIONARY.read().expect("couldn't get dicitonary read");
            if !dictionary.check(&selection) {
                dictionary.suggest(&selection, &mut spell_suggestions);
                self.selected_misspelled_word = Some(selection.clone());

                state.global_store.update_word_count();

                let mut sorted_suggestions: Vec<_> = spell_suggestions
                    .iter()
                    .map(|word| {
                        let word_count = state.global_store.get_word_count(&word.to_lowercase());

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

    fn recompute_search(&mut self, state: &mut SharedAppState) {
        self.search_table.clear();
        self.search_text.clear();

        let search_text = if self.settings.ignore_search_case {
            self.search_content.text().to_lowercase()
        } else {
            self.search_content.text()
        };

        if search_text.is_empty() || search_text == " " {
            return;
        }

        for month_store in state.global_store.month_stores().rev() {
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

    /// performs the content action on the text editor it would apply to
    fn content_perform(&mut self, state: &mut SharedAppState, action: ContentAction) {
        if let Some(active_content) = &self.active_content {
            match active_content {
                ActiveContent::Editor => state.content.perform(action),
                ActiveContent::Search => self.search_content.perform(action),
            }
        }
    }
}
