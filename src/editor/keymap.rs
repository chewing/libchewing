//! Keyboard layout conversion
//!
//! People usually practice Zhuyin input method independently from practicing
//! English typing, they acquire different muscle memory. This module provides APIs
//! to map different English layouts to layout independent key indexes that can be
//! used to drive the phonetic conversion engines.

/// Layout independent key index
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
          K38, K39, K40, K41, K42, K43, K44, K45, K46, K47, K48
}

/// USB HID KeyCodes
#[derive(Debug, Clone, Copy, PartialEq)]
#[rustfmt::skip]
pub enum KeyCode {
    Unknown = 0,
    N1, N2, N3, N4, N5, N6, N7, N8, N9, N0, Minus, Equal, BSlash, Grave,
      Q, W, E, R, T, Y, U, I, O, P, LBracket, RBracket,
       A, S, D, F, G, H, J, K, L, SColon, Quote,
        Z, X, C, V, B, N, M, Comma, Dot, Slash, Space
}

/// Key processed by a keymap
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct KeyEvent {
    pub index: KeyIndex,
    pub code: KeyCode,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Layout {
    name: &'static str,
    map: [KeyCode; 48],
}

pub trait Keymap {
    fn map_key(&self, input: KeyCode) -> KeyEvent;
}

#[derive(Debug)]
pub struct IdentityKeymap {
    inner: RemappingKeymap,
}

impl IdentityKeymap {
    pub fn new(source: Layout) -> IdentityKeymap {
        IdentityKeymap {
            inner: RemappingKeymap::new(source, source),
        }
    }
}

impl Keymap for IdentityKeymap {
    fn map_key(&self, input: KeyCode) -> KeyEvent {
        self.inner.map_key(input)
    }
}

#[derive(Debug)]
pub struct RemappingKeymap {
    source: Layout,
    target: Layout,
}

impl RemappingKeymap {
    pub fn new(source: Layout, target: Layout) -> RemappingKeymap {
        RemappingKeymap { source, target }
    }
}

impl Keymap for RemappingKeymap {
    fn map_key(&self, input: KeyCode) -> KeyEvent {
        let position = self
            .source
            .map
            .iter()
            .position(|&key| input == key)
            .expect("invalid keycode");
        let index = BLANK[position];
        let code = self.target.map[position];
        KeyEvent { index, code }
    }
}

#[rustfmt::skip]
const QWERTY_INDEX: [u8; 48] = [
    b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9', b'0', b'-', b'=', b'\\', b'`',
    b'q', b'w', b'e', b'r', b't', b'y', b'u', b'i', b'o', b'p', b'[', b']',
     b'a', b's', b'd', b'f', b'g', b'h', b'j', b'k', b'l', b';', b'\'',
      b'z', b'x', b'c', b'v', b'b', b'n', b'm', b',', b'.', b'/', b' '
];

pub trait KeyCodeFromQwerty {
    fn as_key_code(&self) -> Option<KeyCode>;
}

impl KeyCodeFromQwerty for u8 {
    fn as_key_code(&self) -> Option<KeyCode> {
        let position = QWERTY_INDEX.iter().position(|key| key == self);
        position.map(|pos| QWERTY.map[pos])
    }
}

use std::fmt::Debug;

use KeyCode::*;
use KeyIndex::*;

#[rustfmt::skip]
const BLANK: [KeyIndex; 48] = [
    K1, K2, K3, K4, K5, K6, K7, K8, K9, K10, K11, K12, K13, K14,
      K15, K16, K17, K18, K19, K20, K21, K22, K23, K24, K25, K26,
        K27, K28, K29, K30, K31, K32, K33, K34, K35, K36, K37,
          K38, K39, K40, K41, K42, K43, K44, K45, K46, K47, K48
];

#[rustfmt::skip]
pub const QWERTY: Layout = Layout {
    name: "QWERTY",
    map: [
        N1, N2, N3, N4, N5, N6, N7, N8, N9, N0, Minus, Equal, BSlash, Grave,
        Q, W, E, R, T, Y, U, I, O, P, LBracket, RBracket,
          A, S, D, F, G, H, J, K, L, SColon, Quote,
            Z, X, C, V, B, N, M, Comma, Dot, Slash, Space
    ],
};

#[rustfmt::skip]
pub const DVORAK: Layout = Layout {
    name: "DVORAK",
    map: [
        N1, N2, N3, N4, N5, N6, N7, N8, N9, N0, LBracket, RBracket, BSlash, Grave,
        Quote, Comma, Dot, P, Y, F, G, C, R, L, Slash, Equal,
          A, O, E, U, I, D, H, T, N, S, Minus,
            SColon, Q, J, K, X, B, M, W, V, Z, Space
    ],
};

#[rustfmt::skip]
pub const CARPALX: Layout = Layout {
    name: "CARPALX (QGMLWY)",
    map: [
        N1, N2, N3, N4, N5, N6, N7, N8, N9, N0, Minus, Equal, BSlash, Grave,
        Q, G, M, L, W, Y, F, U, B, SColon, LBracket, RBracket,
          D, S, T, N, R, I, A, E, O, H, Quote,
            Z, X, C, V, J, K, P, Comma, Dot, Slash, Space
    ],
};
