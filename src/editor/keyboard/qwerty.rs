use super::{
    generic_map_keycode,
    KeyCode::{self, *},
    KeyEvent, KeyboardLayout, Modifiers,
};

/// A standard keyboard.
#[derive(Debug)]
pub struct Qwerty;

#[rustfmt::skip]
static KEYCODE_INDEX: [KeyCode; 49] = [
    Unknown,
    N1, N2, N3, N4, N5, N6, N7, N8, N9, N0, Minus, Equal, BSlash, Grave,
    Q, W, E, R, T, Y, U, I, O, P, LBracket, RBracket,
      A, S, D, F, G, H, J, K, L, SColon, Quote,
        Z, X, C, V, B, N, M, Comma, Dot, Slash, Space
];

#[rustfmt::skip]
static UNICODE_MAP: [char; 49] = [
    '�',
   '1', '2', '3', '4', '5', '6', '7', '8', '9', '0', '-', '=', '\\', '`', 
   'q', 'w', 'e', 'r', 't', 'y', 'u', 'i', 'o', 'p', '[', ']', 
    'a', 's', 'd', 'f', 'g', 'h', 'j', 'k', 'l', ';', '\'', 
     'z', 'x', 'c', 'v', 'b', 'n', 'm', ',', '.', '/', ' '
];

#[rustfmt::skip]
static SHIFT_MAP: [char; 49] = [
    '�',
   '!', '@', '#', '$', '%', '^', '&', '*', '(', ')', '_', '+', '|', '~', 
   'Q', 'W', 'E', 'R', 'T', 'Y', 'U', 'I', 'O', 'P', '{', '}', 
    'A', 'S', 'D', 'F', 'G', 'H', 'J', 'K', 'L', ':', '"', 
     'Z', 'X', 'C', 'V', 'B', 'N', 'M', '<', '>', '?', ' '
];

impl KeyboardLayout for Qwerty {
    fn map_keycode(&self, keycode: KeyCode, modifiers: Modifiers) -> KeyEvent {
        generic_map_keycode(&KEYCODE_INDEX, &UNICODE_MAP, &SHIFT_MAP, keycode, modifiers)
    }
}
