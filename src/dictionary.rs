//! Dictionaries for looking up phrases.

use std::{
    borrow::{Borrow, Cow},
    cmp::Ordering,
    collections::{HashMap, HashSet},
    fmt::{Debug, Display},
    path::Path,
    rc::Rc,
    sync::Arc,
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
    pub source: Box<dyn std::error::Error + Send + Sync>,
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
pub struct Phrase<'a> {
    phrase: Cow<'a, str>,
    freq: u32,
    last_used: Option<u64>,
}

impl<'a> Phrase<'a> {
    /// Creates a new `Phrase`.
    ///
    /// # Examples
    ///
    /// ```
    /// use chewing::dictionary::Phrase;
    ///
    /// let phrase = Phrase::new("新", 1);
    /// ```
    pub fn new<S>(phrase: S, freq: u32) -> Phrase<'a>
    where
        S: Into<Cow<'a, str>>,
    {
        Phrase {
            phrase: phrase.into(),
            freq,
            last_used: None,
        }
    }
    /// Sets the last used time of the phrase.
    pub fn with_time(mut self, last_used: u64) -> Phrase<'a> {
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

    /// Turns the phrase into owned data.
    ///
    /// Clones the data if it is not already owned.
    ///
    /// # Examples
    ///
    /// ```
    /// use chewing::dictionary::Phrase;
    ///
    /// let phrase = Phrase::new("詞", 100);
    ///
    /// assert_eq!("詞", phrase.into_owned().as_str());
    /// ```
    pub fn into_owned(self) -> Phrase<'static> {
        Phrase {
            phrase: Cow::Owned(self.phrase.into_owned()),
            freq: self.freq,
            last_used: self.last_used,
        }
    }
}

/// Phrases are compared by their frequency first, followed by their phrase
/// string.
impl PartialOrd for Phrase<'_> {
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
impl Ord for Phrase<'_> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl AsRef<str> for Phrase<'_> {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl From<Phrase<'_>> for String {
    fn from(phrase: Phrase<'_>) -> Self {
        phrase.phrase.into_owned()
    }
}

impl From<Phrase<'_>> for (String, u32) {
    fn from(phrase: Phrase<'_>) -> Self {
        (phrase.phrase.into_owned(), phrase.freq)
    }
}

impl<'a, S> From<(S, u32)> for Phrase<'a>
where
    S: Into<Cow<'a, str>>,
{
    fn from(tuple: (S, u32)) -> Self {
        Phrase::new(tuple.0, tuple.1)
    }
}

impl<'a, S> From<(S, u32, u64)> for Phrase<'a>
where
    S: Into<Cow<'a, str>>,
{
    fn from(tuple: (S, u32, u64)) -> Self {
        Phrase::new(tuple.0, tuple.1).with_time(tuple.2)
    }
}

impl Display for Phrase<'_> {
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
pub type Phrases<'a, 'p> = Box<dyn Iterator<Item = Phrase<'p>> + 'a>;

/// TODO: doc
pub type DictEntries<'a, 'p> = Box<dyn Iterator<Item = (Vec<Syllable>, Phrase<'p>)> + 'a>;

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
    fn lookup_word(&self, syllable: Syllable) -> Phrases<'_, '_> {
        self.lookup_phrase(&[syllable])
    }
    /// Returns an iterator to all phrases matched by the syllables, if any. The
    /// result should use a stable order each time for the same input.
    fn lookup_phrase(&self, syllables: &[Syllable]) -> Phrases<'_, '_>;
    /// Returns an iterator to all phrases in the dictionary.
    fn entries(&self) -> DictEntries<'_, '_>;
    /// Returns information about the dictionary instance.
    fn about(&self) -> DictionaryInfo;
    /// Returns a mutable reference to the dictionary if the underlying
    /// implementation allows update.
    fn as_mut_dict(&mut self) -> Option<&mut dyn DictionaryMut>;
}

/// An interface for updating dictionaries.
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
pub trait DictionaryMut {
    /// TODO: doc
    fn insert(
        &mut self,
        syllables: &[Syllable],
        phrase: Phrase<'static>,
    ) -> Result<(), DictionaryUpdateError>;

    /// TODO: doc
    fn update(
        &mut self,
        syllables: &[Syllable],
        phrase: Phrase<'_>,
        user_freq: u32,
        time: u64,
    ) -> Result<(), DictionaryUpdateError>;

    /// TODO: doc
    fn remove(
        &mut self,
        syllables: &[Syllable],
        phrase_str: &str,
    ) -> Result<(), DictionaryUpdateError>;
}

