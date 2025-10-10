use chrono::{DateTime, Datelike, Local, NaiveDate};
use iced::{
    Font,
    font::Weight,
    widget::{Button, Column, Row, Text, rich_text, span},
};

use crate::Message;

#[derive(Debug, Clone)]
pub enum CalenderMessage {
    DayButton(u32, Month),
    BackMonth,
    ForwardMonth,
    BackYear,
    ForwardYear,
}

#[derive(Clone, Copy, Debug)]
pub enum Month {
    Last,
    Current,
    Next,
}

pub struct Calender {
    day_list: [u32; 42],
    month_mapping: [Month; 42],
    edited_days: [bool; 42],
    datetime: DateTime<Local>,
    month_text: String,
    year_text: String,
}

impl Calender {
    pub fn view<'a>(&self) -> Column<'a, Message> {
        let mut cal = Column::new();

        let month_back_btn =
            Button::new("<").on_press(Message::Calender(CalenderMessage::BackMonth));
        let month_text = Text::new(self.month_text.clone()).center().size(14);
        let month_forward_btn =
            Button::new(">").on_press(Message::Calender(CalenderMessage::ForwardMonth));

        let year_back_btn = Button::new("<").on_press(Message::Calender(CalenderMessage::BackYear));
        let year_text = Text::new(self.year_text.clone()).center().size(14);
        let year_forward_btn =
            Button::new(">").on_press(Message::Calender(CalenderMessage::ForwardYear));

        let month_year_bar = Row::new()
            .push(month_back_btn)
            .push(month_text)
            .push(month_forward_btn)
            .push(year_back_btn)
            .push(year_text)
            .push(year_forward_btn);

        cal = cal.push(month_year_bar);

        for y in 0..6 {
            let mut row = Row::new();
            for x in 0..7 {
                let pos = y * 7 + x;
                let day_string = (self.day_list[pos]).to_string();

                let button_content = rich_text![span(day_string).font(Font {
                    weight: if self.edited_days[pos] {
                        Weight::Bold
                    } else {
                        Weight::Normal
                    },
                    ..Font::DEFAULT
                })]
                .size(11)
                .center();

                let day_button = Button::new(button_content)
                    .on_press(Message::Calender(CalenderMessage::DayButton(
                        self.day_list[pos],
                        self.month_mapping[pos],
                    )))
                    .width(36)
                    .height(24);
                row = row.push(day_button);
            }
            cal = cal.push(row);
        }

        cal
    }

    fn start_day_offset(active_datetime: DateTime<Local>) -> u32 {
        let nd = NaiveDate::from_ymd_opt(active_datetime.year(), active_datetime.month(), 1)
            .expect("first day is invalid?");
        let mut start_offset = nd.weekday().num_days_from_sunday();

        if start_offset == 0 {
            start_offset = 7;
        }

        start_offset
    }

    pub fn set_edited_days(&mut self, edited_days: [bool; 31]) {
        self.edited_days = [false; 42];

        let start_offset = Self::start_day_offset(self.datetime) as usize;

        self.edited_days[start_offset..(start_offset + 31)].copy_from_slice(&edited_days);
    }

    pub fn update_calender_dates(&mut self, active_datetime: DateTime<Local>) {
        self.datetime = active_datetime;

        let start_offset = Self::start_day_offset(self.datetime);

        let days_in_last_month = if self.datetime.month() == 1 {
            31
        } else {
            let nd = NaiveDate::from_ymd_opt(self.datetime.year(), self.datetime.month() - 1, 1)
                .expect("bad date");

            nd.num_days_in_month() as u32
        };
        let mut cal_first_date = (days_in_last_month - start_offset) + 1;

        let mut current_day_addr = 0;

        for _day_last_month in 0..start_offset {
            self.day_list[current_day_addr] = cal_first_date;
            self.month_mapping[current_day_addr] = Month::Last;
            current_day_addr += 1;
            cal_first_date += 1;
        }

        for day_in_month in 1..=(self.datetime.num_days_in_month() as u32) {
            self.day_list[current_day_addr] = day_in_month;
            self.month_mapping[current_day_addr] = Month::Current;
            current_day_addr += 1;
        }

        let eom = current_day_addr;
        let mut next_month_count = 1;
        for _day_next_month in eom..42 {
            self.day_list[current_day_addr] = next_month_count;
            self.month_mapping[current_day_addr] = Month::Next;
            next_month_count += 1;
            current_day_addr += 1;
        }

        self.month_text = self.datetime.format("%B").to_string();
        self.year_text = self.datetime.format("%Y").to_string();
    }
}

impl Default for Calender {
    fn default() -> Self {
        Self {
            day_list: [0; 42],
            month_mapping: [Month::Last; 42],
            edited_days: [false; 42],
            datetime: Local::now(),
            month_text: String::new(),
            year_text: String::new(),
        }
    }
}
