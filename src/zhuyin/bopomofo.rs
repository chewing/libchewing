use std::{
    error::Error,
    fmt::{Display, Write},
    str::FromStr,
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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BopomofoKind {
    /// Initial sounds: ㄅㄆㄇㄈㄉㄊㄋㄌㄍㄎㄏㄐㄑㄒㄓㄔㄕㄖㄗㄘㄙ
    Initial,
    /// Medial glides: ㄧㄨㄩ
    Medial,
    /// Rimes: ㄚㄛㄜㄝㄞㄟㄠㄡㄢㄣㄤㄥㄦ
    Rime,
    /// Tonal marks: ˙ˊˇˋ
    Tone,
}

/// Zhuyin Fuhao, often shortened as zhuyin and commonly called bopomofo
///
/// <https://simple.m.wikipedia.org/wiki/Zhuyin>
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Bopomofo {
    /// Zhuyin Fuhao: ㄅ
    B,
    /// Zhuyin Fuhao: ㄆ
    P,
    /// Zhuyin Fuhao: ㄇ
    M,
    /// Zhuyin Fuhao: ㄈ
    F,
    /// Zhuyin Fuhao: ㄉ
    D,
    /// Zhuyin Fuhao: ㄊ
    T,
    /// Zhuyin Fuhao: ㄋ
    N,
    /// Zhuyin Fuhao: ㄌ
    L,
    /// Zhuyin Fuhao: ㄍ
    G,
    /// Zhuyin Fuhao: ㄎ
    K,
    /// Zhuyin Fuhao: ㄏ
    H,
    /// Zhuyin Fuhao: ㄐ
    J,
    /// Zhuyin Fuhao: ㄑ
    Q,
    /// Zhuyin Fuhao: ㄒ
    X,
    /// Zhuyin Fuhao: ㄓ
    ZH,
    /// Zhuyin Fuhao: ㄔ
    CH,
    /// Zhuyin Fuhao: ㄕ
    SH,
    /// Zhuyin Fuhao: ㄖ
    R,
    /// Zhuyin Fuhao: ㄗ
    Z,
    /// Zhuyin Fuhao: ㄘ
    C,
    /// Zhuyin Fuhao: ㄙ
    S,
    /// Zhuyin Fuhao: 一
    I,
    /// Zhuyin Fuhao: ㄨ
    U,
    /// Zhuyin Fuhao: ㄩ
    IU,
    /// Zhuyin Fuhao: ㄚ
    A,
    /// Zhuyin Fuhao: ㄛ
    O,
    /// Zhuyin Fuhao: ㄜ
    E,
    /// Zhuyin Fuhao: ㄝ
    EH,
    /// Zhuyin Fuhao: ㄞ
    AI,
    /// Zhuyin Fuhao: ㄟ
    EI,
    /// Zhuyin Fuhao: ㄠ
    AU,
    /// Zhuyin Fuhao: ㄡ
    OU,
    /// Zhuyin Fuhao: ㄢ
    AN,
    /// Zhuyin Fuhao: ㄣ
    EN,
    /// Zhuyin Fuhao: ㄤ
    ANG,
    /// Zhuyin Fuhao: ㄥ
    ENG,
    /// Zhuyin Fuhao: ㄦ
    ER,
    /// Tonal mark: ˙
    TONE5,
    /// Tonal mark: ˊ
    TONE2,
    /// Tonal mark: ˇ
    TONE3,
    /// Tonal mark: ˋ
    TONE4,
    /// Tonal mark: ˉ
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
    /// Returns [`BopomofoKind`] of the [`Bopomofo`] symbol. See [`BopomofoKind`] to know more about
    /// each kind category.
    pub const fn kind(&self) -> BopomofoKind {
        match self {
            B | P | M | F | D | T | N | L | G | K | H | J | Q | X | ZH | CH | SH | R | Z | C
            | S => BopomofoKind::Initial,
            I | U | IU => BopomofoKind::Medial,
            A | O | E | EH | AI | EI | AU | OU | AN | EN | ANG | ENG | ER => BopomofoKind::Rime,
            TONE1 | TONE2 | TONE3 | TONE4 | TONE5 => BopomofoKind::Tone,
        }
    }
    /// Returns a [`Bopomofo`] that is categorized as initial sounds based on the index. It will
    /// return [`None`] if the index is larger than 20. The index order is listed below starting
    /// from 0.
    ///
    /// - Initial sounds: ㄅㄆㄇㄈㄉㄊㄋㄌㄍㄎㄏㄐㄑㄒㄓㄔㄕㄖㄗㄘㄙ
    pub(super) const fn from_initial(index: u16) -> Option<Bopomofo> {
        if index as usize >= INITIAL_MAP.len() {
            return None;
        }
        Some(INITIAL_MAP[index as usize])
    }
    /// Returns a [`Bopomofo`] that is categorized as medial glides based on the index. It will
    /// return [`None`] if the index is larger than 2. The index order is listed below starting
    /// from 0.
    ///
    /// - Medial glides: ㄧㄨㄩ
    pub(super) const fn from_medial(index: u16) -> Option<Bopomofo> {
        if index as usize >= MEDIAL_MAP.len() {
            return None;
        }
        Some(MEDIAL_MAP[index as usize])
    }
    /// Returns a [`Bopomofo`] that is categorized as rimes based on the index. It will
    /// return [`None`] if the index is larger than 12. The index order is listed below starting
    /// from 0.
    ///
    /// - Rimes: ㄚㄛㄜㄝㄞㄟㄠㄡㄢㄣㄤㄥㄦ
    pub(super) const fn from_rime(index: u16) -> Option<Bopomofo> {
        if index as usize >= RIME_MAP.len() {
            return None;
        }
        Some(RIME_MAP[index as usize])
    }
    /// Returns a [`Bopomofo`] that is categorized as tonal marks based on the index. It will
    /// return [`None`] if the index is larger than 3. The index order is listed below starting
    /// from 0.
    ///
    /// - Tonal marks: ˙ˊˇˋ
    pub(super) const fn from_tone(index: u16) -> Option<Bopomofo> {
        if index as usize >= TONE_MAP.len() {
            return None;
        }
        Some(TONE_MAP[index as usize])
    }
    pub(super) const fn index(&self) -> u16 {
        match self {
            B | I | A | TONE5 => 1,
            P | U | O | TONE2 => 2,
            M | IU | E | TONE3 => 3,
            F | EH | TONE4 => 4,
            D | AI | TONE1 => 5,
            T | EI => 6,
            N | AU => 7,
            L | OU => 8,
            G | AN => 9,
            K | EN => 10,
            H | ANG => 11,
            J | ENG => 12,
            Q | ER => 13,
            X => 14,
            ZH => 15,
            CH => 16,
            SH => 17,
            R => 18,
            Z => 19,
            C => 20,
            S => 21,
        }
    }
}

/// Enum to store the various types of errors that can cause parsing a bopomofo
/// symbol to fail.
///
/// # Example
///
/// ```
/// # use std::str::FromStr;
/// # use chewing::zhuyin::Bopomofo;
/// if let Err(e) = Bopomofo::from_str("a12") {
///     println!("Failed conversion to bopomofo: {e}");
/// }
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum BopomofoErrorKind {
    /// Value being parsed is empty.
    Empty,
    /// Contains an invalid symbol.
    InvalidSymbol,
}

/// An error which can be returned when parsing an bopomofo symbol.
///
/// # Potential causes
///
/// Among other causes, `ParseBopomofoError` can be thrown because of leading or trailing whitespace
/// in the string e.g., when it is obtained from the standard input.
/// Using the [`str::trim()`] method ensures that no whitespace remains before parsing.
///
/// # Example
///
/// ```
/// # use std::str::FromStr;
/// # use chewing::zhuyin::Bopomofo;
/// if let Err(e) = Bopomofo::from_str("a12") {
///     println!("Failed conversion to bopomofo: {e}");
/// }
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParseBopomofoError {
    kind: BopomofoErrorKind,
}

impl ParseBopomofoError {
    fn empty() -> ParseBopomofoError {
        Self {
            kind: BopomofoErrorKind::Empty,
        }
    }
    fn invalid_symbol() -> ParseBopomofoError {
        Self {
            kind: BopomofoErrorKind::InvalidSymbol,
        }
    }
    /// Outputs the detailed cause of parsing an bopomofo failing.
    pub fn kind(&self) -> &BopomofoErrorKind {
        &self.kind
    }
}

impl Display for ParseBopomofoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Parse bopomofo error: {:?}", self.kind)
    }
}

