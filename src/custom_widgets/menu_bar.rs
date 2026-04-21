use iced::Alignment::Center;
use iced::Element;
use iced::widget::{self, column, mouse_area, row, stack};

use crate::custom_widgets::context_menu::{ContextMenuItem, build_context_menu};
use crate::ui::layout::{CONTEXT_MENU_HEIGHT, CONTEXT_MENU_PADDING};
use crate::ui::styling::CONTEXT_MENU_SIZE;
use crate::utils::text_tools::string_width;

#[derive(Debug, Default, Clone)]
/// A Dropdown is a context menu that has a name
pub struct Dropdown<M> {
    items: Vec<ContextMenuItem<M>>,
    name: String,
    on_click_dropdown: M,
}

impl<M> Dropdown<M> {
    /// Creates a Dropdown with the given name and click message
    pub fn new(name: &str, on_click_dropdown: M) -> Self {
        Self {
            items: Vec::default(),
            name: name.to_string(),
            on_click_dropdown,
        }
    }

    /// Adds a menu item to the end of the dropdown
    pub fn push_menu_item(&mut self, item: ContextMenuItem<M>) {
        self.items.push(item);
    }

    /// Creates the composite element from the dropdown structure
    fn build_dropdown<'a>(&'a self) -> Element<'a, M>
    where
        M: Clone + 'a,
    {
        build_context_menu(self.items.clone())
    }
}

#[derive(Debug)]
/// The MenuBar is a collection of Dropdowns that make up the top context menus
pub struct MenuBar<M> {
    dropdowns: Vec<Dropdown<M>>,
    dropdown_visible: Option<usize>,
    on_click_away: M,
}

impl<M> MenuBar<M> {
    pub fn new(on_click_away: M) -> Self {
        Self {
            dropdowns: Vec::default(),
            dropdown_visible: None,
            on_click_away,
        }
    }

    /// Adds the new_dropdown to the end of the menu bar
    pub fn push_dropdown(&mut self, new_dropdown: Dropdown<M>) {
        self.dropdowns.push(new_dropdown);
    }

    /// sets the dropdown specified to be visible if Some, and hides the dropdown when None
    pub fn set_active_dropdown(&mut self, dropdown_index: Option<usize>) {
        self.dropdown_visible = dropdown_index;
    }

    /// returns true if there currently is a dropdown on screen, false otherwise
    pub fn is_dropdown_visible(&self) -> bool {
        self.dropdown_visible.is_some()
    }

    /// Creates the composite menu bar element
    fn build_bar<'a>(&self) -> Element<'a, M>
    where
        M: Clone + 'a,
    {
        let mut bar = row![];

        for dropdown in &self.dropdowns {
            let dropdown_name_width =
                string_width(&dropdown.name, CONTEXT_MENU_SIZE) + CONTEXT_MENU_PADDING;

            bar = bar.push(
                widget::button(widget::text(dropdown.name.clone()).size(13).align_x(Center))
                    .on_press(dropdown.on_click_dropdown.clone())
                    .width(dropdown_name_width)
                    .height(CONTEXT_MENU_HEIGHT),
            );
        }

        bar.into()
    }

    /// Builds the current dropdown based on the currently visible dropdown. a dropdown must be visible to call this
    fn build_dropdown<'a>(&'a self) -> Element<'a, M>
    where
        M: Clone + 'a,
    {
        let dropdown_index = self.dropdown_visible.expect("bad dropdown index");

        let dropdown = &self.dropdowns[dropdown_index];

        dropdown.build_dropdown()
    }
}

/// Creates a menu bar vertically on top of the underlay, based on the provided menu_structure
pub fn menu_bar<'a, M>(underlay: Element<'a, M>, menu_structure: &'a MenuBar<M>) -> Element<'a, M>
where
    M: Clone + 'a,
{
    let bar = menu_structure.build_bar();

    let window = column![bar, underlay].into();

    if menu_structure.dropdown_visible.is_none() {
        return window;
    }
    let dropdown_index = menu_structure
        .dropdown_visible
        .expect("dropdown index is None");

    let dropdown = menu_structure.build_dropdown();

    let dropdown_x_alignment = menu_structure
        .dropdowns
        .iter()
        .take(dropdown_index)
        .map(|dropdown| string_width(&dropdown.name, CONTEXT_MENU_SIZE) + CONTEXT_MENU_PADDING)
        .sum::<f32>();

    let pinned_dropdown = widget::pin(dropdown)
        .x(dropdown_x_alignment)
        .y(CONTEXT_MENU_HEIGHT);

    let full_dropdown = mouse_area(pinned_dropdown)
        .on_press(menu_structure.on_click_away.clone())
        .on_right_press(menu_structure.on_click_away.clone())
        .on_middle_press(menu_structure.on_click_away.clone());

    stack!(window, full_dropdown).into()
}
