use iced::widget::text_editor::{Action, Content, Cursor, Edit, Motion, Position};
use std::collections::VecDeque;

#[derive(Debug, Clone, PartialEq)]
/// contains all of the information of the changes to the content
pub struct HistoryEvent {
    pub(crate) text_removed: Option<String>,
    pub(crate) text_added: Option<String>,
    pub(crate) selected_char_count: usize,
    pub(crate) cursor: Cursor,
}

impl Default for HistoryEvent {
    fn default() -> Self {
        Self {
            text_removed: Default::default(),
            text_added: Default::default(),
            selected_char_count: 0,
            cursor: Cursor {
                position: Position { line: 0, column: 0 },
                selection: None,
            },
        }
    }
}

#[derive(Debug)]
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

    pub fn push_undo_action(&mut self, history_event: HistoryEvent) {
        self.stack_undo_action(history_event);
        self.redo_history.clear();
    }

    fn stack_undo_action(&mut self, history_event: HistoryEvent) {
        if history_event == HistoryEvent::default() {
            return;
        }

        self.undo_history.push_front(history_event);

        if let Some(max_undo_size) = self.max_undo_size {
            while self.undo_history.len() > max_undo_size {
                self.undo_history.pop_back();
            }
        }
    }
    fn stack_redo_action(&mut self, history_event: HistoryEvent) {
        self.redo_history.push_front(history_event);

        if let Some(max_redo_size) = self.max_redo_size {
            while self.redo_history.len() > max_redo_size {
                self.redo_history.pop_back();
            }
        }
    }

    fn move_undo_to_redo_stack(&mut self) -> Option<HistoryEvent> {
        if let Some(action_being_undone) = self.undo_history.pop_front() {
            self.stack_redo_action(action_being_undone.clone());
            Some(action_being_undone)
        } else {
            None
        }
    }
    fn move_redo_to_undo_stack(&mut self) -> Option<HistoryEvent> {
        if let Some(action_being_redone) = self.redo_history.pop_front() {
            self.stack_undo_action(action_being_redone.clone());
            Some(action_being_redone)
        } else {
            None
        }
    }

    pub fn perform_undo(&mut self, content: &mut Content) {
        if let Some(history_event) = self.move_undo_to_redo_stack() {
            if (history_event.text_removed.is_some() && history_event.cursor.selection.is_none())
                || content.selection().is_some()
            {
                content.move_to(history_event.cursor);
            }

            let inverse_edits = Self::history_event_to_inverse_edits(&history_event);

            for edit in inverse_edits {
                content.perform(Action::Edit(edit));
            }

            for _i in 0..history_event.selected_char_count {
                content.perform(Action::Select(Motion::Left));
            }
        }
    }

    pub fn perform_redo(&mut self, content: &mut Content) {
        if let Some(history_event) = self.move_redo_to_undo_stack() {
            if let Some(removed) = &history_event.text_removed
                && history_event.cursor.selection.is_none()
            {
                // content_tools::move_cursor(
                //     content,
                //     history_event.cursor_line_idx,
                //     history_event.cursor_char_idx + removed.chars().count(),
                // );
            }

            let redo_edits = Self::history_event_to_edits(history_event);

            for edit in redo_edits {
                content.perform(Action::Edit(edit));
            }
        }
    }

    /// performs an undo but does not move the action into the redo stack
    pub fn revert(&mut self, content: &mut Content) {
        self.perform_undo(content);
        self.redo_history.pop_front();
    }

    /// takes a HistoryEvent and decomposes it into an equivelent set of Edit actions that can reconstruct the original
    /// event for implementing a redo
    fn history_event_to_edits(history_event: HistoryEvent) -> Vec<Edit> {
        let mut edit_sequence = vec![];

        if let Some(added_text) = history_event.text_added.clone() {
            for chara in added_text.chars() {
                if chara == '\n' {
                    edit_sequence.push(Edit::Enter);
                } else {
                    edit_sequence.push(Edit::Insert(chara));
                }
            }
        }

        if history_event.cursor.selection.is_none() {
            if let Some(deleted_text) = history_event.text_removed {
                for _chara in deleted_text.chars() {
                    edit_sequence.push(Edit::Backspace);
                }
            }
        } else if history_event.cursor.selection.is_some() && history_event.text_added.is_none() {
            // remove the selection manually if no text was added (which automatically removes it via the insertion)
            edit_sequence.push(Edit::Backspace);
        }

        edit_sequence
    }

    /// takes a HistoryEvent and decomposes it into an equivelent set of Edit actions that can perform the inverse
    /// of the original event for implementing an undo
    fn history_event_to_inverse_edits(history_event: &HistoryEvent) -> Vec<Edit> {
        let mut inverse_sequence = vec![];

        if let Some(added_text) = &history_event.text_added {
            for _chara in added_text.chars() {
                inverse_sequence.push(Edit::Backspace);
            }
        }

        if let Some(deleted_text) = &history_event.text_removed {
            for chara in deleted_text.chars() {
                if chara == '\n' {
                    inverse_sequence.push(Edit::Enter);
                } else {
                    inverse_sequence.push(Edit::Insert(chara));
                }
            }
        }

        inverse_sequence
    }

    /// returns how many elements are in the undo stack
    pub fn undo_stack_height(&self) -> usize {
        self.undo_history.len()
    }

    /// returns how many elements are in the redo stack
    pub fn redo_stack_height(&self) -> usize {
        self.redo_history.len()
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
