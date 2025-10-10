pub trait Stats {
    /// the number of whitespace separated blocks of characters in a block of text
    fn word_count(&self) -> usize;

    /// the number of characters in a block of text
    fn char_count(&self) -> usize;
}

pub trait BoundedDateStats: Stats {
    // the average number of characters in the structure's timeframe
    fn average_chars(&self) -> f64;

    // the average number of words in the structure's timeframe
    fn average_words(&self) -> f64;
}
