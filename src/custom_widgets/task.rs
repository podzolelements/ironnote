use iced::{
    Element,
    widget::{self, Text, button, column, row, text::Wrapping},
};

use super::calender::TOTAL_CALENDER_WIDTH;

/// Constructs a Task widget. Paremeters:
///
/// checkbox: A completion checkbox is not rendered if None. If Some, the boolean is the current state of the checkbox,
/// and M is the checkbox toggle message
///
/// name: The task name
///
/// expanded: Outer Option: Can the task render an expansion? Inner Option: Some if the expansion is to be rendered.
/// The M is the expansion toggle message
///
/// options_menu_toggle: The message to toggle the task options menu
///
/// options_menu_items: Some if the options menu is to be rendered. The String is the name of the option, and the
/// correponding M is the message that option triggers
pub fn build_task<'a, M: 'a + Clone>(
    checkbox: Option<(bool, M)>,
    name: String,
    expanded: Option<(Option<Element<'a, M>>, M)>,
    options_menu_toggle: M,
    options_menu_items: Option<Vec<(String, M)>>,
) -> Element<'a, M> {
    let main_checkbox = if let Some((checked, check_message)) = checkbox {
        column![widget::checkbox(checked).on_toggle(move |_ticked| { check_message.clone() })]
    } else {
        column![]
    };

    let name_text = Text::new(name)
        .width(TOTAL_CALENDER_WIDTH - 90)
        .wrapping(Wrapping::WordOrGlyph)
        .size(14);

    let (expanded_button, expanded_ui) = if let Some((subel, click_message)) = expanded {
        let is_expanded = subel.is_some();
        let expand_button_text = if is_expanded { "\\/" } else { "<" };

        let expand_button = button(Text::new(expand_button_text)).on_press(click_message);

        (column![expand_button], column![subel])
    } else {
        (column![], column![])
    };

    let options_button = button(Text::new("...")).on_press(options_menu_toggle);

    let menu = if let Some(menu_items) = options_menu_items {
        let mut menus = column![];

        for (menu_name, menu_message) in menu_items {
            let menu_item_button = button(Text::new(menu_name)).on_press(menu_message);

            menus = menus.push(menu_item_button);
        }

        menus
    } else {
        column![]
    };

    let task_ui = row![main_checkbox, name_text, expanded_button, options_button];

    let full_ui = column![task_ui, expanded_ui, menu];

    full_ui.into()
}
