//! Key symbols.
//!
//! Keysyms (short for "key symbol") are translated from keycodes via a
//! keymap. On different layout (qwerty, dvorak, etc.) all keyboards emit
//! the same keycodes but produce different keysyms after translation.

#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
pub enum Keysym {
    #[default]
    NoSymbol,
    /// Non-space printable characters
    Character(char),
    Space,
    Backspace,
    Tab,
    Return,
    Escape,
    Delete,
    Home,
    Left,
    Up,
    Right,
    Down,
    PageUp,
    PageDown,
    End,
    NumLock,
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    Shift,
    Control,
    CapsLock,
    Alt,
    Super,
}
