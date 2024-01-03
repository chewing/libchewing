use std::hash::{Hash, Hasher};

use indexmap::IndexSet;

use crate::zhuyin::SyllableSlice;

use super::{BlockList, DictEntries, Dictionary, DictionaryInfo, DictionaryUpdateError, Phrase};

/// A collection of dictionaries that returns the union of the lookup results.
/// # Examples
///
/// ```
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use std::collections::{HashMap, HashSet};
///
/// use chewing::{dictionary::{LayeredDictionary, Dictionary}, syl, zhuyin::Bopomofo};
///
/// let mut sys_dict = Box::new(HashMap::new());
/// let mut user_dict = Box::new(HashMap::new());
/// sys_dict.insert(
///     vec![syl![Bopomofo::C, Bopomofo::E, Bopomofo::TONE4]],
///     vec![("測", 1).into(), ("冊", 1).into(), ("側", 1).into()]
/// );
/// user_dict.insert(
///     vec![syl![Bopomofo::C, Bopomofo::E, Bopomofo::TONE4]],
///     vec![("策", 100).into(), ("冊", 100).into()]
/// );
///
/// let user_block_list = Box::new(HashSet::from(["側".to_string()]));
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
///     dict.lookup_all_phrases(&[
///         syl![Bopomofo::C, Bopomofo::E, Bopomofo::TONE4]
///     ])
///     .into_iter()
///     .collect::<HashSet<_>>(),
/// );
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
pub struct LayeredDictionary {
    inner: Vec<Box<dyn Dictionary>>,
    blocked: Vec<Box<dyn BlockList>>,
}

impl LayeredDictionary {
    /// Creates a new `LayeredDictionary` with the list of dictionaries and
    /// block lists.
    pub fn new(
        dictionaries: Vec<Box<dyn Dictionary>>,
        block_lists: Vec<Box<dyn BlockList>>,
    ) -> LayeredDictionary {
        LayeredDictionary {
            inner: dictionaries,
            blocked: block_lists,
        }
    }
    fn is_blocked(&self, phrase: &str) -> bool {
        self.blocked.iter().any(|b| b.is_blocked(phrase))
    }
    pub fn base(&mut self) -> &mut dyn Dictionary {
        self.inner[0].as_mut()
    }
}

impl Dictionary for LayeredDictionary {
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
    fn lookup_first_n_phrases(&self, syllables: &dyn SyllableSlice, first: usize) -> Vec<Phrase> {
        let (base, layers) = match self.inner.split_first() {
            Some(d) => d,
            None => return vec![],
        };
        let mut phrases = IndexSet::with_capacity(128);
        phrases.extend(
            base.lookup_all_phrases(syllables)
                .into_iter()
                .map(LookupPhrase),
        );
        for d in layers {
            for phrase in d.lookup_all_phrases(syllables) {
                phrases.replace(LookupPhrase(phrase));
            }
        }
        phrases
            .into_iter()
            .map(|p| p.0)
            .filter(|phrase| !self.is_blocked(&phrase.phrase))
            .take(first)
            .collect()
    }

    fn entries(&self) -> Option<DictEntries> {
        None
    }

    fn about(&self) -> DictionaryInfo {
        DictionaryInfo {
            name: Some("Built-in LayeredDictionary".to_string()),
            ..Default::default()
        }
    }

    fn reopen(&mut self) -> Result<(), DictionaryUpdateError> {
        self.inner.iter_mut().map(|it| it.reopen()).collect()
    }

    fn flush(&mut self) -> Result<(), DictionaryUpdateError> {
        self.inner.iter_mut().map(|it| it.flush()).collect()
    }

    fn add_phrase(
        &mut self,
        syllables: &dyn SyllableSlice,
        phrase: Phrase,
    ) -> Result<(), DictionaryUpdateError> {
        for dict in &mut self.inner {
            // TODO check mutability?
            let _ = dict.add_phrase(syllables, phrase.clone());
        }
        Ok(())
    }

    fn update_phrase(
        &mut self,
        syllables: &dyn SyllableSlice,
        phrase: Phrase,
        user_freq: u32,
        time: u64,
    ) -> Result<(), DictionaryUpdateError> {
        for dict in &mut self.inner {
            // TODO check mutability?
            let _ = dict.update_phrase(syllables, phrase.clone(), user_freq, time);
        }
        Ok(())
    }

    fn remove_phrase(
        &mut self,
        syllables: &dyn SyllableSlice,
        phrase_str: &str,
    ) -> Result<(), DictionaryUpdateError> {
        for dict in &mut self.inner {
            // TODO check mutability?
            let _ = dict.remove_phrase(syllables, phrase_str);
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
