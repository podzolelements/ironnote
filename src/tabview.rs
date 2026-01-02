use iced::{
    Element, Length,
    advanced::widget::Text,
    widget::{button, column, row},
};

/// component that makes up a single tab on a tabview
pub struct TabviewItem<'a, Message> {
    pub(crate) title: String,
    pub(crate) clicked_message: Message,
    pub(crate) content: Element<'a, Message>,
}

/// constructs the tabview in a vertical manor based on the current selected tab from the elements structure:
/// ```txt
///  _____________________    _
/// |Tab0|Tab1|Tab2|      |   ^
/// |---------------------|   |
/// | Tab Content         | height
/// |                     |   |
/// |                     |   v
/// |_____________________|   _
///
/// |<-------width------->|
/// ```
pub fn tabview_content_vertical<'a, Message: Clone + 'a>(
    mut tab_elements: Vec<TabviewItem<'a, Message>>,
    current_tab_index: usize,
    width: Length,
    height: Length,
) -> Element<'a, Message> {
    let mut horizontal_names = row![];

    for tab in &tab_elements {
        let tab_button =
            button(Text::new(tab.title.clone()).size(13)).on_press(tab.clicked_message.clone());

        horizontal_names = horizontal_names.push(tab_button);
    }

    // since the vec owns the element, we need to remove() it in order to take ownership of it to use it. it doesn't
    // matter that the vec is being altered, since the function takes ownership of it, and we no longer have a need for
    // it since information has already been extracted
    let active_tab_content = tab_elements.remove(current_tab_index).content;

    column![horizontal_names, active_tab_content]
        .width(width)
        .height(height)
        .into()
}

/// constructs the tabview in a horizontal manor based on the current selected tab from the elements structure:
/// ```txt
///  ______________________    _
/// |Tab0 | Tab Content    |   ^
/// |Tab1 |                |   |
/// |Tab2 |                | height
/// |     |                |   |
/// |     |                |   v
/// |_____|________________|   _
///
/// |<--------width------->|
/// ```
pub fn tabview_content_horizontal<'a, Message: Clone + 'a>(
    mut tab_elements: Vec<TabviewItem<'a, Message>>,
    current_tab_index: usize,
    width: Length,
    height: Length,
) -> Element<'a, Message> {
    let mut vertical_titles = column![];

    for tab in &tab_elements {
        let tab_button =
            button(Text::new(tab.title.clone()).size(13)).on_press(tab.clicked_message.clone());

        vertical_titles = vertical_titles.push(tab_button);
    }

    // since the vec owns the element, we need to remove() it in order to take ownership of it to use it. it doesn't
    // matter that the vec is being altered, since the function takes ownership of it, and we no longer have a need for
    // it since information has already been extracted
    let active_tab_content = tab_elements.remove(current_tab_index).content;

    row![vertical_titles, active_tab_content]
        .width(width)
        .height(height)
        .into()
}