impl<T> Dictionary for Box<T>
where
    T: Dictionary + ?Sized,
{
    fn lookup_phrase(&self, syllables: &[Syllable]) -> Phrases<'_, '_> {
        self.as_ref().lookup_phrase(syllables)
    }

    fn entries(&self) -> DictEntries<'_, '_> {
        self.as_ref().entries()
    }

    fn about(&self) -> DictionaryInfo {
        self.as_ref().about()
    }

    fn as_mut_dict(&mut self) -> Option<&mut dyn DictionaryMut> {
        self.as_mut().as_mut_dict()
    }
}

impl<T> Dictionary for Rc<T>
where
    T: Dictionary + ?Sized,
{
    fn lookup_phrase(&self, syllables: &[Syllable]) -> Phrases<'_, '_> {
        self.as_ref().lookup_phrase(syllables)
    }

    fn entries(&self) -> DictEntries<'_, '_> {
        self.as_ref().entries()
    }

    fn about(&self) -> DictionaryInfo {
        self.as_ref().about()
    }

    fn as_mut_dict(&mut self) -> Option<&mut dyn DictionaryMut> {
        Rc::get_mut(self).and_then(|this| this.as_mut_dict())
    }
}

impl<T> Dictionary for Arc<T>
where
    T: Dictionary + ?Sized,
{
    fn lookup_phrase(&self, syllables: &[Syllable]) -> Phrases<'_, '_> {
        self.as_ref().lookup_phrase(syllables)
    }

    fn entries(&self) -> DictEntries<'_, '_> {
        self.as_ref().entries()
    }

    fn about(&self) -> DictionaryInfo {
        self.as_ref().about()
    }

    fn as_mut_dict(&mut self) -> Option<&mut dyn DictionaryMut> {
        Arc::get_mut(self).and_then(|this| this.as_mut_dict())
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
        phrase: Phrase<'_>,
    ) -> Result<(), BuildDictionaryError>;
    /// TODO: doc
    fn build(&mut self, path: &Path) -> Result<(), BuildDictionaryError>;
}

impl Dictionary for HashMap<Vec<Syllable>, Vec<Phrase<'static>>> {
    fn lookup_phrase(&self, syllables: &[Syllable]) -> Phrases<'_, '_> {
        self.get(syllables)
            .cloned()
            .map(|v| Box::new(v.into_iter()) as Phrases<'_, '_>)
            .unwrap_or_else(|| Box::new(std::iter::empty()))
    }

    fn entries(&self) -> DictEntries<'_, '_> {
        Box::new(
            self.iter()
                .flat_map(|(k, v)| v.iter().map(|phrase| (k.clone(), phrase.clone()))),
        )
    }

    fn about(&self) -> DictionaryInfo {
        Default::default()
    }

    fn as_mut_dict(&mut self) -> Option<&mut dyn DictionaryMut> {
        Some(self)
    }
}

impl DictionaryMut for HashMap<Vec<Syllable>, Vec<Phrase<'static>>> {
    fn insert(
        &mut self,
        syllables: &[Syllable],
        phrase: Phrase<'static>,
    ) -> Result<(), DictionaryUpdateError> {
        let vec = self.entry(syllables.to_vec()).or_default();
        if vec.iter().any(|it| it.as_str() == phrase.as_str()) {
            return Err(DictionaryUpdateError {
                source: Box::new(DuplicatePhraseError),
            });
        }
        vec.push(phrase);
        Ok(())
    }

    fn update(
        &mut self,
        _syllables: &[Syllable],
        _phrase: Phrase<'_>,
        _user_freq: u32,
        _time: u64,
    ) -> Result<(), DictionaryUpdateError> {
        Ok(())
    }

    fn remove(
        &mut self,
        syllables: &[Syllable],
        phrase_str: &str,
    ) -> Result<(), DictionaryUpdateError> {
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
    fn lookup_phrase(&self, syllables: &[Syllable]) -> Phrases<'_, '_> {
        match self {
            AnyDictionary::SqliteDictionary(dict) => dict.lookup_phrase(syllables),
            AnyDictionary::TrieDictionary(dict) => dict.lookup_phrase(syllables),
        }
    }

    fn entries(&self) -> DictEntries<'_, '_> {
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

    fn as_mut_dict(&mut self) -> Option<&mut dyn DictionaryMut> {
        match self {
            AnyDictionary::SqliteDictionary(dict) => dict.as_mut_dict(),
            AnyDictionary::TrieDictionary(dict) => dict.as_mut_dict(),
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
