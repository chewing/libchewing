//! Dictionaries for looking up phrases.

use std::{
    borrow::{Borrow, Cow},
    cmp::Ordering,
    collections::{HashMap, HashSet},
    fmt::{Debug, Display},
    iter::Peekable,
    path::Path,
    rc::Rc,
    sync::{Arc, RwLock},
};

use thiserror::Error;

use crate::zhuyin::Syllable;

pub use layered::LayeredDictionary;
pub use loader::{SystemDictionaryLoader, UserDictionaryLoader};
pub use sqlite::{SqliteDictionary, SqliteDictionaryBuilder, SqliteDictionaryError};
pub use trie::{TrieDictionary, TrieDictionaryBuilder, TrieDictionaryStatistics};

mod layered;
mod loader;
mod sqlite;
mod trie;

/// The error type which is returned from updating a dictionary.
#[derive(Error, Debug)]
#[error("update dictionary failed")]
pub struct DictionaryUpdateError {
    /// TODO: doc
    /// TODO: change this to anyhow::Error?
    #[from]
    pub source: Option<Box<dyn std::error::Error + Send + Sync>>,
}

/// The error type which is returned from building or updating a dictionary.
#[derive(Error, Debug)]
#[error("found duplicated phrases")]
pub struct DuplicatePhraseError;

/// A collection of metadata of a dictionary.
///
/// The dictionary version and copyright information can be used in
/// configuration application.
///
/// # Examples
///
/// ```no_run
/// # use std::collections::HashMap;
/// # use chewing::dictionary::Dictionary;
/// # let dictionary = HashMap::new();
/// let about = dictionary.about();
/// assert_eq!("libchewing default", about.name.unwrap());
/// assert_eq!("Copyright (c) 2022 libchewing Core Team", about.copyright.unwrap());
/// assert_eq!("LGPL-2.1-or-later", about.license.unwrap());
/// assert_eq!("init_database 0.5.1", about.software.unwrap());
/// ```
#[derive(Debug, Clone, Default)]
pub struct DictionaryInfo {
    /// The name of the dictionary.
    pub name: Option<String>,
    /// The copyright information of the dictionary.
    ///
    /// It's recommended to include the copyright holders' names and email
    /// addresses, separated by semicolons.
    pub copyright: Option<String>,
    /// The license information of the dictionary.
    ///
    /// It's recommended to use the [SPDX license identifier](https://spdx.org/licenses/).
    pub license: Option<String>,
    /// The version of the dictionary.
    ///
    /// It's recommended to use the commit hash or revision if the dictionary is
    /// managed in a source control repository.
    pub version: Option<String>,
    /// The name of the software used to generate the dictionary.
    ///
    /// It's recommended to include the name and the version number.
    pub software: Option<String>,
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
    phrase: String,
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
        S: Into<String>,
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
        match self.freq.partial_cmp(&other.freq) {
            Some(Ordering::Equal) => {}
            ord => return ord,
        }
        self.phrase.partial_cmp(&other.phrase)
    }
}

