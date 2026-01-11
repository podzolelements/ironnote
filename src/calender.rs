use chrono::{Datelike, Days, Local, NaiveDate};
use iced::{
    Alignment::Center,
    Element, Font,
    Length::FillPortion,
    font::Weight,
    never,
    widget::{Button, Row, Text, button, column, rich_text, row, span},
};

#[derive(Debug, Clone)]
pub enum CalenderMessage {
    DayClicked(NaiveDate),
    BackMonth,
    ForwardMonth,
    BackYear,
    ForwardYear,
}

#[derive(Debug)]
pub struct Calender {
    day_mapping: [NaiveDate; 42],
    edited_days: [bool; 42],
    current_date: NaiveDate,
}

impl Calender {
    pub fn build_calender<'a>(&self) -> Element<'a, CalenderMessage> {
        let month_back_btn = Button::new("<")
            .on_press(CalenderMessage::BackMonth)
            .width(30)
            .height(30);
        let month_text = Text::new(self.current_date.format("%B").to_string())
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
        let year_text = Text::new(self.current_date.format("%Y").to_string())
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

        let mut calender = column![month_year_bar, day_bar];

        let mut day_count = 0;
        let mut week_row = row![];
        for (day_index, date) in self.day_mapping.iter().enumerate() {
            let font = Font {
                weight: if self.edited_days[day_index] {
                    Weight::Bold
                } else {
                    Weight::Normal
                },
                ..Font::DEFAULT
            };

            let day_button_content = rich_text![span(date.day().to_string()).font(font)]
                .size(11)
                .on_link_click(never)
                .center();

            let day_button = button(day_button_content)
                .on_press(CalenderMessage::DayClicked(*date))
                .width(36)
                .height(24);

            week_row = week_row.push(day_button);
            day_count += 1;

            if day_count == 7 {
                calender = calender.push(week_row);
                week_row = row![];
                day_count = 0;
            }
        }

        calender.into()
    }

    fn start_day_offset(&self) -> u32 {
        let current_month_first = self.current_date.with_day(1).expect("1st doesn't exist");

        let start_offset = current_month_first.weekday().num_days_from_sunday();

        if start_offset == 0 { 7 } else { start_offset }
    }

    pub fn set_edited_days(&mut self, edited_days: [bool; 31]) {
        self.edited_days = [false; 42];

        let start_offset = self.start_day_offset() as usize;

        self.edited_days[start_offset..(start_offset + 31)].copy_from_slice(&edited_days);
    }

    pub fn set_current_date(&mut self, current_date: NaiveDate) {
        self.current_date = current_date;

        let days_before_first = Days::new(self.start_day_offset() as u64);

        let month_first = self.current_date.with_day(1).expect("first doesn't exist");

        let calender_start_date = month_first
            .checked_sub_days(days_before_first)
            .expect("unable to sub days");

        let mut iterative_date = calender_start_date;

        for day in self.day_mapping.iter_mut() {
            *day = iterative_date;

            iterative_date = iterative_date
                .checked_add_days(Days::new(1))
                .expect("couldn't add day");
        }
    }
}

impl Default for Calender {
    fn default() -> Self {
        Self {
            day_mapping: [NaiveDate::default(); 42],
            edited_days: [false; 42],
            current_date: Local::now().date_naive(),
        }
    }
}
