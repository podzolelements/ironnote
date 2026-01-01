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

/// converts a string in the form of "YYYY-MM-DD" into a DateTime<Local>
pub fn string_to_datetime(input: &str) -> DateTime<Local> {
    let nd = NaiveDate::parse_from_str(input, "%Y-%m-%d").expect("couldn't parse date");
    let ndt = nd.and_hms_opt(0, 0, 0).expect("couldn't create ndt");

    Local.from_local_datetime(&ndt).unwrap()
}

/// computes the number of characters to get to a (char, line) coordinate point in a string
pub fn chars_to_point(text: &str, point_x: usize, point_y: usize) -> usize {
    let lines: Vec<&str> = text.lines().collect();

    lines[0..point_y]
        .iter()
        .map(|line| line.chars().count() + 1)
        .sum::<usize>()
        + (point_x)
}

/// computes where a coordinate point is located in a text block. returns Some(true) if at the start of the text,
/// Some(false) at the end of the text, and None if in the middle. the max_lines accounts for lines with few characters
/// in them, and the max_chars prevents long single lines from registering as 'still on line 1', but being somewhere in
/// the middle of the text block.
pub fn point_on_edge_of_text(
    text: &str,
    point_x: usize,
    point_y: usize,
    max_lines: usize,
    max_chars: usize,
) -> Option<bool> {
    let lines: Vec<&str> = text.split('\n').collect();

    let total_lines = lines.len();
    let total_chars = text.chars().count();

    let chars_to_point = chars_to_point(text, point_x, point_y);

    let at_start = chars_to_point <= max_chars && point_y <= max_lines;

    let at_end = total_chars.saturating_sub(max_chars) <= chars_to_point
        && total_lines.saturating_sub(max_lines) <= point_y;

    if !at_start && !at_end {
        None
    } else if at_start {
        Some(true)
    } else {
        Some(false)
    }
}
