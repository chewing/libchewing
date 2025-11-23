//! Chinese syllables and bopomofo phonetic symbols.

mod bopomofo;
mod syllable;

pub use bopomofo::{Bopomofo, BopomofoErrorKind, BopomofoKind, ParseBopomofoError};
pub use syllable::{
    BuildSyllableError, DecodeSyllableError, ParseSyllableError, Syllable, SyllableBuilder,
    SyllableErrorKind,
};