/// Phrases are compared by their frequency first, followed by their phrase
/// string.
impl Ord for Phrase {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl AsRef<str> for Phrase {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl From<Phrase> for String {
    fn from(phrase: Phrase) -> Self {
        phrase.phrase
    }
}

impl From<Phrase> for (String, u32) {
    fn from(phrase: Phrase) -> Self {
        (phrase.phrase, phrase.freq)
    }
}

impl<S> From<(S, u32)> for Phrase
where
    S: Into<String>,
{
    fn from(tuple: (S, u32)) -> Self {
        Phrase::new(tuple.0, tuple.1)
    }
}

impl<S> From<(S, u32, u64)> for Phrase
where
    S: Into<String>,
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

/// A generic iterator over the phrases and their frequency in a dictionary.
///
/// # Examples
///
/// ```
/// use std::collections::HashMap;
///
/// use chewing::{dictionary::Dictionary, syl, zhuyin::Bopomofo};
///
/// let dict = HashMap::from([
///     (vec![syl![Bopomofo::C, Bopomofo::E, Bopomofo::TONE4]], vec![("測", 100).into()]),
/// ]);
///
/// for phrase in dict.lookup_word(
///     syl![Bopomofo::C, Bopomofo::E, Bopomofo::TONE4]
/// ) {
///     assert_eq!("測", phrase.as_str());
///     assert_eq!(100, phrase.freq());
/// }
/// ```
pub type Phrases<'a> = Box<dyn Iterator<Item = Phrase> + 'a>;

/// TODO: doc
pub type DictEntries<'a> = Box<dyn Iterator<Item = (Vec<Syllable>, Phrase)> + 'a>;

/// An interface for looking up dictionaries.
///
/// This is the main dictionary trait. For more about the concept of
/// dictionaries generally, please see the [module-level
/// documentation][crate::dictionary].
///
/// # Examples
///
/// The std [`HashMap`] implements the `Dictionary` trait so it can be used in
/// tests.
///
/// ```
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use std::collections::HashMap;
///
/// use chewing::{dictionary::Dictionary, syl, zhuyin::Bopomofo};
///
/// let mut dict = HashMap::new();
/// let dict_mut = dict.as_mut_dict().unwrap();
/// dict_mut.insert(&[syl![Bopomofo::C, Bopomofo::E, Bopomofo::TONE4]], ("測", 100).into())?;
///
/// for phrase in dict.lookup_word(
///     syl![Bopomofo::C, Bopomofo::E, Bopomofo::TONE4]
/// ) {
///     assert_eq!("測", phrase.as_str());
///     assert_eq!(100, phrase.freq());
/// }
/// # Ok(())
/// # }
/// ```
pub trait Dictionary: Debug {
    /// Returns an iterator to all single syllable words matched by the
    /// syllable, if any. The result should use a stable order each time for the
    /// same input.
    fn lookup_word(&self, syllable: Syllable) -> Phrases<'_> {
        self.lookup_phrase(&[syllable])
    }
    /// Returns an iterator to all phrases matched by the syllables, if any. The
    /// result should use a stable order each time for the same input.
    fn lookup_phrase<Syl: AsRef<Syllable>>(&self, syllables: &[Syl]) -> Phrases<'_>;
    /// Returns an iterator to all phrases in the dictionary.
    fn entries(&self) -> DictEntries<'_>;
    /// Returns information about the dictionary instance.
    fn about(&self) -> DictionaryInfo;

    /// An method for updating dictionaries.
    ///
    /// For more about the concept of dictionaries generally, please see the
    /// [module-level documentation][crate::dictionary].
    ///
    /// # Examples
    ///
    /// The std [`HashMap`] implements the `DictionaryMut` trait so it can be used in
    /// tests.
    ///
    /// ```
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use std::collections::HashMap;
    ///
    /// use chewing::{dictionary::Dictionary, syl, zhuyin::Bopomofo};
    ///
    /// let mut dict = HashMap::new();
    /// let dict_mut = dict.as_mut_dict().unwrap();
    /// dict_mut.insert(&[syl![Bopomofo::C, Bopomofo::E, Bopomofo::TONE4]], ("測", 100).into())?;
    /// # Ok(())
    /// # }
    /// ```
    /// TODO: doc
    fn insert<Syl: AsRef<Syllable>>(
        &mut self,
        syllables: &[Syl],
        phrase: Phrase,
    ) -> Result<(), DictionaryUpdateError> {
        Err(DictionaryUpdateError { source: None })
    }

    /// TODO: doc
    fn update<Syl: AsRef<Syllable>>(
        &mut self,
        syllables: &[Syl],
        phrase: Phrase,
        user_freq: u32,
        time: u64,
    ) -> Result<(), DictionaryUpdateError> {
        Err(DictionaryUpdateError { source: None })
    }

    /// TODO: doc
    fn remove<Syl: AsRef<Syllable>>(
        &mut self,
        syllables: &[Syl],
        phrase_str: &str,
    ) -> Result<(), DictionaryUpdateError> {
        Err(DictionaryUpdateError { source: None })
    }
}

/// TODO: doc
#[derive(Error, Debug)]
#[error("build dictionary error")]
pub struct BuildDictionaryError {
    #[from]
    source: Box<dyn std::error::Error + Send + Sync>,
}

impl From<std::io::Error> for BuildDictionaryError {
    fn from(source: std::io::Error) -> Self {
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
}

impl Dictionary for HashMap<Vec<Syllable>, Vec<Phrase>> {
    fn lookup_phrase<Syl: AsRef<Syllable>>(&self, syllables: &[Syl]) -> Phrases<'_> {
        let syllables = syllables
            .into_iter()
            .map(|s| s.as_ref().clone())
            .collect::<Vec<_>>();
        self.get(&syllables)
            .cloned()
            .map(|v| Box::new(v.into_iter()) as Phrases<'_>)
            .unwrap_or_else(|| Box::new(std::iter::empty()))
    }

    fn entries(&self) -> DictEntries<'_> {
        Box::new(
            self.iter()
                .flat_map(|(k, v)| v.iter().map(|phrase| (k.clone(), phrase.clone()))),
        )
    }

