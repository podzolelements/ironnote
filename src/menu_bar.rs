use iced::Alignment::Center;
use iced::Element;
use iced::Length::Fill;
use iced::widget::{self, Space, column, mouse_area, opaque, row, stack};

#[derive(Debug)]
pub enum MenuItemType<Message> {
    Button(Message),
}

#[derive(Debug)]
pub struct MenuItem<Message> {
    item_type: MenuItemType<Message>,
    name: String,
}

impl<Message> MenuItem<Message> {
    pub fn new(name: &str, item_type: MenuItemType<Message>) -> Self {
        Self {
            item_type,
            name: name.to_string(),
        }
    }
}

#[derive(Debug, Default)]
pub struct Dropdown<Message> {
    items: Vec<MenuItem<Message>>,
    name: String,
    name_width: u16,
    on_click_dropdown: Message,
}

impl<Message> Dropdown<Message> {
    pub fn new(name: &str, width: u16, on_click_dropdown: Message) -> Self {
        Self {
            items: Vec::default(),
            name: name.to_string(),
            name_width: width,
            on_click_dropdown,
        }
    }

    /// adds a menu item to the end of the dropdown
    pub fn push_menu_item(&mut self, item: MenuItem<Message>) {
        self.items.push(item);
    }

    /// creates the composite element from the dropdown structure
    fn build_dropdown<'a>(&self) -> Element<'a, Message>
    where
        Message: Clone + 'a,
    {
        let mut dropdown = column![];

        for menu_item in &self.items {
            match &menu_item.item_type {
                MenuItemType::Button(message) => {
                    dropdown = dropdown.push(
                        widget::button(widget::text(menu_item.name.clone()).size(13))
                            .width(125)
                            .on_press(message.clone()),
                    )
                }
            }
        }

        dropdown.into()
    }
}

#[derive(Debug)]
pub struct MenuBar<Message> {
    dropdowns: Vec<Dropdown<Message>>,
    dropdown_visible: Option<usize>,
    on_click_away: Message,
}

impl<Message> MenuBar<Message> {
    pub fn new(on_click_away: Message) -> Self {
        Self {
            dropdowns: Vec::default(),
            dropdown_visible: None,
            on_click_away,
        }
    }

    /// adds the new_dropdown to the end of the menu bar
    pub fn push_dropdown(&mut self, new_dropdown: Dropdown<Message>) {
        self.dropdowns.push(new_dropdown);
    }

    /// sets the dropdown specified to be visible if Some, and hides the dropdown when None
    pub fn set_active_dropdown(&mut self, dropdown_index: Option<usize>) {
        self.dropdown_visible = dropdown_index;
    }

    /// creates the composite menu bar element
    fn build_bar<'a>(&self) -> Element<'a, Message>
    where
        Message: Clone + 'a,
    {
        let mut bar = row![];

        for dropdown in &self.dropdowns {
            bar = bar.push(
                widget::button(widget::text(dropdown.name.clone()).size(13).align_x(Center))
                    .on_press(dropdown.on_click_dropdown.clone())
                    .width(dropdown.name_width)
                    .height(25),
            );
        }

        bar.into()
    }

    /// builds the current dropdown based on the currently visible dropdown. a dropdown must be visible to call this
    fn build_dropdown<'a>(&self) -> Element<'a, Message>
    where
        Message: Clone + 'a,
    {
        let dropdown_index = self.dropdown_visible.expect("bad dropdown index");

        let dropdown = &self.dropdowns[dropdown_index];

        dropdown.build_dropdown()
    }
}

/// creates a menu bar vertically on top of the underlay, based on the provided menu_structure
pub fn menu_bar<'a, Message>(
    underlay: Element<'a, Message>,
    menu_structure: &MenuBar<Message>,
) -> Element<'a, Message>
where
    Message: Clone + 'a,
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

    let dropdown_x_alignment: u16 = menu_structure
        .dropdowns
        .iter()
        .take(dropdown_index)
        .map(|dropdown| dropdown.name_width)
        .sum();

    let top_space = Space::new(Fill, 25);
    let left_space = Space::new(dropdown_x_alignment, Fill);

    let padded_dropdown_horizontal = row![left_space, dropdown];
    let padded_dropdown = column![top_space, padded_dropdown_horizontal];

    let full_dropdown = opaque(
        mouse_area(padded_dropdown)
            .on_press(menu_structure.on_click_away.clone())
            .on_right_press(menu_structure.on_click_away.clone())
            .on_middle_press(menu_structure.on_click_away.clone()),
    );

    stack!(window, full_dropdown).into()
}
