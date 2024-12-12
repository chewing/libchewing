use std::{
    borrow::Cow,
    error::Error,
    fmt::{Debug, Display, Write},
    num::NonZeroU16,
    ops::Shl,
    str::FromStr,
};

use super::{Bopomofo, BopomofoKind, ParseBopomofoError};

/// The consonants and vowels that are taken together to make a single sound.
///
/// <https://en.m.wikipedia.org/wiki/Syllable#Chinese_model>
#[derive(Clone, Copy, PartialEq, Eq, Ord, PartialOrd, Hash)]
#[repr(transparent)]
pub struct Syllable {
    value: NonZeroU16,
}

impl Debug for Syllable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Syllable")
            .field("value", &self.value)
            .field("to_string()", &self.to_string())
            .finish()
    }
}

#[allow(clippy::unusual_byte_groupings)]
impl Syllable {
    const EMPTY_PATTERN: u16 = 0b1000000_00_0000_000;
    const EMPTY: Syllable = Syllable {
        value: match NonZeroU16::new(Self::EMPTY_PATTERN) {
            Some(v) => v,
            None => unreachable!(),
        },
    };
    /// Creates a new empty syllable.
    pub const fn new() -> Syllable {
        Syllable::EMPTY
    }
    /// Creates a new syllable builder.
    pub const fn builder() -> SyllableBuilder {
        SyllableBuilder::new()
    }
    /// Returns the initial part of the syllable.
    pub const fn initial(&self) -> Option<Bopomofo> {
        let index = (self.value.get() & 0b0111111_00_0000_000) >> 9;
        if index == 0 {
            None
        } else {
            Bopomofo::from_initial(index - 1)
        }
    }
    /// Returns the medial part of the syllable.
    pub const fn medial(&self) -> Option<Bopomofo> {
        let index = (self.value.get() & 0b0000000_11_0000_000) >> 7;
        if index == 0 {
            None
        } else {
            Bopomofo::from_medial(index - 1)
        }
    }
    /// Returns the rime part of the syllable.
    pub const fn rime(&self) -> Option<Bopomofo> {
        let index = (self.value.get() & 0b0000000_00_1111_000) >> 3;
        if index == 0 {
            None
        } else {
            Bopomofo::from_rime(index - 1)
        }
    }
    /// Returns the tone of the syllable.
    pub const fn tone(&self) -> Option<Bopomofo> {
        let index = self.value.get() & 0b0000000_00_0000_111;
        if index == 0 {
            None
        } else {
            Bopomofo::from_tone(index - 1)
        }
    }
    /// Removes the initial from the syllable.
    pub fn remove_initial(&mut self) -> Option<Bopomofo> {
        let ret = self.initial();
        let value = self.value.get() & 0b0000000_11_1111_111;
        self.value = match value {
            0 => Syllable::EMPTY.value,
            _ => NonZeroU16::new(value).unwrap(),
        };
        ret
    }
    /// Removes the medial from the syllable.
    pub fn remove_medial(&mut self) -> Option<Bopomofo> {
        let ret = self.medial();
        let value = self.value.get() & 0b1111111_00_1111_111;
        self.value = match value {
            0 => Syllable::EMPTY.value,
            _ => NonZeroU16::new(value).unwrap(),
        };
        ret
    }
    /// Removes the rime from the syllable.
    pub fn remove_rime(&mut self) -> Option<Bopomofo> {
        let ret = self.rime();
        let value = self.value.get() & 0b1111111_11_0000_111;
        self.value = match value {
            0 => Syllable::EMPTY.value,
            _ => NonZeroU16::new(value).unwrap(),
        };
        ret
    }
    /// Removes the tone from the syllable.
    pub fn remove_tone(&mut self) -> Option<Bopomofo> {
        let ret = self.tone();
        let value = self.value.get() & 0b1111111_11_1111_000;
        self.value = match value {
            0 => Syllable::EMPTY.value,
            _ => NonZeroU16::new(value).unwrap(),
        };
        ret
    }
    /// Returns whether the syllable is empty.
    pub const fn is_empty(&self) -> bool {
        self.value.get() == Syllable::EMPTY.value.get()
    }
    /// Returns whether the syllable has an initial.
    pub fn has_initial(&self) -> bool {
        self.initial().is_some()
    }
    /// Returns whether the syllable has a medial
    pub fn has_medial(&self) -> bool {
        self.medial().is_some()
    }
    /// Returns whether the syllable has a rime.
    pub fn has_rime(&self) -> bool {
        self.rime().is_some()
    }
    /// Returns whether the syllable has a tone.
    pub fn has_tone(&self) -> bool {
        self.tone().is_some()
    }
    /// Returns whether the syllable partially matches another syllable.
    pub fn starts_with(&self, other: Syllable) -> bool {
        let trailing_zeros = other.to_u16().trailing_zeros();
        let mask = if trailing_zeros >= 9 {
            9
        } else if trailing_zeros >= 7 {
            7
        } else if trailing_zeros >= 3 {
            3
        } else {
            0
        };
        let self_prefix = self.to_u16() >> mask;
        let other_prefix = other.to_u16() >> mask;
        self_prefix == other_prefix
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
    pub fn to_u16(self) -> u16 {
        self.value.get()
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
    fn to_le_bytes(self) -> [u8; 2] {
        self.to_u16().to_le_bytes()
    }
    /// Combines the current syllable with a new sound.
    pub fn update(&mut self, bopomofo: Bopomofo) {
        let orig = self.value.get();
        let value = match bopomofo.kind() {
            BopomofoKind::Initial => (orig & 0b0000000_11_1111_111) | bopomofo.index().shl(9),
            BopomofoKind::Medial => (orig & 0b0111111_00_1111_111) | bopomofo.index().shl(7),
            BopomofoKind::Rime => (orig & 0b0111111_11_0000_111) | bopomofo.index().shl(3),
            BopomofoKind::Tone => (orig & 0b0111111_11_1111_000) | bopomofo.index(),
        };
        self.value = NonZeroU16::new(value).unwrap();
    }
    /// Removes components of the syllable.
    pub fn pop(&mut self) -> Option<Bopomofo> {
        if self.has_tone() {
            return self.remove_tone();
        }
        if self.has_rime() {
            return self.remove_rime();
        }
        if self.has_medial() {
            return self.remove_medial();
        }
        if self.has_initial() {
            return self.remove_initial();
        }
        None
    }
    /// Resets the syllable to empty.
    pub fn clear(&mut self) {
        *self = Syllable::EMPTY
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

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        // TODO check invalid value
        Ok(Syllable {
            value: NonZeroU16::try_from(value).map_err(|_| DecodeSyllableError)?,
        })
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

impl AsRef<Syllable> for Syllable {
    fn as_ref(&self) -> &Syllable {
        self
    }
}

/// A slice that can be converted to a slice of syllables.
pub trait SyllableSlice: Debug {
    fn to_slice(&self) -> Cow<'_, [Syllable]>;
    fn to_bytes(&self) -> Vec<u8> {
        let mut syllables_bytes = vec![];
        self.to_slice()
            .iter()
            .for_each(|syl| syllables_bytes.extend_from_slice(&syl.as_ref().to_le_bytes()));
        syllables_bytes
    }
}

impl SyllableSlice for &[Syllable] {
    fn to_slice(&self) -> Cow<'_, [Syllable]> {
        Cow::Borrowed(*self)
    }
}

impl SyllableSlice for Vec<Syllable> {
    fn to_slice(&self) -> Cow<'_, [Syllable]> {
        Cow::Borrowed(self)
    }
}

impl<const N: usize> SyllableSlice for [Syllable; N] {
    fn to_slice(&self) -> Cow<'_, [Syllable]> {
        Cow::Borrowed(self)
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

/// A syllable builder can be used to construct syllables at compile time.
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
    /// Creates a new syllable builder.
    pub const fn new() -> SyllableBuilder {
        SyllableBuilder {
            value: Syllable::EMPTY_PATTERN,
            step: 0,
        }
    }
    /// Inserts syllable components and checks they follow correct order.
    #[allow(clippy::unusual_byte_groupings)]
    pub const fn insert(
        mut self,
        bopomofo: Bopomofo,
    ) -> Result<SyllableBuilder, BuildSyllableError> {
        match bopomofo.kind() {
            BopomofoKind::Initial => {
                if self.value & 0b0111111_00_0000_000 != 0 {
                    return Err(BuildSyllableError::multiple_initials());
                }
                if self.step > 0 {
                    return Err(BuildSyllableError::incorrect_order());
                }
                self.step = 1;
                self.value &= 0b0000000_11_1111_111;
                self.value |= (bopomofo as u16 + 1) << 9;
            }
            BopomofoKind::Medial => {
                if self.value & 0b0000000_11_0000_000 != 0 {
                    return Err(BuildSyllableError::multiple_medials());
                }
                if self.step > 1 {
                    return Err(BuildSyllableError::incorrect_order());
                }
                self.step = 2;
                self.value &= 0b0111111_00_1111_111;
                self.value |= (bopomofo as u16 - 20) << 7;
            }
            BopomofoKind::Rime => {
                if self.value & 0b0000000_00_1111_000 != 0 {
                    return Err(BuildSyllableError::multiple_rimes());
                }
                if self.step > 2 {
                    return Err(BuildSyllableError::incorrect_order());
                }
                self.step = 3;
                self.value &= 0b0111111_11_0000_111;
                self.value |= (bopomofo as u16 - 23) << 3;
            }
            BopomofoKind::Tone => {
                if self.value & 0b0000000_00_0000_111 != 0 {
                    return Err(BuildSyllableError::multiple_tones());
                }
                if self.step > 3 {
                    return Err(BuildSyllableError::incorrect_order());
                }
                self.step = 4;
                self.value &= 0b0111111_11_1111_000;
                self.value |= bopomofo as u16 - 36;
            }
        };
        Ok(self)
    }
    /// Builds the syllable.
    pub const fn build(self) -> Syllable {
        Syllable {
            value: match NonZeroU16::new(self.value) {
                Some(v) => v,
                None => unreachable!(),
            },
        }
    }
}

/// Errors during decoding a syllable from a u16.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DecodeSyllableError;

impl Display for DecodeSyllableError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "syllable decode error")
    }
}

