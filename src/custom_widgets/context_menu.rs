use iced::alignment::Vertical;
use iced::widget::scrollable::{Direction, Scrollbar};
use iced::widget::{self, Space, Text, column, row, scrollable, stack};
use iced::{Element, Point};

use crate::custom_widgets::rectangle::build_rectangle;
use crate::ui::button_themes::context_menu_style;
use crate::ui::journal_theme::LIGHT;
use crate::ui::layout::{
    CONTEXT_MENU_BORDER_WIDTH, CONTEXT_MENU_HEIGHT, CONTEXT_MENU_MIN_WIDTH,
    CONTEXT_MENU_RADIOCHECK_WIDTH, CONTEXT_MENU_TEXT_PADDING, SCROLLBAR_WIDTH,
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
    /// width, whichever is smaller. The text width includes the text padding at the end
    fn menu_text_width(items: &[ContextMenuItem<M>]) -> f32 {
        let max_menu_width = ContextMenuItem::max_item_width(items);

        let width = if max_menu_width < CONTEXT_MENU_MIN_WIDTH {
            CONTEXT_MENU_MIN_WIDTH
        } else {
            max_menu_width
        };

        width + CONTEXT_MENU_TEXT_PADDING
    }

    /// Menu width in full, includes the spacing required for the radio/checkbox
    pub fn full_menu_width(items: &[ContextMenuItem<M>]) -> f32 {
        Self::menu_text_width(items) + CONTEXT_MENU_RADIOCHECK_WIDTH
    }

    /// Full menu width, including the border. This is the complete width of the widget
    pub fn bordered_menu_width(items: &[ContextMenuItem<M>]) -> f32 {
        Self::full_menu_width(items) + CONTEXT_MENU_BORDER_WIDTH * 2.0
    }

    /// Computes the height of the context menu
    pub fn menu_height(items: &[ContextMenuItem<M>]) -> f32 {
        items
            .iter()
            .map(|item| match item {
                ContextMenuItem::Button(_) => CONTEXT_MENU_HEIGHT,
                ContextMenuItem::Text(_) => CONTEXT_MENU_HEIGHT,
                ContextMenuItem::Break => CONTEXT_MENU_BORDER_WIDTH,
                ContextMenuItem::Scroller((elements, scroller_count)) => {
                    CONTEXT_MENU_HEIGHT * ((*scroller_count).min(elements.len()) as f32)
                }
            })
            .sum()
    }

    /// The total height of the menu, including the border. This is the complete height of the widget
    pub fn bordered_menu_height(items: &[ContextMenuItem<M>]) -> f32 {
        Self::menu_height(items) + CONTEXT_MENU_BORDER_WIDTH * 2.0
    }
}

/// Constructs the context menu from the given collection of items
pub fn build_context_menu<'a, M: 'a + Clone>(
    menu_items: Vec<ContextMenuItem<M>>,
) -> Element<'a, M> {
    let mut menu = column![];

    let full_menu_width = ContextMenuItem::full_menu_width(&menu_items);
    let bordered_menu_width = ContextMenuItem::bordered_menu_width(&menu_items);
    let bordered_menu_height = ContextMenuItem::bordered_menu_height(&menu_items);

    for item in menu_items {
        match item {
            ContextMenuItem::Button(context_menu_element) => {
                let button_content = row![
                    Space::new().width(CONTEXT_MENU_RADIOCHECK_WIDTH),
                    widget::text(context_menu_element.name)
                        .size(CONTEXT_MENU_SIZE)
                        .height(CONTEXT_MENU_HEIGHT)
                        .align_y(Vertical::Center)
                ];

                let button_element = widget::button(button_content)
                    .on_press_maybe(context_menu_element.message)
                    .width(full_menu_width)
                    .height(CONTEXT_MENU_HEIGHT)
                    .padding(0.0)
                    .style(context_menu_style);

                menu = menu.push(button_element);
            }
            ContextMenuItem::Text(text) => {
                let text_component = Text::new(text)
                    .size(CONTEXT_MENU_SIZE)
                    .height(CONTEXT_MENU_HEIGHT)
                    .align_y(Vertical::Center);

                let text_element = row![
                    Space::new().width(CONTEXT_MENU_RADIOCHECK_WIDTH),
                    text_component
                ];

                let text_background = build_rectangle(
                    full_menu_width,
                    CONTEXT_MENU_HEIGHT,
                    LIGHT.context_menu_background,
                );

                let text_full = stack!(text_background, text_element);

                menu = menu.push(text_full);
            }
            ContextMenuItem::Break => {
                menu = menu.push(Space::new().height(CONTEXT_MENU_BORDER_WIDTH));
            }
            ContextMenuItem::Scroller((elements, elements_before_scroll)) => {
                let mut full_scroll = column![];

                let element_count = elements.len();

                for element in elements {
                    let button_content = row![
                        Space::new().width(CONTEXT_MENU_RADIOCHECK_WIDTH),
                        widget::text(element.name)
                            .size(CONTEXT_MENU_SIZE)
                            .height(CONTEXT_MENU_HEIGHT)
                            .align_y(Vertical::Center)
                    ];

                    let button_element = widget::button(button_content)
                        .on_press_maybe(element.message)
                        .width(full_menu_width)
                        .height(CONTEXT_MENU_HEIGHT)
                        .padding(0.0)
                        .style(context_menu_style);

                    full_scroll = full_scroll.push(button_element);
                }

                let scroller = if element_count > elements_before_scroll {
                    column![
                        scrollable(full_scroll)
                            .width(full_menu_width)
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

    let menu_border = build_rectangle(
        bordered_menu_width,
        bordered_menu_height,
        LIGHT.context_menu_border,
    );

    let pinned_menu = widget::pin(menu).position(Point::new(
        CONTEXT_MENU_BORDER_WIDTH,
        CONTEXT_MENU_BORDER_WIDTH,
    ));

    let bordered_menu = stack!(menu_border, pinned_menu);

    bordered_menu.into()
}
