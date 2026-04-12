use chrono::{Datelike, NaiveDate, Weekday};
use iced::{Element, widget::column};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, fs};

use super::{
    MultiBinaryMessage, MultiBinaryTaskElement, StandardMessage, StandardTask, TaskId, TaskType,
    task_data::MultiBinaryTask,
};
use crate::{config::preferences, custom_widgets, utils::month_day::MonthDay};

#[derive(Debug, Default, Serialize, Deserialize)]
/// Contains all the individual task entries in a standard task
pub struct StandardTaskTemplate {
    elements: BTreeMap<NaiveDate, StandardTask>,
}

impl StandardTaskTemplate {
    /// Adds an empty element with the given date to the task elements if it does not exist
    pub fn add_empty_element(&mut self, active_date: NaiveDate) {
        self.elements.entry(active_date).or_default();
    }

    /// Returns mutable access to a element at the given date, if it exists
    pub fn get_element_mut(&mut self, active_date: NaiveDate) -> Option<&mut StandardTask> {
        self.elements.get_mut(&active_date)
    }
}

#[derive(Debug, Serialize, Deserialize)]
/// Contains the common and individual task entries in a MultiBinary task
pub struct MultiBinaryTaskTemplate {
    subtasks: Vec<String>,
    elements: BTreeMap<NaiveDate, MultiBinaryTaskElement>,
}

impl MultiBinaryTaskTemplate {
    /// Creates a new MultiBinaryTaskTemplate with the given set of subtasks
    pub fn new(subtask_names: Vec<String>) -> Self {
        Self {
            subtasks: subtask_names,
            elements: BTreeMap::new(),
        }
    }

    /// Adds an empty element with the given date to the task elements if it does not exist
    pub fn add_empty_element(&mut self, active_date: NaiveDate) {
        let empty_element = MultiBinaryTaskElement::with_empty_subtasks(self.subtasks.len());

        self.elements.entry(active_date).or_insert(empty_element);
    }

