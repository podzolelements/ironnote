use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

#[derive(Debug, Serialize, Deserialize)]
/// the JournalPointer stores PathBuf "pointer" paths for greater configurability. the JournalPointer is fixed at
/// dirs::config_local_dir()/ironnote/journal_pointer.json, so the program is always able to locate the paths to data
/// on a cold start of the application. by altering the paths inside the pointer file but keeping the pointer file
/// location constant, we can freely change where the data is stored, while ensuring configuration files never
/// "disappear" from the knowledge of the program (since we changed their locations)
pub struct JournalPointer {
    /// location of the top level /ironnote journal directory
    journal_path: PathBuf,

    /// location of the preferences file
    preferences_path: PathBuf,
}

impl JournalPointer {
    /// creates a new JournalPointer from the given journal and prefernces path
    pub fn new(journal_path: PathBuf, preferences_path: PathBuf) -> Self {
        JournalPointer {
            journal_path,
            preferences_path,
        }
    }

    /// the path to the file that contains the JournalPointer. this is always located at
    /// dirs::config_local_dir()/ironnote/journal_pointer.json
    fn journal_pointer_file() -> PathBuf {
        let mut journal_path_file =
            dirs::config_local_dir().expect("couldn't open local config dir");
        journal_path_file.push("ironnote");

        fs::create_dir_all(&journal_path_file).expect("unable to create config directory");

        journal_path_file.push("journal_pointer.json");

        journal_path_file
    }

    /// reads the JournalPointer from the disk and returns it. if the file did not exist or was corrupted, it returns
    /// the default JournalPointer and writes that to disk
    pub fn load_from_disk_or_default() -> Self {
        if let Ok(journal_pointer_json) = fs::read_to_string(Self::journal_pointer_file())
            && let Ok(journal_pointer) = serde_json::from_str(&journal_pointer_json)
        {
            return journal_pointer;
        }

        let default_journal_pointer = JournalPointer::default();

        // since the file either didn't exist or was corrupted, save back to the disk so it is there next time
        default_journal_pointer.save_to_disk();

        default_journal_pointer
    }

    /// writes the JournalPointer to the disk at its designated ```journal_pointer_file()``` location
    pub fn save_to_disk(&self) {
        let journal_path_json =
            serde_json::to_string_pretty(self).expect("unable to serialize journal path");

        fs::write(Self::journal_pointer_file(), journal_path_json)
            .expect("unable to write journal path file");
    }

    /// returns the location of the top level /ironnote journal path
    pub fn journal_path(&self) -> PathBuf {
        self.journal_path.clone()
    }

    /// returns the location of the preferences file for the journal
    pub fn preferences_path(&self) -> PathBuf {
        self.preferences_path.clone()
    }
}

impl Default for JournalPointer {
    fn default() -> Self {
        let local_data_dir = dirs::data_local_dir().expect("local data dir unavailable");

        // default journal location: dirs::data_local_dir()/ironnote
        let mut journal_path = local_data_dir;
        journal_path.push("ironnote");

        // default preferences location: dirs::data_local_dir()/ironnote/config/preferences.json
        let mut preferences_path = journal_path.clone();
        preferences_path.push("config");
        preferences_path.push("preferences.json");

        Self {
            journal_path,
            preferences_path,
        }
    }
}
