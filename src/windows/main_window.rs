use chrono::{DateTime, Datelike, Days, Local, Months, NaiveDate};
use iced::Length::Fill;
use iced::widget::operation::snap_to;
use iced::widget::scrollable::{AbsoluteOffset, RelativeOffset, Viewport};
use iced::widget::text_editor::Action;
use iced::widget::{Id, Space, Text, opaque, stack, tooltip};
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
use strum::Display;

use super::window_manager::{WindowType, Windowable};

use crate::config::{preferences, preferences_mut};
use crate::content::{ContentAction, UpgradedContent};
use crate::custom_widgets::calender::{Calender, CalenderColormap, CalenderMessage};
use crate::custom_widgets::context_menu::{
    ContextMenuElement, ContextMenuItem, build_context_menu,
};
use crate::custom_widgets::menu_bar::{MenuBar, menu_bar};
use crate::custom_widgets::menu_bar_builder::{
    EditMessage, FileMessage, MENU_BAR_HEIGHT, MenuMessage, Menus, ToolsMessage, build_menu_bar,
};
use crate::custom_widgets::search_table::{SearchTable, SearchTableMessage};
use crate::custom_widgets::tabview::{TabviewItem, tabview_content_vertical};
use crate::dialogs::DialogType;
use crate::keyboard_manager::{KeyboardAction, TextEdit, UnboundKey};
use crate::md_image::markdown_image::{self, ImageCache, ParsedMarkdown};
use crate::store::{TimedWordCount, WordCount};
use crate::tasks::task_manager::TaskMessage;
use crate::tasks::template_tasks::TemplateData;
use crate::tasks::{StandardMessage, TaskId};
use crate::ui::highlighter::{self, HighlightSettings, SpellHighlighter};
use crate::ui::journal_theme::LIGHT;
use crate::ui::layout::{
    DASHBOARD_TAB_CONTENT_HEIGHT, DASHBOARD_WIDTH, EDITOR_WIDTH, LOGBOX_HEIGHT, SCROLLBAR_WIDTH,
};
use crate::ui::styling::{TOOLTIP_DELAY, TOOLTIP_SIZE};
use crate::ui::ui_tools;
use crate::utils::clipboard::{read_clipboard, write_clipboard};
use crate::utils::dictionary::{self, DICTIONARY};
use crate::utils::logbox::{logbox, logbox_mut};
use crate::utils::misc_tools::point_on_edge_of_text;
use crate::{SharedAppState, UpstreamAction};

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

#[derive(Debug, Clone)]
/// What gets displayed on the editor area
pub enum EditorMode {
    /// Text editor only
    Editor,

    /// Both text editor and live markdown render
    SplitView,

    /// Markdown render only
    View,
}

#[derive(Debug, PartialEq)]
/// options for the currently active text_editor
pub enum ActiveContent {
    Editor,
    Search,
    /// the TemplateTaskMessage stores which task has the editor, so we don't need to store anything else
    Task(TaskId),
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
    show_context_menu: bool,
    mouse_position: Point,
    captured_mouse_position: Point,
    window_size: Size,
    window_mouse_position: Point,
    captured_window_mouse_position: Point,
    menu_bar: MenuBar<MainMessage>,
    editor_scroll_offset: AbsoluteOffset,
    editor_mode: EditorMode,
    editor_markdown: Vec<ParsedMarkdown>,
    markdown_image_cache: ImageCache,
}

#[derive(Debug, Default, Clone)]
pub enum MainMessage {
    #[default]
    EmptyMessage,
    KeyEvent(KeyboardAction),
    WindowEvent(window::Event),

    OpenFileImportWindow,
    OpenFileExportWindow,
    OpenPreferencesWindow,

    BackOneDay,
    ForwardOneDay,
    JumpToToday,
    Edit(text_editor::Action),
    EditSearch(text_editor::Action),
    SwitchEditorMode(EditorMode),
    Markdown,
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
    EditorScrolled(Viewport),
    AddTask,
    TaskAction(TaskMessage),
    Autosave,
}

