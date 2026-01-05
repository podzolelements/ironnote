use std::{
    fs, io,
    path::PathBuf,
    sync::{LazyLock, RwLock, RwLockReadGuard, RwLockWriteGuard},
    time::Duration,
};

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
/// preferences that involve configurable files and directories
pub struct PathPreferences {
    pub(crate) journal_path: PathBuf,
    pub(crate) system_dictionary_dic: PathBuf,
    pub(crate) system_dictionary_aff: PathBuf,
    pub(crate) personal_dictionary_dic: PathBuf,
}

impl Default for PathPreferences {
    fn default() -> Self {
        let journal_path = Self::get_journal_path();

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
        }
    }
}

impl PathPreferences {
    /// the path to the file that contains the location of the top level /ironnote folder
    fn journal_path_file() -> PathBuf {
        let mut journal_path_file =
            dirs::config_local_dir().expect("couldn't open local config dir");
        journal_path_file.push("ironnote");
        journal_path_file.push("journal_path.json");

        journal_path_file
    }

    /// retreives the path of the journal from the journal_path.json pointer file. if the pointer file does not exist
    /// or cannot be parsed, it is created and set to dirs::data_local_dir()/ironnote
    fn get_journal_path() -> PathBuf {
        let journal_path_file_path = Self::journal_path_file();

        if let Ok(existing_journal_path_json) = fs::read_to_string(journal_path_file_path)
            && let Ok(existing_path) = serde_json::from_str(&existing_journal_path_json)
        {
            return existing_path;
        }

        let mut default_journal_path =
            dirs::data_local_dir().expect("local data dir does not exist");
        default_journal_path.push("ironnote");

        let default_journal_path_json = serde_json::to_string_pretty(&default_journal_path)
            .expect("couldn't convert path to json");

        fs::write(Self::journal_path_file(), &default_journal_path_json)
            .expect("unable to write journal path pointer");

        default_journal_path
    }

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

        let personal_dic_parent = self
            .personal_dictionary_dic
            .parent()
            .ok_or_else(|| io::Error::other("no parent directory"))?;
        fs::create_dir_all(personal_dic_parent)?;

        fs::create_dir_all(self.template_tasks_dir())?;

        Ok(())
    }

    /// creates any missing files that are required for ironnote to operate properly. this assumes all required
    /// directories already exist
    fn create_missing_files(&self) -> io::Result<()> {
        if !self.personal_dictionary_dic.exists() {
            fs::write(&self.personal_dictionary_dic, "\n")
        } else {
            Ok(())
        }
    }
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Default, Clone)]
/// stores all of the settings of the application
pub struct UserPreferences {
    pub(crate) general: GeneralPreferences,
    pub(crate) paths: PathPreferences,
    pub(crate) search: SearchPreferences,
}

impl UserPreferences {
    /// ensures that all paths and files that are expected to be present are created
    pub fn initalize_paths_and_files(&self) {
        self.paths
            .create_all_missing_dirs()
            .expect("unable to initalize preference dirs");

        self.paths
            .create_missing_files()
            .expect("unable to initalize preference dirs");
    }
}

/// global preferences object that stores all of the settings of the application
static PREFERENCES: LazyLock<RwLock<UserPreferences>> = LazyLock::new(|| {
    let default_preferences = UserPreferences::default();

    default_preferences.initalize_paths_and_files();

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

/// sets PREFERENCES to the provided new preferences
pub fn overwrite_preferences(new_preferences: UserPreferences) {
    new_preferences.initalize_paths_and_files();

    *preferences_mut() = new_preferences;
}
