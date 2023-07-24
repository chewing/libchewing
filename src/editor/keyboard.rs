//! Keyboard layout conversion
//!
//! People usually practice Zhuyin input method independently from practicing
//! English typing, they acquire different muscle memory. This module provides APIs
//! to map different English layouts to layout independent key indexes that can be
//! used to drive the phonetic conversion engines.

mod dvorak;
mod qgmlwy;
mod qwerty;

pub use dvorak::Dvorak;
pub use qgmlwy::Qgmlwy;
pub use qwerty::Qwerty;

const MATRIX_SIZE: usize = 63;

/// The set of modifier keys you have on a keyboard.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Modifiers {
    /// Any shift key is down
    pub shift: bool,
    /// Any control key is down
    pub ctrl: bool,
    /// The caps lock toggle is on
    pub capslock: bool,
}

fn generic_map_keycode(
    keycode_index: &[KeyCode; MATRIX_SIZE],
    unicode_map: &[char; MATRIX_SIZE],
    shift_map: &[char; MATRIX_SIZE],
    keycode: KeyCode,
    modifiers: Modifiers,
) -> KeyEvent {
    let index = keycode_index
        .iter()
        .position(|key| *key == keycode)
        .expect("invalid keycode");
    let unicode = if modifiers.capslock || modifiers.shift {
        shift_map[index]
    } else {
        unicode_map[index]
    };
    KeyEvent {
        index: INDEX_MAP[index],
        code: keycode,
        unicode,
        modifiers,
    }
}

/// Describe a Keyboard Layout
pub trait KeyboardLayout {
    /// Map the keycode to a key event according to the keyboard layout
    fn map_keycode(&self, keycode: KeyCode, modifiers: Modifiers) -> KeyEvent;
    /// Map the ascii to keycode then to a key event
    fn map_ascii(&self, ascii: u8, modifiers: Modifiers) -> KeyEvent {
        let keycode = KEYCODE_MAP
            .iter()
            .find(|item| item.0 == ascii)
            .map_or(Unknown, |item| item.1);
        self.map_keycode(keycode, modifiers)
    }
}

/// Layout independent key index
///
/// TODO: refactor this to not use enum?
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy, PartialEq)]
#[rustfmt::skip]
pub enum KeyIndex {
    K0 = 0,
//  1   2   3   4   5   6   7   8   9   0    -    =    \    `
    K1, K2, K3, K4, K5, K6, K7, K8, K9, K10, K11, K12, K13, K14,
//    Q    W    E    R    T    Y    U    I    O    P    [    ]
      K15, K16, K17, K18, K19, K20, K21, K22, K23, K24, K25, K26,
//      A    S    D    F    G    H    J    K    L    ;   '
        K27, K28, K29, K30, K31, K32, K33, K34, K35, K36, K37,
//        Z    X    C    V    B    N    M    ,    .    /    SPC
          K38, K39, K40, K41, K42, K43, K44, K45, K46, K47, K48,
//  Other
    K49, K50, K51, K52, K53, K54, K55, K56, K57, K58, K59, K60,
    K61, K62
}

#[rustfmt::skip]
static INDEX_MAP: [KeyIndex; MATRIX_SIZE] = [
    K0,
    K1, K2, K3, K4, K5, K6, K7, K8, K9, K10, K11, K12, K13, K14,
      K15, K16, K17, K18, K19, K20, K21, K22, K23, K24, K25, K26,
        K27, K28, K29, K30, K31, K32, K33, K34, K35, K36, K37,
          K38, K39, K40, K41, K42, K43, K44, K45, K46, K47, K48,
    K49, K50, K51, K52, K53, K54, K55, K56, K57, K58, K59, K60,
    K61, K62,
];

/// USB HID KeyCodes
///
/// TODO: refactor this to not use enum?
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy, PartialEq)]
#[rustfmt::skip]
pub enum KeyCode {
    Unknown = 0,
    N1, N2, N3, N4, N5, N6, N7, N8, N9, N0, Minus, Equal, BSlash, Grave,
      Q, W, E, R, T, Y, U, I, O, P, LBracket, RBracket,
       A, S, D, F, G, H, J, K, L, SColon, Quote,
        Z, X, C, V, B, N, M, Comma, Dot, Slash, Space,
    Esc, Enter, Del, Backspace, Tab, Left, Right, Up, Down, Home, End,
    PageUp, PageDown, NumLock,
}

use KeyCode::*;
use KeyIndex::*;

/// Key processed by a keymap
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct KeyEvent {
    /// TODO: doc
    pub index: KeyIndex,
    /// TODO: doc
    pub code: KeyCode,
    /// TODO: doc
    pub unicode: char,
    /// TODO: doc
    pub modifiers: Modifiers,
}

macro_rules! keycode_map {
    ($($k:expr => $v:expr),* $(,)?) => {{
        [$(($k, $v),)*]
    }};
}

#[rustfmt::skip]
static KEYCODE_MAP: [(u8, KeyCode); 48] = keycode_map! {
  b'1' => N1, b'2' => N2, b'3' => N3, b'4' => N4, b'5' => N5,
  b'6' => N6, b'7' => N7, b'8' => N8, b'9' => N9, b'0' => N0,
  b'-' => Minus, b'=' => Equal, b'\\' => BSlash, b'`' => Grave,
  b'q' => Q, b'w' => W, b'e' => E, b'r' => R, b't' => T, b'y' => Y,
  b'u' => U, b'i' => I, b'o' => O, b'p' => P, b'[' => LBracket, b']' => RBracket,
  b'a' => A, b's' => S, b'd' => D, b'f' => F, b'g' => G, b'h' => H,
  b'j' => J, b'k' => K, b'l' => L, b';' => SColon, b'\'' => Quote,
  b'z' => Z, b'x' => X, b'c' => C, b'v' => V, b'b' => B, b'n' => N,
  b'm' => M, b',' => Comma, b'.' => Dot, b'/' => Slash, b' ' => Space
};
