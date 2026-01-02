use crate::{
    SharedAppState, UpstreamAction,
    keyboard_manager::KeyboardAction,
    upgraded_content::{ContentAction, UpgradedContent},
    window_manager::{WindowType, Windowable},
};
use chrono::{Datelike, Days};
use iced::{
    Task,
    widget::{self, Text, button, column, radio, row, text_editor::Action},
};
use rfd::FileDialog;
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

    FilepathEdit(Action),
    OpenFileDialog,
    SelectedStrategy(FileExportStrategy),
    Cancel,
    Export,
}

#[derive(Debug, Default)]
pub struct FileExport {
    filepath_content: UpgradedContent,
    file_path: PathBuf,
    export_strategy: FileExportStrategy,
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

        let filepath_text = widget::text_editor(self.filepath_content.raw_content())
            .on_action(FileExportMessage::FilepathEdit);

        let filepath_picker = widget::button("open").on_press(FileExportMessage::OpenFileDialog);

        let filepath = row![filepath_text, filepath_picker];

        let cancel_button = button(Text::new("Cancel")).on_press(FileExportMessage::Cancel);

        let export_button = button(Text::new("Export")).on_press(FileExportMessage::Export);

        let bottom_buttons = row![cancel_button, export_button];

        column![
            Text::new("Export File"),
            radio_single_day,
            radio_all_single,
            filepath,
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
            FileExportMessage::FilepathEdit(action) => {
                self.filepath_content
                    .perform(ContentAction::Standard(action));

                self.file_path = self.filepath_content.text().into();
            }
            FileExportMessage::OpenFileDialog => {
                let file_path = match self.export_strategy {
                    FileExportStrategy::SingleDay => FileDialog::new()
                        .set_title("Export File")
                        .add_filter("Text", &["txt", "text", "md"])
                        .add_filter("All formats", &[""])
                        .save_file(),
                    FileExportStrategy::AllSingle => FileDialog::new()
                        .set_title("Export All to Directory")
                        .pick_folder(),
                };

                if let Some(path) = file_path {
                    self.file_path = path.clone();
                    self.filepath_content =
                        UpgradedContent::with_text(path.to_str().expect("path is not valid utf-8"));
                }
            }
            FileExportMessage::SelectedStrategy(strategy) => {
                self.export_strategy = strategy;
            }
            FileExportMessage::Cancel => {
                state.upstream_action = Some(UpstreamAction::CloseWindow(WindowType::FileExport));
            }
            FileExportMessage::Export => match self.export_strategy {
                FileExportStrategy::SingleDay => {
                    let day_text = state.global_store.day().get_day_text();

                    if let Err(_error) = fs::write(self.file_path.clone(), day_text) {}
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

                                let mut root_path = self.file_path.clone();
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
            },
        }

        Task::none()
    }

    fn content_perform(&mut self, _state: &mut SharedAppState, action: ContentAction) {
        self.filepath_content.perform(action);
    }
}
