use crate::{button_themes::standard_button_style, journal_theme::LIGHT};
use chrono::{Datelike, Days, Local, NaiveDate};
use iced::{
    Alignment::Center,
    Background, Border, Color, Element, Font,
    Length::FillPortion,
    Shadow, Theme, Vector,
    font::Weight,
    never,
    widget::{
        self, Button, Row, Space, Text,
        button::{self, Status},
        column, rich_text, row, span,
    },
};

#[derive(Debug, Clone)]
/// types of messages the calender widget can generate
pub enum CalenderMessage {
    DayClicked(NaiveDate),
    BackMonth,
    ForwardMonth,
    BackYear,
    ForwardYear,
}

#[derive(Debug, Clone)]
/// the colormap defines the colors that get applied to the calender
pub struct CalenderColormap {
    /// the weights are values [0.0, 1.0] that represent the "progression" from the floor color to the ceiling color. a
    /// 0.0 represents the color should be the color_floor, and a 1.0 will result in the color being the color_ceiling.
    /// if None, the color will not be applied and be the default
    pub(crate) colormap_weights: [Option<f32>; 42],
    pub(crate) color_floor: Color,
    pub(crate) color_ceiling: Color,

    /// if true, the current day color will overwrite the value normally calculated through the colormap weights
    pub(crate) current_day_overwrite: bool,
}

impl Default for CalenderColormap {
    fn default() -> Self {
        Self {
            colormap_weights: [None; 42],
            color_floor: Color::WHITE,
            color_ceiling: Color::WHITE,
            current_day_overwrite: true,
        }
    }
}

#[derive(Debug)]
/// calender widget that is displayed out as a set of 6 rows by 7 days, with month and year navigations
pub struct Calender {
    day_mapping: [NaiveDate; 42],
    bolded_days: [bool; 42],
    current_date: NaiveDate,
    colormap: CalenderColormap,
}

/// width of each day button on the calender
const DAY_WIDTH: u32 = 34;

/// total width of the calender widget
pub const TOTAL_CALENDER_WIDTH: u32 = DAY_WIDTH * 7;

