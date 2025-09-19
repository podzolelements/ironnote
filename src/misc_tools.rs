// random tools and utilities that don't really fit anywhere in specific

use regex::Regex;
use std::sync::LazyLock;

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

/// pulls the words out from a string, returning the substring and position. the position is defined by its starting
/// and ending indexes in the original string
pub fn extract_words(text: &str) -> Vec<(&str, usize, usize)> {
    // reuse regex on all subsequent calls
    static WORD_REGEX: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"[\w]+").expect("couldn't create regex"));

    WORD_REGEX
        .find_iter(text)
        .map(|regex_match| (regex_match.as_str(), regex_match.start(), regex_match.end()))
        .collect()
}
