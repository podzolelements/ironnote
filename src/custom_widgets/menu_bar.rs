use iced::Alignment::Center;
use iced::widget::{self, button, column, mouse_area, row, stack};
use iced::{Element, Length};

use crate::custom_widgets::context_menu::{ContextMenuItem, build_context_menu};
use crate::ui::button_themes;
use crate::ui::layout::{CONTEXT_MENU_BAR_PADDING, CONTEXT_MENU_HEIGHT};
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

    /// The width of the dropdown text, plus the padding on both sides
    pub fn width(&self) -> f32 {
        string_width(&self.name, CONTEXT_MENU_SIZE) + 2.0 * CONTEXT_MENU_BAR_PADDING
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

    /// Total amount of horizontal space the menu takes up
    pub fn total_bar_width(&self) -> f32 {
        self.dropdowns.iter().map(|dropdown| dropdown.width()).sum()
    }

    /// Determines the menu index from the horizontal position along the menu bar, if the position is within the menu
    /// bar's bounds
    pub fn menu_from_position(&self, horizontal_position: f32) -> Option<usize> {
        let mut accumulator = 0.0;

        for (dropdown_index, dropdown) in self.dropdowns.iter().enumerate() {
            accumulator += dropdown.width();

            if horizontal_position < accumulator {
                return Some(dropdown_index);
            }
        }

        None
    }

    /// Creates the composite menu bar element
    fn build_bar<'a>(&self) -> Element<'a, M>
    where
        M: Clone + 'a,
    {
        let mut bar = row![];

        for dropdown in &self.dropdowns {
            bar = bar.push(
                widget::button(
                    widget::text(dropdown.name.clone())
                        .size(CONTEXT_MENU_SIZE)
                        .align_x(Center)
                        .align_y(Center),
                )
                .on_press(dropdown.on_click_dropdown.clone())
                .width(dropdown.width())
                .height(CONTEXT_MENU_HEIGHT)
                .style(button_themes::context_menu_bar_style),
            );
        }

        let bar_filler = button("")
            .on_press_maybe(None)
            .width(Length::Fill)
            .height(CONTEXT_MENU_HEIGHT)
            .style(button_themes::context_menu_bar_style);
        bar = bar.push(bar_filler);

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

    pub fn map<N, F>(self, mut f: F) -> MenuBar<N>
    where
        F: Clone + FnMut(M) -> N,
    {
        let dropdowns = self
            .dropdowns
            .into_iter()
            .map(|d| Dropdown {
                items: d.items.into_iter().map(|e| e.map(f.clone())).collect(),
                name: d.name,
                on_click_dropdown: f(d.on_click_dropdown),
            })
            .collect();

        MenuBar {
            dropdowns,
            dropdown_visible: self.dropdown_visible,
            on_click_away: f(self.on_click_away),
        }
    }
}

/// Creates a menu bar vertically on top of the underlay, based on the provided menu_structure
pub fn build_full_menu_bar<'a, M>(
    underlay: Element<'a, M>,
    menu_structure: &'a MenuBar<M>,
) -> Element<'a, M>
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
        .map(|dropdown| dropdown.width())
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
