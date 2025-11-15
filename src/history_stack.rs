use crate::content_tools::{self, decrement_cursor_position};
use iced::widget::text_editor::{self, Action, Content, Edit};
use std::collections::VecDeque;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct HistoryEvent {
    /// information about the selection, formatted as ((start_line, start_char), length)
    pub(crate) selection: Option<((usize, usize), usize)>,
    pub(crate) text_removed: Option<String>,
    pub(crate) text_added: Option<String>,
    pub(crate) cursor_line_idx: usize,
    pub(crate) cursor_char_idx: usize,
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
            if (history_event.text_removed.is_some() && history_event.selection.is_none())
                || content.selection().is_some()
            {
                content_tools::move_cursor(
                    content,
                    history_event.cursor_line_idx,
                    history_event.cursor_char_idx,
                );
            }

            let inverse_edits = Self::inverse_edit_action(&history_event);

            for edit in inverse_edits {
                content.perform(Action::Edit(edit));
            }

            if let Some(((line_start, char_start), length)) = history_event.selection {
                content_tools::select_text(content, line_start, char_start, length);
            }
        }
    }

    pub fn perform_redo(&mut self, content: &mut Content) {
        if let Some(history_event) = self.move_redo_to_undo_stack() {
            if let Some(removed) = &history_event.text_removed
                && history_event.selection.is_none()
            {
                content_tools::move_cursor(
                    content,
                    history_event.cursor_line_idx,
                    history_event.cursor_char_idx + removed.chars().count(),
                );
            }

            let redo_edits = Self::edit_action(history_event);

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
    fn edit_action(history_event: HistoryEvent) -> Vec<Edit> {
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

        if history_event.selection.is_none() {
            if let Some(deleted_text) = history_event.text_removed {
                for _i in deleted_text.chars() {
                    edit_sequence.push(Edit::Backspace);
                }
            }
        } else if history_event.selection.is_some() && history_event.text_added.is_none() {
            // remove the selection manually if no text was added (which automatically removes it via the insertion)
            edit_sequence.push(Edit::Backspace);
        }

        edit_sequence
    }

    /// takes a HistoryEvent and decomposes it into an equivelent set of Edit actions that can perform the inverse
    /// of the original event for implementing an undo
    fn inverse_edit_action(history_event: &HistoryEvent) -> Vec<Edit> {
        let mut inverse_sequence = vec![];

        if let Some(added_text) = &history_event.text_added {
            for _ in added_text.chars() {
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

/// converts an Edit action into a HistoryEvent based on the current state of the content
pub fn edit_action_to_history_event(
    content: &Content,
    edit: Edit,
    cursor_line_idx: usize,
    cursor_char_idx: usize,
) -> HistoryEvent {
    let mut history_event = HistoryEvent::default();

    let (cursor_line, cursor_char) = content.cursor_position();
    let content_text = content.text();

    if let Some(selection) = content.selection() {
        let (adjusted_cursor_line, adjusted_cursor_char) =
            content_tools::locate_cursor_start(content, cursor_line_idx, cursor_char_idx);

        let selection_bounds = (
            (adjusted_cursor_line, adjusted_cursor_char),
            selection.chars().count(),
        );

        match edit {
            text_editor::Edit::Insert(inserted_char) => {
                history_event = HistoryEvent {
                    selection: Some(selection_bounds),
                    text_removed: Some(selection),
                    text_added: Some(inserted_char.to_string()),
                    cursor_line_idx: adjusted_cursor_line,
                    cursor_char_idx: adjusted_cursor_char + 1, // cursor is moved one by the insert
                }
            }
            text_editor::Edit::Paste(pasted_text) => {
                let paste_text_string = pasted_text.to_string();
                let pasted_chars = paste_text_string.chars().count();

                history_event = HistoryEvent {
                    selection: Some(selection_bounds),
                    text_removed: Some(selection),
                    text_added: Some(pasted_text.to_string()),
                    cursor_line_idx: adjusted_cursor_line,
                    cursor_char_idx: adjusted_cursor_char + pasted_chars, // cursor moved by the number of chars in paste
                }
            }
            text_editor::Edit::Enter => {
                history_event = HistoryEvent {
                    selection: Some(selection_bounds),
                    text_removed: Some(selection),
                    text_added: Some("\n".to_string()),
                    cursor_line_idx: adjusted_cursor_line + 1, // cursor is moved by the enter
                    cursor_char_idx: 0,
                }
            }
            text_editor::Edit::Backspace => {
                history_event = HistoryEvent {
                    selection: Some(selection_bounds),
                    text_removed: Some(selection),
                    text_added: None,
                    cursor_line_idx: adjusted_cursor_line,
                    cursor_char_idx: adjusted_cursor_char,
                }
            }
            text_editor::Edit::Delete => {
                history_event = HistoryEvent {
                    selection: Some(selection_bounds),
                    text_removed: Some(selection),
                    text_added: None,
                    cursor_line_idx: adjusted_cursor_line,
                    cursor_char_idx: adjusted_cursor_char,
                }
            }
        }
    } else {
        match edit {
            text_editor::Edit::Insert(inserted_char) => {
                history_event = HistoryEvent {
                    selection: None,
                    text_removed: None,
                    text_added: Some(inserted_char.to_string()),
                    cursor_line_idx: cursor_line,
                    cursor_char_idx: cursor_char + 1, // cursor is moved one by the insert
                }
            }
            text_editor::Edit::Paste(pasted_text) => {
                let paste_text_string = pasted_text.to_string();
                let pasted_chars = paste_text_string.chars().count();

                history_event = HistoryEvent {
                    selection: None,
                    text_removed: None,
                    text_added: Some(pasted_text.to_string()),
                    cursor_line_idx: cursor_line,
                    cursor_char_idx: cursor_char + pasted_chars, // cursor moved by the number of chars in paste
                }
            }
            text_editor::Edit::Enter => {
                history_event = HistoryEvent {
                    selection: None,
                    text_removed: None,
                    text_added: Some("\n".to_string()),
                    cursor_line_idx: cursor_line + 1, // cursor is moved by the enter
                    cursor_char_idx: 0,
                }
            }
            text_editor::Edit::Backspace => {
                if let Some(line) = content_text.lines().nth(cursor_line) {
                    if cursor_line == 0 && cursor_char == 0 {
                        // don't log an event since nothing will happen on a backspace at the very start
                    } else if cursor_char > 0 {
                        let removed_char = line
                            .chars()
                            .nth(cursor_char - 1)
                            .expect("couldn't extract char");

                        history_event = HistoryEvent {
                            selection: None,
                            text_removed: Some(removed_char.to_string()),
                            text_added: None,
                            cursor_line_idx: cursor_line,
                            cursor_char_idx: cursor_char - 1,
                        }
                    } else {
                        let removed_char = '\n';

                        let (new_cursor_line, new_cursor_char) =
                            decrement_cursor_position(content, cursor_line, cursor_char);

                        history_event = HistoryEvent {
                            selection: None,
                            text_removed: Some(removed_char.to_string()),
                            text_added: None,
                            cursor_line_idx: new_cursor_line,
                            cursor_char_idx: new_cursor_char,
                        }
                    };
                } else {
                    // backspaced an empty newline at the very end of the text
                    let (new_cursor_line, new_cursor_char) =
                        decrement_cursor_position(content, cursor_line, cursor_char);

                    history_event = HistoryEvent {
                        selection: None,
                        text_removed: Some('\n'.to_string()),
                        text_added: None,
                        cursor_line_idx: new_cursor_line,
                        cursor_char_idx: new_cursor_char,
                    }
                }
            }
            text_editor::Edit::Delete => {
                let line_count = content_text.lines().count();

                let line = match content_text.lines().nth(cursor_line) {
                    Some(line) => line,
                    None => {
                        // this will happen on an attempt to delete at the end of an empty line
                        return HistoryEvent::default();
                    }
                };

                let char_count = line.chars().count();
                let char_to_remove = line.chars().nth(cursor_char);

                if line_count == (cursor_line + 1) && char_count == cursor_char {
                    // nothing to delete at the very end of the text
                    return HistoryEvent::default();
                } else if char_count == cursor_char {
                    // deleting a newline
                    history_event = HistoryEvent {
                        selection: None,
                        text_removed: Some('\n'.to_string()),
                        text_added: None,
                        cursor_line_idx: cursor_line,
                        cursor_char_idx: cursor_char,
                    }
                } else if let Some(removed_char) = char_to_remove {
                    // standard deletion
                    history_event = HistoryEvent {
                        selection: None,
                        text_removed: Some(removed_char.to_string()),
                        text_added: None,
                        cursor_line_idx: cursor_line,
                        cursor_char_idx: cursor_char,
                    }
                }
            }
        }
    }

    history_event
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
