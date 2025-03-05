use super::{
    generic_map_keycode,
    KeyCode::{self, *},
    KeyEvent, KeyboardLayout, Modifiers, MATRIX_SIZE,
};

/// A Colemak keyboard.
#[derive(Debug)]
pub struct Colemak;

#[rustfmt::skip]
static KEYCODE_INDEX: [KeyCode; MATRIX_SIZE] = [
    Unknown,
    N1, N2, N3, N4, N5, N6, N7, N8, N9, N0, Minus, Equal, BSlash, Grave,
    Q, W, F, P, G, J, L, U, Y, SColon, LBracket, RBracket,
      A, R, S, T, D, H, N, E, I, O, Quote,
        Z, X, C, V, B, K, M, Comma, Dot, Slash, Space,
    Esc, Enter, Del, Backspace, Tab, Left, Right, Up, Down, Home, End,
    PageUp, PageDown, NumLock,
];

#[rustfmt::skip]
static UNICODE_MAP: [char; MATRIX_SIZE] = [
    '�',
    '1', '2', '3', '4', '5', '6', '7', '8', '9', '0', '-', '=', '\\', '`',
    'q', 'w', 'f', 'p', 'g', 'j', 'l', 'u', 'y', ';', '[', ']',
    'a', 'r', 's', 't', 'd', 'h', 'n', 'e', 'i', 'o', '\'',
    'z', 'x', 'c', 'v', 'b', 'k', 'm', ',', '.', '/', ' ',
    '�', '�', '�', '�', '�', '�', '�', '�', '�', '�',
    '�', '�', '�', '�',
];

#[rustfmt::skip]
static SHIFT_MAP: [char; MATRIX_SIZE] = [
    '�',
    '!', '@', '#', '$', '%', '^', '&', '*', '(', ')', '_', '+', '|', '~',
    'Q', 'W', 'F', 'P', 'G', 'J', 'L', 'U', 'Y', ':', '{', '}',
    'A', 'R', 'S', 'T', 'D', 'H', 'N', 'E', 'I', 'O', '"',
    'Z', 'X', 'C', 'V', 'B', 'K', 'M', '<', '>', '?', ' ',
    '�', '�', '�', '�', '�', '�', '�', '�', '�', '�',
    '�', '�', '�', '�',
];

impl KeyboardLayout for Colemak {
    fn map_with_mod(&self, keycode: KeyCode, modifiers: Modifiers) -> KeyEvent {
        generic_map_keycode(&KEYCODE_INDEX, &UNICODE_MAP, &SHIFT_MAP, keycode, modifiers)
    }
}
