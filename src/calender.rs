use chrono::{Datelike, Local, NaiveDate};
use iced::{
    Alignment::Center,
    Element, Font,
    Length::FillPortion,
    font::Weight,
    never,
    widget::{Button, Column, Row, Text, rich_text, row, span},
};

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

#[derive(Debug)]
pub struct Calender {
    day_list: [u32; 42],
    month_mapping: [Month; 42],
    edited_days: [bool; 42],
    current_date: NaiveDate,
    month_text: String,
    year_text: String,
}

impl Calender {
    pub fn view<'a>(&self) -> Element<'a, CalenderMessage> {
        let mut cal = Column::new();

        let month_back_btn = Button::new("<")
            .on_press(CalenderMessage::BackMonth)
            .width(30)
            .height(30);
        let month_text = Text::new(self.month_text.clone())
            .center()
            .size(14)
            .width(75)
            .height(34);
        let month_forward_btn = Button::new(">")
            .on_press(CalenderMessage::ForwardMonth)
            .width(30)
            .height(30);

        let month_nav = row![month_back_btn, month_text, month_forward_btn];

        let year_back_btn = Button::new("<")
            .on_press(CalenderMessage::BackYear)
            .width(30)
            .height(30);
        let year_text = Text::new(self.year_text.clone())
            .center()
            .size(14)
            .width(40)
            .height(34);
        let year_forward_btn = Button::new(">")
            .on_press(CalenderMessage::ForwardYear)
            .width(30)
            .height(30);

        let year_nav = row![year_back_btn, year_text, year_forward_btn];

        let month_year_spacing = Text::new("").width(36 * 7 - 30 * 4 - 40 - 75);

        let month_year_bar = row![month_nav, month_year_spacing, year_nav];

        let days_text = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
        let mut day_bar = Row::new().width(7 * 36);

        for day in days_text {
            day_bar = day_bar.push(
                Text::new(day)
                    .width(FillPortion(1))
                    .align_x(Center)
                    .size(14),
            )
        }

        cal = cal.push(month_year_bar);
        cal = cal.push(day_bar);

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
                .on_link_click(never)
                .center();

                let day_button = Button::new(button_content)
                    .on_press(CalenderMessage::DayButton(
                        self.day_list[pos],
                        self.month_mapping[pos],
                    ))
                    .width(36)
                    .height(24);
                row = row.push(day_button);
            }
            cal = cal.push(row);
        }

        cal.into()
    }

    fn start_day_offset(active_date: NaiveDate) -> u32 {
        let active_first = active_date.with_day(1).expect("1st doesn't exist");

        let start_offset = active_first.weekday().num_days_from_sunday();

        if start_offset == 0 { 7 } else { start_offset }
    }

    pub fn set_edited_days(&mut self, edited_days: [bool; 31]) {
        self.edited_days = [false; 42];

        let start_offset = Self::start_day_offset(self.current_date) as usize;

        self.edited_days[start_offset..(start_offset + 31)].copy_from_slice(&edited_days);
    }

    fn days_in_previous_month(current_date: NaiveDate) -> u32 {
        match current_date.month() {
            1 => 31,
            2 => 31,
            3 => {
                if current_date.leap_year() {
                    29
                } else {
                    28
                }
            }
            4 => 31,
            5 => 30,
            6 => 31,
            7 => 30,
            8 => 31,
            9 => 31,
            10 => 30,
            11 => 31,
            12 => 30,
            _ => unreachable!("invalid month"),
        }
    }

    pub fn update_calender_dates(&mut self, current_date: NaiveDate) {
        self.current_date = current_date;

        let start_offset = Self::start_day_offset(self.current_date);

        let days_in_last_month = Self::days_in_previous_month(self.current_date);

        let mut cal_first_date = (days_in_last_month - start_offset) + 1;

        let mut current_day_addr = 0;

        for _day_last_month in 0..start_offset {
            self.day_list[current_day_addr] = cal_first_date;
            self.month_mapping[current_day_addr] = Month::Last;
            current_day_addr += 1;
            cal_first_date += 1;
        }

        for day_in_month in 1..=(self.current_date.num_days_in_month() as u32) {
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

        self.month_text = self.current_date.format("%B").to_string();
        self.year_text = self.current_date.format("%Y").to_string();
    }
}

impl Default for Calender {
    fn default() -> Self {
        Self {
            day_list: [0; 42],
            month_mapping: [Month::Last; 42],
            edited_days: [false; 42],
            current_date: Local::now().date_naive(),
            month_text: String::new(),
            year_text: String::new(),
        }
    }
}
