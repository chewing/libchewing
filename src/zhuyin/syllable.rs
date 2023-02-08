use std::{
    fmt::{Display, Write},
    str::FromStr,
};

use thiserror::Error;

use super::{Bopomofo, BopomofoKind, ParseBopomofoError};

/// The consonants and vowels that are taken together to make a single sound.
///
/// <https://en.m.wikipedia.org/wiki/Syllable#Chinese_model>
#[derive(Clone, Copy, Debug, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub struct Syllable {
    value: u16,
}

impl Syllable {
    /// TODO: docs
    pub const fn new() -> Syllable {
        Syllable { value: 0 }
    }
    /// TODO: docs
    pub const fn builder() -> SyllableBuilder {
        SyllableBuilder::new()
    }
    /// TODO: docs
    pub const fn initial(&self) -> Option<Bopomofo> {
        let index = self.value >> 9;
        if index == 0 {
            None
        } else {
            match Bopomofo::from_initial(index) {
                Ok(v) => Some(v),
                Err(_) => panic!(),
            }
        }
    }
    /// TODO: docs
    #[allow(clippy::unusual_byte_groupings)]
    pub const fn medial(&self) -> Option<Bopomofo> {
        let index = (self.value & 0b0000000_11_0000_000) >> 7;
        if index == 0 {
            None
        } else {
            match Bopomofo::from_medial(index) {
                Ok(v) => Some(v),
                Err(_) => panic!(),
            }
        }
    }
    /// TODO: docs
    #[allow(clippy::unusual_byte_groupings)]
    pub const fn rime(&self) -> Option<Bopomofo> {
        let index = (self.value & 0b0000000_00_1111_000) >> 3;
        if index == 0 {
            None
        } else {
            match Bopomofo::from_rime(index) {
                Ok(v) => Some(v),
                Err(_) => panic!(),
            }
        }
    }
    /// TODO: docs
    #[allow(clippy::unusual_byte_groupings)]
    pub const fn tone(&self) -> Option<Bopomofo> {
        let index = self.value & 0b0000000_00_0000_111;
        if index == 0 {
            None
        } else {
            match Bopomofo::from_tone(index) {
                Ok(v) => Some(v),
                Err(_) => panic!(),
            }
        }
    }
    /// TODO: docs
    pub fn remove_initial(&mut self) -> Option<Bopomofo> {
        let ret = self.initial();
        self.value &= 0b0000_0001_1111_1111;
        ret
    }
    /// TODO: docs
    pub fn remove_medial(&mut self) -> Option<Bopomofo> {
        let ret = self.medial();
        self.value &= 0b1111_1110_0111_1111;
        ret
    }
    /// TODO: docs
    pub fn remove_rime(&mut self) -> Option<Bopomofo> {
        let ret = self.rime();
        self.value &= 0b1111_1111_1000_0111;
        ret
    }
    /// TODO: docs
    pub fn remove_tone(&mut self) -> Option<Bopomofo> {
        let ret = self.tone();
        self.value &= 0b1111_1111_1111_1000;
        ret
    }
    /// TODO: docs
    pub fn is_empty(&self) -> bool {
        self.value == 0
    }
    /// TODO: docs
    pub fn has_initial(&self) -> bool {
        self.initial().is_some()
    }
    /// TODO: docs
    pub fn has_medial(&self) -> bool {
        self.medial().is_some()
    }
    /// TODO: docs
    pub fn has_rime(&self) -> bool {
        self.rime().is_some()
    }
    /// TODO: docs
    pub fn has_tone(&self) -> bool {
        self.tone().is_some()
    }
    /// Returns the `Syllable` encoded in a u16 integer.
    ///
    /// The data layout used:
    ///
    /// ```text
    ///  0                   1
    ///  0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5
    /// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
    /// |   Initial   | M | Rime  |Tone |
    /// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
    /// ```
    pub fn to_u16(&self) -> u16 {
        debug_assert!(
            !self.is_empty(),
            "empty syllable cannot be converted to u16"
        );
        self.value
    }
    /// Returns the `Syllable` encoded in a u16 integer in little-endian bytes.
    ///
    /// The data layout used:
    ///
    /// ```text
    ///  0                   1
    ///  0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5
    /// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
    /// |   Initial   | M | Rime  |Tone |
    /// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
    /// ```
    pub fn to_le_bytes(&self) -> [u8; 2] {
        self.to_u16().to_le_bytes()
    }
    /// TODO: docs
    pub fn update(&mut self, bopomofo: Bopomofo) {
        match bopomofo.kind() {
            BopomofoKind::Initial => {
                self.remove_initial();
                self.value |= (bopomofo as u16 + 1) << 9;
            }
            BopomofoKind::Medial => {
                self.remove_medial();
                self.value |= (bopomofo as u16 - 20) << 7;
            }
            BopomofoKind::Rime => {
                self.remove_rime();
                self.value |= (bopomofo as u16 - 23) << 3;
            }
            BopomofoKind::Tone => {
                self.remove_tone();
                self.value |= bopomofo as u16 - 36;
            }
        };
    }
    /// TODO: docs
    pub fn pop(&mut self) -> Option<Bopomofo> {
        if self.tone().is_some() {
            return self.remove_tone();
        }
        if self.rime().is_some() {
            return self.remove_rime();
        }
        if self.medial().is_some() {
            return self.remove_medial();
        }
        if self.initial().is_some() {
            return self.remove_initial();
        }
        None
    }
    /// TODO: docs
    pub fn clear(&mut self) {
        *self = Syllable::new()
    }
}