impl Error for ParseBopomofoError {}

impl Display for Bopomofo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_char((*self).into())
    }
}

impl FromStr for Bopomofo {
    type Err = ParseBopomofoError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Err(ParseBopomofoError::empty());
        }
        if s.chars().count() != 1 {
            return Err(ParseBopomofoError::invalid_symbol());
        }

        s.chars().next().unwrap().try_into()
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
            _ => Err(ParseBopomofoError::invalid_symbol()),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::zhuyin::{BopomofoErrorKind, ParseBopomofoError};

    use super::Bopomofo;

    #[test]
    fn parse() {
        assert_eq!(Ok(Bopomofo::B), "ㄅ".parse())
    }

    #[test]
    fn parse_empty() {
        assert_eq!(Err(ParseBopomofoError::empty()), "".parse::<Bopomofo>());
        assert_eq!(
            &BopomofoErrorKind::Empty,
            ParseBopomofoError::empty().kind()
        );
    }

    #[test]
    fn parse_invalid() {
        assert_eq!(
            Err(ParseBopomofoError::invalid_symbol()),
            "abc".parse::<Bopomofo>()
        );
        assert_eq!(
            &BopomofoErrorKind::InvalidSymbol,
            ParseBopomofoError::invalid_symbol().kind()
        );
    }

    #[test]
    fn to_string() {
        assert_eq!(Bopomofo::B.to_string(), "ㄅ")
    }
}
