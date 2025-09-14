use iced::widget::text_editor::{Action, Content, Motion};

/// Relocates the cursor to a new position by normalizing and manually moving the cursor there
pub fn move_cursor(content: &mut Content, new_line_idx: usize, new_char_idx: usize) {
    content.perform(Action::Move(Motion::DocumentStart));

    if new_line_idx == 0 && new_char_idx == 0 {
        return;
    }

    let (mut old_cursor_line, mut old_cursor_char) = content.cursor_position();

    loop {
        // TODO: this is really inefficent but other methods are proving unreliable
        content.perform(Action::Move(Motion::Right));
        let (cursor_line, cursor_char) = content.cursor_position();

        // quit if reached target or nothing changed since last iteration
        if (cursor_line == new_line_idx && cursor_char == new_char_idx)
            || (cursor_line == old_cursor_line && cursor_char == old_cursor_char)
        {
            break;
        }

        (old_cursor_line, old_cursor_char) = (cursor_line, cursor_char);
    }
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
    let (current_line, current_char) = content.cursor_position();

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

    for _i in 0..length {
        content.perform(Action::Select(Motion::Right));
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
