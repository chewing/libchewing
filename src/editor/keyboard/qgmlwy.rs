use super::{
    KeyCode::{self, *},
    KeyEvent, KeyboardLayout, MATRIX_SIZE, Modifiers, generic_map_keycode,
};

/// A CARPALX keyboard variant (QGMLWY).
#[derive(Debug)]
pub struct Qgmlwy;

#[rustfmt::skip]
static KEYCODE_INDEX: [KeyCode; MATRIX_SIZE] = [
    Unknown,
    N1, N2, N3, N4, N5, N6, N7, N8, N9, N0, Minus, Equal, BSlash, Grave,
    Q, G, M, L, W, Y, F, U, B, SColon, LBracket, RBracket,
      D, S, T, N, R, I, A, E, O, H, Quote,
        Z, X, C, V, J, K, P, Comma, Dot, Slash, Space,
    Esc, Enter, Del, Backspace, Tab, Left, Right, Up, Down, Home, End,
    PageUp, PageDown, NumLock,
];

#[rustfmt::skip]
static UNICODE_MAP: [char; MATRIX_SIZE] = [
    '�',
    '1', '2', '3', '4', '5', '6', '7', '8', '9', '0', '-', '=', '\\', '`',
    'q', 'g', 'm', 'l', 'w', 'y', 'f', 'u', 'b', ';', '[', ']',
    'd', 's', 't', 'n', 'r', 'i', 'a', 'e', 'o', 'h', '\'',
    'z', 'x', 'c', 'v', 'j', 'k', 'p', ',', '.', '/', ' ',
    '�', '�', '�', '�', '�', '�', '�', '�', '�', '�',
    '�', '�', '�', '�',
];

#[rustfmt::skip]
static SHIFT_MAP: [char; MATRIX_SIZE] = [
    '�',
    '!', '@', '#', '$', '%', '^', '&', '*', '(', ')', '_', '+', '|', '~',
    'Q', 'G', 'M', 'L', 'W', 'Y', 'F', 'U', 'B', ':', '{', '}',
    'D', 'S', 'T', 'N', 'R', 'I', 'A', 'E', 'O', 'H', '"',
    'Z', 'X', 'C', 'V', 'J', 'K', 'P', '<', '>', '?', ' ',
    '�', '�', '�', '�', '�', '�', '�', '�', '�', '�',
    '�', '�', '�', '�',
];

impl KeyboardLayout for Qgmlwy {
    fn map_with_mod(&self, keycode: KeyCode, modifiers: Modifiers) -> KeyEvent {
        generic_map_keycode(&KEYCODE_INDEX, &UNICODE_MAP, &SHIFT_MAP, keycode, modifiers)
    }
}
