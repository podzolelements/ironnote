use crate::{
    SharedAppState,
    warning_dialog::{WarningDialog, WarningMessage},
    window_manager::{WINDOW_HEIGHT, WINDOW_WIDTH, Windowable},
};
use iced::{Element, Size, Task, window};
use std::collections::BTreeMap;

#[derive(Debug, Clone)]
/// types of dialogs that can be triggered. a dialog is a popup window that notifies the user something has happened
pub enum DialogType {
    Warning,
}

#[derive(Debug, Clone)]
/// types of messages that each respective dialog box can generate
pub enum DialogMessage {
    Warning(WarningMessage),
}

#[derive(Debug, Default)]
/// collection of all the dialogs. fixes the double matching problem (see match block in update() in
/// template_tasks.rs@b4a8012d434d0a62cd37436fb41fdd0b92227a0d) by enabling "iteration" over all dialog types, while
/// still allowing access to the individual types without a match
pub struct DialogManager {
    warnings: BTreeMap<window::Id, WarningDialog>,
}

impl DialogManager {
    const DIALOG_WINDOW_SIZE: Size<f32> = Size::new(WINDOW_WIDTH / 3.0, WINDOW_HEIGHT / 5.0);

    /// window settings for the dialog boxes
    pub fn dialog_window_settings() -> window::Settings {
        window::Settings {
            size: Self::DIALOG_WINDOW_SIZE,
            resizable: false,
            position: window::Position::Centered,

            ..Default::default()
        }
    }

    /// gets the title of the dialog window based on the given window Id
    pub fn get_title(&self, dialog_id: window::Id) -> Option<String> {
        self.warnings
            .get(&dialog_id)
            .map(|warning_dialog| warning_dialog.title())
    }

    /// gets the view of the dialog window based on the given Id
    pub fn get_view<'a>(
        &'a self,
        dialog_id: window::Id,
        state: &'a SharedAppState,
    ) -> Option<Element<'a, DialogMessage>> {
        self.warnings
            .get(&dialog_id)
            .map(|warning_dialog| warning_dialog.view(state).map(DialogMessage::Warning))
    }

    /// adds a dialog of the given type to the DialogManager, with the given text and Id
    pub fn insert_dialog(
        &mut self,
        window_id: window::Id,
        dialog_type: DialogType,
        dialog_text: String,
    ) {
        match dialog_type {
            DialogType::Warning => {
                self.warnings
                    .insert(window_id, WarningDialog::new(window_id, dialog_text));
            }
        }
    }

    /// removes the dialog of the given type and Id in the DialogManager
    pub fn remove_dialog(&mut self, window_id: window::Id, dialog_type: DialogType) {
        match dialog_type {
            DialogType::Warning => {
                self.warnings.remove(&window_id);
            }
        }
    }

    /// updates the given dialog Id with the given message
    pub fn update(
        &mut self,
        state: &mut SharedAppState,
        window_id: window::Id,
        dialog_message: DialogMessage,
    ) -> Task<DialogMessage> {
        match dialog_message {
            DialogMessage::Warning(warning_message) => {
                if let Some(warning_dialog) = self.warnings.get_mut(&window_id) {
                    warning_dialog
                        .update(state, warning_message)
                        .map(DialogMessage::Warning)
                } else {
                    Task::none()
                }
            }
        }
    }
}
