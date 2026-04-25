use strum::EnumIter;

use super::menu_bar::{Dropdown, MenuBar};
use crate::custom_widgets::context_menu::{ContextMenuElement, ContextMenuItem};

#[derive(Debug, Clone)]
/// File menu actions
pub enum FileMessage {
    Save,
    Import,
    Export,
}

impl FileMessage {
    /// The text of the file menu item
    pub fn name(&self) -> &'static str {
        match self {
            FileMessage::Save => "Save",
            FileMessage::Import => "Import",
            FileMessage::Export => "Export",
        }
    }
}

#[derive(Debug, Clone)]
/// Edit menu actions
pub enum EditMessage {
    Undo,
    Redo,
    Cut,
    Copy,
    Paste,
}

impl EditMessage {
    /// The text of the edit menu item
    pub fn name(&self) -> &'static str {
        match self {
            EditMessage::Undo => "Undo",
            EditMessage::Redo => "Redo",
            EditMessage::Cut => "Cut",
            EditMessage::Copy => "Copy",
            EditMessage::Paste => "Paste",
        }
    }
}

#[derive(Debug, Clone)]
/// Tools menu actions
pub enum ToolsMessage {
    Preferences,
}

impl ToolsMessage {
    /// The text of the tools menu item
    pub fn name(&self) -> &'static str {
        match self {
            ToolsMessage::Preferences => "Preferences",
        }
    }
}

#[derive(Debug, Clone)]
pub enum MenuMessage {
    ClickedAway,
    ClickedDropdown(usize),
    File(FileMessage),
    Edit(EditMessage),
    Tools(ToolsMessage),
}

#[derive(Debug, Clone, EnumIter)]
/// The types of top bar menus
pub enum DropdownType {
    File,
    Edit,
    Tools,
}

impl DropdownType {
    /// Returns the index of the menu going from left to right starting at 0
    pub fn dropdown_index(&self) -> usize {
        match self {
            DropdownType::File => 0,
            DropdownType::Edit => 1,
            DropdownType::Tools => 2,
        }
    }

    /// The text of the dropdown menus
    pub fn dropdown_name(&self) -> &'static str {
        match self {
            DropdownType::File => "File",
            DropdownType::Edit => "Edit",
            DropdownType::Tools => "Tools",
        }
    }
}

/// Constructs the top menu bar used by the application
pub fn build_menu_bar() -> MenuBar<MenuMessage> {
    let mut menu_bar = MenuBar::new(MenuMessage::ClickedAway);

    let mut file_dropdown = Dropdown::new(
        DropdownType::File.dropdown_name(),
        MenuMessage::ClickedDropdown(DropdownType::File.dropdown_index()),
    );

    file_dropdown.push_menu_item(ContextMenuItem::Button(ContextMenuElement::new(
        FileMessage::Save.name(),
        Some(MenuMessage::File(FileMessage::Save)),
    )));
    file_dropdown.push_menu_item(ContextMenuItem::Break);
    file_dropdown.push_menu_item(ContextMenuItem::Button(ContextMenuElement::new(
        FileMessage::Import.name(),
        Some(MenuMessage::File(FileMessage::Save)),
    )));
    file_dropdown.push_menu_item(ContextMenuItem::Button(ContextMenuElement::new(
        FileMessage::Export.name(),
        Some(MenuMessage::File(FileMessage::Save)),
    )));

    let mut edit_dropdown = Dropdown::new(
        DropdownType::Edit.dropdown_name(),
        MenuMessage::ClickedDropdown(DropdownType::Edit.dropdown_index()),
    );

    edit_dropdown.push_menu_item(ContextMenuItem::Button(ContextMenuElement::new(
        EditMessage::Cut.name(),
        Some(MenuMessage::Edit(EditMessage::Cut)),
    )));
    edit_dropdown.push_menu_item(ContextMenuItem::Button(ContextMenuElement::new(
        EditMessage::Copy.name(),
        Some(MenuMessage::Edit(EditMessage::Copy)),
    )));
    edit_dropdown.push_menu_item(ContextMenuItem::Button(ContextMenuElement::new(
        EditMessage::Paste.name(),
        Some(MenuMessage::Edit(EditMessage::Paste)),
    )));
    edit_dropdown.push_menu_item(ContextMenuItem::Break);
    edit_dropdown.push_menu_item(ContextMenuItem::Button(ContextMenuElement::new(
        EditMessage::Undo.name(),
        Some(MenuMessage::Edit(EditMessage::Undo)),
    )));
    edit_dropdown.push_menu_item(ContextMenuItem::Button(ContextMenuElement::new(
        EditMessage::Redo.name(),
        Some(MenuMessage::Edit(EditMessage::Redo)),
    )));

    let mut tools_dropdown = Dropdown::new(
        DropdownType::Tools.dropdown_name(),
        MenuMessage::ClickedDropdown(DropdownType::Tools.dropdown_index()),
    );

    tools_dropdown.push_menu_item(ContextMenuItem::Button(ContextMenuElement::new(
        ToolsMessage::Preferences.name(),
        Some(MenuMessage::Tools(ToolsMessage::Preferences)),
    )));

    menu_bar.push_dropdown(file_dropdown);
    menu_bar.push_dropdown(edit_dropdown);
    menu_bar.push_dropdown(tools_dropdown);

    menu_bar
}
