use iced::Element;
use iced::alignment::Vertical;
use iced::widget::scrollable::{Direction, Scrollbar};
use iced::widget::{self, Space, Text, column, scrollable};

use crate::ui::layout::{
    CONTEXT_MENU_BREAK_HEIGHT, CONTEXT_MENU_HEIGHT, CONTEXT_MENU_MIN_WIDTH, CONTEXT_MENU_PADDING,
    SCROLLBAR_WIDTH,
};
use crate::ui::styling::CONTEXT_MENU_SIZE;
use crate::utils::text_tools::string_width;

#[derive(Debug, Clone)]
/// The basic data required for a context menu element
pub struct ContextMenuElement<M> {
    pub(crate) name: String,
    pub(crate) message: Option<M>,
}

#[derive(Debug, Clone)]
/// An item that appears in a context menu
pub enum ContextMenuItem<M> {
    Button(ContextMenuElement<M>),
    Text(String),
    Break,
    Scroller((Vec<ContextMenuElement<M>>, usize)),
}

impl<M> ContextMenuItem<M> {
    /// Computes the maximum width of an item in the context menu based on its contents
    fn max_item_width(items: &[ContextMenuItem<M>]) -> f32 {
        let max_width = items
            .iter()
            .map(|item| match item {
                ContextMenuItem::Button(element) => {
                    string_width(&element.name, CONTEXT_MENU_SIZE).ceil() as u32
                }
                ContextMenuItem::Text(text) => string_width(text, CONTEXT_MENU_SIZE).ceil() as u32,
                ContextMenuItem::Break => 0,
                ContextMenuItem::Scroller((elements, _scroller_count)) => elements
                    .iter()
                    .map(|element| string_width(&element.name, CONTEXT_MENU_SIZE).ceil() as u32)
                    .max()
                    .unwrap_or_default(),
            })
            .max()
            .unwrap_or_default();

        max_width as f32
    }

    /// Computes the context menu width, which is either the width of the longest item or the minimum context menu
    /// width, whichever is smaller
    pub fn menu_width(items: &[ContextMenuItem<M>]) -> f32 {
        let max_menu_width = ContextMenuItem::max_item_width(items);

        if max_menu_width < CONTEXT_MENU_MIN_WIDTH {
            CONTEXT_MENU_MIN_WIDTH
        } else {
            max_menu_width
        }
    }

    /// Menu width with the padding
    pub fn padded_menu_width(items: &[ContextMenuItem<M>]) -> f32 {
        Self::menu_width(items) + CONTEXT_MENU_PADDING
    }

    /// Computes the total height of the context menu
    pub fn menu_height(items: &[ContextMenuItem<M>]) -> f32 {
        items
            .iter()
            .map(|item| match item {
                ContextMenuItem::Button(_) => CONTEXT_MENU_HEIGHT,
                ContextMenuItem::Text(_) => CONTEXT_MENU_HEIGHT,
                ContextMenuItem::Break => CONTEXT_MENU_BREAK_HEIGHT,
                ContextMenuItem::Scroller((_elements, scroller_count)) => {
                    CONTEXT_MENU_BREAK_HEIGHT * (*scroller_count as f32)
                }
            })
            .sum()
    }
}

/// Constructs the context menu from the given collection of items
pub fn build_context_menu<'a, M: 'a + Clone>(
    menu_items: Vec<ContextMenuItem<M>>,
) -> Element<'a, M> {
    let mut menu = column![];

    let padded_menu_width = ContextMenuItem::padded_menu_width(&menu_items);

    for item in menu_items {
        match item {
            ContextMenuItem::Button(context_menu_element) => {
                let button_element = widget::button(
                    widget::text(context_menu_element.name)
                        .size(CONTEXT_MENU_SIZE)
                        .height(CONTEXT_MENU_HEIGHT)
                        .align_y(Vertical::Center),
                )
                .on_press_maybe(context_menu_element.message)
                .width(padded_menu_width)
                .height(CONTEXT_MENU_HEIGHT);

                menu = menu.push(button_element);
            }
            ContextMenuItem::Text(text) => {
                let text = Text::new(text)
                    .size(CONTEXT_MENU_SIZE)
                    .height(CONTEXT_MENU_HEIGHT)
                    .align_y(Vertical::Center);

                menu = menu.push(text);
            }
            ContextMenuItem::Break => {
                menu = menu.push(Space::new().height(CONTEXT_MENU_BREAK_HEIGHT));
            }
            ContextMenuItem::Scroller((elements, elements_before_scroll)) => {
                let mut full_scroll = column![];

                let element_count = elements.len();

                for element in elements {
                    let button_element = widget::button(
                        widget::text(element.name)
                            .size(CONTEXT_MENU_SIZE)
                            .height(CONTEXT_MENU_HEIGHT)
                            .align_y(Vertical::Center),
                    )
                    .on_press_maybe(element.message)
                    .width(padded_menu_width)
                    .height(CONTEXT_MENU_HEIGHT);

                    full_scroll = full_scroll.push(button_element);
                }

                let scroller = if element_count > elements_before_scroll {
                    column![
                        scrollable(full_scroll)
                            .width(padded_menu_width)
                            .height(CONTEXT_MENU_HEIGHT * (elements_before_scroll as f32))
                            .direction(Direction::Vertical(
                                Scrollbar::new()
                                    .spacing(0)
                                    .margin(0)
                                    .width(SCROLLBAR_WIDTH)
                                    .scroller_width(SCROLLBAR_WIDTH),
                            ))
                    ]
                } else {
                    column![full_scroll]
                };

                menu = menu.push(scroller);
            }
        }
    }

    menu.into()
}
