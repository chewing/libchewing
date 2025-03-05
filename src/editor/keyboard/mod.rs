//! Keyboard layout conversion
//!
//! People usually practice Zhuyin input method independently from practicing
//! English typing, they acquire different muscle memory. This module provides APIs
//! to map different English layouts to layout independent key indexes that can be
//! used to drive the phonetic conversion engines.

mod colemak;
mod colemak_dh_ansi;
mod colemak_dh_orth;
mod dvorak;
mod dvorak_on_qwerty;
mod qgmlwy;
mod qwerty;
mod workman;

use core::fmt;

pub use colemak::Colemak;
pub use colemak_dh_ansi::ColemakDhAnsi;
pub use colemak_dh_orth::ColemakDhOrth;
pub use dvorak::Dvorak;
pub use dvorak_on_qwerty::DvorakOnQwerty;
pub use qgmlwy::Qgmlwy;
pub use qwerty::Qwerty;
pub use workman::Workman;

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
    /// The num lock toggle is on
    pub numlock: bool,
}
impl Modifiers {
    pub(crate) const fn new() -> Modifiers {
        Modifiers {
            shift: false,
            ctrl: false,
            capslock: false,
            numlock: false,
        }
    }
    pub const fn shift() -> Modifiers {
        Modifiers {
            shift: true,
            ctrl: false,
            capslock: false,
            numlock: false,
        }
    }
    pub const fn control() -> Modifiers {
        Modifiers {
            shift: false,
            ctrl: true,
            capslock: false,
            numlock: false,
        }
    }
    pub const fn capslock() -> Modifiers {
        Modifiers {
            shift: false,
            ctrl: false,
            capslock: true,
            numlock: false,
        }
    }
    pub const fn numlock() -> Modifiers {
        Modifiers {
            shift: false,
            ctrl: false,
            capslock: false,
            numlock: true,
        }
    }
    pub(crate) fn is_none(&self) -> bool {
        !self.shift && !self.ctrl && !self.capslock && !self.numlock
    }
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
    fn map_with_mod(&self, keycode: KeyCode, modifiers: Modifiers) -> KeyEvent;
    fn map(&self, keycode: KeyCode) -> KeyEvent {
        self.map_with_mod(keycode, Modifiers::default())
    }
    /// Map the ascii to keycode then to a key event
    fn map_ascii(&self, ascii: u8) -> KeyEvent {
        let item = KEYCODE_MAP
            .iter()
            .find(|item| item.0 == ascii)
            .map_or((Unknown, Modifiers::default()), |item| item.1);
        self.map_with_mod(item.0, item.1)
    }
    /// Map the ascii to keycode then to a key event with numlock on
    fn map_ascii_numlock(&self, ascii: u8) -> KeyEvent {
        let item = NUMLOCK_MAP
            .iter()
            .find(|item| item.0 == ascii)
            .map_or((Unknown, Modifiers::default()), |item| item.1);
        self.map_with_mod(item.0, item.1)
    }
}

#[derive(Debug)]
#[non_exhaustive]
pub enum AnyKeyboardLayout {
    Qwerty(Qwerty),
    Dvorak(Dvorak),
    DvorakOnQwerty(DvorakOnQwerty),
    Qgmlwy(Qgmlwy),
    Colemak(Colemak),
    ColemakDhAnsi(ColemakDhAnsi),
    ColemakDhOrth(ColemakDhOrth),
    Workman(Workman),
}

impl AnyKeyboardLayout {
    pub fn qwerty() -> AnyKeyboardLayout {
        AnyKeyboardLayout::Qwerty(Qwerty)
    }
    pub fn dvorak() -> AnyKeyboardLayout {
        AnyKeyboardLayout::Dvorak(Dvorak)
    }
    pub fn dvorak_on_qwerty() -> AnyKeyboardLayout {
        AnyKeyboardLayout::DvorakOnQwerty(DvorakOnQwerty)
    }
    pub fn qgmlwy() -> AnyKeyboardLayout {
        AnyKeyboardLayout::Qgmlwy(Qgmlwy)
    }
    pub fn colemak() -> AnyKeyboardLayout {
        AnyKeyboardLayout::Colemak(Colemak)
    }
    pub fn colemak_dh_ansi() -> AnyKeyboardLayout {
        AnyKeyboardLayout::ColemakDhAnsi(ColemakDhAnsi)
    }
    pub fn colemak_dh_orth() -> AnyKeyboardLayout {
        AnyKeyboardLayout::ColemakDhOrth(ColemakDhOrth)
    }
    pub fn workman() -> AnyKeyboardLayout {
        AnyKeyboardLayout::Workman(Workman)
    }
}

