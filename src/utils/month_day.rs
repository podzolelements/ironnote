use chrono::Month;
use serde::{Deserialize, Serialize};
use strum::{Display, VariantArray};

#[derive(Debug, Clone, Copy, PartialEq, Display, VariantArray, Serialize, Deserialize)]
/// a shadow of chrono::Month that derives Display for use in the month select dropdown
pub enum DispMonth {
    January,
    February,
    March,
    April,
    May,
    June,
    July,
    August,
    September,
    October,
    November,
    December,
}

impl DispMonth {
    /// converts it into the equivelent Month
    pub fn chrono_month(&self) -> Month {
        match self {
            DispMonth::January => Month::January,
            DispMonth::February => Month::February,
            DispMonth::March => Month::March,
            DispMonth::April => Month::April,
            DispMonth::May => Month::May,
            DispMonth::June => Month::June,
            DispMonth::July => Month::July,
            DispMonth::August => Month::August,
            DispMonth::September => Month::September,
            DispMonth::October => Month::October,
            DispMonth::November => Month::November,
            DispMonth::December => Month::December,
        }
    }

    /// gets the maximum day count in each month
    pub fn day_count(&self) -> u32 {
        match self {
            DispMonth::January => 31,
            DispMonth::February => 29,
            DispMonth::March => 31,
            DispMonth::April => 30,
            DispMonth::May => 31,
            DispMonth::June => 30,
            DispMonth::July => 31,
            DispMonth::August => 31,
            DispMonth::September => 30,
            DispMonth::October => 31,
            DispMonth::November => 30,
            DispMonth::December => 31,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// structure to store the month and day component of a date. note the day feild represents the calendar day number, so
/// 0 will not ever be a value day is
pub struct MonthDay {
    month: DispMonth,
    day: u32,
}

impl MonthDay {
    /// create a MonthDay from the given month and day
    pub fn new(month: DispMonth, day: u32) -> Self {
        Self { month, day }
    }

    /// returns the Month component as a chrono::Month
    pub fn month(&self) -> Month {
        self.month.chrono_month()
    }

    /// returns the day component
    pub fn day(&self) -> u32 {
        self.day
    }
}
