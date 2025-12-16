use crate::{history_stack::HistoryEvent, misc_tools};
use iced::widget::text_editor::{self, Action, Content, Motion};

/// Relocates the cursor to a new position by manually moving the cursor there
pub fn move_cursor(content: &mut Content, new_line_idx: usize, new_char_idx: usize) {
    content.cursor().position.line = new_line_idx;
    content.cursor().position.column = new_char_idx;

    // let (mut old_cursor_line, mut old_cursor_char) = content.cursor_position();

    // let direction = if old_cursor_line == new_line_idx {
    //     if old_cursor_char < new_char_idx {
    //         Motion::Right
    //     } else if old_cursor_char > new_char_idx {
    //         Motion::Left
    //     } else {
    //         return;
    //     }
    // } else if old_cursor_line < new_line_idx {
    //     Motion::Right
    // } else {
    //     Motion::Left
    // };

    // let mut first_stop = false;

    // loop {
    //     // TODO: this is really inefficent but other methods are proving unreliable
    //     content.perform(Action::Move(direction));
    //     let (cursor_line, cursor_char) = content.cursor_position();

    //     if cursor_line == new_line_idx && cursor_char == new_char_idx {
    //         break;
    //     }
    //     // quit on second repeat location. two are required, as if there is a selection it is possible to have the
    //     // first movement that causes the selction to disappear result in the cursor ending up in the same location as
    //     // it "was" with the selection
    //     if cursor_line == old_cursor_line && cursor_char == old_cursor_char {
    //         if !first_stop {
    //             first_stop = true;
    //         } else {
    //             break;
    //         }
    //     }

    //     (old_cursor_line, old_cursor_char) = (cursor_line, cursor_char);
    // }
}

/// Used for locating new the cursor position when a line gets removed
pub fn decrement_cursor_position(
    content: &Content,
    line_idx: usize,
    char_idx: usize,
) -> (usize, usize) {
    let text = content.text();

    if line_idx == 0 && char_idx == 0 {
        (0, 0)
    } else if char_idx == 0 {
        let line = text
            .lines()
            .nth(line_idx - 1)
            .expect("line index doesn't exist");

        (line_idx - 1, line.chars().count())
    } else {
        (line_idx, char_idx - 1)
    }
}

/// Since the cursor is the literal leading edge of the cursor (where the mouse is located), using cursor_position() at
/// first glance appears to give inconsistant results, depending on the direction that the selection is being dragged.
/// This function always returns the cursor position starting at the start of the selection, and the standard cursor
/// location when no selection is present. cursor_line/char_idx is the last known state of where the cursor was when
/// there was no selection, which corresponds to one of the extrema of the selection, either the beginning (selecting
///  left to right) or the end (selecting right to left)
pub fn locate_cursor_start(
    content: &Content,
    cursor_line_idx: usize,
    cursor_char_idx: usize,
) -> (usize, usize) {
    let (current_line, current_char) = (
        content.cursor().position.line,
        content.cursor().position.column,
    );

    if content.selection().is_none() {
        return (current_line, current_char);
    }

    if current_line > cursor_line_idx {
        (cursor_line_idx, cursor_char_idx)
    } else if current_line < cursor_line_idx {
        (current_line, current_char)
    } else {
        (current_line, current_char.min(cursor_char_idx))
    }
}

/// selects the number of characters specified in length, starting at line index, char index
pub fn select_text(content: &mut Content, line_start: usize, char_start: usize, length: usize) {
    move_cursor(content, line_start, char_start);

    // this is required to ensure the cursor position ends at the start of the selection, so we don't need to pass in
    // the current state for use with locate_cursor_start()
    for _i in 0..length {
        content.perform(Action::Move(Motion::Right));
    }

    for _i in 0..length {
        content.perform(Action::Select(Motion::Left));
    }
}

/// retrives the information about the current selection, returning ((start_of_selection_line_idx, char_idx),
/// length_of_selection)
pub fn get_selection_bounds(
    content: &Content,
    cursor_line_idx: usize,
    cursor_char_idx: usize,
) -> ((usize, usize), usize) {
    let (cursor_line, cursor_char) = locate_cursor_start(content, cursor_line_idx, cursor_char_idx);

    let selection_length = content
        .selection()
        .map_or(0, |selection| selection.chars().count());

    ((cursor_line, cursor_char), selection_length)
}

/// if the cursor is on the first line, hitting the up arrow does not move the cursor to the start of the content, it
/// will remain in place. this also occurs when pressing the down arrow on the last of the document. this fixes this
/// behavior, and properly moves the cursor when an arrow key is pressed on the first/last line of the document
pub fn correct_arrow_movement(
    content: &mut Content,
    old_cursor_position: (usize, usize),
    direction: Motion,
) {
    let new_cursor_position = (
        content.cursor().position.line,
        content.cursor().position.column,
    );

    if old_cursor_position == new_cursor_position {
        match direction {
            text_editor::Motion::Up => {
                content.perform(Action::Move(text_editor::Motion::DocumentStart));
            }
            text_editor::Motion::Down => {
                content.perform(Action::Move(text_editor::Motion::End));
            }
            _ => {}
        }
    }
}

