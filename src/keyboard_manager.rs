use crate::upgraded_content::{ContentAction, CtrlEdit};
use iced::widget::text_editor::{Action, Motion};
use keybinds::Keybinds;

#[derive(Debug, Clone)]
/// these actions are not bound to their shortcuts via the keybinds structure, since the text_editor takes care of
/// handling them. these are called when the action needs to be performed manually without the shortcuts
pub enum UnboundKey {
    Cut,
    Copy,
    Paste,
}

#[derive(Debug, Clone)]
/// keyboard actions specific to text_editors
pub enum TextEdit {
    BackspaceWord,
    BackspaceSentence,
    DeleteWord,
    DeleteSentence,
    Undo,
    Redo,
    JumpToContentStart,
    JumpToContentEnd,
}

impl TextEdit {
    /// converion of the shortcut type into its equivelent ContentAction
    pub fn to_content_action(&self) -> ContentAction {
        match self {
            TextEdit::BackspaceWord => ContentAction::Ctrl(CtrlEdit::BackspaceWord),
            TextEdit::BackspaceSentence => ContentAction::Ctrl(CtrlEdit::BackspaceSentence),
            TextEdit::DeleteWord => ContentAction::Ctrl(CtrlEdit::DeleteWord),
            TextEdit::DeleteSentence => ContentAction::Ctrl(CtrlEdit::DeleteSentence),
            TextEdit::Undo => ContentAction::Undo,
            TextEdit::Redo => ContentAction::Redo,
            TextEdit::JumpToContentStart => {
                ContentAction::Standard(Action::Move(Motion::DocumentStart))
            }
            TextEdit::JumpToContentEnd => {
                ContentAction::Standard(Action::Move(Motion::DocumentEnd))
            }
        }
    }
}

#[derive(Debug, Clone)]
/// keybindings get bound to these physical actions, representing what actually happens after a keybinding is triggered
pub enum KeyboardAction {
    Content(TextEdit),
    Save,
    Debug,
    Unbound(UnboundKey),
}

pub fn bind_keybinds() -> Keybinds<KeyboardAction> {
    let mut keybinds = Keybinds::default();

    keybinds
        .bind("Ctrl+s", KeyboardAction::Save)
        .expect("couldn't bind Ctrl+s");
    keybinds
        .bind("Ctrl+z", KeyboardAction::Content(TextEdit::Undo))
        .expect("couldn't bind Ctrl+z");
    keybinds
        .bind("Ctrl+Z", KeyboardAction::Content(TextEdit::Redo))
        .expect("couldn't bind Ctrl+Z");
    keybinds
        .bind(
            "Ctrl+Backspace",
            KeyboardAction::Content(TextEdit::BackspaceWord),
        )
        .expect("couldn't bind Ctrl+Backspace");
    keybinds
        .bind(
            "Ctrl+Shift+Backspace",
            KeyboardAction::Content(TextEdit::BackspaceSentence),
        )
        .expect("couldn't bind Ctrl+Shift+Backspace");
    keybinds
        .bind("Ctrl+Delete", KeyboardAction::Content(TextEdit::DeleteWord))
        .expect("couldn't bind Ctrl+Delete");
    keybinds
        .bind(
            "Ctrl+Shift+Delete",
            KeyboardAction::Content(TextEdit::DeleteSentence),
        )
        .expect("couldn't bind Ctrl+Shift+Delete");
    keybinds
        .bind("Ctrl+d", KeyboardAction::Debug)
        .expect("couldn't bind Ctrl+d");
    keybinds
        .bind(
            "Ctrl+Up",
            KeyboardAction::Content(TextEdit::JumpToContentStart),
        )
        .expect("couldn't bind Ctrl+Up");
    keybinds
        .bind(
            "Ctrl+Down",
            KeyboardAction::Content(TextEdit::JumpToContentEnd),
        )
        .expect("couldn't bind Ctrl+Down");

    keybinds
}
