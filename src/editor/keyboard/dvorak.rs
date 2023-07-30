use super::{
    generic_map_keycode,
    KeyCode::{self, *},
    KeyEvent, KeyboardLayout, Modifiers, MATRIX_SIZE,
};

/// A Dvorak keyboard.
#[derive(Debug)]
pub struct Dvorak;

#[rustfmt::skip]
static KEYCODE_INDEX: [KeyCode; MATRIX_SIZE] = [
    Unknown,
    N1, N2, N3, N4, N5, N6, N7, N8, N9, N0, LBracket, RBracket, BSlash, Grave,
      Quote, Comma, Dot, P, Y, F, G, C, R, L, Slash, Equal,
        A, O, E, U, I, D, H, T, N, S, Minus,
          SColon, Q, J, K, X, B, M, W, V, Z, Space,
    Esc, Enter, Del, Backspace, Tab, Left, Right, Up, Down, Home, End,
    PageUp, PageDown, NumLock,
];

#[rustfmt::skip]
static UNICODE_MAP: [char; MATRIX_SIZE] = [
    '�',
    '1', '2', '3', '4', '5', '6', '7', '8', '9', '0', '[', ']', '\\', '`',
    '\'', ',', '.', 'p', 'y', 'f', 'g', 'c', 'r', 'l', '/', '=',
    'a', 'o', 'e', 'u', 'i', 'd', 'h', 't', 'n', 's', '-',
    ';', 'q', 'j', 'k', 'x', 'b', 'm', 'w', 'v', 'z', ' ',
    '�', '�', '�', '�', '�', '�', '�', '�', '�', '�',
    '�', '�', '�', '�',
];

#[rustfmt::skip]
static SHIFT_MAP: [char; MATRIX_SIZE] = [
    '�',
    '!', '@', '#', '$', '%', '^', '&', '*', '(', ')', '{', '}', '|', '~',
    '"', '<', '>', 'P', 'Y', 'F', 'G', 'C', 'R', 'L', '?', '+',
    'A', 'O', 'E', 'U', 'I', 'D', 'H', 'T', 'N', 'S', '_',
    ':', 'Q', 'J', 'K', 'X', 'B', 'M', 'W', 'V', 'Z', ' ',
    '�', '�', '�', '�', '�', '�', '�', '�', '�', '�',
    '�', '�', '�', '�',
];

impl KeyboardLayout for Dvorak {
    fn map_with_mod(&self, keycode: KeyCode, modifiers: Modifiers) -> KeyEvent {
        generic_map_keycode(&KEYCODE_INDEX, &UNICODE_MAP, &SHIFT_MAP, keycode, modifiers)
    }
}