impl Default for Syllable {
    fn default() -> Self {
        Syllable::new()
    }
}

impl From<Syllable> for u16 {
    fn from(syl: Syllable) -> Self {
        syl.to_u16()
    }
}

impl From<&Syllable> for u16 {
    fn from(syl: &Syllable) -> Self {
        syl.to_u16()
    }
}

impl TryFrom<u16> for Syllable {
    type Error = DecodeSyllableError;

    #[allow(clippy::unusual_byte_groupings)]
    fn try_from(value: u16) -> Result<Self, Self::Error> {
        // TODO check invalid value
        Ok(Syllable { value })
    }
}

impl FromStr for Syllable {
    type Err = ParseSyllableError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut builder = Syllable::builder();
        for c in s.chars() {
            let bopomofo = Bopomofo::try_from(c)?;
            builder = builder.insert(bopomofo)?;
        }
        Ok(builder.build())
    }
}

/// TODO: docs
pub trait IntoSyllablesBytes {
    /// TODO: docs
    fn into_syllables_bytes(self) -> Vec<u8>;
}

impl<T> IntoSyllablesBytes for T
where
    T: IntoIterator,
    T::Item: Into<u16>,
{
    fn into_syllables_bytes(self) -> Vec<u8> {
        let mut syllables_bytes = vec![];
        self.into_iter()
            .for_each(|syl| syllables_bytes.extend_from_slice(&syl.into().to_le_bytes()));
        syllables_bytes
    }
}

impl Display for Syllable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for &bopomofo in [&self.initial(), &self.medial(), &self.rime(), &self.tone()] {
            if let Some(bopomofo) = bopomofo {
                f.write_char(bopomofo.into())?;
            }
        }
        Ok(())
    }
}

/// TODO: docs
#[derive(Debug)]
pub struct SyllableBuilder {
    value: u16,
    step: u8,
}

impl Default for SyllableBuilder {
    fn default() -> SyllableBuilder {
        SyllableBuilder::new()
    }
}

impl SyllableBuilder {
    /// TODO: docs
    pub const fn new() -> SyllableBuilder {
        SyllableBuilder { value: 0, step: 0 }
    }
    /// TODO: docs
    pub const fn insert(
        mut self,
        bopomofo: Bopomofo,
    ) -> Result<SyllableBuilder, BuildSyllableError> {
        match bopomofo.kind() {
            BopomofoKind::Initial => {
                if self.step > 0 {
                    return Err(BuildSyllableError {
                        msg: "bopomofo is in incorrect order",
                    });
                }
                if self.value & 0b1111_1110_0000_0000 != 0 {
                    return Err(BuildSyllableError {
                        msg: "multiple initial bopomofo",
                    });
                }
                self.step = 1;
                self.value |= (bopomofo as u16 + 1) << 9;
            }
            BopomofoKind::Medial => {
                if self.step > 1 {
                    return Err(BuildSyllableError {
                        msg: "bopomofo is in incorrect order",
                    });
                }
                if self.value & 0b0000_0001_1000_0000 != 0 {
                    return Err(BuildSyllableError {
                        msg: "multiple medial bopomofo",
                    });
                }
                self.step = 2;
                self.value |= (bopomofo as u16 - 20) << 7;
            }
            BopomofoKind::Rime => {
                if self.step > 2 {
                    return Err(BuildSyllableError {
                        msg: "bopomofo is in incorrect order",
                    });
                }
                if self.value & 0b0000_0000_0111_1000 != 0 {
                    return Err(BuildSyllableError {
                        msg: "multiple rime bopomofo",
                    });
                }
                self.step = 3;
                self.value |= (bopomofo as u16 - 23) << 3;
            }
            BopomofoKind::Tone => {
                if self.step > 3 {
                    return Err(BuildSyllableError {
                        msg: "bopomofo is in incorrect order",
                    });
                }
                if self.value & 0b0000_0000_0000_0111 != 0 {
                    return Err(BuildSyllableError {
                        msg: "multiple tone bopomofo",
                    });
                }
                self.step = 4;
                self.value |= bopomofo as u16 - 36;
            }
        };
        Ok(self)
    }
    /// TODO: docs
    pub const fn build(self) -> Syllable {
        Syllable { value: self.value }
    }
}

