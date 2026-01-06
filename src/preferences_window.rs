use crate::{
    SharedAppState, UpstreamAction,
    file_extensions::{AFF_EXT_LIST, DIC_EXT_LIST, JSON_EXT_LIST, build_extensions},
    file_picker::{FilePicker, FilePickerMessage},
    keyboard_manager::KeyboardAction,
    tabview::{TabviewItem, tabview_content_horizontal},
    upgraded_content::{ContentAction, Restriction, UpgradedContent},
    user_preferences::{UserPreferences, overwrite_preferences, preferences},
    window_manager::{WindowType, Windowable},
};
use iced::{
    Length, Task,
    widget::{self, Space, Text, button, checkbox, column, row, text_editor::Action},
};
use std::time::Duration;
use strum::Display;

#[derive(Debug, Default, Clone, PartialEq, Display)]
pub enum PreferencesTab {
    #[default]
    General,
    Paths,
    Keyboard,
}

impl PreferencesTab {
    pub fn to_index(&self) -> usize {
        match self {
            PreferencesTab::General => 0,
            PreferencesTab::Paths => 1,
            PreferencesTab::Keyboard => 2,
        }
    }
}

#[derive(Debug, Clone)]
pub enum GeneralMessage {
    ToggleAutosave(bool),
    EditAutosaveMinute(Action),
    EditAutosaveSecond(Action),
}

#[derive(Debug, Clone)]
pub enum PathsMessage {
    Journal(FilePickerMessage),
    Preferences(FilePickerMessage),
    SystemDic(FilePickerMessage),
    SystemAff(FilePickerMessage),
    PersonalDic(FilePickerMessage),
}

#[derive(Debug, Clone)]
pub enum PreferencesMessage {
    KeyEvent(KeyboardAction),

    TabSwitched(PreferencesTab),
    Cancel,
    Save,
    SaveAndExit,

    General(GeneralMessage),
    Paths(PathsMessage),
}

#[derive(Debug)]
pub enum ActiveContent {
    AutosaveMinute,
    AutosaveSecond,

    JournalPath,
    PreferencesPath,
    SystemDicPath,
    SystemAffPath,
    PersonalDicPath,
}

#[derive(Debug)]
pub struct Preferences {
    working_preferences: UserPreferences,
    edited_preferences: bool,
    preference_edit_requires_restart: bool,

    current_preference_tab: PreferencesTab,
    active_content: Option<ActiveContent>,

    autosave_minute_content: UpgradedContent,
    autosave_minutes: u64,
    autosave_second_content: UpgradedContent,
    autosave_seconds: u64,

    journal_path_picker: FilePicker,
    preferences_path_picker: FilePicker,
    system_dic_path_picker: FilePicker,
    system_aff_path_picker: FilePicker,
    personal_dic_path_picker: FilePicker,
}

impl Default for Preferences {
    fn default() -> Self {
        let working_preferences = preferences().clone();

        Self {
            working_preferences: working_preferences.clone(),
            edited_preferences: false,
            preference_edit_requires_restart: false,

            current_preference_tab: PreferencesTab::default(),
            active_content: None,

            autosave_minute_content: UpgradedContent::with_text("5"),
            autosave_minutes: 5,
            autosave_second_content: UpgradedContent::with_text("0"),
            autosave_seconds: 0,

            journal_path_picker: FilePicker::directory(working_preferences.paths.journal_path),
            preferences_path_picker: FilePicker::file(
                working_preferences.paths.preferences_path,
                &build_extensions(JSON_EXT_LIST),
            ),
            system_dic_path_picker: FilePicker::file(
                working_preferences.paths.system_dictionary_dic,
                &build_extensions(DIC_EXT_LIST),
            ),
            system_aff_path_picker: FilePicker::file(
                working_preferences.paths.system_dictionary_aff,
                &build_extensions(AFF_EXT_LIST),
            ),
            personal_dic_path_picker: FilePicker::file(
                working_preferences.paths.personal_dictionary_dic,
                &build_extensions(DIC_EXT_LIST),
            ),
        }
    }
}

