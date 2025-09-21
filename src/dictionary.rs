use crate::filetools::{self, system_dictionary_path};
use spellbook::Dictionary;
use std::{
    fs,
    sync::{LazyLock, RwLock},
};

/// global static dictionary
pub static DICTIONARY: LazyLock<RwLock<Dictionary>> =
    LazyLock::new(|| RwLock::new(composite_dictionary()));

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
