//! TODO: docs

mod bopomofo;
mod syllable;

pub use bopomofo::{Bopomofo, BopomofoKind, ParseBopomofoError};
pub use syllable::{DecodeSyllableError, Syllable, SyllableBuilder, SyllableSlice};
