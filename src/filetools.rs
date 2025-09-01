use std::{fs, path::PathBuf};

pub fn savedata_path() -> PathBuf {
    let home = dirs::home_dir().expect("couldn't open home directory!");
    let mut save_path = PathBuf::new();
    save_path.push(home);
    save_path.push(".ironnote");
    save_path.push("data");

    save_path
}

pub fn savedata_file_path(filename: String) -> PathBuf {
    let mut save_path = savedata_path();
    save_path.push(filename);

    save_path
}

pub fn setup_savedata_dirs(filename: String) -> PathBuf {
    let save_path = savedata_file_path(filename);
    let save_parent_dir = save_path
        .parent()
        .expect("savedata path has no parent directory");

    fs::create_dir_all(save_parent_dir).expect("couldn't create savedata parent directory(s)");

    save_path
}
