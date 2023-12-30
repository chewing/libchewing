use std::hash::{Hash, Hasher};

use indexmap::IndexSet;

use crate::zhuyin::Syllable;

use super::{
    BlockList, DictEntries, Dictionary, DictionaryInfo, DictionaryUpdateError, Phrase, Phrases,
};

/// A collection of dictionaries that returns the union of the lookup results.
/// # Examples
///
/// ```
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use std::collections::{HashMap, HashSet};
///
/// use chewing::{dictionary::{LayeredDictionary, Dictionary}, syl, zhuyin::Bopomofo};
///
/// let mut sys_dict = HashMap::new();
/// let mut user_dict = HashMap::new();
/// sys_dict.insert(
///     vec![syl![Bopomofo::C, Bopomofo::E, Bopomofo::TONE4]],
///     vec![("測", 1).into(), ("冊", 1).into(), ("側", 1).into()]
/// );
/// user_dict.insert(
///     vec![syl![Bopomofo::C, Bopomofo::E, Bopomofo::TONE4]],
///     vec![("策", 100).into(), ("冊", 100).into()]
/// );
///
/// let user_block_list = HashSet::from(["側".to_string()]);
///
/// let dict = LayeredDictionary::new(vec![sys_dict, user_dict], vec![user_block_list]);
/// assert_eq!(
///     [
///         ("策", 100).into(),
///         ("冊", 100).into(),
///         ("測", 1).into(),
///     ]
///     .into_iter()
///     .collect::<HashSet<_>>(),
///     dict.lookup_phrase(&[
///         syl![Bopomofo::C, Bopomofo::E, Bopomofo::TONE4]
///     ])
///     .collect::<HashSet<_>>(),
/// );
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
pub struct LayeredDictionary<T, B>
where
    T: Dictionary,
    B: BlockList,
{
    inner: Vec<T>,
    blocked: Vec<B>,
}

impl<T, B> LayeredDictionary<T, B>
where
    T: Dictionary,
    B: BlockList,
{
    /// Creates a new `LayeredDictionary` with the list of dictionaries and
    /// block lists.
    pub fn new(dictionaries: Vec<T>, block_lists: Vec<B>) -> LayeredDictionary<T, B> {
        LayeredDictionary {
            inner: dictionaries,
            blocked: block_lists,
        }
    }
    fn is_blocked(&self, phrase: &str) -> bool {
        self.blocked.iter().any(|b| b.is_blocked(phrase))
    }
    pub fn base(&mut self) -> &mut T {
        &mut self.inner[0]
    }
}

impl<T, B> Dictionary for LayeredDictionary<T, B>
where
    T: Dictionary,
    B: BlockList,
{
    /// Lookup phrases from all underlying dictionaries.
    ///
    /// Phrases are ordered by their first apperance in the underlying dictionaries.
    ///
    /// Pseudo code
    ///
    /// ```pseudo_code
    /// Set phrases = list()
    /// Set [d_base, d_layers] = d_list
    /// Foreach phrase, freq in d_base.lookup(syllables)
    ///   Add phrases <- (phrase, freq)
    /// Foreach d in d_layers
    ///   Foreach phrase, freq in d.lookup_syllables)
    ///     If phrase in phrases
    ///       Set phrases[phrase].freq = freq
    ///     Else
    ///       Add phrases <- (phrase, freq)
    /// ```
    fn lookup_phrase<Syl: AsRef<Syllable>>(&self, syllables: &[Syl]) -> Phrases<'_> {
        let (base, layers) = match self.inner.split_first() {
            Some(d) => d,
            None => return Box::new(std::iter::empty()),
        };
        let mut phrases = IndexSet::with_capacity(128);
        phrases.extend(base.lookup_phrase(syllables).map(LookupPhrase));
        for d in layers {
            for phrase in d.lookup_phrase(syllables) {
                phrases.replace(LookupPhrase(phrase));
            }
        }
        Box::new(
            phrases
                .into_iter()
                .map(|p| p.0)
                .filter(|phrase| !self.is_blocked(&phrase.phrase)),
        )
    }

    fn entries(&self) -> DictEntries<'_> {
        todo!("entries from all layers")
        // Box::new(std::iter::empty())
    }

    fn about(&self) -> DictionaryInfo {
        DictionaryInfo {
            name: Some("Built-in LayeredDictionary".to_string()),
            ..Default::default()
        }
    }

    fn insert<Syl: AsRef<Syllable>>(
        &mut self,
        syllables: &[Syl],
        phrase: Phrase,
    ) -> Result<(), DictionaryUpdateError> {
        for dict in &mut self.inner {
            // TODO check mutability?
            let _ = dict.insert(syllables, phrase.clone());
        }
        Ok(())
    }

    fn update<Syl: AsRef<Syllable>>(
        &mut self,
        syllables: &[Syl],
        phrase: Phrase,
        user_freq: u32,
        time: u64,
    ) -> Result<(), DictionaryUpdateError> {
        for dict in &mut self.inner {
            // TODO check mutability?
            let _ = dict.update(syllables, phrase.clone(), user_freq, time);
        }
        Ok(())
    }

    fn remove<Syl: AsRef<Syllable>>(
        &mut self,
        syllables: &[Syl],
        phrase_str: &str,
    ) -> Result<(), DictionaryUpdateError> {
        for dict in &mut self.inner {
            // TODO check mutability?
            let _ = dict.remove(syllables, phrase_str);
        }
        Ok(())
    }
}

#[derive(Debug, Eq)]
struct LookupPhrase(Phrase);

impl Hash for LookupPhrase {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.phrase.hash(state);
    }
}

impl PartialEq for LookupPhrase {
    fn eq(&self, other: &Self) -> bool {
        self.0.phrase == other.0.phrase
    }
}
