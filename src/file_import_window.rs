use crate::window_manager::Windowable;
use iced::{
    Task,
    widget::{
        self, Text, column, radio, row,
        text_editor::{Action, Content},
    },
};
use rfd::FileDialog;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileImportStrategy {
    AppendEnd,
    AppendStart,
    Overwrite,
}

#[derive(Debug, Clone)]
pub enum FileImportMessage {
    FilepathEdit(Action),
    OpenFileDialog,
    SelectedStrategy(FileImportStrategy),
}

#[derive(Debug, Default)]
pub struct FileImport {
    title: String,
    filepath_content: Content,
    import_strategy: Option<FileImportStrategy>,
}

impl Windowable<FileImportMessage> for FileImport {
    fn title(&self) -> String {
        self.title.clone()
    }

    fn view<'a>(&'a self) -> iced::Element<'a, FileImportMessage> {
        let filepath_text =
            widget::text_editor(&self.filepath_content).on_action(FileImportMessage::FilepathEdit);

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

        column![
            Text::new("Import File"),
            filepath,
            radio_append_end,
            radio_append_start,
            radio_overwrite
        ]
        .into()
    }

    fn update(&mut self, message: FileImportMessage) -> iced::Task<FileImportMessage> {
        match message {
            FileImportMessage::FilepathEdit(action) => {
                self.filepath_content.perform(action);
            }
            FileImportMessage::OpenFileDialog => {
                let file_path = FileDialog::new()
                    .set_title("Import File")
                    .add_filter("Text", &["txt", "text", "md"])
                    .add_filter("All formats", &[""])
                    .pick_file();

                if let Some(path) = file_path {
                    self.filepath_content = Content::with_text(path.to_str().unwrap());
                }
            }
            FileImportMessage::SelectedStrategy(strategy) => {
                self.import_strategy = Some(strategy);
            }
        }

        Task::none()
    }
}
