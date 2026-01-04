use crate::{
    SharedAppState, UpstreamAction,
    keyboard_manager::KeyboardAction,
    upgraded_content::{ContentAction, UpgradedContent},
    window_manager::{WindowType, Windowable},
};
use iced::{
    Task,
    widget::{self, Text, button, column, radio, row, text_editor::Action},
};
use rfd::FileDialog;
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

    FilepathEdit(Action),
    OpenFileDialog,
    SelectedStrategy(FileImportStrategy),
    Cancel,
    Import(FileImportStrategy),
}

#[derive(Debug, Default)]
pub struct FileImport {
    filepath_content: UpgradedContent,
    file_path: PathBuf,
    import_strategy: Option<FileImportStrategy>,
}

impl Windowable<FileImportMessage> for FileImport {
    fn title(&self) -> String {
        "Import File".to_string()
    }

    fn view<'a>(&'a self, _state: &SharedAppState) -> iced::Element<'a, FileImportMessage> {
        let filepath_text = widget::text_editor(self.filepath_content.raw_content())
            .on_action(FileImportMessage::FilepathEdit);

        let filepath_picker =
            widget::button("open file").on_press(FileImportMessage::OpenFileDialog);

        let filepath = row![filepath_text, filepath_picker];

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
            filepath,
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
            FileImportMessage::FilepathEdit(action) => {
                self.filepath_content
                    .perform(ContentAction::Standard(action));

                self.file_path = self.filepath_content.text().into();
            }
            FileImportMessage::OpenFileDialog => {
                let file_path = FileDialog::new()
                    .set_title("Import File")
                    .add_filter("Text", &["txt", "text", "md"])
                    .add_filter("All formats", &[""])
                    .pick_file();

                if let Some(path) = file_path {
                    self.file_path = path.clone();
                    self.filepath_content =
                        UpgradedContent::with_text(path.to_str().expect("path is not valid utf-8"));
                }
            }
            FileImportMessage::SelectedStrategy(strategy) => {
                self.import_strategy = Some(strategy);
            }
            FileImportMessage::Cancel => {
                state
                    .upstream_actions
                    .push(UpstreamAction::CloseWindow(WindowType::FileImport));
            }
            FileImportMessage::Import(strategy) => {
                if let Ok(imported_string) = fs::read_to_string(self.file_path.clone()) {
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
        self.filepath_content.perform(action);
    }
}
