use std::fmt::{Display, Write};

use thiserror::Error;

/// The category of the phonetic symbols
///
/// Zhuyin, or Bopomofo, consists of 37 letters and 4 tone marks. They are
/// categorized into one of the four categories:
///
/// 1. Initial sounds: ㄅㄆㄇㄈㄉㄊㄋㄌㄍㄎㄏㄐㄑㄒㄓㄔㄕㄖㄗㄘㄙ
/// 2. Medial glides: ㄧㄨㄩ
/// 3. Rimes: ㄚㄛㄜㄝㄞㄟㄠㄡㄢㄣㄤㄥㄦ
/// 4. Tonal marks: ˙ˊˇˋ
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BopomofoKind {
    Initial = 0,
    Medial,
    Rime,
    Tone,
}

/// Zhuyin Fuhao, often shortened as zhuyin and commonly called bopomofo
///
/// <https://simple.m.wikipedia.org/wiki/Zhuyin>
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Bopomofo {
    /// ㄅ
    B = 0,
    /// ㄆ
    P,
    /// ㄇ
    M,
    /// ㄈ
    F,
    /// ㄉ
    D,
    /// ㄊ
    T,
    /// ㄋ
    N,
    /// ㄌ
    L,
    /// ㄍ
    G,
    /// ㄎ
    K,
    /// ㄏ
    H,
    /// ㄐ
    J,
    /// ㄑ
    Q,
    /// ㄒ
    X,
    /// ㄓ
    ZH,
    /// ㄔ
    CH,
    /// ㄕ
    SH,
    /// ㄖ
    R,
    /// ㄗ
    Z,
    /// ㄘ
    C,
    /// ㄙ
    S,
    /// 一
    I,
    /// ㄨ
    U,
    /// ㄩ
    IU,
    /// ㄚ
    A,
    /// ㄛ
    O,
    /// ㄜ
    E,
    /// ㄝ
    EH,
    /// ㄞ
    AI,
    /// ㄟ
    EI,
    /// ㄠ
    AU,
    /// ㄡ
    OU,
    /// ㄢ
    AN,
    /// ㄣ
    EN,
    /// ㄤ
    ANG,
    /// ㄥ
    ENG,
    /// ㄦ
    ER,
    /// ˙
    TONE5,
    /// ˊ
    TONE2,
    /// ˇ
    TONE3,
    /// ˋ
    TONE4,
    /// ˉ
    TONE1,
}

use Bopomofo::*;

const INITIAL_MAP: [Bopomofo; 21] = [
    B, P, M, F, D, T, N, L, G, K, H, J, Q, X, ZH, CH, SH, R, Z, C, S,
];
const MEDIAL_MAP: [Bopomofo; 3] = [I, U, IU];
const RIME_MAP: [Bopomofo; 13] = [A, O, E, EH, AI, EI, AU, OU, AN, EN, ANG, ENG, ER];
const TONE_MAP: [Bopomofo; 4] = [TONE5, TONE2, TONE3, TONE4];

impl Bopomofo {
    pub const fn kind(&self) -> BopomofoKind {
        match self {
            B | P | M | F | D | T | N | L | G | K | H | J | Q | X | ZH | CH | SH | R | Z | C
            | S => BopomofoKind::Initial,
            I | U | IU => BopomofoKind::Medial,
            A | O | E | EH | AI | EI | AU | OU | AN | EN | ANG | ENG | ER => BopomofoKind::Rime,
            TONE1 | TONE2 | TONE3 | TONE4 | TONE5 => BopomofoKind::Tone,
        }
    }
    pub const fn from_initial(index: u16) -> Result<Bopomofo, ParseBopomofoError> {
        if index < 1 || (index - 1) as usize >= INITIAL_MAP.len() {
            return Err(ParseBopomofoError {
                kind: ParseBopomofoErrorKind::IndexOutOfRange,
            });
        }
        Ok(INITIAL_MAP[(index - 1) as usize])
    }
    pub const fn from_medial(index: u16) -> Result<Bopomofo, ParseBopomofoError> {
        if index < 1 || (index - 1) as usize >= MEDIAL_MAP.len() {
            return Err(ParseBopomofoError {
                kind: ParseBopomofoErrorKind::IndexOutOfRange,
            });
        }
        Ok(MEDIAL_MAP[(index - 1) as usize])
    }
    pub const fn from_rime(index: u16) -> Result<Bopomofo, ParseBopomofoError> {
        if index < 1 || (index - 1) as usize >= RIME_MAP.len() {
            return Err(ParseBopomofoError {
                kind: ParseBopomofoErrorKind::IndexOutOfRange,
            });
        }
        Ok(RIME_MAP[(index - 1) as usize])
    }
    pub const fn from_tone(index: u16) -> Result<Bopomofo, ParseBopomofoError> {
        if index < 1 || (index - 1) as usize >= TONE_MAP.len() {
            return Err(ParseBopomofoError {
                kind: ParseBopomofoErrorKind::IndexOutOfRange,
            });
        }
        Ok(TONE_MAP[(index - 1) as usize])
    }

    pub fn initial_index(&self) -> u16 {
        (INITIAL_MAP.iter().position(|b| b == self).unwrap() + 1) as u16
    }
    pub fn medial_index(&self) -> u16 {
        (MEDIAL_MAP.iter().position(|b| b == self).unwrap() + 1) as u16
    }
    pub fn rime_index(&self) -> u16 {
        (RIME_MAP.iter().position(|b| b == self).unwrap() + 1) as u16
    }
    pub fn tone_index(&self) -> u16 {
        (TONE_MAP.iter().position(|b| b == self).unwrap() + 1) as u16
    }
}