/// performs a ctrl+backspace on the content, for a given set of stopping_chars, which dictates the characters that
/// stop the ctrl+backspace from continuing. the last_known_cursor is the last place the cursor position was known
/// before a selection occurred, used for calculating the start of the selection. returns the corrensponding
/// HistoryEvent that represents the action. Note that the HistoryStack must be reverted before calling this, as the
/// regular backspace before the ctrl+backspace must be undone.
pub fn perform_ctrl_backspace(
    content: &mut Content,
    stopping_chars: &[char],
    last_known_cursor_line_idx: usize,
    last_known_cursor_char_idx: usize,
) -> HistoryEvent {
    if (
        content.cursor().position.line,
        content.cursor().position.column,
    ) == (0, 0)
    {
        return HistoryEvent::default();
    }

    let mut removed_chars = String::new();
    let (cursor_line_start, cursor_char_start) = (
        content.cursor().position.line,
        content.cursor().position.column,
    );

    if let Some(selection) = content.selection() {
        let selection_bounds = get_selection_bounds(
            content,
            last_known_cursor_line_idx,
            last_known_cursor_char_idx,
        );
        let (adjusted_cursor_line, adjusted_cursor_char) = locate_cursor_start(
            content,
            last_known_cursor_line_idx,
            last_known_cursor_char_idx,
        );

        let history_event = HistoryEvent {
            selection: Some(selection_bounds),
            text_removed: Some(selection),
            text_added: None,
            cursor_line_idx: adjusted_cursor_line,
            cursor_char_idx: adjusted_cursor_char,
        };

        content.perform(Action::Edit(text_editor::Edit::Backspace));
        return history_event;
    }

    // on edge of newline
    if cursor_char_start == 0 {
        let (cursor_line, cursor_char) = (
            content.cursor().position.line,
            content.cursor().position.column,
        );

        let (new_cursor_line, new_cursor_char) =
            decrement_cursor_position(content, cursor_line, cursor_char);

        let history_event = HistoryEvent {
            selection: None,
            text_removed: Some("\n".to_string()),
            text_added: None,
            cursor_line_idx: new_cursor_line,
            cursor_char_idx: new_cursor_char,
        };

        content.perform(Action::Edit(text_editor::Edit::Backspace));
        return history_event;
    }

    let content_text = content.text();
    let char_line = content_text
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

    let cursor_line_end = cursor_line_start + 1 - removed_chars.lines().count();
    let cursor_char_end = cursor_char_start - removed_chars.chars().count();

    HistoryEvent {
        selection: None,
        text_removed: Some(removed_chars),
        text_added: None,
        cursor_line_idx: cursor_line_end,
        cursor_char_idx: cursor_char_end,
    }
}

/// performs a ctrl+delete on the content, for a given set of stopping_chars, which dictates the characters that
/// stop the ctrl+delete from continuing. the last_known_cursor is the last place the cursor position was known
/// before a selection occurred, used for calculating the start of the selection. returns the corrensponding
/// HistoryEvent that represents the action
pub fn perform_ctrl_delete(
    content: &mut Content,
    stopping_chars: &[char],
    cursor_line_idx: usize,
    cursor_char_idx: usize,
) -> HistoryEvent {
    let content_text = content.text();

    let (cursor_line_start, cursor_char_start) = (
        content.cursor().position.line,
        content.cursor().position.column,
    );

    let line_count = content_text.lines().count();
    let line = match content_text.lines().nth(cursor_line_start) {
        Some(line) => line,
        None => return HistoryEvent::default(),
    };

    let char_count = line.chars().count();

    if let Some(selection) = content.selection() {
        let selection_bounds = get_selection_bounds(content, cursor_line_idx, cursor_char_idx);
        let (adjusted_cursor_line, adjusted_cursor_char) =
            locate_cursor_start(content, cursor_line_idx, cursor_char_idx);
        let history_event = HistoryEvent {
            selection: Some(selection_bounds),
            text_removed: Some(selection),
            text_added: None,
            cursor_line_idx: adjusted_cursor_line,
            cursor_char_idx: adjusted_cursor_char,
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
            selection: None,
            text_removed: Some('\n'.to_string()),
            text_added: None,
            cursor_line_idx: cursor_line_start,
            cursor_char_idx: cursor_char_start,
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
            selection: None,
            text_removed: Some(removed_chars),
            text_added: None,
            cursor_line_idx: cursor_line_start,
            cursor_char_idx: cursor_char_start,
        }
    }
}
