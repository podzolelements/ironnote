use crate::journal_pointer::JournalPointer;
use serde::{Deserialize, Serialize};
use std::{
    fs, io,
    path::PathBuf,
    sync::{LazyLock, RwLock, RwLockReadGuard, RwLockWriteGuard},
    time::Duration,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
/// general settings
pub struct GeneralPreferences {
    /// if true, the editor will perform the Autosave action at the autosave_interval
    pub(crate) autosave_enabled: bool,
    /// how often the autosave would occour if autosaving is enabled
    pub(crate) autosave_interval: Duration,
}

impl Default for GeneralPreferences {
    fn default() -> Self {
        Self {
            autosave_enabled: false,
            autosave_interval: Duration::from_mins(5),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// preferences that involve configurable files and directories
pub struct PathPreferences {
    pub(crate) journal_path: PathBuf,
    pub(crate) system_dictionary_dic: PathBuf,
    pub(crate) system_dictionary_aff: PathBuf,
    pub(crate) personal_dictionary_dic: PathBuf,
    pub(crate) preferences_path: PathBuf,
}

impl Default for PathPreferences {
    fn default() -> Self {
        let journal_pointer = JournalPointer::load_from_disk_or_default();

        let journal_path = journal_pointer.journal_path();
        let preferences_path = journal_pointer.preferences_path();

        let (aff_path, dic_path) = if cfg!(target_os = "linux") {
            (
                "/usr/share/hunspell/en_US.aff",
                "/usr/share/hunspell/en_US.dic",
            )
        } else if cfg!(target_os = "windows") {
            (
                "C:/Program Files/LibreOffice/share/extensions/dict-en/en_US.aff",
                "C:/Program Files/LibreOffice/share/extensions/dict-en/en_US.dic",
            )
        } else {
            ("", "")
        };

        let mut personal_dictionary_dic = journal_path.clone();
        personal_dictionary_dic.push("dictionary");
        personal_dictionary_dic.push("personal.dic");

        Self {
            journal_path,
            system_dictionary_aff: PathBuf::from(aff_path),
            system_dictionary_dic: PathBuf::from(dic_path),
            personal_dictionary_dic,
            preferences_path,
        }
    }
}

impl PathPreferences {
    /// the /ironnote/data directory
    pub fn savedata_dir(&self) -> PathBuf {
        let mut savedata_dir = self.journal_path.clone();
        savedata_dir.push("data");

        savedata_dir
    }

    /// the /ironnote/tasks directory
    fn tasks_dir(&self) -> PathBuf {
        let mut tasks_path = self.journal_path.clone();
        tasks_path.push("tasks");

        tasks_path
    }

    /// the /ironnote/tasks/templates directory
    pub fn template_tasks_dir(&self) -> PathBuf {
        let mut template_tasks_path = self.tasks_dir();
        template_tasks_path.push("templates");

        template_tasks_path
    }

    /// creates any missing directories that are required for ironnote to operate properly
    fn create_all_missing_dirs(&self) -> io::Result<()> {
        fs::create_dir_all(self.savedata_dir())?;

        let mut personal_dic_parent = self.personal_dictionary_dic.clone();
        if personal_dic_parent.pop() {
            fs::create_dir_all(personal_dic_parent)?;
        }

        fs::create_dir_all(self.template_tasks_dir())?;

        let mut preferences_parent = self.preferences_path.clone();
        if preferences_parent.pop() {
            fs::create_dir_all(preferences_parent)?;
        }

        Ok(())
    }

    /// creates any missing files that are required for ironnote to operate properly. this assumes all required
    /// directories already exist
    fn create_missing_files(&self) -> io::Result<()> {
        if !self.personal_dictionary_dic.exists() {
            fs::write(&self.personal_dictionary_dic, "")?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// settings specific to the search functionality
pub struct SearchPreferences {
    /// if true, the text typed in the search bar will ignore the capitalization the search
    pub(crate) ignore_search_case: bool,
}

impl Default for SearchPreferences {
    fn default() -> Self {
        Self {
            ignore_search_case: true,
        }
    }
}

impl SearchPreferences {
    /// toggles the ignore_search_case setting
    pub fn toggle_ignore_search_case(&mut self) {
        self.ignore_search_case = !self.ignore_search_case;
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
/// stores all of the settings of the application
pub struct UserPreferences {
    pub(crate) general: GeneralPreferences,
    pub(crate) paths: PathPreferences,
    pub(crate) search: SearchPreferences,
}

impl From<&UserPreferences> for JournalPointer {
    fn from(preferences: &UserPreferences) -> Self {
        JournalPointer::new(
            preferences.paths.journal_path.clone(),
            preferences.paths.preferences_path.clone(),
        )
    }
}

impl UserPreferences {
    /// ensures that all paths and files that are expected to be present are created
    pub fn initalize_paths_and_files(&self) -> io::Result<()> {
        self.paths.create_all_missing_dirs()?;

        self.paths.create_missing_files()?;

        if !self.paths.preferences_path.exists() {
            let preferences_json = serde_json::to_string_pretty(self)?;

            fs::write(&self.paths.preferences_path, preferences_json)?
        }

        Ok(())
    }

    /// writes the preferences to the location specified by the paths.preferences_path preference
    pub fn write_to_disk(&self) {
        let preferernces_json =
            serde_json::to_string_pretty(self).expect("serializing preferences failed");

        let preferences_path = self.paths.preferences_path.clone();

        fs::write(preferences_path, preferernces_json).expect("unable to write preferences file");

        let journal_pointer: JournalPointer = self.into();

        journal_pointer.save_to_disk();
    }

    /// returns the preferences loaded from the location of the JournalPointer's preference path. if the specified
    /// path does not exist or contains and invald perferences file, the default preferences are returned
    pub fn load_from_disk_or_default() -> Self {
        let journal_pointer = JournalPointer::load_from_disk_or_default();

        let preferences_path = journal_pointer.preferences_path();

        if let Ok(preferences_json) = fs::read_to_string(preferences_path)
            && let Ok(preferences) = serde_json::from_str(&preferences_json)
        {
            preferences
        } else {
            Self::default()
        }
    }
}

/// global preferences object that stores all of the settings of the application
static PREFERENCES: LazyLock<RwLock<UserPreferences>> = LazyLock::new(|| {
    let default_preferences = UserPreferences::load_from_disk_or_default();

    default_preferences
        .initalize_paths_and_files()
        .expect("unable to initalize preferences files/paths");

    RwLock::new(default_preferences)
});

/// gives read-only access to the global PREFERENCES structure
pub fn preferences() -> RwLockReadGuard<'static, UserPreferences> {
    PREFERENCES.read().expect("unable to get PREFERENCES read")
}

/// gives mutable access to the global PREFERENCES structure
pub fn preferences_mut() -> RwLockWriteGuard<'static, UserPreferences> {
    PREFERENCES
        .write()
        .expect("unable to get PREFERENCES write")
}

/// sets PREFERENCES to the provided new preferences, writing new preferences to disk
pub fn overwrite_preferences(new_preferences: UserPreferences) {
    new_preferences
        .initalize_paths_and_files()
        .expect("unable to initalize paths/files of new preferences");

    new_preferences.write_to_disk();

    *preferences_mut() = new_preferences;
}
