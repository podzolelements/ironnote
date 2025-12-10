use crate::filetools::template_tasks_path;
use chrono::{Datelike, NaiveDate, Weekday};
use iced::{
    Element,
    widget::{self, Space, Text, button, checkbox, column, row, text_editor::Content},
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs};
use strum::Display;

#[derive(Debug, Default)]
/// the standard task with a text box and a single checkbox
pub struct StandardData {
    text_content: Content,
    completed: bool,
}
#[derive(Debug, Serialize, Deserialize)]
/// StandardData task data as stored on disk
pub struct StandardDataDisk {
    text: String,
    completed: bool,
}

impl StandardDataDisk {
    /// converts a StandardDataDisk into the StandardData format
    fn from_disk(&self) -> StandardData {
        StandardData {
            text_content: Content::with_text(&self.text),
            completed: self.completed,
        }
    }
}

impl StandardData {
    /// set if the task was completed or not
    pub fn set_completion(&mut self, completed: bool) {
        self.completed = completed;
    }

    /// converts StandardData into the equivelent disk format
    fn to_disk(&self) -> StandardDataDisk {
        let mut text = self.text_content.text();
        text.pop();

        StandardDataDisk {
            text,
            completed: self.completed,
        }
    }
}

#[derive(Debug, Default)]
pub struct DualBinaryData {
    text_content: Content,
    name_first: String,
    name_second: String,
    completed_first: bool,
    completed_second: bool,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct DualBinaryDataDisk {
    text: String,
    name_first: String,
    name_second: String,
    completed_first: bool,
    completed_second: bool,
}

impl DualBinaryDataDisk {
    fn from_disk(&self) -> DualBinaryData {
        DualBinaryData {
            text_content: Content::with_text(&self.text),
            name_first: self.name_first.clone(),
            name_second: self.name_second.clone(),
            completed_first: self.completed_first,
            completed_second: self.completed_second,
        }
    }
}

impl DualBinaryData {
    pub fn new(first_name: String, second_name: String) -> Self {
        Self {
            text_content: Content::new(),
            name_first: first_name,
            name_second: second_name,
            completed_first: false,
            completed_second: false,
        }
    }

    pub fn set_completion(&mut self, first: bool, second: bool) {
        self.completed_first = first;
        self.completed_second = second;
    }

