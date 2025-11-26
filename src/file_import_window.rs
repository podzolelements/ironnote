use crate::window_manager::Windowable;
use iced::widget::{Text, column};

#[derive(Debug, Clone)]
pub enum FileImportMessage {}

#[derive(Debug, Default)]
pub struct FileImport {
    title: String,
}

impl Windowable<FileImportMessage> for FileImport {
    fn title(&self) -> String {
        self.title.clone()
    }

    fn view<'a>(&'a self) -> iced::Element<'a, FileImportMessage> {
        column![Text::new("Import File")].into()
    }

    fn update(&mut self, message: FileImportMessage) -> iced::Task<FileImportMessage> {
        match message {}
    }
}