    fn about(&self) -> DictionaryInfo {
        Default::default()
    }

    fn insert<Syl: AsRef<Syllable>>(
        &mut self,
        syllables: &[Syl],
        phrase: Phrase,
    ) -> Result<(), DictionaryUpdateError> {
        let syllables = syllables
            .into_iter()
            .map(|s| s.as_ref().clone())
            .collect::<Vec<_>>();
        let vec = self.entry(syllables.to_vec()).or_default();
        if vec.iter().any(|it| it.as_str() == phrase.as_str()) {
            return Err(DictionaryUpdateError {
                source: Some(Box::new(DuplicatePhraseError)),
            });
        }
        vec.push(phrase);
        Ok(())
    }

    fn update<Syl: AsRef<Syllable>>(
        &mut self,
        _syllables: &[Syl],
        _phrase: Phrase,
        _user_freq: u32,
        _time: u64,
    ) -> Result<(), DictionaryUpdateError> {
        Ok(())
    }

    fn remove<Syl: AsRef<Syllable>>(
        &mut self,
        syllables: &[Syl],
        phrase_str: &str,
    ) -> Result<(), DictionaryUpdateError> {
        let syllables = syllables
            .into_iter()
            .map(|s| s.as_ref().clone())
            .collect::<Vec<_>>();
        let vec = self.entry(syllables.to_vec()).or_default();
        *vec = vec
            .iter()
            .cloned()
            .filter(|p| p.as_str() != phrase_str)
            .collect::<Vec<_>>();
        Ok(())
    }
}

/// A block list contains unwanted phrases.
pub trait BlockList: Debug {
    /// Returns if whether a phrase is in the block list.
    fn is_blocked(&self, phrase: &str) -> bool;
}

impl BlockList for HashSet<String> {
    fn is_blocked(&self, phrase: &str) -> bool {
        self.contains(phrase)
    }
}

impl BlockList for () {
    fn is_blocked(&self, _phrase: &str) -> bool {
        false
    }
}

#[derive(Debug)]
pub enum AnyDictionary {
    SqliteDictionary(SqliteDictionary),
    TrieDictionary(TrieDictionary),
}

impl Dictionary for AnyDictionary {
    fn lookup_phrase<Syl: AsRef<Syllable>>(&self, syllables: &[Syl]) -> Phrases<'_> {
        match self {
            AnyDictionary::SqliteDictionary(dict) => dict.lookup_phrase(syllables),
            AnyDictionary::TrieDictionary(dict) => dict.lookup_phrase(syllables),
        }
    }

    fn entries(&self) -> DictEntries<'_> {
        match self {
            AnyDictionary::SqliteDictionary(dict) => dict.entries(),
            AnyDictionary::TrieDictionary(dict) => dict.entries(),
        }
    }

    fn about(&self) -> DictionaryInfo {
        match self {
            AnyDictionary::SqliteDictionary(dict) => dict.about(),
            AnyDictionary::TrieDictionary(dict) => dict.about(),
        }
    }

    fn insert<Syl: AsRef<Syllable>>(
        &mut self,
        syllables: &[Syl],
        phrase: Phrase,
    ) -> Result<(), DictionaryUpdateError> {
        match self {
            AnyDictionary::SqliteDictionary(dict) => dict.insert(syllables, phrase),
            AnyDictionary::TrieDictionary(dict) => dict.insert(syllables, phrase),
        }
    }

    fn update<Syl: AsRef<Syllable>>(
        &mut self,
        syllables: &[Syl],
        phrase: Phrase,
        user_freq: u32,
        time: u64,
    ) -> Result<(), DictionaryUpdateError> {
        match self {
            AnyDictionary::SqliteDictionary(dict) => {
                dict.update(syllables, phrase, user_freq, time)
            }
            AnyDictionary::TrieDictionary(dict) => dict.update(syllables, phrase, user_freq, time),
        }
    }

    fn remove<Syl: AsRef<Syllable>>(
        &mut self,
        syllables: &[Syl],
        phrase_str: &str,
    ) -> Result<(), DictionaryUpdateError> {
        match self {
            AnyDictionary::SqliteDictionary(dict) => dict.remove(syllables, phrase_str),
            AnyDictionary::TrieDictionary(dict) => dict.remove(syllables, phrase_str),
        }
    }
}

impl From<SqliteDictionary> for AnyDictionary {
    fn from(value: SqliteDictionary) -> Self {
        Self::SqliteDictionary(value)
    }
}

impl From<TrieDictionary> for AnyDictionary {
    fn from(value: TrieDictionary) -> Self {
        Self::TrieDictionary(value)
    }
}
