//! Systems and user phrase dictionaries.

use std::{
    any::Any,
    borrow::Borrow,
    cmp::Ordering,
    error::Error,
    fmt::{Debug, Display},
    io,
    path::Path,
};

use crate::zhuyin::{Syllable, SyllableSlice};

pub use layered::Layered;
pub use loader::{LoadDictionaryError, SystemDictionaryLoader, UserDictionaryLoader};
#[cfg(feature = "sqlite")]
pub use sqlite::{SqliteDictionary, SqliteDictionaryBuilder, SqliteDictionaryError};
pub use trie::{Trie, TrieBuilder, TrieOpenOptions, TrieStatistics};
pub use trie_buf::TrieBuf;

mod layered;
mod loader;
#[cfg(feature = "sqlite")]
mod sqlite;
mod trie;
mod trie_buf;
mod uhash;

/// The error type which is returned from updating a dictionary.
#[derive(Debug)]
pub struct UpdateDictionaryError {
    /// TODO: doc
    source: Option<Box<dyn Error + Send + Sync>>,
}

impl UpdateDictionaryError {
    pub(crate) fn new() -> UpdateDictionaryError {
        UpdateDictionaryError { source: None }
    }
}

impl From<io::Error> for UpdateDictionaryError {
    fn from(value: io::Error) -> Self {
        UpdateDictionaryError {
            source: Some(Box::new(value)),
        }
    }
}

impl Display for UpdateDictionaryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "update dictionary failed")
    }
}

impl Error for UpdateDictionaryError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.source.as_ref().map(|err| err.as_ref() as &dyn Error)
    }
}

/// A collection of metadata of a dictionary.
///
/// The dictionary version and copyright information can be used in
/// configuration application.
///
/// # Examples
///
/// ```no_run
/// # use chewing::dictionary::{Dictionary, TrieBuf};
/// # let dictionary = TrieBuf::new_in_memory();
/// let about = dictionary.about();
/// assert_eq!("libchewing default", about.name);
/// assert_eq!("Copyright (c) 2022 libchewing Core Team", about.copyright);
/// assert_eq!("LGPL-2.1-or-later", about.license);
/// assert_eq!("init_database 0.5.1", about.software);
/// ```
#[derive(Debug, Clone, Default)]
pub struct DictionaryInfo {
    /// The name of the dictionary.
    pub name: String,
    /// The copyright information of the dictionary.
    ///
    /// It's recommended to include the copyright holders' names and email
    /// addresses, separated by semicolons.
    pub copyright: String,
    /// The license information of the dictionary.
    ///
    /// It's recommended to use the [SPDX license identifier](https://spdx.org/licenses/).
    pub license: String,
    /// The version of the dictionary.
    ///
    /// It's recommended to use the commit hash or revision if the dictionary is
    /// managed in a source control repository.
    pub version: String,
    /// The name of the software used to generate the dictionary.
    ///
    /// It's recommended to include the name and the version number.
    pub software: String,
}

/// A type containing a phrase string and its frequency.
///
/// # Examples
///
/// A `Phrase` can be created from/to a tuple.
///
/// ```
/// use chewing::dictionary::Phrase;
///
/// let phrase = Phrase::new("測", 1);
/// assert_eq!(phrase, ("測", 1).into());
/// assert_eq!(("測".to_string(), 1u32), phrase.into());
/// ```
///
/// Phrases are ordered by their frequency.
///
/// ```
/// use chewing::dictionary::Phrase;
///
/// assert!(Phrase::new("測", 100) > Phrase::new("冊", 1));
/// ```
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Phrase {
    phrase: Box<str>,
    freq: u32,
    last_used: Option<u64>,
}

impl Phrase {
    /// Creates a new `Phrase`.
    ///
    /// # Examples
    ///
    /// ```
    /// use chewing::dictionary::Phrase;
    ///
    /// let phrase = Phrase::new("新", 1);
    /// ```
    pub fn new<S>(phrase: S, freq: u32) -> Phrase
    where
        S: Into<Box<str>>,
    {
        Phrase {
            phrase: phrase.into(),
            freq,
            last_used: None,
        }
    }
    /// Sets the last used time of the phrase.
    pub fn with_time(mut self, last_used: u64) -> Phrase {
        self.last_used = Some(last_used);
        self
    }
    /// Returns the frequency of the phrase.
    ///
    /// # Examples
    ///
    /// ```
    /// use chewing::dictionary::Phrase;
    ///
    /// let phrase = Phrase::new("詞頻", 100);
    ///
    /// assert_eq!(100, phrase.freq());
    /// ```
    pub fn freq(&self) -> u32 {
        self.freq
    }
    /// Returns the last time this phrase was selected as user phrase.
    ///
    /// The time is a counter increased by one for each keystroke.
    pub fn last_used(&self) -> Option<u64> {
        self.last_used
    }
    /// Returns the inner str of the phrase.
    ///
    /// # Examples
    ///
    /// ```
    /// use chewing::dictionary::Phrase;
    ///
    /// let phrase = Phrase::new("詞", 100);
    ///
    /// assert_eq!("詞", phrase.as_str());
    /// ```
    pub fn as_str(&self) -> &str {
        self.phrase.borrow()
    }
}

