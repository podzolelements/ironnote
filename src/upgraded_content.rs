use crate::history_stack::{HistoryEvent, HistoryStack};
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
/// types of extended actions the UpgradedContent can perform
pub enum ContentAction {
    Standard(text_editor::Action),
    Ctrl(CtrlEdit),
    Undo,
    Redo,
    ClearHistoryStack,
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
        let selected_char_count = selection.clone().unwrap_or_default().chars().count();

        let history_event = match content_action {
            ContentAction::Standard(action) => {
                self.content.perform(action.clone());

                let new_cursor = self.content.cursor();

                match action {
                    text_editor::Action::Edit(edit) => match edit {
                        Edit::Insert(inserted_char) => Some(HistoryEvent {
                            text_removed: selection,
                            text_added: Some(inserted_char.to_string()),
                            selected_char_count,
                            cursor: old_cursor,
                        }),
                        Edit::Paste(pasted_text) => {
                            let pasted_string = pasted_text.to_string();

                            Some(HistoryEvent {
                                text_removed: selection,
                                text_added: Some(pasted_string),
                                selected_char_count,
                                cursor: old_cursor,
                            })
                        }
                        Edit::Enter => Some(HistoryEvent {
                            text_removed: selection,
                            text_added: Some("\n".to_string()),
                            selected_char_count,
                            cursor: old_cursor,
                        }),
                        Edit::Indent => todo!(),
                        Edit::Unindent => todo!(),
                        Edit::Backspace => {
                            if old_text.is_empty() {
                                return;
                            }

                            if selection.is_some() {
                                Some(HistoryEvent {
                                    text_removed: selection,
                                    text_added: None,
                                    selected_char_count,
                                    cursor: new_cursor,
                                })
                            } else if old_cursor.position.column == 0 {
                                Some(HistoryEvent {
                                    text_removed: Some("\n".to_string()),
                                    text_added: None,
                                    selected_char_count,
                                    cursor: new_cursor,
                                })
                            } else {
                                let text_removed = Some(
                                    old_text
                                        .lines()
                                        .nth(old_cursor.position.line)
                                        .expect("couldn't get line")
                                        .chars()
                                        .nth(old_cursor.position.column - 1)
                                        .expect("couldn't get char")
                                        .to_string(),
                                );

                                Some(HistoryEvent {
                                    text_removed,
                                    text_added: None,
                                    selected_char_count,
                                    cursor: new_cursor,
                                })
                            }
                        }
                        Edit::Delete => {
                            if old_text.is_empty() {
                                return;
                            }

                            let max_char_index = old_text
                                .lines()
                                .nth(old_cursor.position.line)
                                .expect("couldn't get line")
                                .chars()
                                .count()
                                .saturating_sub(1);
                            let delete_index = old_cursor.position.column + 1;

                            let max_line_index = old_text.lines().count() - 1;
                            let next_line_index = old_cursor.position.line + 1;

                            if selection.is_some() {
                                Some(HistoryEvent {
                                    text_removed: selection,
                                    text_added: None,
                                    selected_char_count,
                                    cursor: old_cursor,
                                })
                            } else if delete_index > max_char_index
                                && (next_line_index > max_line_index)
                            {
                                // deleting at the end of text has no effect
                                None
                            } else if delete_index > max_char_index {
                                // deleting a newline not at the end of the text
                                Some(HistoryEvent {
                                    text_removed: Some("\n".to_string()),
                                    text_added: None,
                                    selected_char_count,
                                    cursor: old_cursor,
                                })
                            } else {
                                let text_removed = Some(
                                    old_text
                                        .lines()
                                        .nth(old_cursor.position.line)
                                        .expect("couldn't get line")
                                        .chars()
                                        .nth(old_cursor.position.column + 1)
                                        .expect("couldn't get char")
                                        .to_string(),
                                );

                                Some(HistoryEvent {
                                    text_removed,
                                    text_added: None,
                                    selected_char_count,
                                    cursor: old_cursor,
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

                        None
                    }
                    _ => None,
                }
            }
            ContentAction::Ctrl(ctrl_type) => {
                let stopping_chars = ctrl_type.stopping_char_set();

                match ctrl_type {
                    CtrlEdit::BackspaceWord | CtrlEdit::BackspaceSentence => {
                        // revert the backspace handled automatically by the content
                        self.history_stack.revert(&mut self.content);

                        Some(Self::perform_ctrl_backspace(
                            &mut self.content,
                            stopping_chars,
                        ))
                    }
                    CtrlEdit::DeleteWord | CtrlEdit::DeleteSentence => {
                        // not sure why ctrl+delete doesn't get caught and need to be revert()ed but ctrl+backspace
                        // does...
                        Some(Self::perform_ctrl_delete(&mut self.content, stopping_chars))
                    }
                }
            }
            ContentAction::Undo => {
                self.history_stack.perform_undo(&mut self.content);

                None
            }
            ContentAction::Redo => {
                self.history_stack.perform_redo(&mut self.content);

                None
            }
            ContentAction::ClearHistoryStack => {
                self.history_stack.clear();

                None
            }
        };

        if let Some(valid_history_event) = history_event {
            self.history_stack.push_undo_action(valid_history_event);
        }
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
    /// stop the ctrl+backspace from continuing. returns the corresponding HistoryEvent that represents the action.
    /// Note that the HistoryStack must be reverted before calling this, as the regular backspace before the
    /// ctrl+backspace must be undone.
    pub fn perform_ctrl_backspace(content: &mut Content, stopping_chars: &[char]) -> HistoryEvent {
        if content.cursor().position.line == 0 && content.cursor().position.column == 0 {
            return HistoryEvent::default();
        }

        let mut removed_chars = String::new();
        let (cursor_line_start, cursor_char_start) = (
            content.cursor().position.line,
            content.cursor().position.column,
        );

        let old_cursor = content.cursor();
        let old_text = content.text();

        let selection = content.selection();
        let selected_char_count = selection.clone().unwrap_or_default().chars().count();

        if selection.is_some() {
            let history_event = HistoryEvent {
                text_removed: selection,
                text_added: None,
                selected_char_count,
                cursor: old_cursor,
            };

            content.perform(Action::Edit(Edit::Backspace));
            return history_event;
        }

        // on edge of newline
        if cursor_char_start == 0 {
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
                text_removed: Some("\n".to_string()),
                text_added: None,
                selected_char_count,
                cursor: new_cursor,
            };

            content.perform(Action::Edit(text_editor::Edit::Backspace));
            return history_event;
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

        HistoryEvent {
            text_removed: Some(removed_chars),
            text_added: None,
            selected_char_count,
            cursor: new_cursor,
        }
    }

    /// performs a ctrl+delete on the content, for a given set of stopping_chars, which dictates the characters that
    /// stop the ctrl+delete from continuing. returns the corresponding HistoryEvent that represents the action
    pub fn perform_ctrl_delete(content: &mut Content, stopping_chars: &[char]) -> HistoryEvent {
        let content_text = content.text();

        let cursor_line_start = content.cursor().position.line;
        let cursor_char_start = content.cursor().position.column;

        let old_cursor = content.cursor();

        let selection = content.selection();
        let selected_char_count = selection.clone().unwrap_or_default().chars().count();

        let line_count = content_text.lines().count();
        let line = match content_text.lines().nth(cursor_line_start) {
            Some(line) => line,
            None => return HistoryEvent::default(),
        };

        let char_count = line.chars().count();

        if let Some(selection_text) = selection {
            let history_event = HistoryEvent {
                text_removed: Some(selection_text),
                text_added: None,
                selected_char_count,
                cursor: old_cursor,
            };

            content.perform(Action::Edit(text_editor::Edit::Backspace));
            return history_event;
        }

        if line_count == (cursor_line_start + 1) && char_count == cursor_char_start {
            // nothing to delete, end of text
            HistoryEvent::default()
        } else if char_count == cursor_char_start {
            // deletes following newline
            let history_event = HistoryEvent {
                text_removed: Some('\n'.to_string()),
                text_added: None,
                selected_char_count,
                cursor: old_cursor,
            };
            content.perform(Action::Edit(text_editor::Edit::Delete));

            history_event
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

            HistoryEvent {
                text_removed: Some(removed_chars),
                text_added: None,
                selected_char_count,
                cursor: old_cursor,
            }
        }
    }
}
