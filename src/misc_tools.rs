// random tools and utilities that don't really fit anywhere in specific

use chrono::{DateTime, Local, NaiveDate, TimeZone};

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

/// finds the index of the input that contains a whitespace character
pub fn first_whitespace_left(input: &str, starting_idx: usize) -> usize {
    let head = &input[0..starting_idx];

    let mut characters_moved = 0;

    for chara in head.chars().rev() {
        if chara.is_whitespace() {
            break;
        }
        characters_moved += 1;
    }

    starting_idx - characters_moved
}

/// converts a string in the form of "YYYY-MM-DD" into a DateTime<Local>
pub fn string_to_datetime(input: &str) -> DateTime<Local> {
    let nd = NaiveDate::parse_from_str(input, "%Y-%m-%d").expect("couldn't parse date");
    let ndt = nd.and_hms_opt(0, 0, 0).expect("couldn't create ndt");

    Local.from_local_datetime(&ndt).unwrap()
}