    /// Returns mutable access to the task if it exists
    pub fn get_element_mut(
        &mut self,
        active_date: NaiveDate,
    ) -> Option<&mut MultiBinaryTaskElement> {
        self.elements.get_mut(&active_date)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// the Frequency represents the schedule of how often the templates trigger
pub enum Frequency {
    Daily,
    Weekly([bool; 7]),
    Monthly([bool; 31]),
    Dated(MonthDay),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// a Frequency without any information about the schedule
pub enum FrequencyType {
    Daily,
    Weekly,
    Monthly,
    Dated,
}

impl Frequency {
    /// returns if the frequency would be scheduled to be active on the given date
    pub fn is_active(&self, active_date: NaiveDate) -> bool {
        match self {
            Frequency::Daily => {
                return true;
            }
            Frequency::Weekly(daymap) => {
                let current_day = active_date.weekday();

                if Self::weekly_is(current_day, daymap, Weekday::Sun) {
                    return true;
                }
            }
            Frequency::Monthly(daymap) => {
                let day_of_month = active_date.day0() as usize;

                if daymap[day_of_month] {
                    return true;
                }
            }
            Frequency::Dated(month_day) => {
                if active_date.month() == month_day.month().number_from_month()
                    && active_date.day() == month_day.day()
                {
                    return true;
                }
            }
        }

        false
    }

    /// checks if the given weekday would be active based on the daymap and the weekday defined as daymap[0]
    fn weekly_is(current_day: Weekday, daymap: &[bool; 7], week_start_day: Weekday) -> bool {
        let mut day_index = 0;

        let mut iterative_day = week_start_day;

        while iterative_day != current_day {
            iterative_day = iterative_day.succ();
            day_index += 1;
        }

        daymap[day_index]
    }
}

#[derive(Debug, Clone, PartialEq)]
/// types of messages that all TemplateTasks are able to create
pub enum CommonMessage {
    ExpandToggled,
    ExpandOptions,
    EndTask,
    DeleteTemplate,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum TemplateData {
    Standard(StandardTaskTemplate),
    MultiBinary(MultiBinaryTaskTemplate),
}

impl TemplateData {
    /// Conversion to TaskTypes
    pub fn task_type(&self) -> TaskType {
        match self {
            TemplateData::Standard(_) => TaskType::Standard,
            TemplateData::MultiBinary(_) => TaskType::MultiBinary,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
/// Data that is used by all TemplateTasks
pub struct TemplateTask {
    name: String,
    creation_date: NaiveDate,
    ended_date: Option<NaiveDate>,
    frequency: Frequency,
    #[serde(skip)]
    expanded: bool,
    #[serde(skip)]
    options_expanded: bool,
    template_data: TemplateData,
}

impl TemplateTask {
    /// Creates new TemplateTask instance with the given values
    pub fn new(
        name: String,
        creation_date: NaiveDate,
        frequency: Frequency,
        template_data: TemplateData,
    ) -> Self {
        Self {
            name,
            creation_date,
            ended_date: None,
            frequency,
            expanded: false,
            options_expanded: false,
            template_data,
        }
    }

    /// Returns mutable access to the template data
    pub fn get_template_mut(&mut self) -> &mut TemplateData {
        &mut self.template_data
    }

    /// If the TemplateTask does not have an entry for the given day and it should based on its Frequency, a blank
    /// element is inserted into the elements
    pub fn generate_template_entry(&mut self, active_date: NaiveDate) {
        if self.frequency.is_active(active_date) {
            match &mut self.template_data {
                TemplateData::Standard(standard_task) => {
                    standard_task.add_empty_element(active_date);
                }
                TemplateData::MultiBinary(multi_binary_task) => {
                    multi_binary_task.add_empty_element(active_date);
                }
            }
        }
    }

    /// Constructs the template ui element at the given date, if it exists
    pub fn built_template<'a>(&'a self, active_date: NaiveDate) -> Element<'a, TemplateMessage> {
        let checkbox = match &self.template_data {
            TemplateData::Standard(standard_task_template) => {
                if let Some(standard_task) = standard_task_template.elements.get(&active_date) {
                    Some((
                        standard_task.is_completed(),
                        TemplateMessage::Standard(StandardMessage::ToggledCheckbox),
                    ))
                } else {
                    None
                }
            }
            TemplateData::MultiBinary(multi_binary_task_template) => {
                if let Some(multi_binary_task) =
                    multi_binary_task_template.elements.get(&active_date)
                {
                    Some((
                        multi_binary_task.is_completed(),
                        TemplateMessage::MultiBinary(MultiBinaryMessage::ToggledOverride),
                    ))
                } else {
                    None
                }
            }
        };

        let expanded_ui = match &self.template_data {
            TemplateData::Standard(standard_task) => {
                if let Some(task_element) = standard_task.elements.get(&active_date) {
                    task_element
                        .expanded_ui()
                        .map(|standard_message| TemplateMessage::Standard(standard_message))
                } else {
                    column![].into()
                }
            }
            TemplateData::MultiBinary(multi_binary_task) => {
                if let Some(task_element) = multi_binary_task.elements.get(&active_date) {
                    MultiBinaryTask::expanded_ui(task_element, &multi_binary_task.subtasks).map(
                        |multi_binary_message| TemplateMessage::MultiBinary(multi_binary_message),
                    )
                } else {
                    column![].into()
                }
            }
        };

        let expanded = if self.expanded {
            Some((
                Some(expanded_ui),
                TemplateMessage::Common(CommonMessage::ExpandToggled),
            ))
        } else {
            Some((None, TemplateMessage::Common(CommonMessage::ExpandToggled)))
        };

        let end_task_text = if self.ended_date.is_none() {
            "End Task".to_string()
        } else {
            "Resume Task".to_string()
        };

        let menu_items = vec![
            (
                end_task_text,
                TemplateMessage::Common(CommonMessage::EndTask),
            ),
            (
                "Delete Task".to_string(),
                TemplateMessage::Common(CommonMessage::DeleteTemplate),
            ),
        ];

        let options_menu = if self.options_expanded {
            Some(menu_items)
        } else {
            None
        };

        custom_widgets::task::build_task(
            checkbox,
            self.name.clone(),
            expanded,
            TemplateMessage::Common(CommonMessage::ExpandOptions),
            options_menu,
        )
    }
}

#[derive(Debug, Clone)]
/// Messages that TemplateTasks can generate, represents some action to perform
pub enum TemplateMessage {
    Common(CommonMessage),
    Standard(StandardMessage),
    MultiBinary(MultiBinaryMessage),
}

#[derive(Debug, Clone)]
/// The actual TemplateTask message. The TaskId is which TemplateTask the message should be performed on
pub struct TemplateTaskMessage {
    pub(crate) message: TemplateMessage,
    pub(crate) task_id: TaskId,
}

#[derive(Debug, Default, Serialize, Deserialize)]
/// Collection of all the loaded templates
pub struct TemplateTasks {
    tasks: BTreeMap<TaskId, TemplateTask>,
}

impl<'a> TemplateTasks {
    /// Writes all template tasks to disk, into the template_tasks directory defined in the preferences
    pub fn save_templates(&self) {
        let mut template_path = preferences().paths.template_tasks_dir();
        template_path.push("templates.json");

        let template_json =
            serde_json::to_string_pretty(self).expect("couldn't serialize disk templates");

        fs::write(template_path, template_json).expect("couldn't save template json");
    }

    /// Loads all template tasks from disk, from the template_tasks directory defined in the preferences. If the
    /// templates file cannot be read, it returns an empty TemplateTasks
    pub fn load_templates() -> Self {
        let mut template_path = preferences().paths.template_tasks_dir();
        template_path.push("templates.json");

        if let Ok(template_string) = fs::read_to_string(template_path)
            && let Ok(template_disk) = serde_json::from_str::<TemplateTasks>(&template_string)
        {
            if let Some(max_id) = template_disk.tasks.keys().max() {
                TaskId::set_if_greater(max_id.as_u32() + 1);
            }

            template_disk
        } else {
            Self::default()
        }
    }

    /// Inserts a new template task into the structure
    pub fn create_task(&mut self, mut template: TemplateTask) {
        let task_id = TaskId::new_unique_id();

        let task_date = template.creation_date;

        template.generate_template_entry(task_date);

        self.tasks.insert(task_id, template);
    }

    /// Generate any missing entries for tasks scheduled on the given date
    pub fn generate_template_entries(&mut self, active_date: NaiveDate) {
        let active_templates = self.get_active_template_ids(active_date);

        for task_id in active_templates {
            if let Some(task) = self.tasks.get_mut(&task_id) {
                task.generate_template_entry(active_date);
            }
        }
    }

    /// Returns the task at the given TaskId, if it exists
    pub fn get_task(&self, task_id: TaskId) -> Option<&TemplateTask> {
        self.tasks.get(&task_id)
    }

    /// Returns mutable access the task at the given TaskId, if it exists
    pub fn get_task_mut(&mut self, task_id: TaskId) -> Option<&mut TemplateTask> {
        self.tasks.get_mut(&task_id)
    }

    /// Returns true if given name and task type are present in the same TemplateTask
    pub fn task_exists(&self, task_name: &str, task_type: TaskType) -> bool {
        for task in self.tasks.values() {
            if task.name == task_name && task.template_data.task_type() == task_type {
                return true;
            }
        }

        false
    }

    /// Performs the action specified by the message on the TemplateTask on the given date
    pub fn update(&mut self, active_date: NaiveDate, message: TemplateTaskMessage) {
        match message.message {
            TemplateMessage::Common(common_message) => {
                if let Some(template) = self.tasks.get_mut(&message.task_id) {
                    match common_message {
                        CommonMessage::ExpandToggled => {
                            template.expanded = !template.expanded;
                        }
                        CommonMessage::ExpandOptions => {
                            template.options_expanded = !template.options_expanded
                        }
                        CommonMessage::EndTask => {
                            if template.ended_date.is_none() {
                                template.ended_date = Some(active_date);
                            } else {
                                template.ended_date = None;
                            }

                            template.options_expanded = false;
                        }
                        CommonMessage::DeleteTemplate => {
                            self.tasks.remove(&message.task_id);
                        }
                    }
                }
            }
            TemplateMessage::Standard(standard_message) => {
                if let Some(task) = self.tasks.get_mut(&message.task_id)
                    && let TemplateData::Standard(standard_task) = &mut task.template_data
                    && let Some(task_element) = standard_task.elements.get_mut(&active_date)
                {
                    task_element.update(standard_message);
                }
            }
            TemplateMessage::MultiBinary(multi_binary_message) => {
                if let Some(task) = self.tasks.get_mut(&message.task_id)
                    && let TemplateData::MultiBinary(multi_binary_task) = &mut task.template_data
                    && let Some(task_element) = multi_binary_task.elements.get_mut(&active_date)
                {
                    task_element.update(multi_binary_message);
                }
            }
        }
    }

    /// Returns a list of all the templates that are active on a given date
    pub fn get_active_template_ids(&self, active_date: NaiveDate) -> Vec<TaskId> {
        self.tasks
            .iter()
            .filter(|(_id, data)| {
                let after_end_date = data
                    .ended_date
                    .is_some_and(|ended_date| active_date > ended_date);

                data.frequency.is_active(active_date)
                    && active_date >= data.creation_date
                    && !after_end_date
            })
            .map(|(id, _data)| *id)
            .collect::<Vec<TaskId>>()
    }

    /// Full TemplateTask graphical element
    pub fn build_template(
        &'a self,
        task_id: TaskId,
        active_date: NaiveDate,
    ) -> Element<'a, TemplateTaskMessage> {
        if let Some(task_data) = self.get_task(task_id) {
            task_data
                .built_template(active_date)
                .map(move |template_message| TemplateTaskMessage {
                    message: template_message,
                    task_id,
                })
        } else {
            column![].into()
        }
    }
}
