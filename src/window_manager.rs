use iced::{Element, Size, Task, window};

use crate::SharedAppState;

#[derive(Debug, Clone, PartialEq)]
/// all of the types windows that can be created
pub enum WindowType {
    Main,
    FileImport,
}

impl WindowType {
    /// window settings based on the type of window
    pub fn settings(&self) -> window::Settings {
        match self {
            WindowType::Main => window::Settings {
                size: Size::new(1024.0, 768.0),
                ..Default::default()
            },
            WindowType::FileImport => window::Settings {
                size: Size::new(1024.0 / 2.0, 768.0 / 2.0),
                resizable: false,
                position: window::Position::Centered,
                ..Default::default()
            },
        }
    }
}

/// trait that outlines the required functionality to create a new window instance
pub trait Windowable<Message> {
    /// the title displayed at the top of the window
    fn title(&self) -> String;

    /// the contents on the window
    fn view<'a>(&'a self, state: &'a SharedAppState) -> Element<'a, Message>;

    /// update the internal state based on the message received
    fn update(&mut self, state: &mut SharedAppState, message: Message) -> Task<Message>;
}
