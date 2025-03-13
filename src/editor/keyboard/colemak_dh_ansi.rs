use super::{
    KeyCode::{self, *},
    KeyEvent, KeyboardLayout, MATRIX_SIZE, Modifiers, generic_map_keycode,
};

/// A Colemak-DH Ansiolinear keyboard.
#[derive(Debug)]
pub struct ColemakDhAnsi;

#[rustfmt::skip]
static KEYCODE_INDEX: [KeyCode; MATRIX_SIZE] = [
    Unknown,
    N1, N2, N3, N4, N5, N6, N7, N8, N9, N0, Minus, Equal, BSlash, Grave,
    Q, W, F, P, B, J, L, U, Y, SColon, LBracket, RBracket,
      A, R, S, T, G, M, N, E, I, O, Quote,
        X, C, D, V, Z, K, H, Comma, Dot, Slash, Space,
    Esc, Enter, Del, Backspace, Tab, Left, Right, Up, Down, Home, End,
    PageUp, PageDown, NumLock,
];

#[rustfmt::skip]
static UNICODE_MAP: [char; MATRIX_SIZE] = [
    '�',
    '1', '2', '3', '4', '5', '6', '7', '8', '9', '0', '-', '=', '\\', '`',
    'q', 'w', 'f', 'p', 'b', 'j', 'l', 'u', 'y', ';', '[', ']',
    'a', 'r', 's', 't', 'g', 'm', 'n', 'e', 'i', 'o', '\'',
    'x', 'c', 'd', 'v', 'z', 'k', 'h', ',', '.', '/', ' ',
    '�', '�', '�', '�', '�', '�', '�', '�', '�', '�',
    '�', '�', '�', '�',
];

#[rustfmt::skip]
static SHIFT_MAP: [char; MATRIX_SIZE] = [
    '�',
    '!', '@', '#', '$', '%', '^', '&', '*', '(', ')', '_', '+', '|', '~',
    'Q', 'W', 'F', 'P', 'B', 'J', 'L', 'U', 'Y', ':', '{', '}',
    'A', 'R', 'S', 'T', 'G', 'M', 'N', 'E', 'I', 'O', '"',
    'X', 'C', 'D', 'V', 'Z', 'K', 'H', '<', '>', '?', ' ',
    '�', '�', '�', '�', '�', '�', '�', '�', '�', '�',
    '�', '�', '�', '�',
];

impl KeyboardLayout for ColemakDhAnsi {
    fn map_with_mod(&self, keycode: KeyCode, modifiers: Modifiers) -> KeyEvent {
        generic_map_keycode(&KEYCODE_INDEX, &UNICODE_MAP, &SHIFT_MAP, keycode, modifiers)
    }
}
