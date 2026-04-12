use chrono::NaiveDate;
use iced::{Element, widget::column};

use super::template_tasks::{TemplateTaskMessage, TemplateTasks};
use super::{
    TaskId,
    event_tasks::{EventTaskMessage, EventTasks},
};

#[derive(Debug)]
/// Structure storing all the different types of tasks together
pub struct TaskManager {
    pub(crate) template_tasks: TemplateTasks,
    pub(crate) event_tasks: EventTasks,
}

impl Default for TaskManager {
    fn default() -> Self {
        TaskManager::load_all()
    }
}

#[derive(Debug, Clone)]
/// Messages the task manager operates on
pub enum TaskMessageAction {
    Template(TemplateTaskMessage),
    Event(EventTaskMessage),
}

#[derive(Debug, Clone)]
/// Tagged task messages
pub struct TaskMessage {
    message: TaskMessageAction,
    task_id: TaskId,
}

impl TaskMessage {
    /// Returns the task id out of the message
    pub fn get_id(&self) -> TaskId {
        self.task_id
    }
}

impl TaskManager {
    /// Updates tasks for the given date and message
    pub fn update(&mut self, active_date: NaiveDate, message: TaskMessage) {
        match message.message {
            TaskMessageAction::Template(template_task_message) => {
                self.template_tasks
                    .update(active_date, template_task_message);
            }
            TaskMessageAction::Event(event_task_message) => {
                self.event_tasks.update(event_task_message);
            }
        }
    }

    /// Constructs all tasks scheduled to be active on the given date
    pub fn build_tasks<'a>(&'a self, active_date: NaiveDate) -> Element<'a, TaskMessage> {
        let mut tasks = column![];

        let event_ids = self.event_tasks.get_active_event_ids(active_date);

        for id in event_ids {
            tasks = tasks.push(self.event_tasks.build_event(id).map(move |event_message| {
                TaskMessage {
                    message: TaskMessageAction::Event(event_message),
                    task_id: id,
                }
            }));
        }

        let template_ids = self.template_tasks.get_active_template_ids(active_date);

        for id in template_ids {
            tasks = tasks.push(self.template_tasks.build_template(id, active_date).map(
                move |template_message| TaskMessage {
                    message: TaskMessageAction::Template(template_message),
                    task_id: id,
                },
            ));
        }

        tasks.into()
    }

    /// Saves all tasks to disk
    pub fn save_all(&self) {
        self.template_tasks.save_templates();
        self.event_tasks.save_events();
    }

    /// Returns a TaskManager containing all tasks stored on disk
    pub fn load_all() -> Self {
        let template_tasks = TemplateTasks::load_templates();
        let event_tasks = EventTasks::load_events();

        Self {
            template_tasks,
            event_tasks,
        }
    }
}
