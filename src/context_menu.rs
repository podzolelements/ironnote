use iced::widget::{self, mouse_area, opaque, stack};
use iced::{Element, Point};

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

    let pinned_menu = widget::pin(menu).position(position);

    let menu_content = opaque(
        mouse_area(pinned_menu)
            .on_press(on_click_away.clone())
            .on_right_press(on_click_away.clone())
            .on_middle_press(on_click_away.clone()),
    );

    stack![underlay.into(), menu_content].into()
}
