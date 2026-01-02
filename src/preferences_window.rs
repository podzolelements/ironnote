use crate::{
    SharedAppState,
    keyboard_manager::KeyboardAction,
    tabview::{TabviewItem, tabview_content_horizontal},
    upgraded_content::ContentAction,
    window_manager::Windowable,
};
use iced::{
    Length, Task,
    widget::{Text, column},
};
use strum::Display;

#[derive(Debug, Clone, PartialEq, Display)]
pub enum PreferencesTab {
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
pub enum PreferencesMessage {
    KeyEvent(KeyboardAction),

    TabSwitched(PreferencesTab),
}

#[derive(Debug)]
pub struct Preferences {
    current_preference_tab: PreferencesTab,
}

impl Default for Preferences {
    fn default() -> Self {
        Self {
            current_preference_tab: PreferencesTab::General,
        }
    }
}

impl Windowable<PreferencesMessage> for Preferences {
    fn title(&self) -> String {
        "Preferences".to_string()
    }

    fn view<'a>(&'a self, _state: &SharedAppState) -> iced::Element<'a, PreferencesMessage> {
        let general_tab_content = { column![Text::new("General Settings")] };

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
                self.current_preference_tab = new_preferences_tab;
            }
        }

        Task::none()
    }

    fn content_perform(&mut self, _state: &mut SharedAppState, _action: ContentAction) {}
}
