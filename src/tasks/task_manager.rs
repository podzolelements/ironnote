use super::template_tasks::{TemplateTaskMessage, TemplateTasks};

use chrono::NaiveDate;
use iced::{Element, widget::column};

#[derive(Debug)]
/// structure storing all the different types of tasks together
pub struct TaskManager {
    pub(crate) template_tasks: TemplateTasks,
}

impl Default for TaskManager {
    fn default() -> Self {
        TaskManager::load_all()
    }
}

impl TaskManager {
    /// constructs all tasks scheduled for the given date
    pub fn build_tasks<'a>(&'a self, active_date: NaiveDate) -> Element<'a, TemplateTaskMessage> {
        let mut tasks = column![];

        let template_ids = self.template_tasks.get_active_template_ids(active_date);

        for id in template_ids {
            tasks = tasks.push(self.template_tasks.build_template(id, active_date));
        }

        tasks.into()
    }

    /// saves all tasks to disk
    pub fn save_all(&self) {
        self.template_tasks.save_templates();
    }

    /// Returns a TaskManager containing all tasks stored on disk
    pub fn load_all() -> Self {
        let template_tasks = TemplateTasks::load_templates();

        Self { template_tasks }
    }
}