impl KeyboardLayout for AnyKeyboardLayout {
    fn map_with_mod(&self, keycode: KeyCode, modifiers: Modifiers) -> KeyEvent {
        match self {
            AnyKeyboardLayout::Qwerty(kb) => kb.map_with_mod(keycode, modifiers),
            AnyKeyboardLayout::Dvorak(kb) => kb.map_with_mod(keycode, modifiers),
            AnyKeyboardLayout::DvorakOnQwerty(kb) => kb.map_with_mod(keycode, modifiers),
            AnyKeyboardLayout::Qgmlwy(kb) => kb.map_with_mod(keycode, modifiers),
            AnyKeyboardLayout::Colemak(kb) => kb.map_with_mod(keycode, modifiers),
            AnyKeyboardLayout::ColemakDhAnsi(kb) => kb.map_with_mod(keycode, modifiers),
            AnyKeyboardLayout::ColemakDhOrth(kb) => kb.map_with_mod(keycode, modifiers),
            AnyKeyboardLayout::Workman(kb) => kb.map_with_mod(keycode, modifiers),
        }
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

impl KeyCode {
    pub const fn to_digit(self) -> Option<u8> {
        match self {
            code @ (N1 | N2 | N3 | N4 | N5 | N6 | N7 | N8 | N9 | N0) => Some(code as u8),
            _ => None,
        }
    }
    #[rustfmt::skip]
    pub const fn is_atoz(&self) -> bool {
        matches!(self, A|B|C|D|E|F|G|H|I|J|K|L|M|N|O|P|Q|R|S|T|U|V|W|X|Y|Z)
    }
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

impl KeyEvent {
    pub(crate) fn is_printable(&self) -> bool {
        // FIXME refactor
        self.unicode != '�'
    }
}

impl fmt::Display for KeyEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "key-{:?}-{:?}-{}-", self.index, self.code, self.unicode)?;
        if self.modifiers.capslock {
            write!(f, "C")?;
        }
        if self.modifiers.ctrl {
            write!(f, "c")?;
        }
        if self.modifiers.shift {
            write!(f, "S")?;
        }
        Ok(())
    }
}

macro_rules! keycode_map {
    ($($k:expr => $v:expr),* $(,)?) => {{
        [$(($k, $v),)*]
    }};
}