#[derive(Error, Debug)]
#[error("parse bopomofo error: {:?}", kind)]
pub struct ParseBopomofoError {
    pub kind: ParseBopomofoErrorKind,
}

#[derive(Debug)]
pub enum ParseBopomofoErrorKind {
    UnknownSymbol,
    IndexOutOfRange,
}

impl Display for Bopomofo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_char((*self).into())
    }
}

impl From<Bopomofo> for char {
    fn from(bopomofo: Bopomofo) -> Self {
        match bopomofo {
            Bopomofo::B => 'ㄅ',
            Bopomofo::P => 'ㄆ',
            Bopomofo::M => 'ㄇ',
            Bopomofo::F => 'ㄈ',
            Bopomofo::D => 'ㄉ',
            Bopomofo::T => 'ㄊ',
            Bopomofo::N => 'ㄋ',
            Bopomofo::L => 'ㄌ',
            Bopomofo::G => 'ㄍ',
            Bopomofo::K => 'ㄎ',
            Bopomofo::H => 'ㄏ',
            Bopomofo::J => 'ㄐ',
            Bopomofo::Q => 'ㄑ',
            Bopomofo::X => 'ㄒ',
            Bopomofo::ZH => 'ㄓ',
            Bopomofo::CH => 'ㄔ',
            Bopomofo::SH => 'ㄕ',
            Bopomofo::R => 'ㄖ',
            Bopomofo::Z => 'ㄗ',
            Bopomofo::C => 'ㄘ',
            Bopomofo::S => 'ㄙ',
            Bopomofo::A => 'ㄚ',
            Bopomofo::O => 'ㄛ',
            Bopomofo::E => 'ㄜ',
            Bopomofo::EH => 'ㄝ',
            Bopomofo::AI => 'ㄞ',
            Bopomofo::EI => 'ㄟ',
            Bopomofo::AU => 'ㄠ',
            Bopomofo::OU => 'ㄡ',
            Bopomofo::AN => 'ㄢ',
            Bopomofo::EN => 'ㄣ',
            Bopomofo::ANG => 'ㄤ',
            Bopomofo::ENG => 'ㄥ',
            Bopomofo::ER => 'ㄦ',
            Bopomofo::I => 'ㄧ',
            Bopomofo::U => 'ㄨ',
            Bopomofo::IU => 'ㄩ',
            Bopomofo::TONE1 => 'ˉ',
            Bopomofo::TONE5 => '˙',
            Bopomofo::TONE2 => 'ˊ',
            Bopomofo::TONE3 => 'ˇ',
            Bopomofo::TONE4 => 'ˋ',
        }
    }
}

impl TryFrom<char> for Bopomofo {
    type Error = ParseBopomofoError;

    fn try_from(c: char) -> Result<Bopomofo, ParseBopomofoError> {
        match c {
            'ㄅ' => Ok(Bopomofo::B),
            'ㄆ' => Ok(Bopomofo::P),
            'ㄇ' => Ok(Bopomofo::M),
            'ㄈ' => Ok(Bopomofo::F),
            'ㄉ' => Ok(Bopomofo::D),
            'ㄊ' => Ok(Bopomofo::T),
            'ㄋ' => Ok(Bopomofo::N),
            'ㄌ' => Ok(Bopomofo::L),
            'ㄍ' => Ok(Bopomofo::G),
            'ㄎ' => Ok(Bopomofo::K),
            'ㄏ' => Ok(Bopomofo::H),
            'ㄐ' => Ok(Bopomofo::J),
            'ㄑ' => Ok(Bopomofo::Q),
            'ㄒ' => Ok(Bopomofo::X),
            'ㄓ' => Ok(Bopomofo::ZH),
            'ㄔ' => Ok(Bopomofo::CH),
            'ㄕ' => Ok(Bopomofo::SH),
            'ㄖ' => Ok(Bopomofo::R),
            'ㄗ' => Ok(Bopomofo::Z),
            'ㄘ' => Ok(Bopomofo::C),
            'ㄙ' => Ok(Bopomofo::S),
            'ㄚ' => Ok(Bopomofo::A),
            'ㄛ' => Ok(Bopomofo::O),
            'ㄜ' => Ok(Bopomofo::E),
            'ㄝ' => Ok(Bopomofo::EH),
            'ㄞ' => Ok(Bopomofo::AI),
            'ㄟ' => Ok(Bopomofo::EI),
            'ㄠ' => Ok(Bopomofo::AU),
            'ㄡ' => Ok(Bopomofo::OU),
            'ㄢ' => Ok(Bopomofo::AN),
            'ㄣ' => Ok(Bopomofo::EN),
            'ㄤ' => Ok(Bopomofo::ANG),
            'ㄥ' => Ok(Bopomofo::ENG),
            'ㄦ' => Ok(Bopomofo::ER),
            'ㄧ' => Ok(Bopomofo::I),
            'ㄨ' => Ok(Bopomofo::U),
            'ㄩ' => Ok(Bopomofo::IU),
            'ˉ' => Ok(Bopomofo::TONE1),
            '˙' => Ok(Bopomofo::TONE5),
            'ˊ' => Ok(Bopomofo::TONE2),
            'ˇ' => Ok(Bopomofo::TONE3),
            'ˋ' => Ok(Bopomofo::TONE4),
            _ => Err(ParseBopomofoError {
                kind: ParseBopomofoErrorKind::UnknownSymbol,
            }),
        }
    }
}