impl Error for DecodeSyllableError {}

/// Errors when parsing a str to a syllable.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum SyllableErrorKind {
    MultipleInitials,
    MultipleMedials,
    MultipleRimes,
    MultipleTones,
    IncorrectOrder,
    InvalidBopomofo,
}

/// Errors when building a new syllable.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BuildSyllableError {
    kind: SyllableErrorKind,
}

impl BuildSyllableError {
    const fn multiple_initials() -> BuildSyllableError {
        Self {
            kind: SyllableErrorKind::MultipleInitials,
        }
    }
    const fn multiple_medials() -> BuildSyllableError {
        Self {
            kind: SyllableErrorKind::MultipleMedials,
        }
    }
    const fn multiple_rimes() -> BuildSyllableError {
        Self {
            kind: SyllableErrorKind::MultipleRimes,
        }
    }
    const fn multiple_tones() -> BuildSyllableError {
        Self {
            kind: SyllableErrorKind::MultipleTones,
        }
    }
    const fn incorrect_order() -> BuildSyllableError {
        Self {
            kind: SyllableErrorKind::IncorrectOrder,
        }
    }
    pub fn kind(&self) -> &SyllableErrorKind {
        &self.kind
    }
}

impl Display for BuildSyllableError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Syllable build error: {:?}", self.kind)
    }
}

