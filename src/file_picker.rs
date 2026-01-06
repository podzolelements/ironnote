use crate::upgraded_content::{ContentAction, UpgradedContent};
use iced::{
    Element,
    advanced::widget::Text,
    widget::{self, row, tooltip::Position},
};
use rfd::FileDialog;
use std::path::PathBuf;

#[derive(Debug, Clone)]
/// types of messages the FilePicker can produce
pub enum FilePickerMessage {
    FilepathEdit(ContentAction),
    OpenFileDialog,
}

#[derive(Debug)]
/// selects whether the widget picks a file or a directory
pub enum PickerType {
    File(Vec<(String, Vec<String>)>),
    Directory,
}

#[derive(Debug)]
/// the FilePicker is a custom widget with a text box to type out a path, that also has a button to open a file dialog
/// for selecting files. works on both directories and files through the PickerType selector
pub struct FilePicker {
    picker_type: PickerType,
    filepath_content: UpgradedContent,
    filepath: PathBuf,
}

impl<'a> FilePicker {
    /// creates a new FilePicker that picks out files.
    pub fn file(inital_path: PathBuf, extension_filters: &[(String, Vec<String>)]) -> Self {
        let inital_path_str = inital_path.to_str().expect("path is not valid utf-8");

        Self {
            picker_type: PickerType::File(extension_filters.to_vec()),
            filepath_content: UpgradedContent::with_text(inital_path_str),
            filepath: inital_path,
        }
    }

    /// creates a new FilePicker that picks out directories
    pub fn directory(inital_path: PathBuf) -> Self {
        let inital_path_str = inital_path.to_str().expect("path is not valid utf-8");

        Self {
            picker_type: PickerType::Directory,
            filepath_content: UpgradedContent::with_text(inital_path_str),
            filepath: inital_path,
        }
    }

    /// returns the current path in the FilePicker
    pub fn path(&self) -> PathBuf {
        self.filepath.clone()
    }

    /// builds the FilePicker for rendering. Note it returns a FilePickerMessage, which will need to be .map()ed
    /// to the upstream message type
    pub fn view(&'a self) -> Element<'a, FilePickerMessage> {
        let filepath_text = widget::text_editor(self.filepath_content.raw_content())
            .on_action(|action| FilePickerMessage::FilepathEdit(ContentAction::Standard(action)));

        let picker_button_content = match &self.picker_type {
            // TODO: icons
            PickerType::File(_extension_filters) => Text::new("open file"),
            PickerType::Directory => Text::new("open directory"),
        };

        let picker_button_hover_content = match &self.picker_type {
            PickerType::File(_extension_filters) => Text::new("Select a file"),
            PickerType::Directory => Text::new("Select a directory"),
        };

        let filepath_button =
            widget::button(picker_button_content).on_press(FilePickerMessage::OpenFileDialog);

        let filepath_tooltiped =
            widget::tooltip(filepath_button, picker_button_hover_content, Position::Top);

        let filepath = row![filepath_text, filepath_tooltiped];

        filepath.into()
    }

    /// updates the internal state of the FilePicker based on the given message
    pub fn update(&mut self, message: FilePickerMessage) {
        match message {
            FilePickerMessage::FilepathEdit(action) => {
                self.filepath_content.perform(action);
            }
            FilePickerMessage::OpenFileDialog => {
                let file_path = match &self.picker_type {
                    PickerType::File(extension_filters) => {
                        let mut file_dialog = FileDialog::new().set_title("Select File");

                        for (name, extensions) in extension_filters {
                            file_dialog = file_dialog.add_filter(name, extensions)
                        }

                        file_dialog = file_dialog.add_filter("All formats", &[""]);

                        file_dialog.pick_file()
                    }
                    PickerType::Directory => FileDialog::new()
                        .set_title("Select Directory")
                        .pick_folder(),
                };

                if let Some(path) = file_path {
                    self.filepath = path.clone();
                    self.filepath_content =
                        UpgradedContent::with_text(path.to_str().expect("path is not valid utf-8"));
                }
            }
        }
    }
}
