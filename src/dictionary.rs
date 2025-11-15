use crate::filetools::{self, system_dictionary_path};
use regex::Regex;
use spellbook::Dictionary;
use std::{
    fs,
    sync::{LazyLock, RwLock},
};

/// global static dictionary
pub static DICTIONARY: LazyLock<RwLock<Dictionary>> =
    LazyLock::new(|| RwLock::new(composite_dictionary()));

/// regex that seperates out words. allows ' and - to show up in the middle of words
pub static WORD_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\b[\w'-]+\b").expect("couldn't create regex"));

/// pulls the words out from a string, returning the substring and position. the position is defined by its starting
/// and ending indexes in the original string
pub fn extract_words(text: &str) -> Vec<(&str, usize, usize)> {
    // since regex can't do lookahead/lookbehind, anything matching IGNORE_REGEX gets removed from the word list.
    // removes letter/number combinations, snake and camel case
    static IGNORE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"([\d]+[\w]*)|([a-zA-Z](_|-))+|([a-zA-Z][A-Z])+")
            .expect("couldn't create regex")
    });

    WORD_REGEX
        .find_iter(text)
        .filter(|regex_match| {
            let first_pass_valid_word = regex_match.as_str();
            !IGNORE_REGEX.is_match(first_pass_valid_word)
        })
        .map(|regex_match| (regex_match.as_str(), regex_match.start(), regex_match.end()))
        .collect()
}

/// generates a dictionary composed from the system dictionary combined with the personal dictionary
pub fn composite_dictionary() -> Dictionary {
    let (sys_aff_path, sys_dic_path) = system_dictionary_path();

    let sys_aff = fs::read_to_string(sys_aff_path).expect("couldn't read aff");
    let sys_dic = fs::read_to_string(sys_dic_path).expect("couldn't read dic");

    let personal_dic_path = filetools::personal_dictionary_path("personal.dic");
    if !personal_dic_path.exists() {
        fs::write(&personal_dic_path, "").expect("couldn't create new personal dic");
    }
    let personal_dic = fs::read_to_string(personal_dic_path).expect("couldn't read personal dic");

    let composite_dic = sys_dic + "\n" + &personal_dic;

    Dictionary::new(&sys_aff, &composite_dic).expect("couldn't create dictionary")
}

/// adds a word to the personal dictionary. the global dictionary is updated through .add(), and the personal
/// dictionary file is updated
pub fn add_word_to_personal_dictionary(new_word: &str) {
    let personal_dic_path = filetools::personal_dictionary_path("personal.dic");
    let personal_dic =
        fs::read_to_string(&personal_dic_path).expect("couldn't read personal dic to string");
    let mut dic_entries: Vec<&str> = personal_dic.lines().collect();

    if !dic_entries.contains(&new_word) {
        dic_entries.push(new_word);

        dic_entries.sort();

        let new_dic = dic_entries.join("\n");

        fs::write(personal_dic_path, new_dic).expect("couldn't save new dic");

        let mut dictionary = DICTIONARY.write().expect("couldn't get dictionary write");

        dictionary.add(new_word).expect("error word to dictionary");
    }
}
