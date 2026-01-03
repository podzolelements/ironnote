use crate::{
    SharedAppState,
    keyboard_manager::KeyboardAction,
    tabview::{TabviewItem, tabview_content_horizontal},
    upgraded_content::{ContentAction, Restriction, UpgradedContent},
    user_preferences::{UserPreferences, preferences},
    window_manager::Windowable,
};
use iced::{
    Length, Task,
    widget::{self, Space, Text, checkbox, column, row, text_editor::Action},
};
use std::time::Duration;
use strum::Display;

#[derive(Debug, Default, Clone, PartialEq, Display)]
pub enum PreferencesTab {
    #[default]
    General,
    Keyboard,
}

impl PreferencesTab {
    pub fn to_index(&self) -> usize {
        match self {
            PreferencesTab::General => 0,
            PreferencesTab::Keyboard => 1,
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
pub enum PreferencesMessage {
    KeyEvent(KeyboardAction),

    TabSwitched(PreferencesTab),
    General(GeneralMessage),
}

#[derive(Debug)]
pub enum ActiveContent {
    AutosaveMinute,
    AutosaveSecond,
}

#[derive(Debug)]
pub struct Preferences {
    working_preferences: UserPreferences,

    current_preference_tab: PreferencesTab,
    active_content: Option<ActiveContent>,

    autosave_minute_content: UpgradedContent,
    autosave_minutes: u64,
    autosave_second_content: UpgradedContent,
    autosave_seconds: u64,
}

impl Default for Preferences {
    fn default() -> Self {
        Self {
            working_preferences: preferences().clone(),
            current_preference_tab: PreferencesTab::default(),
            active_content: None,
            autosave_minute_content: UpgradedContent::with_text("5"),
            autosave_minutes: 5,
            autosave_second_content: UpgradedContent::with_text("0"),
            autosave_seconds: 0,
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

        let keyboard_tab_content = { column![Text::new("Keyboard Settings")] };

        let keyboard_tab = TabviewItem {
            title: PreferencesTab::Keyboard.to_string(),
            clicked_message: PreferencesMessage::TabSwitched(PreferencesTab::Keyboard),
            content: keyboard_tab_content.into(),
        };

        let tab_elements = vec![general_tab, keyboard_tab];

        tabview_content_horizontal(
            tab_elements,
            self.current_preference_tab.to_index(),
            Length::Fill,
            Length::Fill,
        )
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
                KeyboardAction::Save => todo!(),
                KeyboardAction::Debug => {}
                KeyboardAction::Unbound(_unbound_key) => {}
            },
            PreferencesMessage::TabSwitched(new_preferences_tab) => {
                self.active_content = None;

                self.current_preference_tab = new_preferences_tab;
            }
            PreferencesMessage::General(general_message) => match general_message {
                GeneralMessage::ToggleAutosave(is_checked) => {
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

                    self.working_preferences.general.autosave_interval =
                        Duration::from_mins(self.autosave_minutes)
                            + Duration::from_secs(self.autosave_seconds);
                }
            },
        }

        Task::none()
    }

    fn content_perform(&mut self, _state: &mut SharedAppState, action: ContentAction) {
        if let Some(active_content) = &self.active_content {
            match active_content {
                ActiveContent::AutosaveMinute => self.autosave_minute_content.perform(action),
                ActiveContent::AutosaveSecond => self.autosave_second_content.perform(action),
            }
        }
    }
}
