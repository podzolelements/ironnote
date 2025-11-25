use crate::{
    main_window::{Main, MainMessage},
    window_manager::{WindowType, Windowable},
};
use iced::{
    Element, Event, Subscription, Task,
    event::listen_with,
    keyboard::{self},
    widget::column,
};
use iced_core::window;
use keybinds::Keybinds;
use std::collections::BTreeMap;

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
mod main_window;
mod menu_bar;
mod menu_bar_builder;
mod misc_tools;
mod month_store;
mod search_table;
mod window_manager;
mod word_count;

struct App {
    keybinds: Keybinds<KeyboardAction>,
    // clipboard: ClipboardContext,
    windows: BTreeMap<window::Id, WindowType>,
    main_window: Main,
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

#[derive(Debug, Clone)]
pub enum Message {
    CapturedKeyEvent((keyboard::Event, window::Id)),
    KeyEvent((KeyboardAction, window::Id)),
    WindowEvent((window::Event, window::Id)),
    WindowOpened(window::Id, WindowType),
    WindowClosed(window::Id),
    RenderAll,

    MainWindow(MainMessage),
}

impl App {
    fn new() -> (Self, Task<Message>) {
        let window_type = WindowType::Main;

        let (_new_id, task) = iced::window::open(window_type.settings());

        (
            Self::default(),
            task.map(move |id| Message::WindowOpened(id, window_type.clone())),
        )
    }

    fn title(&self, id: window::Id) -> String {
        if let Some(window_type) = self.windows.get(&id) {
            match window_type {
                WindowType::Main => self.main_window.title(),
            }
        } else {
            "orphaned window".to_string()
        }
    }

    pub fn view(&'_ self, id: window::Id) -> Element<'_, Message> {
        if let Some(window_type) = self.windows.get(&id) {
            match window_type {
                WindowType::Main => self.main_window.view().map(Message::MainWindow),
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
                    }
                }

                Task::none()
            }
            Message::MainWindow(main_message) => self
                .main_window
                .update(main_message)
                .map(Message::MainWindow),
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

        // let clipboard = ClipboardContext::new().expect("couldn't get clipboard");

        Self {
            keybinds,
            // clipboard,
            windows: BTreeMap::new(),
            main_window: Main::default(),
        }
    }
}

fn main() -> iced::Result {
    iced::daemon(App::title, App::update, App::view)
        .subscription(App::subscription)
        .run_with(App::new)
}
