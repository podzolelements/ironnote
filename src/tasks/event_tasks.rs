use chrono::{Days, NaiveDate};
use iced::{Element, widget::column};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, fs};

use super::{MultiBinaryMessage, MultiBinaryTask, StandardMessage, TaskData, TaskId};
use crate::{
    config::preferences,
    custom_widgets::{
        self,
        context_menu::{ContextMenuElement, ContextMenuItem, build_context_menu},
    },
};

#[derive(Debug, Serialize, Deserialize)]
/// How task-based events are carried into future days if they are not completed
pub enum PastDate {
    Ignore,
    RetainDays(u32),
    RetainIndefinite,
}

#[derive(Debug, Serialize, Deserialize)]
/// An event that operates like a task. It has a checkbox scheme, and if not checked off, the PastDate determines if it
/// is brought forward to the next day or discarded
pub struct TaskEventTask {
    task: TaskData,
    #[serde(skip)]
    expanded: bool,
    end_behavior: PastDate,
}

#[derive(Debug, Clone)]
pub enum EventTaskAction {
    PressMenu,
    DeleteEvent,

    ToggleDown,
    StandardTask(StandardMessage),
    MultiBinaryTask(MultiBinaryMessage),
}

#[derive(Debug, Clone)]
pub struct EventTaskMessage {
    pub(crate) message: EventTaskAction,
    task_id: TaskId,
}

#[derive(Debug, Serialize, Deserialize)]
/// An EventTask is an event that happens once and does not repeat.
pub struct EventTask {
    name: String,
    date: NaiveDate,
    length_days: Option<u32>,
    preview_days: Option<u32>,
    // #[serde(skip)]
    // options_expanded: bool,
    task_data: Option<TaskEventTask>,
}

impl EventTask {
    /// Returns if the event should show up on the given date
    pub fn is_active(&self, active_date: NaiveDate) -> bool {
        if active_date < self.date {
            if let Some(preview_days) = self.preview_days
                && self.date.checked_sub_days(Days::new(preview_days as u64)) <= Some(active_date)
            {
                return true;
            } else {
                return false;
            }
        }

        if active_date == self.date {
            return true;
        }

        if let Some(extended_days) = self.length_days
            && self.date.checked_add_days(Days::new(extended_days as u64)) > Some(active_date)
        {
            return true;
        }

        if let Some(task) = &self.task_data {
            match task.end_behavior {
                PastDate::Ignore => return false,
                PastDate::RetainDays(retained_days) => {
                    let days_past_end = if let Some(extended_days) = self.length_days {
                        extended_days + retained_days
                    } else {
                        retained_days
                    };

                    return self.date.checked_add_days(Days::new(days_past_end as u64))
                        >= Some(active_date);
                }
                PastDate::RetainIndefinite => return true,
            }
        }

        false
    }

    /// Constructs the event task ui element
    pub fn build_event<'a>(&'a self, options_expanded: bool) -> Element<'a, EventTaskAction> {
        let checkbox = if let Some(task) = &self.task_data {
            match &task.task {
                TaskData::Standard(standard_task) => Some((
                    standard_task.is_completed(),
                    EventTaskAction::StandardTask(StandardMessage::ToggledCheckbox),
                )),
                TaskData::MultiBinary(multi_binary_task) => Some((
                    multi_binary_task.element.is_completed(),
                    EventTaskAction::MultiBinaryTask(MultiBinaryMessage::ToggledOverride),
                )),
            }
        } else {
            None
        };

        let expanded = if let Some(task) = &self.task_data {
            let expanded_ui = match &task.task {
                TaskData::Standard(standard_task) => standard_task
                    .expanded_ui()
                    .map(EventTaskAction::StandardTask),
                TaskData::MultiBinary(multi_binary_task) => MultiBinaryTask::expanded_ui(
                    &multi_binary_task.element,
                    &multi_binary_task.subtasks,
                )
                .map(EventTaskAction::MultiBinaryTask),
            };

            if task.expanded {
                Some((Some(expanded_ui), EventTaskAction::ToggleDown))
            } else {
                Some((None, EventTaskAction::ToggleDown))
            }
        } else {
            None
        };

        let event_menu_items = vec![ContextMenuItem::Button(ContextMenuElement::new(
            "Delete Event",
            Some(EventTaskAction::DeleteEvent),
        ))];

        let event_menu = build_context_menu(event_menu_items);

        let menu = if options_expanded {
            Some(event_menu)
        } else {
            None
        };

        custom_widgets::task::build_task(
            checkbox,
            self.name.clone(),
            expanded,
            EventTaskAction::PressMenu,
            menu,
        )
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
/// Collection of the event tasks
pub struct EventTasks {
    events: BTreeMap<TaskId, EventTask>,
}

impl EventTasks {
    /// Loads events from disk, returning an empty EventTasks if the load fails
    pub fn load_events() -> Self {
        let mut event_path = preferences().paths.event_tasks_dir();
        event_path.push("events.json");

        if let Ok(event_string) = fs::read_to_string(event_path)
            && let Ok(event_disk) = serde_json::from_str::<EventTasks>(&event_string)
        {
            if let Some(max_id) = event_disk.events.keys().max() {
                TaskId::set_if_greater(max_id.as_u32() + 1);
            }

            event_disk
        } else {
            Self::default()
        }
    }

    /// Write all events to disk
    pub fn save_events(&self) {
        let mut template_path = preferences().paths.event_tasks_dir();
        template_path.push("events.json");

        let template_json =
            serde_json::to_string_pretty(self).expect("couldn't serialize event tasks");

        fs::write(template_path, template_json).expect("couldn't save event task json");
    }

    /// Updates interal state of the EventTasks based on the message
    pub fn update(&mut self, message: EventTaskMessage) {
        if let Some(event_task) = self.events.get_mut(&message.task_id) {
            match message.message {
                EventTaskAction::PressMenu => {}
                EventTaskAction::DeleteEvent => {
                    self.events.remove(&message.task_id);
                }
                EventTaskAction::StandardTask(standard_task_message) => {
                    if let Some(once_task) = &mut event_task.task_data
                        && let TaskData::Standard(standard_task) = &mut once_task.task
                    {
                        standard_task.update(standard_task_message);
                    }
                }
                EventTaskAction::MultiBinaryTask(multi_binary_task_message) => {
                    if let Some(once_task) = &mut event_task.task_data
                        && let TaskData::MultiBinary(multi_binary_task) = &mut once_task.task
                    {
                        multi_binary_task.element.update(multi_binary_task_message);
                    }
                }
                EventTaskAction::ToggleDown => {
                    if let Some(once_task) = &mut event_task.task_data {
                        once_task.expanded = !once_task.expanded;
                    }
                }
            }
        }
    }

    /// Constructs the task ui element for the given task id, if it exists
    pub fn build_event<'a>(
        &'a self,
        task_id: TaskId,
        options_expanded: bool,
    ) -> Element<'a, EventTaskMessage> {
        if let Some(event_task) = self.events.get(&task_id) {
            event_task
                .build_event(options_expanded)
                .map(move |event_message| EventTaskMessage {
                    message: event_message,
                    task_id,
                })
        } else {
            column![].into()
        }
    }

    /// Returns the events that are active on the given date
    pub fn get_active_event_ids(&self, active_date: NaiveDate) -> Vec<TaskId> {
        self.events
            .iter()
            .filter(|(_id, event)| event.is_active(active_date))
            .map(|(&id, _event)| id)
            .collect()
    }
}
