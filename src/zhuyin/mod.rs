//! Chinese syllables and bopomofo phonetic symbols.

mod bopomofo;
mod syllable;

pub use bopomofo::{Bopomofo, BopomofoErrorKind, BopomofoKind, ParseBopomofoError};
pub use syllable::{DecodeSyllableError, Syllable, SyllableBuilder, SyllableSlice};