const LOG_EDIT_AREA_ID: &str = "log_edit_area";

impl Windowable<MainMessage> for Main {
    fn title(&self) -> String {
        self.title.clone()
    }

    fn view<'a>(&'a self, state: &'a SharedAppState) -> Element<'a, MainMessage> {
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

        let daily_nav_bar = row![back_button, today_button, forward_button].width(DASHBOARD_WIDTH);

        let calender = self.calender.build_calender().map(MainMessage::Calender);

        let (tasks_tab_content, tasks_tab_overlay) = {
            const ADD_BUTTON_WIDTH: f32 = 40.0;
            const ADD_MARGIN: f32 = 5.0;

            let content = column![
                state
                    .task_manager
                    .build_tasks(state.global_store.current_date())
                    .map(MainMessage::TaskAction),
                Space::new().height(ADD_BUTTON_WIDTH + ADD_MARGIN * 2.0)
            ];

            let add_button_h_padding = Space::new()
                .width(DASHBOARD_WIDTH - ADD_BUTTON_WIDTH - ADD_MARGIN - SCROLLBAR_WIDTH);
            let add_button_v_padding =
                Space::new().height(DASHBOARD_TAB_CONTENT_HEIGHT - ADD_BUTTON_WIDTH);

            let add_button = widget::Button::new(Text::new("+").align_x(Center).align_y(Center))
                .on_press(MainMessage::AddTask)
                .width(ADD_BUTTON_WIDTH)
                .height(ADD_BUTTON_WIDTH);

            let add_button_tooltip = tooltip(
                add_button,
                Text::new("Create New Task").size(TOOLTIP_SIZE),
                tooltip::Position::Left,
            )
            .delay(TOOLTIP_DELAY);

            let add_button_h_padded = row![add_button_h_padding, add_button_tooltip];

            let overlay = column![add_button_v_padding, add_button_h_padded];

            (content, overlay)
        };

        let tasks_tab = TabviewItem {
            title: Tab::Tasks.to_string(),
            clicked_message: MainMessage::TabSwitched(Tab::Tasks),
            content: tasks_tab_content.into(),
            overlay: Some(tasks_tab_overlay.into()),
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
                Text::new("Clear Search").size(TOOLTIP_SIZE),
                tooltip::Position::Top,
            )
            .delay(TOOLTIP_DELAY);

            let match_case_tooltip_text = if preferences().search.ignore_search_case {
                "Match Case"
            } else {
                "Ignore Case"
            };

            let match_case_tooltip = tooltip(
                match_case_button,
                Text::new(match_case_tooltip_text).size(TOOLTIP_SIZE),
                tooltip::Position::Top,
            )
            .delay(TOOLTIP_DELAY);

            let search_line = row![
                searchbar,
                clear_search_tooltip,
                match_case_tooltip,
                Space::new().width(SCROLLBAR_WIDTH)
            ]
            .width(DASHBOARD_WIDTH);

            let search_results =
                SearchTable::view(&self.search_table).map(MainMessage::TableSearch);

            column![search_line, search_results]
        };

        let search_tab = TabviewItem {
            title: Tab::Search.to_string(),
            clicked_message: MainMessage::TabSwitched(Tab::Search),
            content: search_tab_content.into(),
            overlay: None,
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
            title: Tab::Stats.to_string(),
            clicked_message: MainMessage::TabSwitched(Tab::Stats),
            content: stats_tab_content.into(),
            overlay: None,
        };

        let tab_elements = vec![tasks_tab, search_tab, stats_tab];

        let tab_view = tabview_content_vertical(
            tab_elements,
            self.current_tab.to_index(),
            Length::Fixed(DASHBOARD_WIDTH),
            Length::Fill,
        );

        let left_ui = column![daily_nav_bar, calender, tab_view];

