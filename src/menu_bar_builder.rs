use crate::{
    main_window::MainMessage,
    menu_bar::{Dropdown, MenuBar, MenuItem, MenuItemType},
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

    /// returns how much horizontal space the menu requires to properly render
    pub fn width(&self) -> u32 {
        match self {
            Menus::File => 45,
            Menus::Edit => 45,
            Menus::Tools => 50,
        }
    }

    /// returns the total horizontal space the menu bar takes up
    pub fn total_bar_width() -> u32 {
        Menus::iter().map(|menu| menu.width()).sum()
    }

    /// returns the menu bar present at the given position, and None if the position is outside of the range the menu
    /// bar takes up
    pub fn menu_from_position(position: u32) -> Option<Menus> {
        let mut accumulator = 0;

        for menu in Menus::iter() {
            accumulator += menu.width();

            if position < accumulator {
                return Some(menu);
            }
        }

        None
    }
}

/// vertical space the menu bar takes up
pub const MENU_BAR_HEIGHT: u32 = 25;

/// constructs the top menu bar used by the application
pub fn build_menu_bar() -> MenuBar<crate::MainMessage> {
    let mut menu_bar = MenuBar::new(
        MENU_BAR_HEIGHT,
        MainMessage::MenuBar(MenuMessage::ClickedAway),
    );

    let mut file_dropdown = Dropdown::new(
        &Menus::File.to_string(),
        Menus::File.width(),
        MainMessage::MenuBar(MenuMessage::ClickedMenu(Menus::File)),
    );

    for file_message in FileMessage::iter() {
        file_dropdown.push_menu_item(MenuItem::new(
            &file_message.to_string(),
            MenuItemType::Button(MainMessage::MenuBar(MenuMessage::File(file_message))),
        ));
    }

    let mut edit_dropdown = Dropdown::new(
        &Menus::Edit.to_string(),
        Menus::Edit.width(),
        MainMessage::MenuBar(MenuMessage::ClickedMenu(Menus::Edit)),
    );

    for edit_message in EditMessage::iter() {
        edit_dropdown.push_menu_item(MenuItem::new(
            &edit_message.to_string(),
            MenuItemType::Button(MainMessage::MenuBar(MenuMessage::Edit(edit_message))),
        ));
    }

    let mut tools_dropdown = Dropdown::new(
        &Menus::Tools.to_string(),
        Menus::Tools.width(),
        MainMessage::MenuBar(MenuMessage::ClickedMenu(Menus::Tools)),
    );

    for tools_message in ToolsMessage::iter() {
        tools_dropdown.push_menu_item(MenuItem::new(
            &tools_message.to_string(),
            MenuItemType::Button(MainMessage::MenuBar(MenuMessage::Tools(tools_message))),
        ));
    }

    menu_bar.push_dropdown(file_dropdown);
    menu_bar.push_dropdown(edit_dropdown);
    menu_bar.push_dropdown(tools_dropdown);

    menu_bar
}
