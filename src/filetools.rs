use std::{fs, path::PathBuf};

pub fn savedata_path() -> PathBuf {
    let home = dirs::home_dir().expect("couldn't open home directory!");
    let mut save_path = PathBuf::new();
    save_path.push(home);
    save_path.push(".ironnote");
    save_path.push("data");

    save_path
}

pub fn savedata_file_path(filename: &str) -> PathBuf {
    let mut save_path = savedata_path();
    save_path.push(filename);

    save_path
}

pub fn setup_savedata_dirs(filename: &str) -> PathBuf {
    let save_path = savedata_file_path(filename);
    let save_parent_dir = save_path
        .parent()
        .expect("savedata path has no parent directory");

    fs::create_dir_all(save_parent_dir).expect("couldn't create savedata parent directory(s)");

    save_path
}

/// returns the paths of the (aff, dic) dictionary files. TODO: make configurable
pub fn system_dictionary_path() -> (PathBuf, PathBuf) {
    let mut aff_path = PathBuf::new();
    let mut dic_path = PathBuf::new();

    if cfg!(target_os = "windows") {
        aff_path.push("C:/Program Files/LibreOffice/share/extensions/dict-en/en_US.aff");
        dic_path.push("C:/Program Files/LibreOffice/share/extensions/dict-en/en_US.dic");
    } else if cfg!(target_os = "linux") {
        aff_path.push("/usr/share/hunspell/en_US.aff");
        dic_path.push("/usr/share/hunspell/en_US.dic");
    } else {
        todo!("configurable dictionary path");
    }

    (aff_path, dic_path)
}

/// returns the path of the personal dictionary, setting up the directories and making a custom.dic if it doesn't
/// already exist
pub fn personal_dictionary_path(filename: &str) -> PathBuf {
    let home = dirs::home_dir().expect("couldn't open home directory");
    let mut dictionary_path = PathBuf::new();
    dictionary_path.push(home);
    dictionary_path.push(".ironnote");
    dictionary_path.push("dictionary");
    dictionary_path.push(filename);

    let parent_dir = dictionary_path
        .parent()
        .expect("savedata path has no parent directory");

    if !dictionary_path.exists() {
        fs::create_dir_all(parent_dir).expect("couldn't create savedata parent directory(s)");
        fs::write(&dictionary_path, "").expect("couldn't create new custom dic");
    }

    dictionary_path
}
