use crate::{
    SharedAppState, UpstreamAction,
    month_day::{DispMonth, MonthDay},
    template_tasks::{
        Frequency, FrequencyType, MultiBinaryCommonData, TaskCommonDataFormat, TaskType,
        TemplateTask,
    },
    window_manager::{WindowType, Windowable},
};
use iced::{
    Alignment::Center,
    Task,
    advanced::widget::Text,
    widget::{
        self, Space, button, checkbox, column, hover, pick_list, radio, row, scrollable,
        text_editor::{Action, Content},
    },
};
use strum::VariantArray;

#[derive(Debug, Clone)]
pub enum TaskCreatorMessage {
    SelectedTask(TaskType),
    SelectedFrequency(FrequencyType),
    EditedName(Action),
    CheckedWeekday(usize, bool),
    CheckedMonth(usize, bool),
    IncreasedMultiBinCount,
    DecreasedMultiBinCount,
    EditedMultiBinName((usize, Action)),
    SelectedMonth(DispMonth),
    SelectedDay(u32),
    Cancel,
    CreateTask,
}

#[derive(Debug)]
pub struct TaskCreator {
    selected_task_type: TaskType,
    name_content: Content,
    selected_frequency: FrequencyType,
    freq_weekmap: [bool; 7],
    freq_monthmap: [bool; 31],
    freq_day: u32,
    freq_month: DispMonth,
    multi_binary_contents: Vec<Content>,
}

impl Default for TaskCreator {
    fn default() -> Self {
        Self {
            selected_task_type: TaskType::Standard,
            name_content: Content::default(),
            selected_frequency: FrequencyType::Daily,
            freq_weekmap: [false; 7],
            freq_monthmap: [false; 31],
            freq_day: 1,
            freq_month: DispMonth::January,
            multi_binary_contents: vec![Content::new(), Content::new()],
        }
    }
}

impl TaskCreator {
    /// returns true if all the information required to create a task is present and false if any information is missing
    pub fn is_valid_task(&self, state: &SharedAppState) -> bool {
        let mut name_text = self.name_content.text();
        name_text.pop();

        if name_text.is_empty() {
            return false;
        }

        match self.selected_frequency {
            FrequencyType::Daily => {}
            FrequencyType::Weekly => {
                let selected_day_count: u32 = self
                    .freq_weekmap
                    .iter()
                    .map(|selected| *selected as u32)
                    .sum();

                if selected_day_count == 0 {
                    return false;
                }
            }
            FrequencyType::Monthly => {
                let selected_day_count: u32 = self
                    .freq_monthmap
                    .iter()
                    .map(|selected| *selected as u32)
                    .sum();

                if selected_day_count == 0 {
                    return false;
                }
            }
            FrequencyType::Dated => {}
        }

        for template in state.all_tasks.template_tasks.get_all_templates() {
            let existing_name = template.get_name();
            let existing_type = template.get_type();

            if name_text == existing_name && self.selected_task_type == existing_type {
                return false;
            }
        }

        true
    }
}

impl Windowable<TaskCreatorMessage> for TaskCreator {
    fn title(&self) -> String {
        "Task Creator".to_string()
    }

