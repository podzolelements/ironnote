use copypasta::{ClipboardContext, ClipboardProvider, x11_clipboard::X11ClipboardContext};
use std::sync::{LazyLock, RwLock};

/// global clipboard, interfaced through read/write_clipboard()
static CLIPBOARD: LazyLock<RwLock<X11ClipboardContext>> =
    LazyLock::new(|| RwLock::new(ClipboardContext::new().expect("couldn't get clipboard")));

/// returns the current contents of the clipboard
pub fn read_clipboard() -> String {
    let mut clipboard = CLIPBOARD
        .write()
        .expect("couldn't get clipboard write lock");

    clipboard
        .get_contents()
        .expect("couldn't read clipboard contents")
}

/// writes the provided string into the system's clipboard
pub fn write_clipboard(new_clipboard_contents: String) {
    let mut clipboard = CLIPBOARD
        .write()
        .expect("couldn't get clipboard write lock");

    clipboard
        .set_contents(new_clipboard_contents)
        .expect("couldn't write to clipboard");
}
