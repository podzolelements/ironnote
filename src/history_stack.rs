use iced::widget::text_editor::{Action, Content, Cursor, Edit, Motion, Position};
use std::collections::VecDeque;

#[derive(Debug, Clone, PartialEq)]
/// contains information about what and how text gets removed
pub struct TextRemoval {
    text: String,
    /// if the removal is caused by a delete, the characters get removed on the right of the cursor, which is
    /// needs to be known for reconstructing events in the HistoryStack
    removed_right_of_cursor: bool,
}

impl TextRemoval {
    /// creates a new TextRemoval with the text that was removed, and if it was caused by a delete action
    pub fn new(text: String, is_delete_removal: bool) -> Self {
        Self {
            text,
            removed_right_of_cursor: is_delete_removal,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
/// contains all of the information of the changes to the content
pub struct HistoryEvent {
    pub(crate) text_removed: Option<TextRemoval>,
    pub(crate) text_added: Option<String>,
    pub(crate) selection_char_count: usize,
    /// what the cursor should be set to when moving along the redo stack
    pub(crate) redo_cursor: Cursor,
    /// what the cursor should be set to when moving along the undo stack; the final cursor state after the event has
    /// been completed
    pub(crate) undo_cursor: Cursor,
}

impl Default for HistoryEvent {
    fn default() -> Self {
        Self {
            text_removed: Default::default(),
            text_added: Default::default(),
            selection_char_count: 0,
            redo_cursor: Cursor {
                position: Position { line: 0, column: 0 },
                selection: None,
            },
            undo_cursor: Cursor {
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
    /// removes all contents from both the undo and redo stacks
    pub fn clear(&mut self) {
        self.undo_history.clear();
        self.redo_history.clear();
    }

    /// adds a new event onto the undo stack. the redo stack gets cleared when doing this, since the redo actions are
    /// no longer valid when a new edit is added to the undo stack
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

    /// undoes the last HistoryEvent on the undo stack, applying its effects to the provided content and moving the event
    /// into the redo stack
    pub fn perform_undo(&mut self, content: &mut Content) {
        if let Some(history_event) = self.move_undo_to_redo_stack() {
            if content.selection().is_some() {
                // this clears any existing selection since move_to() doesn't work right when there is one
                content.perform(Action::Move(Motion::DocumentStart));
            }
            content.move_to(history_event.undo_cursor);

            let inverse_actions = Self::history_event_to_inverse_actions(&history_event);

            for action in inverse_actions {
                content.perform(action);
            }

            for _i in 0..history_event.selection_char_count {
                content.perform(Action::Select(Motion::Left));
            }
        }
    }

    /// redoes the last HistoryEvent on the redo stack, applying its effects to the provided content and moving the event
    /// back onto the undo stack
    pub fn perform_redo(&mut self, content: &mut Content) {
        if let Some(history_event) = self.move_redo_to_undo_stack() {
            content.move_to(history_event.redo_cursor);

            let redo_actions = Self::history_event_to_actions(history_event);

            for action in redo_actions {
                content.perform(action);
            }
        }
    }

    /// performs an undo but does not move the action into the redo stack
    pub fn revert(&mut self, content: &mut Content) {
        self.perform_undo(content);
        self.redo_history.pop_front();
    }

    /// takes a HistoryEvent and decomposes it into an equivalent set of Actions that can reconstruct the original
    /// event for implementing a redo
    fn history_event_to_actions(history_event: HistoryEvent) -> Vec<Action> {
        let mut edit_sequence = vec![];

        if let Some(added_text) = history_event.text_added.clone() {
            for chara in added_text.chars() {
                if chara == '\n' {
                    edit_sequence.push(Action::Edit(Edit::Enter));
                } else {
                    edit_sequence.push(Action::Edit(Edit::Insert(chara)));
                }
            }
        }

        if history_event.redo_cursor.selection.is_none() {
            if let Some(removed_text) = history_event.text_removed {
                for _chara in removed_text.text.chars() {
                    if removed_text.removed_right_of_cursor {
                        edit_sequence.push(Action::Edit(Edit::Delete));
                    } else {
                        edit_sequence.push(Action::Edit(Edit::Backspace));
                    }
                }
            }
        } else if history_event.redo_cursor.selection.is_some()
            && history_event.text_added.is_none()
        {
            // remove the selection manually if no text was added
            edit_sequence.push(Action::Edit(Edit::Backspace));
        }

        edit_sequence
    }

    /// takes a HistoryEvent and decomposes it into an equivalent set of Actions that can perform the inverse
    /// of the original event for implementing an undo
    fn history_event_to_inverse_actions(history_event: &HistoryEvent) -> Vec<Action> {
        let mut inverse_sequence = vec![];

        if let Some(added_text) = &history_event.text_added {
            for _chara in added_text.chars() {
                inverse_sequence.push(Action::Edit(Edit::Backspace));
            }
        }

        if let Some(removed_text) = &history_event.text_removed {
            for chara in removed_text.text.chars() {
                if chara == '\n' {
                    inverse_sequence.push(Action::Edit(Edit::Enter));
                } else {
                    inverse_sequence.push(Action::Edit(Edit::Insert(chara)));
                }
            }

            // since the cursor moves right when performing Inserts, if it was a delete we need to move the cursor back
            // to where it should be
            if removed_text.removed_right_of_cursor {
                for _chara in removed_text.text.chars() {
                    inverse_sequence.push(Action::Move(Motion::Left));
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