#[rustfmt::skip]
static KEYCODE_MAP: [(u8, (KeyCode, Modifiers)); 95] = keycode_map! {
  b'1' => (N1, Modifiers::new()),
  b'2' => (N2, Modifiers::new()),
  b'3' => (N3, Modifiers::new()),
  b'4' => (N4, Modifiers::new()),
  b'5' => (N5, Modifiers::new()),
  b'6' => (N6, Modifiers::new()),
  b'7' => (N7, Modifiers::new()),
  b'8' => (N8, Modifiers::new()),
  b'9' => (N9, Modifiers::new()),
  b'0' => (N0, Modifiers::new()),
  b'-' => (Minus, Modifiers::new()),
  b'=' => (Equal, Modifiers::new()),
  b'\\' => (BSlash, Modifiers::new()),
  b'`' => (Grave, Modifiers::new()),
  b'q' => (Q, Modifiers::new()),
  b'w' => (W, Modifiers::new()),
  b'e' => (E, Modifiers::new()),
  b'r' => (R, Modifiers::new()),
  b't' => (T, Modifiers::new()),
  b'y' => (Y, Modifiers::new()),
  b'u' => (U, Modifiers::new()),
  b'i' => (I, Modifiers::new()),
  b'o' => (O, Modifiers::new()),
  b'p' => (P, Modifiers::new()),
  b'[' => (LBracket, Modifiers::new()),
  b']' => (RBracket, Modifiers::new()),
  b'a' => (A, Modifiers::new()),
  b's' => (S, Modifiers::new()),
  b'd' => (D, Modifiers::new()),
  b'f' => (F, Modifiers::new()),
  b'g' => (G, Modifiers::new()),
  b'h' => (H, Modifiers::new()),
  b'j' => (J, Modifiers::new()),
  b'k' => (K, Modifiers::new()),
  b'l' => (L, Modifiers::new()),
  b';' => (SColon, Modifiers::new()),
  b'\'' => (Quote, Modifiers::new()),
  b'z' => (Z, Modifiers::new()),
  b'x' => (X, Modifiers::new()),
  b'c' => (C, Modifiers::new()),
  b'v' => (V, Modifiers::new()),
  b'b' => (B, Modifiers::new()),
  b'n' => (N, Modifiers::new()),
  b'm' => (M, Modifiers::new()),
  b',' => (Comma, Modifiers::new()),
  b'.' => (Dot, Modifiers::new()),
  b'/' => (Slash, Modifiers::new()),
  b' ' => (Space, Modifiers::new()),

  b'A' => (A, Modifiers::shift()),
  b'B' => (B, Modifiers::shift()),
  b'C' => (C, Modifiers::shift()),
  b'D' => (D, Modifiers::shift()),
  b'E' => (E, Modifiers::shift()),
  b'F' => (F, Modifiers::shift()),
  b'G' => (G, Modifiers::shift()),
  b'H' => (H, Modifiers::shift()),
  b'I' => (I, Modifiers::shift()),
  b'J' => (J, Modifiers::shift()),
  b'K' => (K, Modifiers::shift()),
  b'L' => (L, Modifiers::shift()),
  b'M' => (M, Modifiers::shift()),
  b'N' => (N, Modifiers::shift()),
  b'O' => (O, Modifiers::shift()),
  b'P' => (P, Modifiers::shift()),
  b'Q' => (Q, Modifiers::shift()),
  b'R' => (R, Modifiers::shift()),
  b'S' => (S, Modifiers::shift()),
  b'T' => (T, Modifiers::shift()),
  b'U' => (U, Modifiers::shift()),
  b'V' => (V, Modifiers::shift()),
  b'W' => (W, Modifiers::shift()),
  b'X' => (X, Modifiers::shift()),
  b'Y' => (Y, Modifiers::shift()),
  b'Z' => (Z, Modifiers::shift()),
  b'"' => (Quote, Modifiers::shift()),
  b'<' => (Comma, Modifiers::shift()),
  b'>' => (Dot, Modifiers::shift()),
  b'{' => (LBracket, Modifiers::shift()),
  b'}' => (RBracket, Modifiers::shift()),
  b'+' => (Equal, Modifiers::shift()),
  b'_' => (Minus, Modifiers::shift()),
  b':' => (SColon, Modifiers::shift()),
  b'~' => (Grave, Modifiers::shift()),
  b'!' => (N1, Modifiers::shift()),
  b'@' => (N2, Modifiers::shift()),
  b'#' => (N3, Modifiers::shift()),
  b'$' => (N4, Modifiers::shift()),
  b'%' => (N5, Modifiers::shift()),
  b'^' => (N6, Modifiers::shift()),
  b'&' => (N7, Modifiers::shift()),
  b'*' => (N8, Modifiers::shift()),
  b'(' => (N9, Modifiers::shift()),
  b')' => (N0, Modifiers::shift()),
  b'|' => (BSlash, Modifiers::shift()),
  b'?' => (Slash, Modifiers::shift()),
};

// FIXME should we map to real numlock keycode?
#[rustfmt::skip]
static NUMLOCK_MAP: [(u8, (KeyCode, Modifiers)); 15] = keycode_map! {
  b'1' => (N1, Modifiers::numlock()),
  b'2' => (N2, Modifiers::numlock()),
  b'3' => (N3, Modifiers::numlock()),
  b'4' => (N4, Modifiers::numlock()),
  b'5' => (N5, Modifiers::numlock()),
  b'6' => (N6, Modifiers::numlock()),
  b'7' => (N7, Modifiers::numlock()),
  b'8' => (N8, Modifiers::numlock()),
  b'9' => (N9, Modifiers::numlock()),
  b'0' => (N0, Modifiers::numlock()),
  b'+' => (Equal, Modifiers { shift: true, ctrl: false, capslock: false, numlock: true }),
  b'-' => (Minus, Modifiers::numlock()),
  b'*' => (N8, Modifiers { shift: true, ctrl: false, capslock: false, numlock: true }),
  b'/' => (Slash, Modifiers::numlock()),
  b'.' => (Dot, Modifiers::numlock()),
};
