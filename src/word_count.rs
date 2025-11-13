use std::collections::{HashMap, HashSet};

pub trait WordCount {
    /// recursively checks all elements below itself in the word count hierarchy to see if any subelements have gone
    /// out of sync, updating itself to reflect the current synchronization status. if any subelements have gone out of
    /// sync, return false
    fn is_word_count_in_sync(&mut self) -> bool;

    /// updates the 'current' table to reflect the actual current word counts
    fn reload_current_counts(&mut self);

    /// computes the difference between the 'current' and 'upstream' tables. a positive difference relays a word has
    /// been added to the 'current' table with respect to upstream, and vice versa. for example, ("bees", +3) indicates
    /// 'upstream' has 3 less of the word "bees" in it than the 'current' table does
    fn word_diff(&self) -> Vec<(String, i32)>;

    /// copies the 'current' table to the 'upstream' table, updating the sync status to be in sync
    fn sync_current_to_upstream(&mut self);

    /// updates the 'upstream' word count to reflect the current state of the word counts, returning the difference to
    /// pass up the hierarchy
    fn update_word_count(&mut self) -> Vec<(String, i32)> {
        self.reload_current_counts();

        let word_diff = self.word_diff();

        self.sync_current_to_upstream();

        word_diff
    }

    /// returns the upstream word count
    fn get_word_count(&self, word: &str) -> usize;

    /// returns the total number of words present in the structure
    fn total_word_count(&self) -> usize;

    /// returns the total number of characters present in the structure
    fn total_char_count(&self) -> usize;
}

pub trait TimedWordCount: WordCount {
    // the average number of characters in the structure's timeframe
    fn average_chars(&self) -> f64;

    // the average number of words in the structure's timeframe
    fn average_words(&self) -> f64;
}

#[derive(Debug, Clone, Default)]
/// structure to carry information of word counts between hierarchical groupings. rather than recomputing every single
/// word count on every update, the current word count for the structure can be computed into the 'current' table. from
/// the upstream caller, only the difference needs to be computed between the 'current' and 'upstream' tables, and the
/// upstream table can be updated to reflect this.
pub struct WordCounts {
    /// the current word count table. this reflects the actual word count at the current instant in time, regardless of
    /// the upstream status
    current: HashMap<String, usize>,

    /// the word count that the upstream word counter is aware of. likely to not reflect the current actual word counts
    upstream: HashMap<String, usize>,

    /// the synchronization status between upstream and current word counts
    in_sync: bool,

    /// total number of characters in the structure
    total_chars: usize,
}

impl WordCounts {
    /// returns true if the current and upstream tables are synchronized (contain identical contents)
    pub fn in_sync(&self) -> bool {
        self.in_sync
    }

    /// returns a set of all of the words in both the upstream and current tables
    fn complete_word_set(&self) -> HashSet<String> {
        let mut word_set = HashSet::new();

        for upstream_word in self.upstream.keys() {
            word_set.insert(upstream_word.to_string());
        }
        for current_word in self.current.keys() {
            word_set.insert(current_word.to_string());
        }

        word_set
    }

    /// gets the total number of words in the 'upstream' table
    pub fn total_word_count(&self) -> usize {
        self.upstream.iter().map(|(_word, count)| count).sum()
    }

    /// returns the total character count stored in the structure
    pub fn total_char_count(&self) -> usize {
        self.total_chars
    }

    /// sets the total character count of the structure
    pub fn set_total_char_count(&mut self, new_count: usize) {
        self.total_chars = new_count;
    }

    /// gets the word count of the specified word from the 'upstream' table
    pub fn get_word_count(&self, word: &str) -> usize {
        *self.upstream.get(word).unwrap_or(&0)
    }

    /// gets the word count of the specified word from the 'current' table
    fn current_count(&self, word: &str) -> usize {
        *self.current.get(word).unwrap_or(&0)
    }

    /// sets the synchronization status to the new_sync value
    pub fn set_sync(&mut self, new_sync: bool) {
        self.in_sync = new_sync;
    }

    /// clears the 'current' table
    pub fn clear_current(&mut self) {
        self.current.clear();
    }

    /// changes the 'current' table's number of words by count, inserting it if it not already present. care must be
    /// taken to ensure the number of words stored is accurate and never goes negative
    pub fn insert_or_add(&mut self, word: &str, count: i32) {
        self.current
            .entry(word.to_string())
            .and_modify(|current_count| {
                let count_magnitude = count.unsigned_abs() as usize;

                if count < 0 {
                    *current_count -= count_magnitude
                } else {
                    *current_count += count_magnitude
                }
            })
            .or_insert(count as usize);
    }

    /// computes the difference between the 'current' and 'upstream' tables. a positive difference relays a word has
    /// been added to the 'current' table with respect to upstream, and vice versa. for example, ("bees", +3) indicates
    /// 'upstream' has 3 less of the word "bees" in it than the 'current' table does
    pub fn word_diff(&self) -> Vec<(String, i32)> {
        let word_set = self.complete_word_set();

        let mut word_diff = vec![];

        for word in word_set {
            let current_count = self.current_count(&word) as i32;
            let upstream_count = self.get_word_count(&word) as i32;

            let diff = current_count - upstream_count;

            if diff != 0 {
                word_diff.push((word.clone(), diff));
            }
        }

        word_diff
    }

    /// copies the 'current' word count to the 'upstream' word count
    pub fn sync_current_to_upstream(&mut self) {
        self.upstream = self.current.clone();
        self.in_sync = true;
    }
}
