use crate::{
    SharedAppState, UpstreamAction, dialog_manager::DialogType, upgraded_content::ContentAction,
    window_manager::Windowable,
};
use iced::{
    Element, Task,
    widget::{Text, button, column},
    window,
};

#[derive(Debug, Clone)]
/// types of warning messages
pub enum WarningMessage {
    Ok,
}

#[derive(Debug, Clone, PartialEq)]
/// structure representing a dialog of the warning severity. a warning is used to notify the user something has gone
/// wrong, but the program is able to continue operating
pub struct WarningDialog {
    /// text of what happened that is displayed to the user
    warning_text: String,

    /// window Id of the dialog box
    window_id: window::Id,
}

impl WarningDialog {
    /// creates a new WarningDialog structure with the given Id and a description of what went wrong
    pub fn new(window_id: window::Id, warning_text: String) -> Self {
        Self {
            warning_text,
            window_id,
        }
    }
}

impl Windowable<WarningMessage> for WarningDialog {
    fn title(&self) -> String {
        "Warning".to_string()
    }

    fn view<'a>(&'a self, _state: &'a SharedAppState) -> Element<'a, WarningMessage> {
        let warning_message = Text::new(&self.warning_text);

        let ok_button = button("Ok").on_press(WarningMessage::Ok);

        column![warning_message, ok_button].into()
    }

    fn update(
        &mut self,
        state: &mut SharedAppState,
        message: WarningMessage,
    ) -> Task<WarningMessage> {
        match message {
            WarningMessage::Ok => {
                state.upstream_actions.push(UpstreamAction::CloseDialog(
                    self.window_id,
                    DialogType::Warning,
                ));
            }
        }

        Task::none()
    }

    fn content_perform(&mut self, _state: &mut SharedAppState, _action: ContentAction) {}
}
