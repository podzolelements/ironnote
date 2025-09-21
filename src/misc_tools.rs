// random tools and utilities that don't really fit anywhere in specific

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
