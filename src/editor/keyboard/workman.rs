use super::{
    KeyCode::{self, *},
    KeyEvent, KeyboardLayout, MATRIX_SIZE, Modifiers, generic_map_keycode,
};

/// A Workman keyboard.
#[derive(Debug)]
pub struct Workman;

#[rustfmt::skip]
pub(crate) static KEYCODE_INDEX: [KeyCode; MATRIX_SIZE] = [
    Unknown,
    N1, N2, N3, N4, N5, N6, N7, N8, N9, N0, Minus, Equal, BSlash, Grave,
      Q, D, R, W, B, J, F, U, P, SColon, LBracket, RBracket,
        A, S, H, T, G, Y, N, E, O, I, Quote,
          Z, X, M, C, V, K, L, Comma, Dot, Slash, Space,
    Esc, Enter, Del, Backspace, Tab, Left, Right, Up, Down, Home, End,
    PageUp, PageDown, NumLock,
];

#[rustfmt::skip]
pub(crate) static UNICODE_MAP: [char; MATRIX_SIZE] = [
    '�',
    '1', '2', '3', '4', '5', '6', '7', '8', '9', '0', '-', '=', '\\', '`',
    'q', 'd', 'r', 'w', 'b', 'j', 'f', 'u', 'p', ';', '[', ']',
    'a', 's', 'h', 't', 'g', 'y', 'n', 'e', 'o', 'i', '\'',
    'z', 'x', 'm', 'c', 'v', 'k', 'l', ',', '.', '/', ' ',
    '�', '�', '�', '�', '�', '�', '�', '�', '�', '�',
    '�', '�', '�', '�',
];

#[rustfmt::skip]
pub(crate) static SHIFT_MAP: [char; MATRIX_SIZE] = [
    '�',
    '!', '@', '#', '$', '%', '^', '&', '*', '(', ')', '_', '+', '|', '~',
    'Q', 'D', 'R', 'W', 'B', 'J', 'F', 'U', 'P', ':', '{', '}',
    'A', 'S', 'H', 'T', 'G', 'Y', 'N', 'E', 'O', 'I', '"',
    'Z', 'X', 'M', 'C', 'V', 'K', 'L', '<', '>', '?', ' ',
    '�', '�', '�', '�', '�', '�', '�', '�', '�', '�',
    '�', '�', '�', '�',
];

impl KeyboardLayout for Workman {
    fn map_with_mod(&self, keycode: KeyCode, modifiers: Modifiers) -> KeyEvent {
        generic_map_keycode(&KEYCODE_INDEX, &UNICODE_MAP, &SHIFT_MAP, keycode, modifiers)
    }
}