/// Phrases are compared by their frequency first, followed by their phrase
/// string.
impl PartialOrd for Phrase {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Phrases are compared by their frequency first, followed by their phrase
/// string.
impl Ord for Phrase {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.freq.cmp(&other.freq) {
            Ordering::Equal => {}
            ord => return ord,
        }
        self.phrase.cmp(&other.phrase)
    }
}

impl AsRef<str> for Phrase {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl From<Phrase> for String {
    fn from(phrase: Phrase) -> Self {
        phrase.phrase.into_string()
    }
}

impl From<Phrase> for Box<str> {
    fn from(phrase: Phrase) -> Self {
        phrase.phrase
    }
}

impl From<Phrase> for (String, u32) {
    fn from(phrase: Phrase) -> Self {
        (phrase.phrase.into_string(), phrase.freq)
    }
}

impl<S> From<(S, u32)> for Phrase
where
    S: Into<Box<str>>,
{
    fn from(tuple: (S, u32)) -> Self {
        Phrase::new(tuple.0, tuple.1)
    }
}

impl<S> From<(S, u32, u64)> for Phrase
where
    S: Into<Box<str>>,
{
    fn from(tuple: (S, u32, u64)) -> Self {
        Phrase::new(tuple.0, tuple.1).with_time(tuple.2)
    }
}

impl Display for Phrase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A boxed iterator over the phrases and their frequency in a dictionary.
///
/// # Examples
///
/// ```
/// use chewing::{dictionary::{Dictionary, LookupStrategy, TrieBuf}, syl, zhuyin::Bopomofo};
///
/// let dict = TrieBuf::from([
///     (vec![syl![Bopomofo::C, Bopomofo::E, Bopomofo::TONE4]], vec![("測", 100)]),
/// ]);
///
/// for phrase in dict.lookup_all_phrases(
///     &[syl![Bopomofo::C, Bopomofo::E, Bopomofo::TONE4]], LookupStrategy::Standard
/// ) {
///     assert_eq!("測", phrase.as_str());
///     assert_eq!(100, phrase.freq());
/// }
/// ```
pub type Phrases<'a> = Box<dyn Iterator<Item = Phrase> + 'a>;

/// A boxed iterator over all the entries in a dictionary.
///
/// # Examples
///
/// ```
/// use chewing::{dictionary::{Dictionary, TrieBuf}, syl, zhuyin::Bopomofo};
///
/// let dict = TrieBuf::from([
///     (vec![syl![Bopomofo::C, Bopomofo::E, Bopomofo::TONE4]], vec![("測", 100)]),
/// ]);
///
/// for (syllables, phrase) in dict.entries() {
///     for bopomofos in syllables {
///         println!("{bopomofos} -> {phrase}");
///     }
/// }
/// ```
pub type Entries<'a> = Box<dyn Iterator<Item = (Vec<Syllable>, Phrase)> + 'a>;

/// The lookup strategy hint for dictionary.
///
/// If the dictionary supports the lookup strategy it should try to use.
/// Otherwise fallback to standard.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum LookupStrategy {
    /// The native lookup strategy supported by the dictionary.
    #[default]
    Standard,
    /// Try to fuzzy match partial syllables using only preffix.
    FuzzyPartialPrefix,
}

