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
/// keybindings get bound to these physical actions, representing what actually happens after a keybinding is triggered
pub enum KeyboardAction {
    Save,
    BackspaceWord,
    BackspaceSentence,
    DeleteWord,
    DeleteSentence,
    Undo,
    Redo,
    Debug,
    JumpToContentStart,
    JumpToContentEnd,
    Unbound(UnboundKey),
}

pub fn bind_keybinds() -> Keybinds<KeyboardAction> {
    let mut keybinds = Keybinds::default();

    keybinds
        .bind("Ctrl+s", KeyboardAction::Save)
        .expect("couldn't bind Ctrl+s");
    keybinds
        .bind("Ctrl+z", KeyboardAction::Undo)
        .expect("couldn't bind Ctrl+z");
    keybinds
        .bind("Ctrl+Z", KeyboardAction::Redo)
        .expect("couldn't bind Ctrl+Z");
    keybinds
        .bind("Ctrl+Backspace", KeyboardAction::BackspaceWord)
        .expect("couldn't bind Ctrl+Backspace");
    keybinds
        .bind("Ctrl+Shift+Backspace", KeyboardAction::BackspaceSentence)
        .expect("couldn't bind Ctrl+Shift+Backspace");
    keybinds
        .bind("Ctrl+Delete", KeyboardAction::DeleteWord)
        .expect("couldn't bind Ctrl+Delete");
    keybinds
        .bind("Ctrl+Shift+Delete", KeyboardAction::DeleteSentence)
        .expect("couldn't bind Ctrl+Shift+Delete");
    keybinds
        .bind("Ctrl+d", KeyboardAction::Debug)
        .expect("couldn't bind Ctrl+d");
    keybinds
        .bind("Ctrl+Up", KeyboardAction::JumpToContentStart)
        .expect("couldn't bind Ctrl+Up");
    keybinds
        .bind("Ctrl+Down", KeyboardAction::JumpToContentEnd)
        .expect("couldn't bind Ctrl+Down");

    keybinds
}
