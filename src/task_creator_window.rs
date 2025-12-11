use crate::{SharedAppState, template_tasks::TaskType, window_manager::Windowable};
use iced::{
    Task,
    advanced::widget::Text,
    widget::{self, column, radio, text_editor::Content},
};

#[derive(Debug, Clone)]
pub enum TaskCreatorMessage {
    SelectedTask(TaskType),
}

#[derive(Debug, Default)]
pub struct TaskCreator {
    selected_task_type: Option<TaskType>,
    name_content: Content,
}

impl Windowable<TaskCreatorMessage> for TaskCreator {
    fn title(&self) -> String {
        "Task Creator".to_string()
    }

    fn view<'a>(&'a self, _state: &SharedAppState) -> iced::Element<'a, TaskCreatorMessage> {
        let intro_message = Text::new("Select a task type:");

        let radio_standard = radio(
            "Standard task",
            TaskType::Standard,
            self.selected_task_type,
            TaskCreatorMessage::SelectedTask,
        );

        let radio_multi_binary = radio(
            "Task with any number of sub-tasks",
            TaskType::MultiBinary,
            self.selected_task_type,
            TaskCreatorMessage::SelectedTask,
        );

        let name_entry = widget::text_editor(&self.name_content).placeholder("Enter task name...");

        let config_menu = if let Some(task_type) = self.selected_task_type {
            let task_specifc = match task_type {
                TaskType::Standard => {
                    column![Text::new("standard task")]
                }
                TaskType::MultiBinary => {
                    column![Text::new("multi binary task")]
                }
            };

            column![task_specifc]
        } else {
            column![Text::new("Choose a task type to create a task")]
        };

        let selection = column![intro_message, radio_standard, radio_multi_binary];

        column![selection, name_entry, config_menu].into()
    }

    fn update(
        &mut self,
        _state: &mut SharedAppState,
        message: TaskCreatorMessage,
    ) -> iced::Task<TaskCreatorMessage> {
        match message {
            TaskCreatorMessage::SelectedTask(task_type) => {
                self.selected_task_type = Some(task_type);
            }
        }

        Task::none()
    }
}
