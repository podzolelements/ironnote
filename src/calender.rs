use chrono::{DateTime, Datelike, Local, NaiveDate};
use iced::widget::{Button, Column, Row, Text};

use crate::Message;

#[derive(Debug, Clone)]
pub enum CalenderMessage {
    DayButton(u32, Month),
}

#[derive(Clone, Copy, Debug)]
pub enum Month {
    LastMonth,
    CurrentMonth,
    NextMonth,
}

pub struct Calender {
    day_list: [u32; 42],
    month_mapping: [Month; 42],
}

impl Calender {
    pub fn view<'a>(&self) -> Column<'a, Message> {
        let mut cal = Column::new();

        for y in 0..6 {
            let mut row = Row::new();
            for x in 0..7 {
                let pos = y * 7 + x;
                let day_string = (self.day_list[pos]).to_string();
                let day_button = Button::new(Text::new(day_string).size(11).center())
                    .on_press(Message::Calender(CalenderMessage::DayButton(
                        self.day_list[pos],
                        self.month_mapping[pos],
                    )))
                    .width(34)
                    .height(21);
                row = row.push(day_button);
            }
            cal = cal.push(row);
        }

        cal
    }

    pub fn update_calender_dates(&mut self, active_datetime: DateTime<Local>) {
        let nd = NaiveDate::from_ymd_opt(active_datetime.year(), active_datetime.month(), 1)
            .expect("first day is invalid?");
        let mut start_offset = nd.weekday().num_days_from_sunday();

        if start_offset == 0 {
            start_offset = 7;
        }

        let days_in_last_month: u32;
        if active_datetime.month() == 1 {
            days_in_last_month = 31;
        } else {
            let nd =
                NaiveDate::from_ymd_opt(active_datetime.year(), active_datetime.month() - 1, 1)
                    .expect("bad date");

            days_in_last_month = nd.num_days_in_month() as u32;
        }
        let mut cal_first_date = (days_in_last_month - start_offset) + 1;

        let mut current_day_addr = 0;

        for _day_last_month in 0..start_offset {
            self.day_list[current_day_addr] = cal_first_date;
            self.month_mapping[current_day_addr] = Month::LastMonth;
            current_day_addr += 1;
            cal_first_date += 1;
        }

        for day_in_month in 1..=(nd.num_days_in_month() as u32) {
            self.day_list[current_day_addr] = day_in_month;
            self.month_mapping[current_day_addr] = Month::CurrentMonth;
            current_day_addr += 1;
        }

        let eom = current_day_addr.clone();
        let mut next_month_count = 1;
        for _day_next_month in eom..42 {
            self.day_list[current_day_addr] = next_month_count;
            self.month_mapping[current_day_addr] = Month::NextMonth;
            next_month_count += 1;
            current_day_addr += 1;
        }
    }
}

impl Default for Calender {
    fn default() -> Self {
        Self {
            day_list: [0; 42],
            month_mapping: [Month::LastMonth; 42],
        }
    }
}
