use super::menu_bar::{Dropdown, MenuBar};
use crate::{
    custom_widgets::context_menu::{ContextMenuElement, ContextMenuItem},
    ui::{layout::CONTEXT_MENU_TEXT_PADDING, styling::CONTEXT_MENU_SIZE},
    utils::text_tools::string_width,
    windows::main_window::MainMessage,
};

use strum::{Display, EnumIter, IntoEnumIterator};

#[derive(Debug, Clone, EnumIter, Display)]
pub enum FileMessage {
    Save,
    Import,
    Export,
}

#[derive(Debug, Clone, EnumIter, Display)]
pub enum EditMessage {
    Undo,
    Redo,
    Cut,
    Copy,
    Paste,
}

#[derive(Debug, Clone, EnumIter, Display)]
pub enum ToolsMessage {
    Preferences,
}

#[derive(Debug, Clone)]
pub enum MenuMessage {
    ClickedAway,
    ClickedMenu(Menus),
    File(FileMessage),
    Edit(EditMessage),
    Tools(ToolsMessage),
}

#[derive(Debug, Clone, EnumIter, Display)]
/// the types of top bar menus
pub enum Menus {
    File,
    Edit,
    Tools,
}

impl Menus {
    /// returns the index of the menu going from left to right starting at 0
    pub fn menu_index(&self) -> usize {
        match self {
            Menus::File => 0,
            Menus::Edit => 1,
            Menus::Tools => 2,
        }
    }

    /// Returns how much horizontal space the menu item takes up
    pub fn width(&self) -> f32 {
        string_width(&self.to_string(), CONTEXT_MENU_SIZE) + CONTEXT_MENU_TEXT_PADDING
    }

    /// Returns the total horizontal space the menu bar takes up
    pub fn total_bar_width() -> f32 {
        Menus::iter().map(|menu| menu.width()).sum()
    }

    /// Returns the menu bar present at the given position, and None if the position is outside of the range the menu
    /// bar takes up
    pub fn menu_from_position(horizontal_position: f32) -> Option<Menus> {
        let mut accumulator = 0.0;

        for menu in Menus::iter() {
            accumulator += menu.width();

            if horizontal_position < accumulator {
                return Some(menu);
            }
        }

        None
    }
}

/// Constructs the top menu bar used by the application
pub fn build_menu_bar() -> MenuBar<crate::MainMessage> {
    let mut menu_bar = MenuBar::new(MainMessage::MenuBar(MenuMessage::ClickedAway));

    let mut file_dropdown = Dropdown::new(
        &Menus::File.to_string(),
        MainMessage::MenuBar(MenuMessage::ClickedMenu(Menus::File)),
    );

    for file_message in FileMessage::iter() {
        file_dropdown.push_menu_item(ContextMenuItem::Button(ContextMenuElement {
            name: file_message.to_string(),
            message: Some(MainMessage::MenuBar(MenuMessage::File(file_message))),
        }));
    }

    let mut edit_dropdown = Dropdown::new(
        &Menus::Edit.to_string(),
        MainMessage::MenuBar(MenuMessage::ClickedMenu(Menus::Edit)),
    );

    for edit_message in EditMessage::iter() {
        edit_dropdown.push_menu_item(ContextMenuItem::Button(ContextMenuElement {
            name: edit_message.to_string(),
            message: Some(MainMessage::MenuBar(MenuMessage::Edit(edit_message))),
        }));
    }

    let mut tools_dropdown = Dropdown::new(
        &Menus::Tools.to_string(),
        MainMessage::MenuBar(MenuMessage::ClickedMenu(Menus::Tools)),
    );

    for tools_message in ToolsMessage::iter() {
        tools_dropdown.push_menu_item(ContextMenuItem::Button(ContextMenuElement {
            name: tools_message.to_string(),
            message: Some(MainMessage::MenuBar(MenuMessage::Tools(tools_message))),
        }));
    }

    menu_bar.push_dropdown(file_dropdown);
    menu_bar.push_dropdown(edit_dropdown);
    menu_bar.push_dropdown(tools_dropdown);

    menu_bar
}
