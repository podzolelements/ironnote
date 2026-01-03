use crate::history_stack::{HistoryEvent, HistoryStack, TextRemoval};
use crate::misc_tools;
use iced::widget::text_editor::{self, Action, Content, Cursor, Edit, Position};

#[derive(Debug, Clone, PartialEq)]
/// edits that bulk delete several characters at once
pub enum CtrlEdit {
    BackspaceWord,
    BackspaceSentence,
    DeleteWord,
    DeleteSentence,
}

impl CtrlEdit {
    /// the characters that cause the CtrlEdit to stop removing characters
    pub fn stopping_char_set(&self) -> &'static [char] {
        match self {
            CtrlEdit::BackspaceWord | CtrlEdit::DeleteWord => &[
                ' ', '.', '!', '?', ',', '-', '_', '\"', ';', ':', '(', ')', '{', '}', '[', ']',
            ],
            CtrlEdit::BackspaceSentence | CtrlEdit::DeleteSentence => {
                &['.', '!', '?', '\"', ';', ':']
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
/// a Restriction is a subset of the ContentAction::Standard variant, which imposes additional requirements on the
/// types of Actions that can be performed on the UpgradedContent. Note a Restriction only ever blocks Actions from
/// being perform()ed on the content, it will NEVER retroactively apply the Restriction rules to the content
pub enum Restriction {
    /// any Edits that would result in non-number characters (anything other than ASCII '0'-'9') being added to the
    /// content are blocked
    NumbersOnly,
}

#[derive(Debug, Clone, PartialEq)]
/// types of extended actions the UpgradedContent can perform
pub enum ContentAction {
    Standard(text_editor::Action),
    Restricted((Restriction, text_editor::Action)),
    Ctrl(CtrlEdit),
    Undo,
    Redo,
    ClearHistoryStack,
}

/// the result in terms of HistoryEvents of a ContentAction
pub enum ActionHistoryEvent {
    /// successfully created a valid HistoryEvent that should be pushed onto the undo stack
    Push(HistoryEvent),

    /// a HistoryEvent should have been created but wasn't, likely due to nothing changing on the editor (backspacing
    /// at the start of the document, deleting at the end...etc). since an event was not written to the the undo stack
    /// but should have, revert()ing must be disabled to ensure the stack remains valid
    DisableRevert,

    /// the ContentAction doesn't produce events for the HistoryStack
    Ignore,
}

#[derive(Debug, Default)]
/// the UpgradedContent is a wrapper around iced's text_editor::Content that provides significant additional
/// functionality through the ContentAction extended actions
pub struct UpgradedContent {
    content: Content,
    history_stack: HistoryStack,
}

impl UpgradedContent {
    /// creates a new UpgradedContent that starts with the provided text already present
    pub fn with_text(starting_text: &str) -> Self {
        Self {
            content: Content::with_text(starting_text),
            history_stack: HistoryStack::default(),
        }
    }

    /// performs extended ContentAction actions on the UpgradedContent
    pub fn perform(&mut self, content_action: ContentAction) {
        let old_cursor = self.content.cursor();
        let old_text = self.text();

        let selection = self.content.selection();
        let selection_char_count = selection.clone().unwrap_or_default().chars().count();

        let selection_text_removal = selection
            .clone()
            .map(|selection_text| TextRemoval::new(selection_text, false));

        let content_action_status = match content_action {
            ContentAction::Standard(action) => {
                self.content.perform(action.clone());

                let new_cursor = self.content.cursor();

                match action {
                    text_editor::Action::Edit(edit) => match edit {
                        Edit::Insert(inserted_char) => ActionHistoryEvent::Push(HistoryEvent {
                            text_removed: selection_text_removal,
                            text_added: Some(inserted_char.to_string()),
                            selection_char_count,
                            redo_cursor: old_cursor,
                            undo_cursor: new_cursor,
                        }),
                        Edit::Paste(pasted_text) => {
                            let pasted_string = pasted_text.to_string();

                            ActionHistoryEvent::Push(HistoryEvent {
                                text_removed: selection_text_removal,
                                text_added: Some(pasted_string),
                                selection_char_count,
                                redo_cursor: old_cursor,
                                undo_cursor: new_cursor,
                            })
                        }
                        Edit::Enter => ActionHistoryEvent::Push(HistoryEvent {
                            text_removed: selection_text_removal,
                            text_added: Some("\n".to_string()),
                            selection_char_count,
                            redo_cursor: old_cursor,
                            undo_cursor: new_cursor,
                        }),
                        Edit::Indent => todo!(),
                        Edit::Unindent => todo!(),
                        Edit::Backspace => {
                            if old_text.is_empty() {
                                ActionHistoryEvent::DisableRevert
                            } else if selection.is_some() {
                                ActionHistoryEvent::Push(HistoryEvent {
                                    text_removed: selection_text_removal,
                                    text_added: None,
                                    selection_char_count,
                                    redo_cursor: new_cursor,
                                    undo_cursor: new_cursor,
                                })
                            } else if Self::cursor_at_start_of_text(&old_cursor) {
                                ActionHistoryEvent::DisableRevert
                            } else if Self::cursor_at_start_of_line(&old_cursor) {
                                ActionHistoryEvent::Push(HistoryEvent {
                                    text_removed: Some(TextRemoval::new("\n".to_string(), false)),
                                    text_added: None,
                                    selection_char_count,
                                    redo_cursor: new_cursor,
                                    undo_cursor: new_cursor,
                                })
                            } else {
                                let text_removed = old_text
                                    .lines()
                                    .nth(old_cursor.position.line)
                                    .expect("couldn't get line")
                                    .chars()
                                    .nth(old_cursor.position.column - 1)
                                    .expect("couldn't get char")
                                    .to_string();

                                ActionHistoryEvent::Push(HistoryEvent {
                                    text_removed: Some(TextRemoval::new(text_removed, false)),
                                    text_added: None,
                                    selection_char_count,
                                    redo_cursor: new_cursor,
                                    undo_cursor: new_cursor,
                                })
                            }
                        }
                        Edit::Delete => {
                            if old_text.is_empty() {
                                ActionHistoryEvent::DisableRevert
                            } else if let Some(selection_text) = selection {
                                // note that this isn't a delete removal since a delete with a selection is identical
                                // to a backspace with a selection
                                ActionHistoryEvent::Push(HistoryEvent {
                                    text_removed: Some(TextRemoval::new(selection_text, false)),
                                    text_added: None,
                                    selection_char_count,
                                    redo_cursor: old_cursor,
                                    undo_cursor: new_cursor,
                                })
                            } else if Self::cursor_at_end_of_text(&old_cursor, &old_text) {
                                ActionHistoryEvent::DisableRevert
                            } else if Self::cursor_at_end_of_line(&old_cursor, &old_text) {
                                ActionHistoryEvent::Push(HistoryEvent {
                                    text_removed: Some(TextRemoval::new("\n".to_string(), true)),
                                    text_added: None,
                                    selection_char_count,
                                    redo_cursor: old_cursor,
                                    undo_cursor: old_cursor,
                                })
                            } else {
                                let text_removed = old_text
                                    .lines()
                                    .nth(old_cursor.position.line)
                                    .expect("couldn't get line")
                                    .chars()
                                    .nth(old_cursor.position.column)
                                    .expect("couldn't get char")
                                    .to_string();

                                ActionHistoryEvent::Push(HistoryEvent {
                                    text_removed: Some(TextRemoval::new(text_removed, true)),
                                    text_added: None,
                                    selection_char_count,
                                    redo_cursor: old_cursor,
                                    undo_cursor: old_cursor,
                                })
                            }
                        }
                    },
                    text_editor::Action::Move(motion) => {
                        // if the cursor position didn't change when up/down was pressed, move cursor to start/end
                        if old_cursor.position == new_cursor.position {
                            match motion {
                                text_editor::Motion::Up => {
                                    self.content
                                        .perform(Action::Move(text_editor::Motion::DocumentStart));
                                }
                                text_editor::Motion::Down => {
                                    self.content
                                        .perform(Action::Move(text_editor::Motion::DocumentEnd));
                                }
                                _ => {}
                            }
                        }

                        ActionHistoryEvent::Ignore
                    }
                    _ => ActionHistoryEvent::Ignore,
                }
            }
            ContentAction::Restricted((restriction, action)) => {
                match restriction {
                    Restriction::NumbersOnly => {
                        let edit_contains_only_numbers = if let Action::Edit(edit) = &action {
                            match edit {
                                Edit::Insert(inserted_char) => inserted_char.is_ascii_digit(),
                                Edit::Paste(pasted_string) => pasted_string
                                    .to_string()
                                    .chars()
                                    .all(|character| character.is_ascii_digit()),
                                Edit::Enter => false,
                                Edit::Indent => false,
                                Edit::Unindent => false,
                                Edit::Backspace => true,
                                Edit::Delete => true,
                            }
                        } else {
                            // no non-Edit actions could add a non-number character, so all non-Edits will be valid
                            true
                        };

                        if edit_contains_only_numbers {
                            self.perform(ContentAction::Standard(action));
                        }
                    }
                }

                // the self.perform() will take care of any HistoryEvents it generates on its own. if the ContentAction
                // fails to meet the restriction requirements, the content is not altered in any way so no
                // HistoryEvents could be generated, making it safe to ignore Restricted ContentActions
                ActionHistoryEvent::Ignore
            }
            ContentAction::Ctrl(ctrl_type) => {
                let stopping_chars = ctrl_type.stopping_char_set();

                match ctrl_type {
                    CtrlEdit::BackspaceWord | CtrlEdit::BackspaceSentence => {
                        // revert the backspace handled automatically by the content
                        self.history_stack.revert(&mut self.content);

                        if let Some(history_event) =
                            Self::perform_ctrl_backspace(&mut self.content, stopping_chars)
                        {
                            ActionHistoryEvent::Push(history_event)
                        } else {
                            ActionHistoryEvent::DisableRevert
                        }
                    }
                    CtrlEdit::DeleteWord | CtrlEdit::DeleteSentence => {
                        // revert delete handled automatically by the content
                        self.history_stack.revert(&mut self.content);

                        if let Some(history_event) =
                            Self::perform_ctrl_delete(&mut self.content, stopping_chars)
                        {
                            ActionHistoryEvent::Push(history_event)
                        } else {
                            ActionHistoryEvent::DisableRevert
                        }
                    }
                }
            }
            ContentAction::Undo => {
                self.history_stack.perform_undo(&mut self.content);

                ActionHistoryEvent::Ignore
            }
            ContentAction::Redo => {
                self.history_stack.perform_redo(&mut self.content);

                ActionHistoryEvent::Ignore
            }
            ContentAction::ClearHistoryStack => {
                self.history_stack.clear();

                ActionHistoryEvent::Ignore
            }
        };

        match content_action_status {
            ActionHistoryEvent::Push(history_event) => {
                self.history_stack.push_undo_action(history_event);
            }
            ActionHistoryEvent::DisableRevert => {
                self.history_stack.set_unrevertable();
            }
            ActionHistoryEvent::Ignore => {}
        }
    }

    /// returns true if the given cursor is at the start of the current line
    fn cursor_at_start_of_line(cursor: &Cursor) -> bool {
        cursor.position.column == 0
    }

    /// returns true if the given cursor is at the end of the current line in the provided text. also returns true if
    /// the text is empty. the cursor must exist positionally inside of the text
    fn cursor_at_end_of_line(cursor: &Cursor, text: &str) -> bool {
        if text.is_empty() {
            return true;
        }

        let max_char_index = text
            .lines()
            .nth(cursor.position.line)
            .expect("couldn't extract line")
            .chars()
            .count();

        let current_char_index = cursor.position.column;

        if current_char_index == max_char_index {
            return true;
        }

        false
    }

    /// returns true if the given cursor is at the start of the text, ie at (0,0)
    fn cursor_at_start_of_text(cursor: &Cursor) -> bool {
        cursor.position.column == 0 && cursor.position.line == 0
    }

    /// returns true if the given cursor is at the end of the current line and at the last character in the provided
    /// text. returns true if the text is the empty string
    fn cursor_at_end_of_text(cursor: &Cursor, text: &str) -> bool {
        if text.is_empty() {
            return true;
        }

        let max_line_index = text.lines().count() - 1;

        let current_line_index = cursor.position.line;

        if Self::cursor_at_end_of_line(cursor, text) && current_line_index == max_line_index {
            return true;
        }

        false
    }

    /// returns the zero-indexed line number of the cursor
    pub fn cursor_line(&self) -> usize {
        self.content.cursor().position.line
    }

    /// returns the column/character number of the cursor
    pub fn cursor_column(&self) -> usize {
        self.content.cursor().position.column
    }

    /// returns the underlying Content. this should only be used when constructing a text_editor
    pub fn raw_content(&self) -> &Content {
        &self.content
    }

    /// returns the selected text in the content, if it exists
    pub fn selection(&self) -> Option<String> {
        self.content.selection()
    }

    /// returns the Content text as a string
    pub fn text(&self) -> String {
        self.content.text()
    }

    /// returns the number of elements in the undo stack
    pub fn undo_stack_height(&self) -> usize {
        self.history_stack.undo_stack_height()
    }

    /// returns the number of elements in the redo stack
    pub fn redo_stack_height(&self) -> usize {
        self.history_stack.redo_stack_height()
    }

    /// performs a ctrl+backspace on the content, for a given set of stopping_chars, which dictates the characters that
    /// stop the ctrl+backspace from continuing. returns the corresponding HistoryEvent that represents the action, if
    /// the action changed the state of the content, None otherwise. Note that the HistoryStack must be reverted before
    /// calling this, as the regular backspace before the ctrl+backspace must be undone.
    pub fn perform_ctrl_backspace(
        content: &mut Content,
        stopping_chars: &[char],
    ) -> Option<HistoryEvent> {
        if Self::cursor_at_start_of_text(&content.cursor()) {
            return None;
        }

        let mut removed_chars = String::new();

        let cursor_line_start = content.cursor().position.line;
        let cursor_char_start = content.cursor().position.column;

        let old_cursor = content.cursor();
        let old_text = content.text();

        let selection = content.selection();
        let selection_char_count = selection.clone().unwrap_or_default().chars().count();

        if let Some(selection_text) = selection {
            content.perform(Action::Edit(Edit::Backspace));

            let new_cursor = content.cursor();

            let history_event = HistoryEvent {
                text_removed: Some(TextRemoval::new(selection_text, false)),
                text_added: None,
                selection_char_count,
                redo_cursor: old_cursor,
                undo_cursor: new_cursor,
            };

            return Some(history_event);
        }

        if Self::cursor_at_start_of_line(&old_cursor) {
            let previous_line_char_count = old_text
                .lines()
                .nth(old_cursor.position.line - 1)
                .expect("couldn't extract line")
                .chars()
                .count();

            let new_cursor = Cursor {
                position: Position {
                    line: old_cursor.position.line - 1,
                    column: previous_line_char_count,
                },
                selection: None,
            };

            let history_event = HistoryEvent {
                text_removed: Some(TextRemoval::new("\n".to_string(), false)),
                text_added: None,
                selection_char_count,
                redo_cursor: new_cursor,
                undo_cursor: new_cursor,
            };

            content.perform(Action::Edit(text_editor::Edit::Backspace));
            return Some(history_event);
        }

        let char_line = old_text
            .lines()
            .nth(cursor_line_start)
            .expect("couldn't extract line");

        let mut backspace_head = cursor_char_start - 1;

        let first_char_removed = char_line
            .chars()
            .nth(backspace_head)
            .expect("couldn't extract char from line");

        let mut removing_seqence_of_stops = false;

        loop {
            let char_to_remove = char_line
                .chars()
                .nth(backspace_head)
                .expect("couldn't extract char from line");

            removed_chars.push(char_to_remove);
            content.perform(Action::Edit(text_editor::Edit::Backspace));

            if backspace_head > 0 {
                backspace_head -= 1;
            } else {
                break;
            }

            let next_char_to_remove = char_line
                .chars()
                .nth(backspace_head)
                .expect("couldn't extract char from line");

            if stopping_chars.contains(&first_char_removed)
                && first_char_removed == next_char_to_remove
                && misc_tools::chars_all_same_in_string(&removed_chars)
                && (removed_chars.chars().count() == 1 || removing_seqence_of_stops)
            {
                removing_seqence_of_stops = true;
                continue;
            } else if removing_seqence_of_stops {
                break;
            }

            if stopping_chars.contains(&next_char_to_remove) {
                break;
            }
        }

        removed_chars = removed_chars.chars().rev().collect();

        let new_cursor = content.cursor();

        Some(HistoryEvent {
            text_removed: Some(TextRemoval::new(removed_chars, false)),
            text_added: None,
            selection_char_count,
            redo_cursor: new_cursor,
            undo_cursor: new_cursor,
        })
    }

    /// performs a ctrl+delete on the content, for a given set of stopping_chars, which dictates the characters that
    /// stop the ctrl+delete from continuing. returns the corresponding HistoryEvent that represents the action if the
    /// state of the content was changed, None otherwise
    pub fn perform_ctrl_delete(
        content: &mut Content,
        stopping_chars: &[char],
    ) -> Option<HistoryEvent> {
        let old_text = content.text();
        let old_cursor = content.cursor();

        let cursor_line_start = old_cursor.position.line;
        let cursor_char_start = old_cursor.position.column;

        let selection = content.selection();
        let selection_char_count = selection.clone().unwrap_or_default().chars().count();

        let line = old_text.lines().nth(cursor_line_start)?;

        let char_count = line.chars().count();

        if let Some(selection_text) = selection {
            content.perform(Action::Edit(text_editor::Edit::Backspace));

            let new_cursor = content.cursor();

            let history_event = HistoryEvent {
                // again, this isn't a delete removal since a ctrl+delete with a selection is simply a backspace
                text_removed: Some(TextRemoval::new(selection_text, false)),
                text_added: None,
                selection_char_count,
                redo_cursor: old_cursor,
                undo_cursor: new_cursor,
            };

            return Some(history_event);
        }

        if Self::cursor_at_end_of_text(&old_cursor, &old_text) {
            None
        } else if Self::cursor_at_end_of_line(&old_cursor, &old_text) {
            let history_event = HistoryEvent {
                text_removed: Some(TextRemoval::new('\n'.to_string(), true)),
                text_added: None,
                selection_char_count,
                redo_cursor: old_cursor,
                undo_cursor: old_cursor,
            };
            content.perform(Action::Edit(text_editor::Edit::Delete));

            Some(history_event)
        } else {
            // standard ctrl+delete
            let mut removed_chars = String::new();
            let first_char_removed = line
                .chars()
                .nth(cursor_char_start)
                .expect("couldn't extract char from line");

            let mut delete_head = cursor_char_start;

            let mut removing_sequence_of_stops = false;

            loop {
                let char_to_remove = line
                    .chars()
                    .nth(delete_head)
                    .expect("couldn't extract char from line");

                removed_chars.push(char_to_remove);
                content.perform(Action::Edit(text_editor::Edit::Delete));

                if (delete_head + 1) < char_count {
                    delete_head += 1;
                } else {
                    break;
                }

                let next_char_to_remove = line
                    .chars()
                    .nth(delete_head)
                    .expect("couldn't extract char from line");

                if stopping_chars.contains(&first_char_removed)
                    && first_char_removed == next_char_to_remove
                    && misc_tools::chars_all_same_in_string(&removed_chars)
                    && (removed_chars.chars().count() == 1 || removing_sequence_of_stops)
                {
                    removing_sequence_of_stops = true;
                    continue;
                } else if removing_sequence_of_stops {
                    break;
                }

                if stopping_chars.contains(&next_char_to_remove) {
                    break;
                }
            }

            Some(HistoryEvent {
                text_removed: Some(TextRemoval::new(removed_chars, true)),
                text_added: None,
                selection_char_count,
                redo_cursor: old_cursor,
                undo_cursor: old_cursor,
            })
        }
    }
}
