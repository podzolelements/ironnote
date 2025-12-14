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

#[derive(Debug, Clone)]
pub enum MenuMessage {
    ClickedAway,
    ClickedMenu(usize),
    File(FileMessage),
    Edit(EditMessage),
}

/// constructs the top menu bar used by the application
pub fn build_menu_bar() -> MenuBar<crate::MainMessage> {
    let mut menu_bar = MenuBar::new(MainMessage::MenuBar(MenuMessage::ClickedAway));

    let mut file_dropdown = Dropdown::new(
        "File",
        45,
        MainMessage::MenuBar(MenuMessage::ClickedMenu(0)),
    );

    for file_message in FileMessage::iter() {
        file_dropdown.push_menu_item(MenuItem::new(
            &file_message.to_string(),
            MenuItemType::Button(MainMessage::MenuBar(MenuMessage::File(file_message))),
        ));
    }

    let mut edit_dropdown = Dropdown::new(
        "Edit",
        45,
        MainMessage::MenuBar(MenuMessage::ClickedMenu(1)),
    );

    for edit_message in EditMessage::iter() {
        edit_dropdown.push_menu_item(MenuItem::new(
            &edit_message.to_string(),
            MenuItemType::Button(MainMessage::MenuBar(MenuMessage::Edit(edit_message))),
        ));
    }

    menu_bar.push_dropdown(file_dropdown);
    menu_bar.push_dropdown(edit_dropdown);

    menu_bar
}
