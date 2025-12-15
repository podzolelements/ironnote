use iced::{
    Element, Length,
    advanced::widget::Text,
    widget::{button, column, row},
};

/// component that makes up a single tab on a tabview
pub struct TabviewItem<'a, Message> {
    pub(crate) tab_name: String,
    pub(crate) tab_clicked: Message,
    pub(crate) content: Element<'a, Message>,
}

/// renders the tabview based on the current selected tab from the elements structure.
pub fn tab_view<'a, Message: Clone + 'a>(
    mut elements: Vec<TabviewItem<'a, Message>>,
    current_tab: usize,
    width: Length,
    height: Length,
) -> Element<'a, Message> {
    let mut titles = row![];

    for element in &elements {
        let tab_button = button(Text::new(element.tab_name.clone()).size(13))
            .on_press(element.tab_clicked.clone());

        titles = titles.push(tab_button);
    }

    // since the vec owns the element, we need to remove() it in order to take ownership of it
    let active_element_content = elements.remove(current_tab).content;

    column![titles, active_element_content]
        .width(width)
        .height(height)
        .into()
}