    fn to_disk(&self) -> DualBinaryDataDisk {
        let mut text = self.text_content.text();
        text.pop();

        DualBinaryDataDisk {
            text,
            name_first: self.name_first.clone(),
            name_second: self.name_second.clone(),
            completed_first: self.completed_first,
            completed_second: self.completed_second,
        }
    }
}

#[derive(Debug)]
/// types of different data formats used by the template tasks
pub enum TaskDataFormat {
    Standard(StandardData),
    DualBinary(DualBinaryData),
}

impl TaskDataFormat {
    fn to_disk(&self) -> TaskDataDiskFormat {
        match self {
            TaskDataFormat::Standard(standard_data) => {
                TaskDataDiskFormat::Standard(standard_data.to_disk())
            }
            TaskDataFormat::DualBinary(dual_binary_data) => {
                TaskDataDiskFormat::DualBinary(dual_binary_data.to_disk())
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
/// types of data formats stored on disk by the template tasks
pub enum TaskDataDiskFormat {
    Standard(StandardDataDisk),
    DualBinary(DualBinaryDataDisk),
}

impl TaskDataDiskFormat {
    fn from_disk(&self) -> TaskDataFormat {
        match self {
            TaskDataDiskFormat::Standard(standard_data_disk) => {
                TaskDataFormat::Standard(standard_data_disk.from_disk())
            }
            TaskDataDiskFormat::DualBinary(dual_binary_data_disk) => {
                TaskDataFormat::DualBinary(dual_binary_data_disk.from_disk())
            }
        }
    }
}

#[derive(Debug, Clone, Copy, Display, Serialize, Deserialize)]
/// these are the types of templates that can be created
pub enum TaskType {
    Standard,
    DualBinary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// the Frequency represents the schedule of how often the templates trigger
pub enum Frequency {
    Daily,
    Weekly([bool; 7]),
    Monthly([bool; 31]),
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

#[derive(Debug)]
/// the actual template that stores the repeated tasks. for the TemplateTask to operate properly, its TaskType MUST
/// align with its TaskDataFormat
pub struct TemplateTask {
    name: String,
    task_type: TaskType,
    creation_date: NaiveDate,
    ended_date: Option<NaiveDate>,
    frequency: Frequency,
    entries: HashMap<NaiveDate, TaskDataFormat>,
    expanded: bool,
}

#[derive(Debug, Serialize, Deserialize)]
/// disk format of the TemplateTask
pub struct TemplateTaskDisk {
    name: String,
    task_type: TaskType,
    creation_date: NaiveDate,
    ended_date: Option<NaiveDate>,
    frequency: Frequency,
    entries: Vec<(NaiveDate, TaskDataDiskFormat)>,
    expanded: bool,
}

impl TemplateTaskDisk {
    fn from_disk(&self) -> TemplateTask {
        let mut entries = HashMap::new();

        for (date, disk_data) in &self.entries {
            entries.insert(*date, disk_data.from_disk());
        }

        TemplateTask {
            name: self.name.clone(),
            task_type: self.task_type,
            creation_date: self.creation_date,
            ended_date: self.ended_date,
            frequency: self.frequency.clone(),
            entries,
            expanded: self.expanded,
        }
    }
}

impl TemplateTask {
    /// creates a new template task and automatically creates an entry if the creation date would have an entry
    pub fn new(
        name: String,
        task_type: TaskType,
        creation_date: NaiveDate,
        frequency: Frequency,
    ) -> Self {
        let mut new_task = Self {
            name,
            task_type,
            creation_date,
            ended_date: None,
            frequency,
            entries: HashMap::new(),
            expanded: false,
        };

        if new_task.is_active(creation_date) {
            new_task.add_empty_entry(task_type, creation_date);
        }

        new_task
    }

    /// returns if the template is scheduled for an entry on the given date
    pub fn is_active(&self, active_date: NaiveDate) -> bool {
        if active_date < self.creation_date {
            return false;
        }

        if let Some(ended_date) = self.ended_date
            && active_date > ended_date
        {
            return false;
        }

        self.frequency.is_active(active_date)
    }

    /// adds a default entry to the entries list. does not perform validation against the frequency of the template
    pub fn add_empty_entry(&mut self, entry_type: TaskType, entry_date: NaiveDate) {
        let empty_entry = match entry_type {
            TaskType::Standard => TaskDataFormat::Standard(StandardData::default()),
            TaskType::DualBinary => TaskDataFormat::DualBinary(DualBinaryData::default()),
        };

        self.entries.insert(entry_date, empty_entry);
    }

    /// returns the entry for template on the given date, returning None if entry is nonexistant
    pub fn get_entry(&self, entry_date: NaiveDate) -> Option<&TaskDataFormat> {
        self.entries.get(&entry_date)
    }

    /// returns mutable access to the entry on the given date, returning None if entry is nonexistant
    pub fn get_entry_mut(&mut self, entry_date: NaiveDate) -> Option<&mut TaskDataFormat> {
        self.entries.get_mut(&entry_date)
    }

    /// sets if the entry is expanded (true) or collapsed (false) when rendered
    pub fn set_expansion(&mut self, expanded: bool) {
        self.expanded = expanded;
    }

    fn to_disk(&self) -> TemplateTaskDisk {
        let mut entries = vec![];

        for (date, data) in &self.entries {
            entries.push((*date, data.to_disk()));
        }

        entries.sort_by_key(|(date, _disk)| *date);

        TemplateTaskDisk {
            name: self.name.clone(),
            task_type: self.task_type,
            creation_date: self.creation_date,
            ended_date: self.ended_date,
            frequency: self.frequency.clone(),
            entries,
            expanded: self.expanded,
        }
    }

    /// builds the template to an element for the given date. if the entry doesn't exist, a zero width space is returned
    pub fn build_template<'a, Message: 'a + Clone>(
        &'a self,
        entry_date: NaiveDate,
    ) -> Element<'a, Message> {
        let name = Text::new(self.name.clone());

        let expand_button_text = if self.expanded { "\\/" } else { "<" };

        let expand_button = button(Text::new(expand_button_text));

        if let Some(entry) = self.entries.get(&entry_date) {
            match entry {
                TaskDataFormat::Standard(standard_data) => {
                    let checkbox = checkbox("", standard_data.completed);

                    let minimized_task = row![name, expand_button, checkbox];

                    let text = widget::text_editor(&standard_data.text_content);

                    if !self.expanded {
                        minimized_task.into()
                    } else {
                        column![minimized_task, text].into()
                    }
                }
                TaskDataFormat::DualBinary(dual_binary_data) => {
                    let check_first = checkbox("", dual_binary_data.completed_first);
                    let check_second = checkbox("", dual_binary_data.completed_second);

                    let minimized_task = row![name, expand_button, check_first, check_second];

                    let text = widget::text_editor(&dual_binary_data.text_content);

                    if !self.expanded {
                        minimized_task.into()
                    } else {
                        column![minimized_task, text].into()
                    }
                }
            }
        } else {
            Space::new(0, 0).into()
        }
    }
}

#[derive(Debug, Default)]
/// collection of all the loaded templates
pub struct TemplateTasks {
    all_templates: Vec<TemplateTask>,
}

impl TemplateTasks {
    /// inserts a new template into the structure
    pub fn add_template(&mut self, new_template: TemplateTask) {
        self.all_templates.push(new_template);
    }

    /// returns a Vec of all the templates that are scheduled to be active on the given date
    pub fn get_active_templates(&self, active_date: NaiveDate) -> Vec<&TemplateTask> {
        self.all_templates
            .iter()
            .filter(|task| task.is_active(active_date))
            .collect()
    }

    /// returns a Vec of mutable templates that are scheduled to be active on the given date
    pub fn get_active_templates_mut(&mut self, active_date: NaiveDate) -> Vec<&mut TemplateTask> {
        self.all_templates
            .iter_mut()
            .filter(|task| task.is_active(active_date))
            .collect()
    }

    /// generate any missing entries for tasks scheduled on the given date
    pub fn generate_template_entries(&mut self, active_date: NaiveDate) {
        let active_templates = self.get_active_templates_mut(active_date);

        for template in active_templates {
            if template.get_entry(active_date).is_none() {
                template.add_empty_entry(template.task_type, active_date);
            }
        }
    }

    /// writes all templates to disk
    pub fn save_templates(&self) {
        for template in &self.all_templates {
            let template_disk = template.to_disk();

            let task_filename = "task_".to_string()
                + &template_disk.name.clone()
                + &template_disk.task_type.to_string()
                + ".json";

            let mut task_path = template_tasks_path();
            task_path.push(task_filename);

            let template_json = serde_json::to_string_pretty(&template_disk)
                .expect("couldn't serialize template_disk");

            fs::write(task_path, template_json).expect("couldn't save template json");
        }
    }

    /// loads all templates from disk
    pub fn load_templates(&mut self) {
        let mut template_paths = Vec::new();

        if let Ok(files) = fs::read_dir(template_tasks_path()) {
            for file in files.flatten() {
                template_paths.push(file.path());
            }
        }

        for path in template_paths {
            if let Ok(template_string) = fs::read_to_string(path)
                && let Ok(template_disk) =
                    serde_json::from_str::<TemplateTaskDisk>(&template_string)
            {
                self.add_template(template_disk.from_disk());
            }
        }
    }
}
