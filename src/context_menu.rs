use iced::widget::{Space, column, mouse_area, opaque, row, stack};
use iced::{Element, Fill, Point};

/// draws the menu element on top of the underlay element using a stack widget at the given position, if the
/// show_context_menu flag is set. takes a Message to send when the context menu is clicked off of.
pub fn context_menu<'a, Message>(
    underlay: impl Into<Element<'a, Message>>,
    menu: impl Into<Element<'a, Message>>,
    show_context_menu: bool,
    position: Point,
    on_click_away: Message,
) -> impl Into<Element<'a, Message>>
where
    Message: Clone + 'a,
{
    if !show_context_menu {
        return underlay.into();
    }

    // to get the menu to show up at a specific position, fixed with spacing is used to pad it out
    let space_top = Space::new().width(Fill).height(position.y);
    let space_left = Space::new().width(position.x).height(Fill);

    let padded_menu_horizontal = row![space_left, menu.into()];
    let padded_menu = column![space_top, padded_menu_horizontal];

    let menu_content = opaque(
        mouse_area(padded_menu)
            .on_press(on_click_away.clone())
            .on_right_press(on_click_away.clone())
            .on_middle_press(on_click_away.clone()),
    );

    stack![underlay.into(), menu_content].into()
}
