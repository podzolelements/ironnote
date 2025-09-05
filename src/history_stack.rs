use std::collections::VecDeque;

use iced::widget::text_editor::{Action, Content, Edit};

#[derive(Debug, Clone, Default)]
pub struct HistoryEvent {
    pub(crate) text_removed: Option<String>,
    pub(crate) text_added: Option<String>,
}

pub struct HistoryStack {
    undo_history: VecDeque<HistoryEvent>,
    max_undo_size: Option<usize>,
    redo_history: VecDeque<HistoryEvent>,
    max_redo_size: Option<usize>,
}

impl HistoryStack {
    pub fn clear(&mut self) {
        self.undo_history.clear();
        self.redo_history.clear();
    }

    pub fn clear_redo_stack(&mut self) {
        self.redo_history.clear();
    }

    pub fn push_undo_action(&mut self, history_event: HistoryEvent) {
        if history_event.text_added.is_none() && history_event.text_removed.is_none() {
            return;
        }

        self.undo_history.push_front(history_event);

        if let Some(max_undo_size) = self.max_undo_size {
            while self.undo_history.len() > max_undo_size {
                self.undo_history.pop_back();
            }
        }
    }
    pub fn push_redo_action(&mut self, history_event: HistoryEvent) {
        self.redo_history.push_front(history_event);

        if let Some(max_redo_size) = self.max_redo_size {
            while self.redo_history.len() > max_redo_size {
                self.redo_history.pop_back();
            }
        }
    }

    fn move_undo_to_redo_stack(&mut self) -> Option<HistoryEvent> {
        if let Some(action_being_undone) = self.undo_history.pop_front() {
            self.push_redo_action(action_being_undone.clone());
            Some(action_being_undone)
        } else {
            None
        }
    }
    fn move_redo_to_undo_stack(&mut self) -> Option<HistoryEvent> {
        if let Some(action_being_redone) = self.redo_history.pop_front() {
            self.push_undo_action(action_being_redone.clone());
            Some(action_being_redone)
        } else {
            None
        }
    }

    pub fn perform_undo(&mut self, content: &mut Content) {
        if let Some(history_event) = self.move_undo_to_redo_stack() {
            let inverse_edits = Self::inverse_edit_action(history_event);

            for edit in inverse_edits {
                content.perform(Action::Edit(edit));
            }
        }
    }

    pub fn perform_redo(&mut self, content: &mut Content) {
        if let Some(history_event) = self.move_redo_to_undo_stack() {
            let redo_edits = Self::edit_action(history_event);

            for edit in redo_edits {
                content.perform(Action::Edit(edit));
            }
        }
    }

    /// takes a HistoryEvent and decomposes it into an equivelent set of Edit actions that can reconstruct the original
    /// event for implementing a redo
    fn edit_action(history_event: HistoryEvent) -> Vec<Edit> {
        let mut edit_sequence = vec![];

        if let Some(added_text) = history_event.text_added {
            for chara in added_text.chars() {
                edit_sequence.push(Edit::Insert(chara));
            }
        }

        if let Some(deleted_text) = history_event.text_removed {
            for _ in deleted_text.chars() {
                edit_sequence.push(Edit::Backspace);
            }
        }

        edit_sequence
    }

    /// takes a HistoryEvent and decomposes it into an equivelent set of Edit actions that can perform the inverse
    /// of the original event for implementing an undo
    fn inverse_edit_action(history_event: HistoryEvent) -> Vec<Edit> {
        let mut inverse_sequence = vec![];

        if let Some(added_text) = history_event.text_added {
            for _ in added_text.chars() {
                inverse_sequence.push(Edit::Backspace);
            }
        }

        if let Some(deleted_text) = history_event.text_removed {
            for chara in deleted_text.chars() {
                inverse_sequence.push(Edit::Insert(chara));
            }
        }

        inverse_sequence
    }
}

impl Default for HistoryStack {
    fn default() -> Self {
        Self {
            undo_history: Default::default(),
            max_undo_size: Some(1000),
            redo_history: Default::default(),
            max_redo_size: Some(1000),
        }
    }
}
