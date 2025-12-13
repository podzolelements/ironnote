use iced::{Element, Size, Task, window};

use crate::SharedAppState;

#[derive(Debug, Clone, PartialEq)]
/// all of the types windows that can be created
pub enum WindowType {
    Main,
    FileImport,
    TaskCreator,
}

impl WindowType {
    const WINDOW_WIDTH: f32 = 1024.0;
    const WINDOW_HEIGHT: f32 = 768.0;
    const SMALL_WINDOW_SIZE: Size<f32> =
        Size::new(Self::WINDOW_WIDTH / 2.0, Self::WINDOW_HEIGHT / 2.0);
    const MEDIUM_WINDOW_SIZE: Size<f32> =
        Size::new(Self::WINDOW_WIDTH / 1.5, Self::WINDOW_HEIGHT / 1.5);
    const WINDOW_SIZE: Size<f32> = Size::new(Self::WINDOW_WIDTH, Self::WINDOW_HEIGHT);

    /// window settings based on the type of window
    pub fn settings(&self) -> window::Settings {
        match self {
            WindowType::Main => window::Settings {
                size: Self::WINDOW_SIZE,
                ..Default::default()
            },
            WindowType::FileImport => window::Settings {
                size: Self::SMALL_WINDOW_SIZE,
                resizable: false,
                position: window::Position::Centered,
                ..Default::default()
            },
            WindowType::TaskCreator => window::Settings {
                size: Self::MEDIUM_WINDOW_SIZE,
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