/// An interface for looking up dictionaries.
///
/// This is the main dictionary trait. For more about the concept of
/// dictionaries generally, please see the [module-level
/// documentation][crate::dictionary].
///
/// # Examples
///
/// ```
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
///
/// use chewing::{dictionary::{Dictionary, DictionaryMut, LookupStrategy, TrieBuf}, syl, zhuyin::Bopomofo};
///
/// let mut dict = TrieBuf::new_in_memory();
/// dict.add_phrase(&[syl![Bopomofo::C, Bopomofo::E, Bopomofo::TONE4]], ("測", 100).into())?;
///
/// for phrase in dict.lookup_all_phrases(
///     &[syl![Bopomofo::C, Bopomofo::E, Bopomofo::TONE4]], LookupStrategy::Standard
/// ) {
///     assert_eq!("測", phrase.as_str());
///     assert_eq!(100, phrase.freq());
/// }
/// # Ok(())
/// # }
/// ```
pub trait Dictionary: Debug {
    /// Returns first N phrases matched by the syllables.
    ///
    /// The result should use a stable order each time for the same input.
    fn lookup_first_n_phrases(
        &self,
        syllables: &dyn SyllableSlice,
        first: usize,
        strategy: LookupStrategy,
    ) -> Vec<Phrase>;
    /// Returns the first phrase matched by the syllables.
    ///
    /// The result should use a stable order each time for the same input.
    fn lookup_first_phrase(
        &self,
        syllables: &dyn SyllableSlice,
        strategy: LookupStrategy,
    ) -> Option<Phrase> {
        self.lookup_first_n_phrases(syllables, 1, strategy)
            .into_iter()
            .next()
    }
    /// Returns all phrases matched by the syllables.
    ///
    /// The result should use a stable order each time for the same input.
    fn lookup_all_phrases(
        &self,
        syllables: &dyn SyllableSlice,
        strategy: LookupStrategy,
    ) -> Vec<Phrase> {
        self.lookup_first_n_phrases(syllables, usize::MAX, strategy)
    }
    /// Returns an iterator to all phrases in the dictionary.
    fn entries(&self) -> Entries<'_>;
    /// Returns information about the dictionary instance.
    fn about(&self) -> DictionaryInfo;
    /// Returns the dictionary file path if it's backed by a file.
    fn path(&self) -> Option<&Path>;
    fn as_dict_mut(&mut self) -> Option<&mut dyn DictionaryMut>;
}

/// An interface for updating dictionaries.
pub trait DictionaryMut: Debug {
    /// Reopens the dictionary if it was changed by a different process
    ///
    /// It should not fail if the dictionary is read-only or able to sync across
    /// processes automatically.
    fn reopen(&mut self) -> Result<(), UpdateDictionaryError>;
    /// Flushes all the changes back to the filesystem
    ///
    /// The change made to the dictionary might not be persisted without
    /// calling this method.
    fn flush(&mut self) -> Result<(), UpdateDictionaryError>;
    /// An method for updating dictionaries.
    ///
    /// For more about the concept of dictionaries generally, please see the
    /// [module-level documentation][crate::dictionary].
    ///
    /// # Examples
    ///
    /// ```
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///
    /// use chewing::{dictionary::{DictionaryMut, TrieBuf}, syl, zhuyin::Bopomofo};
    ///
    /// let mut dict = TrieBuf::new_in_memory();
    /// dict.add_phrase(&[syl![Bopomofo::C, Bopomofo::E, Bopomofo::TONE4]], ("測", 100).into())?;
    /// # Ok(())
    /// # }
    /// ```
    /// TODO: doc
    fn add_phrase(
        &mut self,
        syllables: &dyn SyllableSlice,
        phrase: Phrase,
    ) -> Result<(), UpdateDictionaryError>;

    /// TODO: doc
    fn update_phrase(
        &mut self,
        syllables: &dyn SyllableSlice,
        phrase: Phrase,
        user_freq: u32,
        time: u64,
    ) -> Result<(), UpdateDictionaryError>;

    /// TODO: doc
    fn remove_phrase(
        &mut self,
        syllables: &dyn SyllableSlice,
        phrase_str: &str,
    ) -> Result<(), UpdateDictionaryError>;
}

/// Errors during dictionary construction.
#[derive(Debug)]
pub struct BuildDictionaryError {
    source: Box<dyn Error + Send + Sync>,
}

impl Display for BuildDictionaryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "build dictionary error")
    }
}

impl Error for BuildDictionaryError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(self.source.as_ref())
    }
}

impl From<io::Error> for BuildDictionaryError {
    fn from(source: io::Error) -> Self {
        BuildDictionaryError {
            source: Box::new(source),
        }
    }
}

/// TODO: doc
pub trait DictionaryBuilder {
    /// TODO: doc
    fn set_info(&mut self, info: DictionaryInfo) -> Result<(), BuildDictionaryError>;
    /// TODO: doc
    fn insert(
        &mut self,
        syllables: &[Syllable],
        phrase: Phrase,
    ) -> Result<(), BuildDictionaryError>;
    /// TODO: doc
    fn build(&mut self, path: &Path) -> Result<(), BuildDictionaryError>;
    fn as_any(&self) -> &dyn Any;
}

#[cfg(test)]
mod tests {
    use crate::dictionary::{Dictionary, DictionaryBuilder, DictionaryMut};

    #[test]
    fn ensure_object_safe() {
        const _: Option<&dyn Dictionary> = None;
        const _: Option<&dyn DictionaryMut> = None;
        const _: Option<&dyn DictionaryBuilder> = None;
    }
}
