use std::sync::{LazyLock, RwLock, RwLockReadGuard, RwLockWriteGuard};

#[derive(Debug, Default, Clone)]
/// general settings
pub struct GeneralPreferences {}

#[derive(Debug, Default, Clone)]
/// settings specific to the search functionality
pub struct SearchPreferences {
    /// if true, the text typed in the search bar will ignore the capitalization the search
    pub(crate) ignore_search_case: bool,
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
    pub(crate) _general: GeneralPreferences,
    pub(crate) search: SearchPreferences,
}

/// global preferences object that stores all of the settings of the application
static PREFERENCES: LazyLock<RwLock<UserPreferences>> = LazyLock::new(|| {
    let preferences = UserPreferences::default();

    RwLock::new(preferences)
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
