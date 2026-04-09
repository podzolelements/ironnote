use crate::template_tasks::{TemplateTaskMessage, TemplateTasks};

use chrono::NaiveDate;
use iced::{Element, widget::column};

#[derive(Debug)]
/// structure storing all the different types of tasks together
pub struct Tasks {
    pub(crate) template_tasks: TemplateTasks,
}

impl Default for Tasks {
    fn default() -> Self {
        Tasks::load_all()
    }
}

impl Tasks {
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

    /// returns a Tasks containing all tasks stored on disk
    pub fn load_all() -> Self {
        let template_tasks = TemplateTasks::load_templates();

        Self { template_tasks }
    }
}