impl Calender {
    /// constructs the actual iced Element from the calender structure
    pub fn build_calender<'a>(&'a self) -> Element<'a, CalenderMessage> {
        const NAV_BUTTON_WIDTH: u32 = 25;
        const MONTH_TEXT_WIDTH: u32 = 65;
        const YEAR_TEXT_WIDTH: u32 = 35;

        let month_back_btn = Button::new(Text::new("<").center())
            .on_press(CalenderMessage::BackMonth)
            .width(NAV_BUTTON_WIDTH)
            .height(30)
            .style(standard_button_style);
        let month_text = Text::new(self.current_date.format("%B").to_string())
            .center()
            .size(12)
            .width(MONTH_TEXT_WIDTH)
            .height(34);
        let month_forward_btn = Button::new(Text::new(">").center())
            .on_press(CalenderMessage::ForwardMonth)
            .width(NAV_BUTTON_WIDTH)
            .height(30)
            .style(standard_button_style);

        let month_nav = row![month_back_btn, month_text, month_forward_btn];

        let year_back_btn = Button::new(Text::new("<").center())
            .on_press(CalenderMessage::BackYear)
            .width(NAV_BUTTON_WIDTH)
            .height(30)
            .style(standard_button_style);
        let year_text = Text::new(self.current_date.format("%Y").to_string())
            .center()
            .size(12)
            .width(YEAR_TEXT_WIDTH)
            .height(34);
        let year_forward_btn = Button::new(Text::new(">").center())
            .on_press(CalenderMessage::ForwardYear)
            .width(NAV_BUTTON_WIDTH)
            .height(30)
            .style(standard_button_style);

        let year_nav = row![year_back_btn, year_text, year_forward_btn];

        let month_year_bar = row![
            month_nav,
            Space::new().width(
                TOTAL_CALENDER_WIDTH - (4 * NAV_BUTTON_WIDTH) - MONTH_TEXT_WIDTH - YEAR_TEXT_WIDTH
            ),
            year_nav
        ];

        let days_text = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
        let mut day_bar = Row::new().width(TOTAL_CALENDER_WIDTH);

        for day in days_text {
            day_bar = day_bar.push(
                Text::new(day)
                    .width(FillPortion(1))
                    .align_x(Center)
                    .size(12),
            )
        }

        let mut calender = column![month_year_bar, day_bar];

        let mut day_count = 0;
        let mut week_row = row![];
        for (day_index, date) in self.day_mapping.iter().enumerate() {
            let font = Font {
                weight: if self.bolded_days[day_index] {
                    Weight::Bold
                } else {
                    Weight::Normal
                },
                ..Font::DEFAULT
            };

            let day_button_content = rich_text![span(date.day().to_string()).font(font)]
                .size(13)
                .on_link_click(never)
                .center();

            let day_button = widget::button(day_button_content)
                .on_press(CalenderMessage::DayClicked(*date))
                .width(DAY_WIDTH)
                .height(24)
                .style(self.day_button_color(day_index));

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

    /// calculates the number of days from the previous month should be included at the start of the calender. the
    /// value that is returned will always be at least 1, in the range of [1, 7]
    fn previous_month_days(&self) -> u32 {
        let current_month_first = self.current_date.with_day(1).expect("1st doesn't exist");

        let previous_month_days = current_month_first.weekday().num_days_from_sunday();

        if previous_month_days == 0 {
            7
        } else {
            previous_month_days
        }
    }

    /// sets the days from the current month that should be bolded. only days that are in the current month are able to
    /// be bolded
    pub fn set_bolded_days(&mut self, bolded_days: &[bool; 31]) {
        self.bolded_days = [false; 42];

        let start_offset = self.previous_month_days() as usize;

        self.bolded_days[start_offset..(start_offset + 31)].clone_from_slice(bolded_days);
    }

    /// sets the current date of the calender, updating the calender structure to reflect the current date
    pub fn set_current_date(&mut self, current_date: NaiveDate) {
        self.current_date = current_date;

        let days_before_first = Days::new(self.previous_month_days() as u64);

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

    /// sets the colormap of the calender
    pub fn set_colormap(&mut self, new_colormap: CalenderColormap) {
        self.colormap = new_colormap;
    }

    /// returns the date of the top left calender day
    pub fn calender_start_date(&self) -> NaiveDate {
        self.day_mapping[0]
    }

    /// linearly interpolates a new color on the range [color_floor, color_ceiling] based on the weight value's range
    /// from [0.0, 1.0]
    fn color_linear_interpolate(color_floor: Color, color_ceiling: Color, weight: f32) -> Color {
        let weight = weight.clamp(0.0, 1.0);

        let new_r = (color_ceiling.r - color_floor.r) * weight + color_floor.r;
        let new_g = (color_ceiling.g - color_floor.g) * weight + color_floor.g;
        let new_b = (color_ceiling.b - color_floor.b) * weight + color_floor.b;
        let new_a = (color_ceiling.a - color_floor.a) * weight + color_floor.a;

        Color::from_linear_rgba(new_r, new_g, new_b, new_a)
    }

    /// returns the styling function used to color in the day buttons on the calender
    fn day_button_color(&self, day_index: usize) -> impl Fn(&Theme, Status) -> button::Style {
        // the 8th day is the first day in the calender that is guaranteed to be in the current month
        let calender_main_month = self.day_mapping[7].month();

        let current_days_month = self.day_mapping[day_index].month();

        let is_current_month = calender_main_month == current_days_month;
        let is_current_day = self.day_mapping[day_index] == self.current_date;

        let background_color = if is_current_day && self.colormap.current_day_overwrite {
            LIGHT.selection
        } else if let Some(color_weight) = self.colormap.colormap_weights[day_index] {
            Self::color_linear_interpolate(
                self.colormap.color_floor,
                self.colormap.color_ceiling,
                color_weight,
            )
        } else {
            LIGHT.default_background
        };

        let text_color = if !is_current_month {
            LIGHT.dimmed_text
        } else if is_current_day && self.colormap.current_day_overwrite {
            LIGHT.selection_text
        } else {
            LIGHT.default_text
        };

        let border_color = if is_current_day && !self.colormap.current_day_overwrite {
            LIGHT.selection
        } else {
            LIGHT.default_background
        };

        let (boarder_radius, border_width) = if is_current_day {
            (2.0, 1.0)
        } else {
            (6.0, 2.0)
        };

        move |_theme: &Theme, status: Status| {
            let modified_background = match status {
                Status::Active => background_color,
                Status::Hovered | Status::Pressed | Status::Disabled => {
                    LIGHT.darken(background_color)
                }
            };

            button::Style {
                background: Some(Background::Color(modified_background)),
                text_color,
                border: Border {
                    color: border_color,
                    width: border_width,
                    radius: boarder_radius.into(),
                },
                shadow: Shadow {
                    color: Color::TRANSPARENT,
                    offset: Vector::ZERO,
                    blur_radius: 0.0,
                },
                snap: true,
            }
        }
    }
}

impl Default for Calender {
    fn default() -> Self {
        Self {
            day_mapping: [NaiveDate::default(); 42],
            bolded_days: [false; 42],
            current_date: Local::now().date_naive(),
            colormap: CalenderColormap::default(),
        }
    }
}
