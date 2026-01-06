use crate::{
    SharedAppState, UpstreamAction,
    file_extensions::{TEXT_EXT_LIST, build_extensions},
    file_picker::{FilePicker, FilePickerMessage},
    keyboard_manager::KeyboardAction,
    upgraded_content::ContentAction,
    window_manager::{WindowType, Windowable},
};
use chrono::{Datelike, Days};
use iced::{
    Task,
    widget::{Text, button, column, radio, row},
};
use std::{fs, path::PathBuf};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum FileExportStrategy {
    #[default]
    SingleDay,
    AllSingle,
}

#[derive(Debug, Clone)]
pub enum FileExportMessage {
    KeyEvent(KeyboardAction),

    FilePicker(FilePickerMessage),
    SelectedStrategy(FileExportStrategy),
    Cancel,
    Export,
}

#[derive(Debug)]
pub struct FileExport {
    individial_file_picker: FilePicker,
    bulk_directory_picker: FilePicker,
    filepicker_content_is_active: bool,
    export_strategy: FileExportStrategy,
}

impl Default for FileExport {
    fn default() -> Self {
        Self {
            individial_file_picker: FilePicker::file(
                PathBuf::new(),
                &build_extensions(TEXT_EXT_LIST),
            ),
            bulk_directory_picker: FilePicker::directory(PathBuf::new()),
            filepicker_content_is_active: false,
            export_strategy: Default::default(),
        }
    }
}

impl Windowable<FileExportMessage> for FileExport {
    fn title(&self) -> String {
        "Export File".to_string()
    }

    fn view<'a>(&'a self, _state: &SharedAppState) -> iced::Element<'a, FileExportMessage> {
        let radio_single_day = radio(
            "Export current day",
            FileExportStrategy::SingleDay,
            (self.export_strategy == FileExportStrategy::SingleDay)
                .then_some(FileExportStrategy::SingleDay),
            FileExportMessage::SelectedStrategy,
        );

        let radio_all_single = radio(
            "Export all days individually as plaintext",
            FileExportStrategy::AllSingle,
            (self.export_strategy == FileExportStrategy::AllSingle)
                .then_some(FileExportStrategy::AllSingle),
            FileExportMessage::SelectedStrategy,
        );

        let file_picker = match self.export_strategy {
            FileExportStrategy::SingleDay => self
                .individial_file_picker
                .view()
                .map(FileExportMessage::FilePicker),
            FileExportStrategy::AllSingle => self
                .bulk_directory_picker
                .view()
                .map(FileExportMessage::FilePicker),
        };

        let cancel_button = button(Text::new("Cancel")).on_press(FileExportMessage::Cancel);

        let export_button = button(Text::new("Export")).on_press(FileExportMessage::Export);

        let bottom_buttons = row![cancel_button, export_button];

        column![
            Text::new("Export File"),
            radio_single_day,
            radio_all_single,
            file_picker,
            bottom_buttons
        ]
        .into()
    }

    fn update(
        &mut self,
        state: &mut SharedAppState,
        message: FileExportMessage,
    ) -> iced::Task<FileExportMessage> {
        match message {
            FileExportMessage::KeyEvent(keyboard_action) => {
                match keyboard_action {
                    KeyboardAction::Content(text_edit) => {
                        self.content_perform(state, text_edit.to_content_action());
                    }
                    KeyboardAction::Save => {}
                    KeyboardAction::Debug => {}
                    KeyboardAction::Unbound(_unbound_key) => {}
                };
            }
            FileExportMessage::FilePicker(message) => {
                self.filepicker_content_is_active =
                    matches!(&message, FilePickerMessage::FilepathEdit(_content_action));

                match self.export_strategy {
                    FileExportStrategy::SingleDay => self.individial_file_picker.update(message),
                    FileExportStrategy::AllSingle => self.bulk_directory_picker.update(message),
                }
            }
            FileExportMessage::SelectedStrategy(strategy) => {
                self.filepicker_content_is_active = false;

                self.export_strategy = strategy;
            }
            FileExportMessage::Cancel => {
                self.filepicker_content_is_active = false;

                state
                    .upstream_actions
                    .push(UpstreamAction::CloseWindow(WindowType::FileExport));
            }
            FileExportMessage::Export => {
                self.filepicker_content_is_active = false;

                match self.export_strategy {
                    FileExportStrategy::SingleDay => {
                        let day_text = state.global_store.day().get_day_text();

                        if let Err(_error) = fs::write(self.individial_file_picker.path(), day_text)
                        {
                        }
                    }
                    FileExportStrategy::AllSingle => {
                        if let Some(first_edited_day) = state.global_store.first_edited_day()
                            && let Some(last_edited_day) = state.global_store.last_edited_day()
                        {
                            let mut iterative_day = first_edited_day;

                            while iterative_day <= last_edited_day {
                                if let Some(day_store) = state.global_store.get_day(iterative_day)
                                    && day_store.contains_entry()
                                {
                                    let year = iterative_day.year().to_string();
                                    let filename = iterative_day.date_naive().to_string();

                                    let mut root_path = self.bulk_directory_picker.path();
                                    root_path.push(year);
                                    if let Err(_error) = fs::create_dir_all(&root_path) {}

                                    root_path.push(filename);

                                    let day_text = day_store.get_day_text();

                                    if let Err(_error) = fs::write(root_path, day_text) {}
                                }

                                iterative_day = iterative_day
                                    .checked_add_days(Days::new(1))
                                    .expect("couldn't add day");
                            }
                        }
                    }
                }
            }
        }

        Task::none()
    }

    fn content_perform(&mut self, _state: &mut SharedAppState, action: ContentAction) {
        if self.filepicker_content_is_active {
            match self.export_strategy {
                FileExportStrategy::SingleDay => self
                    .individial_file_picker
                    .update(FilePickerMessage::FilepathEdit(action)),
                FileExportStrategy::AllSingle => self
                    .bulk_directory_picker
                    .update(FilePickerMessage::FilepathEdit(action)),
            }
        }
    }
}
