// random tools and utilities that don't really fit anywhere in specific

use chrono::NaiveDate;

/// returns true if all of the characters in the input string are the same character. returns true on an empty string
pub fn chars_all_same_in_string(input: &str) -> bool {
    if let Some(first_char) = input.chars().next() {
        for chara in input.chars() {
            if chara != first_char {
                return false;
            }
        }
        true
    } else {
        // no characters in string
        true
    }
}

/// converts a string in the form of "YYYY-MM-DD" into a NaiveDate
pub fn yyyy_mm_dd_string_to_date(input: &str) -> NaiveDate {
    NaiveDate::parse_from_str(input, "%Y-%m-%d").expect("couldn't parse date")
}