impl Error for BuildSyllableError {}

/// Errors when parsing a str to a syllable.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParseSyllableError {
    kind: SyllableErrorKind,
}

impl ParseSyllableError {
    pub fn kind(&self) -> &SyllableErrorKind {
        &self.kind
    }
}

impl Display for ParseSyllableError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Syllable parse error")
    }
}

impl Error for ParseSyllableError {}

impl From<ParseBopomofoError> for ParseSyllableError {
    fn from(_: ParseBopomofoError) -> Self {
        ParseSyllableError {
            kind: SyllableErrorKind::InvalidBopomofo,
        }
    }
}

impl From<BuildSyllableError> for ParseSyllableError {
    fn from(value: BuildSyllableError) -> Self {
        ParseSyllableError { kind: value.kind }
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
    fn syllable_update_as_u16() {
        let mut syl = Syllable::new();
        syl.update(Bopomofo::I);
        assert_eq!(128, syl.to_u16());

        syl.update(Bopomofo::TONE2);
        assert_eq!(130, syl.to_u16());

        syl.update(Bopomofo::X);
        assert_eq!(7298, syl.to_u16());
    }

    #[test]
    fn empty_syllable_as_u16() {
        Syllable::builder().build().to_u16();
    }

    #[test]
    fn syllable_as_u16_roundtrip() {
        let syl = Syllable::builder().insert(Bopomofo::S).unwrap().build();
        assert_eq!(syl, syl.to_u16().try_into().unwrap());
    }

    #[test]
    fn syllable_starts_with() {
        assert!(
            syl![Bopomofo::X, Bopomofo::I, Bopomofo::EN, Bopomofo::TONE4].starts_with(syl![
                Bopomofo::X,
                Bopomofo::I,
                Bopomofo::EN
            ])
        );
        assert!(
            syl![Bopomofo::X, Bopomofo::I, Bopomofo::EN, Bopomofo::TONE4]
                .starts_with(syl![Bopomofo::X, Bopomofo::I])
        );
        assert!(
            syl![Bopomofo::X, Bopomofo::I, Bopomofo::EN, Bopomofo::TONE4]
                .starts_with(syl![Bopomofo::X])
        );
        assert!(
            !syl![Bopomofo::X, Bopomofo::I, Bopomofo::EN, Bopomofo::TONE4]
                .starts_with(syl![Bopomofo::Q])
        );
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