impl Windowable<PreferencesMessage> for Preferences {
    fn title(&self) -> String {
        "Preferences".to_string()
    }

    fn view<'a>(&'a self, _state: &SharedAppState) -> iced::Element<'a, PreferencesMessage> {
        const SUB_OPTION_SPACE_WIDTH: u32 = 50;

        let general_tab_content = {
            let general_prefs = &self.working_preferences.general;

            let title = Text::new("General Settings");

            let autosave_checkbox = checkbox(general_prefs.autosave_enabled)
                .on_toggle(|checked| {
                    PreferencesMessage::General(GeneralMessage::ToggleAutosave(checked))
                })
                .label("Enable auto saving");

            let autosave_minute_text = Text::new("Minutes");
            let autosave_minute_editor = if self.working_preferences.general.autosave_enabled {
                widget::text_editor(self.autosave_minute_content.raw_content())
                    .on_action(|action| {
                        PreferencesMessage::General(GeneralMessage::EditAutosaveMinute(action))
                    })
                    .width(50)
            } else {
                widget::text_editor(self.autosave_minute_content.raw_content()).width(50)
            };

            let autosave_second_text = Text::new("Seconds");
            let autosave_second_editor = if self.working_preferences.general.autosave_enabled {
                widget::text_editor(self.autosave_second_content.raw_content())
                    .on_action(|action| {
                        PreferencesMessage::General(GeneralMessage::EditAutosaveSecond(action))
                    })
                    .width(50)
            } else {
                widget::text_editor(self.autosave_second_content.raw_content()).width(50)
            };

            let autosave_time = row![
                Space::new().width(SUB_OPTION_SPACE_WIDTH),
                autosave_minute_text,
                autosave_minute_editor,
                Space::new().width(25),
                autosave_second_text,
                autosave_second_editor
            ];

            let autosave = column![autosave_checkbox, autosave_time];

            column![title, autosave]
        };

        let general_tab = TabviewItem {
            title: PreferencesTab::General.to_string(),
            clicked_message: PreferencesMessage::TabSwitched(PreferencesTab::General),
            content: general_tab_content.into(),
        };

        let paths_tab_content = {
            let title = Text::new("Path Settings");

            let journal_location_path = self
                .journal_path_picker
                .view()
                .map(|message| PreferencesMessage::Paths(PathsMessage::Journal(message)));
            let journal_location =
                column![Text::new("Journal Save Location"), journal_location_path];

            let preferences_path_editor = self
                .preferences_path_picker
                .view()
                .map(|message| PreferencesMessage::Paths(PathsMessage::Preferences(message)));
            let preferences_path = column![
                Text::new("Preferences Save Location"),
                preferences_path_editor
            ];

            let system_dic_path = self
                .system_dic_path_picker
                .view()
                .map(|message| PreferencesMessage::Paths(PathsMessage::SystemDic(message)));
            let system_dic = column![Text::new("System Dictionary .dic"), system_dic_path];

            let system_aff_path = self
                .system_aff_path_picker
                .view()
                .map(|message| PreferencesMessage::Paths(PathsMessage::SystemAff(message)));
            let system_aff = column![Text::new("System Dictionary .aff"), system_aff_path];

            let personal_dic_path = self
                .personal_dic_path_picker
                .view()
                .map(|message| PreferencesMessage::Paths(PathsMessage::PersonalDic(message)));
            let personal_dic = column![Text::new("Personal Dictionary .dic"), personal_dic_path];

            column![
                title,
                journal_location,
                preferences_path,
                system_dic,
                system_aff,
                personal_dic
            ]
            .into()
        };

        let paths_tab = TabviewItem {
            title: PreferencesTab::Paths.to_string(),
            clicked_message: PreferencesMessage::TabSwitched(PreferencesTab::Paths),
            content: paths_tab_content,
        };

        let keyboard_tab_content = { column![Text::new("Keyboard Settings")] };

        let keyboard_tab = TabviewItem {
            title: PreferencesTab::Keyboard.to_string(),
            clicked_message: PreferencesMessage::TabSwitched(PreferencesTab::Keyboard),
            content: keyboard_tab_content.into(),
        };

        let tab_elements = vec![general_tab, paths_tab, keyboard_tab];

        let preference_editor = tabview_content_horizontal(
            tab_elements,
            self.current_preference_tab.to_index(),
            Length::Fill,
            Length::Fill,
        );

        let cancel_button = button(Text::new("Cancel")).on_press(PreferencesMessage::Cancel);
        let save_button = button(Text::new("Save"))
            .on_press_maybe(self.edited_preferences.then_some(PreferencesMessage::Save));
        let save_exit_button = button(Text::new("Save and Exit")).on_press_maybe(
            self.edited_preferences
                .then_some(PreferencesMessage::SaveAndExit),
        );

        let save_options = row![
            Space::new().width(Length::Fill),
            cancel_button,
            save_button,
            save_exit_button
        ];

        column![preference_editor, save_options].into()
    }

    fn update(
        &mut self,
        state: &mut SharedAppState,
        message: PreferencesMessage,
    ) -> iced::Task<PreferencesMessage> {
        match message {
            PreferencesMessage::KeyEvent(keyboard_action) => match keyboard_action {
                KeyboardAction::Content(text_edit) => {
                    self.content_perform(state, text_edit.to_content_action());
                }
                KeyboardAction::Save => {
                    if self.edited_preferences {
                        return self.update(state, PreferencesMessage::Save);
                    }
                }
                KeyboardAction::Debug => {}
                KeyboardAction::Unbound(_unbound_key) => {}
            },
            PreferencesMessage::TabSwitched(new_preferences_tab) => {
                self.active_content = None;

                self.current_preference_tab = new_preferences_tab;
            }
            PreferencesMessage::General(general_message) => match general_message {
                GeneralMessage::ToggleAutosave(is_checked) => {
                    self.edited_preferences = true;

                    self.working_preferences.general.autosave_enabled = is_checked;
                }
                GeneralMessage::EditAutosaveMinute(action) => {
                    self.active_content = Some(ActiveContent::AutosaveMinute);

                    self.autosave_minute_content
                        .perform(ContentAction::Restricted((
                            Restriction::NumbersOnly,
                            action,
                        )));

                    let minute_text = self.autosave_minute_content.text();
                    let minutes = minute_text.parse::<u64>().unwrap_or(0).min(9999);

                    self.autosave_minutes = minutes;

                    // this prevents leading 0s from being entered (though the with_text resets the cursor position to
                    // the start, it doesn't matter since leading 0s can only be added at the start) and enforces the
                    // max minutes limit for absurd values
                    if self.autosave_minute_content.text() != self.autosave_minutes.to_string() {
                        self.autosave_minute_content =
                            UpgradedContent::with_text(&self.autosave_minutes.to_string())
                    }

                    self.edited_preferences = true;

                    self.working_preferences.general.autosave_interval =
                        Duration::from_mins(self.autosave_minutes)
                            + Duration::from_secs(self.autosave_seconds);
                }
                GeneralMessage::EditAutosaveSecond(action) => {
                    self.active_content = Some(ActiveContent::AutosaveSecond);

                    self.autosave_second_content
                        .perform(ContentAction::Restricted((
                            Restriction::NumbersOnly,
                            action,
                        )));

                    let second_text = self.autosave_second_content.text();
                    let seconds = second_text.parse::<u64>().unwrap_or(0).min(59);

                    self.autosave_seconds = seconds;

                    // this prevents leading 0s from being entered (though the with_text resets the cursor position to
                    // the start, it doesn't matter since leading 0s can only be added at the start) and enforces the
                    // 59 minute limit max
                    if self.autosave_second_content.text() != self.autosave_seconds.to_string() {
                        self.autosave_second_content =
                            UpgradedContent::with_text(&self.autosave_seconds.to_string())
                    }

                    self.edited_preferences = true;

                    self.working_preferences.general.autosave_interval =
                        Duration::from_mins(self.autosave_minutes)
                            + Duration::from_secs(self.autosave_seconds);
                }
            },
            PreferencesMessage::Paths(paths_message) => {
                match paths_message {
                    PathsMessage::Journal(message) => {
                        self.active_content =
                            matches!(&message, FilePickerMessage::FilepathEdit(_content_action))
                                .then_some(ActiveContent::JournalPath);

                        self.journal_path_picker.update(message);

                        self.working_preferences.paths.journal_path =
                            self.journal_path_picker.path();
                    }
                    PathsMessage::Preferences(message) => {
                        self.active_content =
                            matches!(&message, FilePickerMessage::FilepathEdit(_content_action))
                                .then_some(ActiveContent::PreferencesPath);

                        self.preferences_path_picker.update(message);

                        self.working_preferences.paths.preferences_path =
                            self.preferences_path_picker.path();
                    }
                    PathsMessage::SystemDic(message) => {
                        self.active_content =
                            matches!(&message, FilePickerMessage::FilepathEdit(_content_action))
                                .then_some(ActiveContent::SystemDicPath);

                        self.system_dic_path_picker.update(message);

                        self.working_preferences.paths.system_dictionary_dic =
                            self.system_dic_path_picker.path();
                    }
                    PathsMessage::SystemAff(message) => {
                        self.active_content =
                            matches!(&message, FilePickerMessage::FilepathEdit(_content_action))
                                .then_some(ActiveContent::SystemAffPath);

                        self.system_aff_path_picker.update(message);

                        self.working_preferences.paths.system_dictionary_aff =
                            self.system_aff_path_picker.path();
                    }
                    PathsMessage::PersonalDic(message) => {
                        self.active_content =
                            matches!(&message, FilePickerMessage::FilepathEdit(_content_action))
                                .then_some(ActiveContent::PersonalDicPath);

                        self.personal_dic_path_picker.update(message);

                        self.working_preferences.paths.personal_dictionary_dic =
                            self.personal_dic_path_picker.path();
                    }
                }

                self.edited_preferences = true;
                self.preference_edit_requires_restart = true;
            }

            PreferencesMessage::Cancel => {
                state
                    .upstream_actions
                    .push(UpstreamAction::CloseWindow(WindowType::Preferences));
            }
            PreferencesMessage::Save => {
                self.save_preferences();

                if self.preference_edit_requires_restart {
                    state.upstream_actions.push(UpstreamAction::Autosave);

                    state
                        .upstream_actions
                        .push(UpstreamAction::RestartApplication);
                }
            }
            PreferencesMessage::SaveAndExit => {
                let save_task = self.update(state, PreferencesMessage::Save);

                state
                    .upstream_actions
                    .push(UpstreamAction::CloseWindow(WindowType::Preferences));

                return save_task;
            }
        }

        Task::none()
    }

    fn content_perform(&mut self, _state: &mut SharedAppState, action: ContentAction) {
        if let Some(active_content) = &self.active_content {
            match active_content {
                ActiveContent::AutosaveMinute => self.autosave_minute_content.perform(action),
                ActiveContent::AutosaveSecond => self.autosave_second_content.perform(action),
                ActiveContent::JournalPath => self
                    .journal_path_picker
                    .update(FilePickerMessage::FilepathEdit(action)),
                ActiveContent::PreferencesPath => self
                    .preferences_path_picker
                    .update(FilePickerMessage::FilepathEdit(action)),
                ActiveContent::SystemDicPath => self
                    .system_dic_path_picker
                    .update(FilePickerMessage::FilepathEdit(action)),
                ActiveContent::SystemAffPath => self
                    .system_aff_path_picker
                    .update(FilePickerMessage::FilepathEdit(action)),
                ActiveContent::PersonalDicPath => self
                    .personal_dic_path_picker
                    .update(FilePickerMessage::FilepathEdit(action)),
            }
        }
    }
}

impl Preferences {
    /// copies the current working preferences as stored in the preference editor into the actual preferences. since
    /// the working preferences are now up to date with the actual ones, the current state is now "no preferences have
    /// been changed"
    fn save_preferences(&mut self) {
        overwrite_preferences(self.working_preferences.clone());

        self.edited_preferences = false;
    }
}