    fn view<'a>(&'a self, state: &SharedAppState) -> iced::Element<'a, TaskCreatorMessage> {
        let intro_message = Text::new("Select a task type:");

        let radio_standard = radio(
            "Standard task",
            TaskType::Standard,
            (self.selected_task_type == TaskType::Standard).then_some(TaskType::Standard),
            TaskCreatorMessage::SelectedTask,
        );

        let radio_multi_binary = radio(
            "Task with any number of components",
            TaskType::MultiBinary,
            (self.selected_task_type == TaskType::MultiBinary).then_some(TaskType::MultiBinary),
            TaskCreatorMessage::SelectedTask,
        );

        let type_selection = column![intro_message, radio_standard, radio_multi_binary];

        let name_entry = widget::text_editor(&self.name_content)
            .placeholder("Enter task name...")
            .on_action(TaskCreatorMessage::EditedName);

        let type_config = {
            let task_specifc = match self.selected_task_type {
                TaskType::Standard => {
                    row![]
                }
                TaskType::MultiBinary => {
                    let mut subtasks = column![];

                    for (task_index, content) in self.multi_binary_contents.iter().enumerate() {
                        let index_text = Text::new(format!("Task {}:", task_index + 1));
                        let name_editor = widget::text_editor(content).on_action(move |action| {
                            TaskCreatorMessage::EditedMultiBinName((task_index, action))
                        });

                        let name_entry = row![index_text, name_editor];

                        subtasks = subtasks.push(name_entry);
                    }

                    let subtasks_scrollable = scrollable(subtasks).height(100);

                    let increase_button = button(Text::new("Add"))
                        .on_press(TaskCreatorMessage::IncreasedMultiBinCount);
                    let decrease_button = button(Text::new("Remove")).on_press_maybe(
                        (self.multi_binary_contents.len() > 1)
                            .then_some(TaskCreatorMessage::DecreasedMultiBinCount),
                    );

                    let inc_dec = column![decrease_button, increase_button];

                    row![subtasks_scrollable, inc_dec]
                }
            };

            column![task_specifc]
        };

        let frequency_select_message = Text::new("Select task frequency:");

        let radio_freq_daily = radio(
            "Daily",
            FrequencyType::Daily,
            (self.selected_frequency == FrequencyType::Daily).then_some(FrequencyType::Daily),
            TaskCreatorMessage::SelectedFrequency,
        );
        let freq_daily = if self.selected_frequency == FrequencyType::Daily {
            column![radio_freq_daily, Text::new("A task that happens every day")]
        } else {
            column![radio_freq_daily]
        };

        let radio_freq_weekly = radio(
            "Weekly",
            FrequencyType::Weekly,
            (self.selected_frequency == FrequencyType::Weekly).then_some(FrequencyType::Weekly),
            TaskCreatorMessage::SelectedFrequency,
        );
        let freq_weekly = if self.selected_frequency == FrequencyType::Weekly {
            let weekday_width = 2;
            let weekdays = row![
                Text::new("Sun"),
                Space::with_width(weekday_width),
                Text::new("Mon"),
                Space::with_width(weekday_width),
                Text::new("Tue"),
                Space::with_width(weekday_width),
                Text::new("Wed"),
                Space::with_width(weekday_width),
                Text::new("Thu"),
                Space::with_width(weekday_width),
                Text::new("Fri"),
                Space::with_width(weekday_width),
                Text::new("Sat"),
            ];

            let mut weekmap = row![];

            for week_index in 0..7 {
                weekmap = weekmap.push(checkbox("", self.freq_weekmap[week_index]).on_toggle(
                    move |checked| TaskCreatorMessage::CheckedWeekday(week_index, checked),
                ));

                weekmap = weekmap.push(Space::with_width(6));
            }

            let schedule = column![weekdays, weekmap];

            column![
                radio_freq_weekly,
                Text::new("A task that happens on a weekly basis, with a defined schedule:"),
                schedule,
            ]
        } else {
            column![radio_freq_weekly]
        };

        let radio_freq_monthly = radio(
            "Monthly",
            FrequencyType::Monthly,
            (self.selected_frequency == FrequencyType::Monthly).then_some(FrequencyType::Monthly),
            TaskCreatorMessage::SelectedFrequency,
        );
        let freq_monthly = if self.selected_frequency == FrequencyType::Monthly {
            let mut schedule = column![];

            let mut week = row![];

            let mut day_counter = 0;

            for month_index in 0..31 {
                let day_checkbox =
                    checkbox("", self.freq_monthmap[month_index]).on_toggle(move |checked| {
                        TaskCreatorMessage::CheckedMonth(month_index, checked)
                    });

                let checkbox_with_day = hover(
                    day_checkbox,
                    Text::new(month_index + 1).align_x(Center).align_y(Center),
                );

                week = week.push(checkbox_with_day);

                day_counter += 1;
                if day_counter == 7 {
                    day_counter = 0;

                    schedule = schedule.push(week);

                    week = row![];
                }
            }
            schedule = schedule.push(week);

            column![
                radio_freq_monthly,
                Text::new("A task that happens on a monthly basis, with a defined schedule:"),
                schedule,
            ]
        } else {
            column![radio_freq_monthly]
        };

        let radio_freq_dated = radio(
            "Fixed Date",
            FrequencyType::Dated,
            (self.selected_frequency == FrequencyType::Dated).then_some(FrequencyType::Dated),
            TaskCreatorMessage::SelectedFrequency,
        );
        let freq_dated = if self.selected_frequency == FrequencyType::Dated {
            let month_picklist = pick_list(
                DispMonth::VARIANTS,
                Some(self.freq_month),
                TaskCreatorMessage::SelectedMonth,
            );

            let max_days = self.freq_month.day_count();

            let days = (1..=max_days).collect::<Vec<u32>>();

            let day_picklist =
                pick_list(days, Some(self.freq_day), TaskCreatorMessage::SelectedDay);

            let month_day_select = row![month_picklist, day_picklist];

            column![
                radio_freq_dated,
                Text::new("A task that happens on a specific day of the year:"),
                month_day_select,
            ]
        } else {
            column![radio_freq_dated]
        };

        let frequency_config = column![
            frequency_select_message,
            freq_daily,
            freq_weekly,
            freq_monthly,
            freq_dated,
        ];

        let cancel_button = button(Text::new("Cancel")).on_press(TaskCreatorMessage::Cancel);

        let create_message = self
            .is_valid_task(state)
            .then_some(TaskCreatorMessage::CreateTask);

        let create_button = button(Text::new("Create Task")).on_press_maybe(create_message);

        let action_buttons = row![cancel_button, create_button];

        column![
            type_selection,
            name_entry,
            type_config,
            frequency_config,
            action_buttons
        ]
        .into()
    }

    fn update(
        &mut self,
        state: &mut SharedAppState,
        message: TaskCreatorMessage,
    ) -> iced::Task<TaskCreatorMessage> {
        match message {
            TaskCreatorMessage::SelectedTask(task_type) => {
                self.selected_task_type = task_type;
            }
            TaskCreatorMessage::SelectedFrequency(frequency) => {
                self.selected_frequency = frequency;
            }
            TaskCreatorMessage::EditedName(action) => {
                self.name_content.perform(action);
            }
            TaskCreatorMessage::CheckedWeekday(weekday_index, checked) => {
                self.freq_weekmap[weekday_index] = checked;
            }
            TaskCreatorMessage::CheckedMonth(month_index, checked) => {
                self.freq_monthmap[month_index] = checked;
            }
            TaskCreatorMessage::IncreasedMultiBinCount => {
                self.multi_binary_contents.push(Content::new());
            }
            TaskCreatorMessage::DecreasedMultiBinCount => {
                if self.multi_binary_contents.len() > 1 {
                    self.multi_binary_contents.pop();
                }
            }
            TaskCreatorMessage::EditedMultiBinName((index, action)) => {
                self.multi_binary_contents[index].perform(action);
            }

            TaskCreatorMessage::SelectedMonth(month) => {
                self.freq_month = month;

                if self.freq_day > self.freq_month.day_count() {
                    self.freq_day = 1;
                }
            }
            TaskCreatorMessage::SelectedDay(day) => {
                self.freq_day = day;
            }
            TaskCreatorMessage::Cancel => {
                state.upstream_action = Some(UpstreamAction::CloseWindow(WindowType::TaskCreator));
            }
            TaskCreatorMessage::CreateTask => {
                let mut name_text = self.name_content.text();
                name_text.pop();

                let active_date = state.global_store.date_time().date_naive();

                let (common_data, task_type) = match self.selected_task_type {
                    TaskType::Standard => (TaskCommonDataFormat::Standard, TaskType::Standard),
                    TaskType::MultiBinary => {
                        let subtask_names = self
                            .multi_binary_contents
                            .iter()
                            .map(|content| {
                                let mut name = content.text();
                                name.pop();

                                name
                            })
                            .collect();

                        let common_data = MultiBinaryCommonData::new(subtask_names);
                        (
                            TaskCommonDataFormat::MultiBinary(common_data),
                            TaskType::MultiBinary,
                        )
                    }
                };

                let frequency = match self.selected_frequency {
                    FrequencyType::Daily => Frequency::Daily,
                    FrequencyType::Weekly => Frequency::Weekly(self.freq_weekmap),
                    FrequencyType::Monthly => Frequency::Monthly(self.freq_monthmap),
                    FrequencyType::Dated => {
                        Frequency::Dated(MonthDay::new(self.freq_month, self.freq_day))
                    }
                };

                let template =
                    TemplateTask::new(name_text, task_type, common_data, active_date, frequency);

                state.all_tasks.template_tasks.add_template(template);
                state.all_tasks.save_all();

                *self = Self::default();
                state.upstream_action = Some(UpstreamAction::CloseWindow(WindowType::TaskCreator));
            }
        }

        Task::none()
    }
}
