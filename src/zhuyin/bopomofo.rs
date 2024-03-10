use std::{
    error::Error,
    fmt::{Display, Write},
};

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
    /// TODO: docs
    Initial = 0,
    /// TODO: docs
    Medial,
    /// TODO: docs
    Rime,
    /// TODO: docs
    Tone,
}

/// Zhuyin Fuhao, often shortened as zhuyin and commonly called bopomofo
///
/// TODO: refactor this to not use enum?
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
    /// TODO: docs
    pub const fn kind(&self) -> BopomofoKind {
        match self {
            B | P | M | F | D | T | N | L | G | K | H | J | Q | X | ZH | CH | SH | R | Z | C
            | S => BopomofoKind::Initial,
            I | U | IU => BopomofoKind::Medial,
            A | O | E | EH | AI | EI | AU | OU | AN | EN | ANG | ENG | ER => BopomofoKind::Rime,
            TONE1 | TONE2 | TONE3 | TONE4 | TONE5 => BopomofoKind::Tone,
        }
    }
    /// TODO: docs
    pub const fn from_initial(index: u16) -> Result<Bopomofo, ParseBopomofoError> {
        if index < 1 || (index - 1) as usize >= INITIAL_MAP.len() {
            return Err(ParseBopomofoError {
                kind: ParseBopomofoErrorKind::IndexOutOfRange,
            });
        }
        Ok(INITIAL_MAP[(index - 1) as usize])
    }
    /// TODO: docs
    pub const fn from_medial(index: u16) -> Result<Bopomofo, ParseBopomofoError> {
        if index < 1 || (index - 1) as usize >= MEDIAL_MAP.len() {
            return Err(ParseBopomofoError {
                kind: ParseBopomofoErrorKind::IndexOutOfRange,
            });
        }
        Ok(MEDIAL_MAP[(index - 1) as usize])
    }
    /// TODO: docs
    pub const fn from_rime(index: u16) -> Result<Bopomofo, ParseBopomofoError> {
        if index < 1 || (index - 1) as usize >= RIME_MAP.len() {
            return Err(ParseBopomofoError {
                kind: ParseBopomofoErrorKind::IndexOutOfRange,
            });
        }
        Ok(RIME_MAP[(index - 1) as usize])
    }
    /// TODO: docs
    pub const fn from_tone(index: u16) -> Result<Bopomofo, ParseBopomofoError> {
        if index < 1 || (index - 1) as usize >= TONE_MAP.len() {
            return Err(ParseBopomofoError {
                kind: ParseBopomofoErrorKind::IndexOutOfRange,
            });
        }
        Ok(TONE_MAP[(index - 1) as usize])
    }
    /// TODO: docs
    pub fn initial_index(&self) -> u16 {
        (INITIAL_MAP.iter().position(|b| b == self).unwrap() + 1) as u16
    }
    /// TODO: docs
    pub fn medial_index(&self) -> u16 {
        (MEDIAL_MAP.iter().position(|b| b == self).unwrap() + 1) as u16
    }
    /// TODO: docs
    pub fn rime_index(&self) -> u16 {
        (RIME_MAP.iter().position(|b| b == self).unwrap() + 1) as u16
    }
    /// TODO: docs
    pub fn tone_index(&self) -> u16 {
        (TONE_MAP.iter().position(|b| b == self).unwrap() + 1) as u16
    }
}

/// TODO: docs
/// TODO: refactor to enum?
#[derive(Debug)]
pub struct ParseBopomofoError {
    /// TODO: docs
    pub kind: ParseBopomofoErrorKind,
}

impl Display for ParseBopomofoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "parse bopomofo error: {:?}", self.kind)
    }
}

impl Error for ParseBopomofoError {}

/// TODO: docs
#[derive(Debug)]
pub enum ParseBopomofoErrorKind {
    /// TODO: docs
    UnknownSymbol,
    /// TODO: docs
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
            B => 'ㄅ',
            P => 'ㄆ',
            M => 'ㄇ',
            F => 'ㄈ',
            D => 'ㄉ',
            T => 'ㄊ',
            N => 'ㄋ',
            L => 'ㄌ',
            G => 'ㄍ',
            K => 'ㄎ',
            H => 'ㄏ',
            J => 'ㄐ',
            Q => 'ㄑ',
            X => 'ㄒ',
            ZH => 'ㄓ',
            CH => 'ㄔ',
            SH => 'ㄕ',
            R => 'ㄖ',
            Z => 'ㄗ',
            C => 'ㄘ',
            S => 'ㄙ',
            A => 'ㄚ',
            O => 'ㄛ',
            E => 'ㄜ',
            EH => 'ㄝ',
            AI => 'ㄞ',
            EI => 'ㄟ',
            AU => 'ㄠ',
            OU => 'ㄡ',
            AN => 'ㄢ',
            EN => 'ㄣ',
            ANG => 'ㄤ',
            ENG => 'ㄥ',
            ER => 'ㄦ',
            I => 'ㄧ',
            U => 'ㄨ',
            IU => 'ㄩ',
            TONE1 => 'ˉ',
            TONE5 => '˙',
            TONE2 => 'ˊ',
            TONE3 => 'ˇ',
            TONE4 => 'ˋ',
        }
    }
}

impl TryFrom<char> for Bopomofo {
    type Error = ParseBopomofoError;

    fn try_from(c: char) -> Result<Bopomofo, ParseBopomofoError> {
        match c {
            'ㄅ' => Ok(B),
            'ㄆ' => Ok(P),
            'ㄇ' => Ok(M),
            'ㄈ' => Ok(F),
            'ㄉ' => Ok(D),
            'ㄊ' => Ok(T),
            'ㄋ' => Ok(N),
            'ㄌ' => Ok(L),
            'ㄍ' => Ok(G),
            'ㄎ' => Ok(K),
            'ㄏ' => Ok(H),
            'ㄐ' => Ok(J),
            'ㄑ' => Ok(Q),
            'ㄒ' => Ok(X),
            'ㄓ' => Ok(ZH),
            'ㄔ' => Ok(CH),
            'ㄕ' => Ok(SH),
            'ㄖ' => Ok(R),
            'ㄗ' => Ok(Z),
            'ㄘ' => Ok(C),
            'ㄙ' => Ok(S),
            'ㄚ' => Ok(A),
            'ㄛ' => Ok(O),
            'ㄜ' => Ok(E),
            'ㄝ' => Ok(EH),
            'ㄞ' => Ok(AI),
            'ㄟ' => Ok(EI),
            'ㄠ' => Ok(AU),
            'ㄡ' => Ok(OU),
            'ㄢ' => Ok(AN),
            'ㄣ' => Ok(EN),
            'ㄤ' => Ok(ANG),
            'ㄥ' => Ok(ENG),
            'ㄦ' => Ok(ER),
            'ㄧ' => Ok(I),
            'ㄨ' => Ok(U),
            'ㄩ' => Ok(IU),
            'ˉ' => Ok(TONE1),
            '˙' => Ok(TONE5),
            'ˊ' => Ok(TONE2),
            'ˇ' => Ok(TONE3),
            'ˋ' => Ok(TONE4),
            _ => Err(ParseBopomofoError {
                kind: ParseBopomofoErrorKind::UnknownSymbol,
            }),
        }
    }
}
