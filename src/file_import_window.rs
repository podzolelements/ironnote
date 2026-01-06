use crate::{
    SharedAppState, UpstreamAction,
    file_extensions::{TEXT_EXT_LIST, build_extensions},
    file_picker::{FilePicker, FilePickerMessage},
    keyboard_manager::KeyboardAction,
    upgraded_content::{ContentAction, UpgradedContent},
    window_manager::{WindowType, Windowable},
};
use iced::{
    Task,
    widget::{Text, button, column, radio, row},
};
use std::{fs, path::PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileImportStrategy {
    AppendEnd,
    AppendStart,
    Overwrite,
}

#[derive(Debug, Clone)]
pub enum FileImportMessage {
    KeyEvent(KeyboardAction),

    FilePicker(FilePickerMessage),
    SelectedStrategy(FileImportStrategy),
    Cancel,
    Import(FileImportStrategy),
}

#[derive(Debug)]
pub struct FileImport {
    filepicker: FilePicker,
    filepicker_content_is_active: bool,
    import_strategy: Option<FileImportStrategy>,
}

impl Default for FileImport {
    fn default() -> Self {
        Self {
            filepicker: FilePicker::file(PathBuf::new(), &build_extensions(TEXT_EXT_LIST)),
            filepicker_content_is_active: false,
            import_strategy: None,
        }
    }
}

impl Windowable<FileImportMessage> for FileImport {
    fn title(&self) -> String {
        "Import File".to_string()
    }

    fn view<'a>(&'a self, _state: &SharedAppState) -> iced::Element<'a, FileImportMessage> {
        let filepicker = self.filepicker.view().map(FileImportMessage::FilePicker);

        let radio_append_end = radio(
            "Append to end of current day",
            FileImportStrategy::AppendEnd,
            self.import_strategy,
            FileImportMessage::SelectedStrategy,
        );

        let radio_append_start = radio(
            "Append to beginning of current day",
            FileImportStrategy::AppendStart,
            self.import_strategy,
            FileImportMessage::SelectedStrategy,
        );

        let radio_overwrite = radio(
            "Overwrite contents of current day",
            FileImportStrategy::Overwrite,
            self.import_strategy,
            FileImportMessage::SelectedStrategy,
        );

        let cancel_button = button(Text::new("Cancel")).on_press(FileImportMessage::Cancel);

        let import_message = self.import_strategy.map(FileImportMessage::Import);
        let import_button = button(Text::new("Import")).on_press_maybe(import_message);

        let bottom_buttons = row![cancel_button, import_button];

        column![
            Text::new("Import File"),
            filepicker,
            radio_append_end,
            radio_append_start,
            radio_overwrite,
            bottom_buttons,
        ]
        .into()
    }

    fn update(
        &mut self,
        state: &mut SharedAppState,
        message: FileImportMessage,
    ) -> iced::Task<FileImportMessage> {
        match message {
            FileImportMessage::KeyEvent(keyboard_action) => {
                match keyboard_action {
                    KeyboardAction::Content(text_edit) => {
                        self.content_perform(state, text_edit.to_content_action());
                    }
                    KeyboardAction::Save => {}
                    KeyboardAction::Debug => {}
                    KeyboardAction::Unbound(_unbound_key) => {}
                };
            }
            FileImportMessage::FilePicker(message) => {
                self.filepicker_content_is_active =
                    matches!(&message, FilePickerMessage::FilepathEdit(_content_action));

                self.filepicker.update(message);
            }
            FileImportMessage::SelectedStrategy(strategy) => {
                self.filepicker_content_is_active = false;

                self.import_strategy = Some(strategy);
            }
            FileImportMessage::Cancel => {
                self.filepicker_content_is_active = false;

                state
                    .upstream_actions
                    .push(UpstreamAction::CloseWindow(WindowType::FileImport));
            }
            FileImportMessage::Import(strategy) => {
                self.filepicker_content_is_active = false;

                if let Ok(imported_string) = fs::read_to_string(self.filepicker.path()) {
                    let new_text = match strategy {
                        FileImportStrategy::AppendEnd => state.content.text() + &imported_string,
                        FileImportStrategy::AppendStart => imported_string + &state.content.text(),
                        FileImportStrategy::Overwrite => imported_string,
                    };

                    state.content = UpgradedContent::with_text(&new_text);
                } else {
                    //TODO: unable to import
                }

                state
                    .upstream_actions
                    .push(UpstreamAction::CloseWindow(WindowType::FileImport));
            }
        }

        Task::none()
    }

    fn content_perform(&mut self, _state: &mut SharedAppState, action: ContentAction) {
        if self.filepicker_content_is_active {
            self.filepicker
                .update(FilePickerMessage::FilepathEdit(action));
        }
    }
}