        let right_top_bar = row![
            widget::button("Edit Mode")
                .on_press(MainMessage::SwitchEditorMode(EditorMode::Editor))
                .height(100),
            widget::button("Live Preview")
                .on_press(MainMessage::SwitchEditorMode(EditorMode::SplitView))
                .height(100),
            widget::button("View Mode")
                .on_press(MainMessage::SwitchEditorMode(EditorMode::View))
                .height(100),
        ];

        let editor_area = {
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
                        ignore_search_case: preferences().search.ignore_search_case,
                    },
                    highlighter::highlight_to_format,
                );

            match self.editor_mode {
                EditorMode::Editor => row![log_text_input],
                EditorMode::SplitView => {
                    let half_editor = log_text_input.width(EDITOR_WIDTH / 2.0);
                    let half_viewer = markdown_image::build_markdown(
                        &self.editor_markdown,
                        &self.markdown_image_cache,
                        MainMessage::Markdown,
                    );

                    row![half_editor, half_viewer]
                }
                EditorMode::View => {
                    row![markdown_image::build_markdown(
                        &self.editor_markdown,
                        &self.markdown_image_cache,
                        MainMessage::Markdown
                    )]
                }
            }
        };

        let mut spellcheck_context_menu_contents: Vec<(String, MainMessage)> = vec![];

        if self.selected_misspelled_word.is_some() {
            for (i, suggestion) in self.spell_suggestions.iter().enumerate() {
                spellcheck_context_menu_contents
                    .push((suggestion.to_string(), MainMessage::AcceptSpellcheck(i)));
            }
        }

        let mut context_menu_items = Vec::new();

        let spell_suggestions = spellcheck_context_menu_contents
            .into_iter()
            .map(|(text, message)| ContextMenuElement {
                name: text,
                message: Some(message),
            })
            .collect::<Vec<ContextMenuElement<MainMessage>>>();

        if let Some(word) = &self.selected_misspelled_word {
            if spell_suggestions.len() > 0 {
                context_menu_items.push(ContextMenuItem::Text("Did you mean:".to_string()));
                context_menu_items.push(ContextMenuItem::Scroller((spell_suggestions, 6)));
            }

            let contains_whitespace = word.chars().any(|chara| chara.is_whitespace());

            if !contains_whitespace {
                let add_to_dictionary = ContextMenuElement {
                    name: format!("Add \"{}\" to dictionary", word),
                    message: Some(MainMessage::AddToDictionary(word.clone())),
                };

                context_menu_items.push(ContextMenuItem::Button(add_to_dictionary));
            }

            context_menu_items.push(ContextMenuItem::Break);
        }

        let cut_message = state
            .content
            .selection()
            .map(|_selection| MainMessage::KeyEvent(KeyboardAction::Unbound(UnboundKey::Cut)));

        let cut = ContextMenuElement {
            name: "Cut".to_string(),
            message: cut_message,
        };

        let copy_message = state
            .content
            .selection()
            .map(|_selection| MainMessage::KeyEvent(KeyboardAction::Unbound(UnboundKey::Copy)));

        let copy = ContextMenuElement {
            name: "Copy".to_string(),
            message: copy_message,
        };

        let paste_message = MainMessage::KeyEvent(KeyboardAction::Unbound(UnboundKey::Paste));

        let paste = ContextMenuElement {
            name: "Paste".to_string(),
            message: Some(paste_message),
        };

        context_menu_items.push(ContextMenuItem::Button(cut));
        context_menu_items.push(ContextMenuItem::Button(copy));
        context_menu_items.push(ContextMenuItem::Button(paste));
        context_menu_items.push(ContextMenuItem::Break);

        // TODO: extend context menu to all text_editors
        let undo_message = if state.content.undo_stack_height() > 0 {
            Some(MainMessage::KeyEvent(KeyboardAction::Content(
                TextEdit::Undo,
            )))
        } else {
            None
        };

        let undo = ContextMenuElement {
            name: "Undo".to_string(),
            message: undo_message,
        };

        let redo_message = if state.content.redo_stack_height() > 0 {
            Some(MainMessage::KeyEvent(KeyboardAction::Content(
                TextEdit::Redo,
            )))
        } else {
            None
        };

        let redo = ContextMenuElement {
            name: "Redo".to_string(),
            message: redo_message,
        };

        context_menu_items.push(ContextMenuItem::Button(undo));
        context_menu_items.push(ContextMenuItem::Button(redo));

        let mut context_menu_position = self.captured_mouse_position;
        context_menu_position.y += self.editor_scroll_offset.y;

        let context_menu_width = ContextMenuItem::padded_menu_width(&context_menu_items);
        let context_menu_height = ContextMenuItem::menu_height(&context_menu_items);

        let distance_to_window_edge =
            self.window_size.width - self.captured_window_mouse_position.x;

        if distance_to_window_edge < (context_menu_width + 15.0) as f32 {
            context_menu_position.x -= context_menu_width as f32;
        }

        let distance_to_window_bottom =
            self.window_size.height - self.captured_window_mouse_position.y - LOGBOX_HEIGHT;

        if distance_to_window_bottom < context_menu_height {
            context_menu_position.y -= context_menu_height;
        }

        let context_menu = build_context_menu(context_menu_items);

        let aligned_context_menu = if self.show_context_menu {
            let pinned = widget::pin(context_menu).position(context_menu_position);

            let menu_content = opaque(
                mouse_area(pinned)
                    .on_press(MainMessage::ExitContextMenu)
                    .on_right_press(MainMessage::ExitContextMenu)
                    .on_middle_press(MainMessage::ExitContextMenu),
            );

            Some(menu_content)
        } else {
            None
        };

        // TODO: extend to entire window
        let editor_with_menu = stack!(editor_area, aligned_context_menu);

        let scrollable_editor = widget::scrollable(editor_with_menu)
            .width(Length::Fill)
            .on_scroll(MainMessage::EditorScrolled)
            .height(Length::Fill)
            .direction(Direction::Vertical(
                Scrollbar::new()
                    .spacing(0)
                    .margin(0)
                    .width(SCROLLBAR_WIDTH)
                    .scroller_width(SCROLLBAR_WIDTH),
            ))
            .id(Id::new(LOG_EDIT_AREA_ID));

        let mouse_editor_area = mouse_area(scrollable_editor)
            .on_right_release(MainMessage::RightClickEditArea)
            .on_move(MainMessage::MouseMoved);

        let right_ui = column![right_top_bar, mouse_editor_area];

        let top_ui = row![left_ui, right_ui];

        let logbox = widget::text(logbox().get_log_at_time())
            .size(14)
            .font(Font::DEFAULT)
            .height(LOGBOX_HEIGHT);

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
                self.active_content = None;

                let previous_day = state
                    .global_store
                    .current_date()
                    .checked_sub_days(Days::new(1))
                    .expect("failed to go to previous day");

                let new_date = if state.global_store.day().contains_entry()
                    || !preferences().general.smart_navigation
                {
                    previous_day
                } else {
                    state
                        .global_store
                        .get_previous_edited_day(state.global_store.current_date())
                        .unwrap_or(previous_day)
                };

                self.reload_date(state, new_date);

                snap_to(Id::new(LOG_EDIT_AREA_ID), RelativeOffset::START)
            }
            MainMessage::ForwardOneDay => {
                self.active_content = None;

                let next_day = state
                    .global_store
                    .current_date()
                    .checked_add_days(Days::new(1))
                    .expect("failed to go to next day");

                let new_date = if state.global_store.day().contains_entry()
                    || !preferences().general.smart_navigation
                {
                    next_day
                } else {
                    state
                        .global_store
                        .get_next_edited_day(state.global_store.current_date())
                        .unwrap_or(next_day)
                };

                self.reload_date(state, new_date);

                snap_to(Id::new(LOG_EDIT_AREA_ID), RelativeOffset::START)
            }
            MainMessage::JumpToToday => {
                self.active_content = None;

                self.reload_date(state, Local::now().date_naive());

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

                if matches!(self.editor_mode, EditorMode::SplitView) {
                    self.parse_markdown(state);
                }

                match cursor_location {
                    Some(true) => snap_to(Id::new(LOG_EDIT_AREA_ID), RelativeOffset::START),
                    Some(false) => snap_to(Id::new(LOG_EDIT_AREA_ID), RelativeOffset::END),
                    None => Task::none(),
                }
            }
            MainMessage::EditSearch(search_action) => {
                if self.active_content != Some(ActiveContent::Search) {
                    self.write_active_entry_to_store(state);
                }
                self.active_content = Some(ActiveContent::Search);

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
            MainMessage::SwitchEditorMode(new_editor_mode) => {
                self.active_content = None;

                self.editor_mode = new_editor_mode;

                match self.editor_mode {
                    EditorMode::Editor => {}
                    EditorMode::SplitView | EditorMode::View => {
                        self.parse_markdown(state);
                    }
                }

                Task::none()
            }
            MainMessage::Markdown => {
                println!("md!");

                Task::none()
            }
            MainMessage::Calender(calender_message) => {
                self.active_content = None;

                match calender_message {
                    CalenderMessage::DayClicked(new_date) => {
                        self.reload_date(state, new_date);
                    }
                    CalenderMessage::BackMonth => {
                        let new_date = state
                            .global_store
                            .current_date()
                            .checked_sub_months(Months::new(1))
                            .expect("couldn't go back a month");

                        self.reload_date(state, new_date);
                    }
                    CalenderMessage::ForwardMonth => {
                        let new_date = state
                            .global_store
                            .current_date()
                            .checked_add_months(Months::new(1))
                            .expect("couldn't go forward a month");

                        self.reload_date(state, new_date);
                    }
                    CalenderMessage::BackYear => {
                        let new_date = state
                            .global_store
                            .current_date()
                            .checked_sub_months(Months::new(12))
                            .expect("couldn't go back a year");

                        self.reload_date(state, new_date);
                    }
                    CalenderMessage::ForwardYear => {
                        let new_date = state
                            .global_store
                            .current_date()
                            .checked_add_months(Months::new(12))
                            .expect("couldn't go forward a year");

                        self.reload_date(state, new_date);
                    }
                }

                Task::none()
            }
            MainMessage::KeyEvent(event) => {
                self.show_context_menu = false;

                match event {
                    KeyboardAction::Content(text_edit) => {
                        self.content_perform(state, text_edit.to_content_action());

                        self.last_edit_time = Local::now();
                    }
                    KeyboardAction::Save => {
                        self.save_all(state);

                        logbox_mut().log("Saved");
                    }
                    KeyboardAction::Debug => {
                        let dialog_text = "debug!".to_string();

                        state
                            .upstream_actions
                            .push(UpstreamAction::OpenDialog(DialogType::Warning, dialog_text));
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
                self.active_content = None;

                let SearchTableMessage::EntryClicked(table_date) = table_message;

                self.reload_date(state, table_date);

                snap_to(Id::new(LOG_EDIT_AREA_ID), RelativeOffset::START)
            }
            MainMessage::TabSwitched(tab) => {
                self.active_content = None;

                self.write_active_entry_to_store(state);

                self.current_tab = tab;

                match self.current_tab {
                    Tab::Tasks => {
                        self.calender.set_colormap(CalenderColormap::default());
                    }
                    Tab::Search => {
                        self.calender.set_colormap(CalenderColormap::default());
                    }
                    Tab::Stats => {
                        state.global_store.update_word_count();

                        self.calender
                            .set_colormap(self.compute_word_count_colormap(state));
                    }
                }

                Task::none()
            }
            MainMessage::AcceptSpellcheck(suggestion_idx) => {
                let exit_message = self.update(state, MainMessage::ExitContextMenu);

                let selected_suggestion = self.spell_suggestions[suggestion_idx].clone();

                let equivalent_edit = text_editor::Edit::Paste(selected_suggestion.into());

                state
                    .content
                    .perform(ContentAction::Standard(Action::Edit(equivalent_edit)));

                exit_message
            }
            MainMessage::AddToDictionary(word) => {
                let exit_message = self.update(state, MainMessage::ExitContextMenu);

                dictionary::add_word_to_personal_dictionary(&word);

                exit_message
            }
            MainMessage::ClearSearch => {
                // TODO: auto focus
                // self.active_content = Some(ActiveContent::Search);

                self.search_content = UpgradedContent::default();

                self.update(
                    state,
                    MainMessage::EditSearch(Action::Move(text_editor::Motion::DocumentEnd)),
                )
            }
            MainMessage::ToggleSearchCase => {
                // TODO: keep focus?
                self.active_content = None;

                preferences_mut().search.toggle_ignore_search_case();

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

                if state.content.selection().is_none() {
                    state.content.perform(ContentAction::Standard(Action::Click(
                        self.captured_mouse_position,
                    )));

                    state
                        .content
                        .perform(ContentAction::Standard(Action::SelectWord));

                    self.update_spellcheck(state);
                }

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
                            return self.update(
                                state,
                                MainMessage::KeyEvent(KeyboardAction::Content(TextEdit::Undo)),
                            );
                        }
                        EditMessage::Redo => {
                            return self.update(
                                state,
                                MainMessage::KeyEvent(KeyboardAction::Content(TextEdit::Redo)),
                            );
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
                    MenuMessage::Tools(tools_message) => match tools_message {
                        ToolsMessage::Preferences => {
                            return self.update(state, MainMessage::OpenPreferencesWindow);
                        }
                    },
                }

                Task::none()
            }
            MainMessage::OpenFileImportWindow => {
                self.active_content = None;

                state
                    .upstream_actions
                    .push(UpstreamAction::CreateWindow(WindowType::FileImport));

                Task::none()
            }
            MainMessage::OpenFileExportWindow => {
                self.active_content = None;

                state
                    .upstream_actions
                    .push(UpstreamAction::CreateWindow(WindowType::FileExport));

                Task::none()
            }
            MainMessage::OpenPreferencesWindow => {
                self.active_content = None;

                state
                    .upstream_actions
                    .push(UpstreamAction::CreateWindow(WindowType::Preferences));

                Task::none()
            }
            MainMessage::EditorScrolled(viewport) => {
                self.editor_scroll_offset = viewport.absolute_offset();

                Task::none()
            }
            MainMessage::AddTask => {
                self.active_content = None;

                state
                    .upstream_actions
                    .push(UpstreamAction::CreateWindow(WindowType::TaskCreator));

                Task::none()
            }
            MainMessage::TaskAction(template_message) => {
                self.active_content = Some(ActiveContent::Task(template_message.get_id()));

                state
                    .task_manager
                    .update(state.global_store.current_date(), template_message);

                Task::none()
            }
            MainMessage::Autosave => {
                self.save_all(state);

                logbox_mut().log("Autosaved");

                Task::none()
            }
        }
    }

    fn content_perform(&mut self, state: &mut SharedAppState, action: ContentAction) {
        if let Some(active_content) = &self.active_content {
            match active_content {
                ActiveContent::Editor => state.content.perform(action),
                ActiveContent::Search => self.search_content.perform(action),
                ActiveContent::Task(task_id) => {
                    if let Some(task) = state.task_manager.template_tasks.get_task_mut(*task_id) {
                        match task.get_template_mut() {
                            TemplateData::Standard(standard_task) => {
                                if let Some(task_element) =
                                    standard_task.get_element_mut(state.global_store.current_date())
                                {
                                    task_element.update(StandardMessage::TextEdit(action));
                                }
                            }
                            TemplateData::MultiBinary(multi_binary_task) => {
                                if let Some(task_element) = multi_binary_task
                                    .get_element_mut(state.global_store.current_date())
                                {
                                    task_element.content_perform(action);
                                }
                            }
                        }
                    }
                }
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
            show_context_menu: false,
            mouse_position: Point::default(),
            captured_mouse_position: Point::default(),
            window_size: Size::default(),
            window_mouse_position: Point::default(),
            captured_window_mouse_position: Point::default(),
            menu_bar: build_menu_bar(),
            editor_scroll_offset: AbsoluteOffset::default(),
            editor_mode: EditorMode::Editor,
            editor_markdown: Vec::default(),
            markdown_image_cache: ImageCache::default(),
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
            .set_bolded_days(&state.global_store.month().edited_days());

        state.global_store.update_word_count();

        if self.current_tab == Tab::Stats {
            self.calender
                .set_colormap(self.compute_word_count_colormap(state));
        }
    }

    /// writes current entry to store, saves the store to disk, and saves task list to disk
    fn save_all(&mut self, state: &mut SharedAppState) {
        self.write_active_entry_to_store(state);
        state.global_store.save_all();

        state.task_manager.save_all();
    }

    /// reloads the window's title based on the current active date
    fn update_window_title(&mut self, state: &mut SharedAppState) {
        let formated_date = state
            .global_store
            .current_date()
            .format("%A, %B %d, %Y")
            .to_string();
        let new_title = "ironnote - ".to_string() + &formated_date;

        self.title = new_title;
    }

    /// writes the current entry into the store and changes the date of the current entry
    fn reload_date(&mut self, state: &mut SharedAppState, new_date: NaiveDate) {
        self.write_active_entry_to_store(state);

        state.global_store.set_current_store_date(new_date);

        self.update_window_title(state);
        self.calender
            .set_current_date(state.global_store.current_date());
        self.load_active_entry(state);

        self.calender
            .set_bolded_days(&state.global_store.month().edited_days());

        if self.current_tab == Tab::Stats {
            self.calender
                .set_colormap(self.compute_word_count_colormap(state));
        }

        match self.editor_mode {
            EditorMode::Editor => {}
            EditorMode::SplitView | EditorMode::View => {
                self.parse_markdown(state);
            }
        }

        state
            .task_manager
            .template_tasks
            .generate_template_entries(state.global_store.current_date());

        self.last_edit_time = Local::now();

        self.content_perform(state, ContentAction::ClearHistoryStack);
    }

    fn update_spellcheck(&mut self, state: &mut SharedAppState) {
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
            && let Some(dictionary) = DICTIONARY
                .read()
                .expect("couldn't get dictionary read")
                .as_ref()
        {
            let mut spell_suggestions = vec![];

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

    /// Parses the editor text into their format for rendering
    fn parse_markdown(&mut self, state: &SharedAppState) {
        self.editor_markdown =
            markdown_image::parse(&state.content.text(), &mut self.markdown_image_cache);
    }

    fn recompute_search(&mut self, state: &mut SharedAppState) {
        self.search_table.clear();
        self.search_text.clear();

        let search_text = if preferences().search.ignore_search_case {
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

                let content_text = if preferences().search.ignore_search_case {
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

                    let start_text = (day_store.date().to_string()
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

                    self.search_text = bolded_text.clone();

                    self.search_table.insert_element(
                        start_text,
                        bolded_text,
                        end_text,
                        day_store.date(),
                    );
                }
            }
        }
    }

    /// maps the word counts for all days of the calender into the corresponding colormap
    fn compute_word_count_colormap(&self, state: &SharedAppState) -> CalenderColormap {
        let mut char_counts = [0; 42];

        let mut iterative_date = self.calender.calender_start_date();

        for char_count in char_counts.iter_mut() {
            if let Some(day_store) = state.global_store.get_day(iterative_date)
                && iterative_date.month() == state.global_store.current_date().month()
            {
                let day_char_count = day_store.total_char_count();

                *char_count = day_char_count;
            }

            iterative_date = iterative_date
                .checked_add_days(Days::new(1))
                .expect("couldn't add day");
        }

        let colormap_weights = ui_tools::smooth_color_map(char_counts);

        CalenderColormap {
            colormap_weights,
            color_floor: LIGHT.char_count_floor,
            color_ceiling: LIGHT.char_count_ceiling,
            current_day_overwrite: false,
        }
    }
}
