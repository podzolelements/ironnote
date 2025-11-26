use crate::{
    keyboard_manager::{KeyboardAction, bind_keybinds},
    main_window::{Main, MainMessage},
    window_manager::{WindowType, Windowable},
};
use iced::window;
use iced::{Element, Event, Subscription, Task, event::listen_with, keyboard, widget::column};
use keybinds::Keybinds;
use std::collections::BTreeMap;

mod calender;
mod clipboard;
mod config;
mod content_tools;
mod context_menu;
mod day_store;
mod dictionary;
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

struct App {
    keybinds: Keybinds<KeyboardAction>,
    windows: BTreeMap<window::Id, WindowType>,
    main_window: Main,
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
        Self {
            keybinds: bind_keybinds(),
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
