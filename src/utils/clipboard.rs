use arboard::Clipboard;
use std::sync::{LazyLock, RwLock};

/// global clipboard, interfaced through read/write_clipboard()
static CLIPBOARD: LazyLock<RwLock<Clipboard>> =
    LazyLock::new(|| RwLock::new(Clipboard::new().expect("couldn't get clipboard")));

/// returns the current contents of the clipboard
pub fn read_clipboard() -> String {
    let mut clipboard = CLIPBOARD
        .write()
        .expect("couldn't get clipboard write lock");

    clipboard
        .get_text()
        .expect("couldn't read clipboard contents")
}

/// writes the provided string into the system's clipboard
pub fn write_clipboard(new_clipboard_contents: String) {
    let mut clipboard = CLIPBOARD
        .write()
        .expect("couldn't get clipboard write lock");

    clipboard
        .set_text(new_clipboard_contents)
        .expect("couldn't write to clipboard");
}
