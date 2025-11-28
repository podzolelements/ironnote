use crate::{
    file_import_window::{FileImport, FileImportMessage},
    global_store::GlobalStore,
    keyboard_manager::{KeyboardAction, bind_keybinds},
    main_window::{Main, MainMessage},
    window_manager::{WindowType, Windowable},
    word_count::WordCount,
};
use iced::{Element, Event, Subscription, Task, event::listen_with, keyboard, widget::column};
use iced::{
    widget::text_editor::{self, Content},
    window,
};
use keybinds::Keybinds;
use std::collections::BTreeMap;

mod calender;
mod clipboard;
mod config;
mod content_tools;
mod context_menu;
mod day_store;
mod dictionary;
mod file_import_window;
mod filetools;
mod global_store;
mod highlighter;
mod history_stack;
mod keyboard_manager;
mod logbox;
mod main_window;
mod menu_bar;
mod menu_bar_builder;
mod misc_tools;
mod month_store;
mod search_table;
mod window_manager;
mod word_count;

#[derive(Debug)]
/// stores the application state that needs to be shared between different windows
struct SharedAppState {
    content: text_editor::Content,
    global_store: GlobalStore,
}

impl Default for SharedAppState {
    fn default() -> Self {
        let mut global_store = GlobalStore::default();
        global_store.load_all();
        global_store.update_word_count();

        let content = Content::with_text(&global_store.day().get_day_text());

        Self {
            content,
            global_store,
        }
    }
}

struct App {
    shared_state: SharedAppState,
    keybinds: Keybinds<KeyboardAction>,
    windows: BTreeMap<window::Id, WindowType>,
    main_window: Main,
    file_import_window: FileImport,
}

#[derive(Debug, Clone)]
pub enum Message {
    CapturedKeyEvent((keyboard::Event, window::Id)),
    KeyEvent((KeyboardAction, window::Id)),
    WindowEvent((window::Event, window::Id)),
    WindowOpened(window::Id, WindowType),
    WindowClosed(window::Id),
    RenderAll,

    MainWindow(MainMessage),
    FileImportWindow(FileImportMessage),
}

#[derive(Debug)]
/// allows for windows to pass up requests to be done by the main application, since they don't have access to the main
// application Messages
pub enum UpsteamAction {
    CreateWindow(WindowType),
}

impl App {
    fn new() -> (Self, Task<Message>) {
        let window_type = WindowType::Main;

        let (_new_id, task) = iced::window::open(window_type.settings());

        let mut app = Self::default();

        let generate_window = task.map(move |id| Message::WindowOpened(id, window_type.clone()));
        let jump_today = app.update(Message::MainWindow(MainMessage::JumpToToday));

        let tasks = vec![generate_window, jump_today];

        (app, Task::batch(tasks))
    }

    fn title(&self, id: window::Id) -> String {
        if let Some(window_type) = self.windows.get(&id) {
            match window_type {
                WindowType::Main => self.main_window.title(),
                WindowType::FileImport => self.file_import_window.title(),
            }
        } else {
            "orphaned window".to_string()
        }
    }

    pub fn view(&'_ self, id: window::Id) -> Element<'_, Message> {
        if let Some(window_type) = self.windows.get(&id) {
            match window_type {
                WindowType::Main => self
                    .main_window
                    .view(&self.shared_state)
                    .map(Message::MainWindow),
                WindowType::FileImport => self
                    .file_import_window
                    .view(&self.shared_state)
                    .map(Message::FileImportWindow),
            }
        } else {
            column![].into()
        }
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::WindowOpened(new_window_id, new_window_type) => {
                self.windows.insert(new_window_id, new_window_type);

                Task::none()
            }
            Message::WindowClosed(id) => {
                if let Some(window_closed) = self.windows.get(&id)
                    && *window_closed == WindowType::Main
                {
                    return iced::exit();
                }

                self.windows.remove(&id);

                Task::none()
            }
            Message::RenderAll => {
                for window_id in self.windows.keys() {
                    self.view(*window_id);
                }

                Task::none()
            }
            Message::CapturedKeyEvent((event, id)) => {
                if let Some(action) = self.keybinds.dispatch(event) {
                    let key_action = action.clone();

                    return self.update(Message::KeyEvent((key_action, id)));
                }

                Task::none()
            }
            Message::KeyEvent((keyboard_action, window_id)) => {
                if let Some(window_type) = self.windows.get(&window_id) {
                    match window_type {
                        WindowType::Main => {
                            return self.update(Message::MainWindow(MainMessage::KeyEvent(
                                keyboard_action,
                            )));
                        }
                        WindowType::FileImport => {}
                    }
                }

                Task::none()
            }
            Message::WindowEvent((event, window_id)) => {
                if let Some(window_type) = self.windows.get(&window_id) {
                    match window_type {
                        WindowType::Main => {
                            return self
                                .update(Message::MainWindow(MainMessage::WindowEvent(event)));
                        }
                        WindowType::FileImport => {}
                    }
                }

                Task::none()
            }
            Message::MainWindow(main_message) => {
                let mut tasks = vec![];

                tasks.push(
                    self.main_window
                        .update(&mut self.shared_state, main_message)
                        .map(Message::MainWindow),
                );

                if let Some(action) = &self.main_window.upstream_action {
                    match action {
                        UpsteamAction::CreateWindow(window_type) => {
                            let new_window_type = window_type.clone();

                            let mut window_already_exists = false;

                            for (window_id, window_type) in &self.windows {
                                if new_window_type == *window_type {
                                    tasks.push(window::gain_focus(*window_id));
                                    window_already_exists = true;
                                    break;
                                }
                            }

                            if !window_already_exists {
                                let (_new_id, task) =
                                    iced::window::open(new_window_type.settings());
                                tasks.push(task.map(move |id| {
                                    Message::WindowOpened(id, new_window_type.clone())
                                }));
                            }
                        }
                    }

                    self.main_window.upstream_action = None;
                }

                Task::batch(tasks)
            }
            Message::FileImportWindow(file_import_message) => self
                .file_import_window
                .update(&mut self.shared_state, file_import_message)
                .map(Message::FileImportWindow),
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        let subscriptions = vec![
            iced::window::close_events().map(Message::WindowClosed),
            listen_with(|event, _status, id| match event {
                Event::Keyboard(key_event) => Some(Message::CapturedKeyEvent((key_event, id))),
                Event::Window(window_event) => Some(Message::WindowEvent((window_event, id))),
                _ => None,
            }),
            // ensure view() gets called at a minimum of 10 FPS
            iced::time::every(std::time::Duration::from_millis(100))
                .map(|_instant| Message::RenderAll),
        ];

        Subscription::batch(subscriptions)
    }
}

impl Default for App {
    fn default() -> Self {
        Self {
            shared_state: SharedAppState::default(),
            keybinds: bind_keybinds(),
            windows: BTreeMap::new(),
            main_window: Main::default(),
            file_import_window: FileImport::default(),
        }
    }
}

fn main() -> iced::Result {
    iced::daemon(App::title, App::update, App::view)
        .subscription(App::subscription)
        .run_with(App::new)
}