/// TODO: docs
#[derive(Error, Debug)]
#[error("syllable decode error: {msg}")]
pub struct DecodeSyllableError {
    msg: String,
    source: Box<dyn std::error::Error>,
}

#[derive(Error, Debug)]
#[error("syllable build error: {msg}")]
pub struct BuildSyllableError {
    msg: &'static str,
}

#[derive(Error, Debug)]
#[error("syllable parse error")]
pub struct ParseSyllableError {
    source: Box<dyn std::error::Error>,
}

impl From<ParseBopomofoError> for ParseSyllableError {
    fn from(value: ParseBopomofoError) -> Self {
        ParseSyllableError {
            source: value.into(),
        }
    }
}

impl From<BuildSyllableError> for ParseSyllableError {
    fn from(value: BuildSyllableError) -> Self {
        ParseSyllableError {
            source: value.into(),
        }
    }
}

/// Builds a syllable from bopomofos.
///
/// `syl!` can be used in const context. It is meant for
/// embedding const syllables or writing tests.
///
/// # Examples
///
/// Build a syllable
/// ```
/// use chewing::zhuyin::Bopomofo::{K, U, TONE4};
/// use chewing::syl;
///
/// let syl = syl![K, U, TONE4];
///
/// assert_eq!("ㄎㄨˋ", syl.to_string());
/// ```
///
/// # Panics
///
/// `syl!` can panic if the bopomofos are not well formed.
/// For example, multiple initials or incorrect orders both cause
/// the macro to panic.
#[macro_export]
macro_rules! syl {
    () => { $crate::zhuyin::Syllable::new() };
    ($($bopomofo:expr),+) => {
        {
            let mut builder = $crate::zhuyin::Syllable::builder();
            $(builder = match builder.insert($bopomofo) {
                Ok(b) => b,
                Err(_) => panic!("unable to build syllable"),
            };)+
            builder.build()
        }
    };
}

#[cfg(test)]
mod test {

    use super::{Bopomofo, Syllable};

    #[test]
    fn syllable_hsu_sdf_as_u16() {
        let syl = Syllable::builder().insert(Bopomofo::S).unwrap().build();
        assert_eq!(0x2A00, syl.to_u16());

        let syl = Syllable::builder().insert(Bopomofo::D).unwrap().build();
        assert_eq!(0xA00, syl.to_u16());

        let syl = Syllable::builder().insert(Bopomofo::F).unwrap().build();
        assert_eq!(0x800, syl.to_u16());
    }

    #[test]
    #[should_panic]
    fn empty_syllable_as_u16() {
        Syllable::builder().build().to_u16();
    }

    #[test]
    fn syllable_as_u16_roundtrip() {
        let syl = Syllable::builder().insert(Bopomofo::S).unwrap().build();
        assert_eq!(syl, syl.to_u16().try_into().unwrap());
    }

    #[test]
    fn syl_macro_rules() {
        let syl = syl![];
        assert_eq!(Syllable::new(), syl);

        let syl = syl![Bopomofo::S];
        assert_eq!(
            Syllable::builder().insert(Bopomofo::S).unwrap().build(),
            syl
        );

        let syl = syl![Bopomofo::S, Bopomofo::I, Bopomofo::EN, Bopomofo::TONE4];
        assert_eq!(
            Syllable::builder()
                .insert(Bopomofo::S)
                .unwrap()
                .insert(Bopomofo::I)
                .unwrap()
                .insert(Bopomofo::EN)
                .unwrap()
                .insert(Bopomofo::TONE4)
                .unwrap()
                .build(),
            syl
        );
    }

    #[test]
    #[should_panic]
    fn syl_macro_rules_fool_proof() {
        syl![Bopomofo::S, Bopomofo::D];
    }

    #[test]
    fn syl_macro_rules_comiles_in_const() {
        const SYLLABLE: Syllable = syl![Bopomofo::S, Bopomofo::I, Bopomofo::EN];
        assert_eq!(
            Syllable::builder()
                .insert(Bopomofo::S)
                .unwrap()
                .insert(Bopomofo::I)
                .unwrap()
                .insert(Bopomofo::EN)
                .unwrap()
                .build(),
            SYLLABLE
        );
    }

    #[test]
    fn new_and_pop_bopomofo() {
        let mut syl = syl![Bopomofo::S, Bopomofo::I, Bopomofo::EN, Bopomofo::TONE4];
        assert_eq!(Some(Bopomofo::TONE4), syl.pop());
        assert_eq!(Some(Bopomofo::EN), syl.pop());
        assert_eq!(Some(Bopomofo::I), syl.pop());
        assert_eq!(Some(Bopomofo::S), syl.pop());
        assert_eq!(None, syl.pop());
        assert_eq!(syl![], syl);
    }
}
