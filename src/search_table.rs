use chrono::{DateTime, Local};
use iced::{
    Font, Length,
    font::Weight,
    widget::{
        self, Column, Scrollable, rich_text,
        scrollable::{Direction, Scrollbar},
        span,
    },
};

use crate::Message;

#[derive(Debug, Clone)]
pub enum SearchTableMessage {
    EntryClicked(DateTime<Local>),
}

#[derive(Debug, Default)]
struct SearchEntry {
    start_text: String,
    bolded_text: String,
    end_text: String,
    date: DateTime<Local>,
}

#[derive(Debug, Default)]
pub struct SearchTable {
    entries: Vec<SearchEntry>,
}

impl SearchTable {
    pub fn view(&self) -> Scrollable<'_, Message> {
        let mut table = Column::new();

        for entry in self.entries.iter() {
            let rich_text = rich_text![
                span(entry.start_text.clone()),
                span(entry.bolded_text.clone()).font(Font {
                    weight: Weight::Semibold,
                    ..Font::DEFAULT
                }),
                span(entry.end_text.clone()),
            ]
            .size(12);

            table = table.push(
                widget::button(rich_text)
                    .on_press(Message::TableSearch(SearchTableMessage::EntryClicked(
                        entry.date,
                    )))
                    .width(500),
            );
        }

        widget::scrollable(table)
            .width(Length::Fixed(250.0))
            .height(Length::Fixed(500.0))
            .direction(Direction::Both {
                vertical: Scrollbar::new(),
                horizontal: Scrollbar::new(),
            })
    }

    pub fn insert_element(
        &mut self,
        start_text: String,
        bolded_text: String,
        end_text: String,
        date: DateTime<Local>,
    ) {
        let new_entry = SearchEntry {
            start_text,
            bolded_text,
            end_text,
            date,
        };

        self.entries.push(new_entry);
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }
}
