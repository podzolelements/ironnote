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
        let mut template_tasks = TemplateTasks::default();

        template_tasks.load_templates();

        Self { template_tasks }
    }
}

impl Tasks {
    /// constructs all tasks scheduled for the given date
    pub fn build_tasks<'a>(&'a self, active_date: NaiveDate) -> Element<'a, TemplateTaskMessage> {
        let mut tasks = column![];

        let templates = self.template_tasks.get_active_templates(active_date);

        for template in templates {
            tasks = tasks.push(template.build_template(active_date));
        }

        tasks.into()
    }

    /// saves all tasks to disk
    pub fn save_all(&self) {
        self.template_tasks.save_templates();
    }
}
