use crate::{
    SharedAppState,
    content::ContentAction,
    ui::layout::{MEDIUM_WINDOW_SIZE, SMALL_WINDOW_SIZE, WINDOW_SIZE},
};

use iced::{Element, Task, window};

#[derive(Debug, Clone, PartialEq)]
/// all of the types windows that can be created
pub enum WindowType {
    Main,
    FileImport,
    FileExport,
    TaskCreator,
    Preferences,
}

impl WindowType {
    /// window settings based on the type of window
    pub fn settings(&self) -> window::Settings {
        match self {
            WindowType::Main => window::Settings {
                size: WINDOW_SIZE,
                ..Default::default()
            },
            WindowType::FileImport => window::Settings {
                size: SMALL_WINDOW_SIZE,
                resizable: false,
                position: window::Position::Centered,
                ..Default::default()
            },
            WindowType::FileExport => window::Settings {
                size: SMALL_WINDOW_SIZE,
                resizable: false,
                position: window::Position::Centered,
                ..Default::default()
            },
            WindowType::TaskCreator => window::Settings {
                size: MEDIUM_WINDOW_SIZE,
                resizable: false,
                position: window::Position::Centered,
                ..Default::default()
            },
            WindowType::Preferences => window::Settings {
                size: MEDIUM_WINDOW_SIZE,
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

    /// performs the provided action on the window's UpgradedContents. internal tracking should be used to ensure the
    /// proper content has the action applied
    fn content_perform(&mut self, state: &mut SharedAppState, action: ContentAction);
}
