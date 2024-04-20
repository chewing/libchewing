use std::{
    cmp,
    collections::{btree_map::Entry, BTreeMap},
    iter,
};

use log::error;

use crate::zhuyin::SyllableSlice;

use super::{Dictionary, DictionaryInfo, Entries, Phrase, UpdateDictionaryError};

/// A collection of dictionaries that returns the union of the lookup results.
/// # Examples
///
/// ```
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
///
/// use chewing::{dictionary::{LayeredDictionary, TrieBufDictionary, Dictionary, Phrase}, syl, zhuyin::Bopomofo};
///
/// let sys_dict = TrieBufDictionary::from([(
///     vec![syl![Bopomofo::C, Bopomofo::E, Bopomofo::TONE4]],
///     vec![("測", 1).into(), ("冊", 1).into(), ("側", 1).into()]
/// )]);
/// let user_dict = TrieBufDictionary::from([(
///     vec![syl![Bopomofo::C, Bopomofo::E, Bopomofo::TONE4]],
///     vec![("策", 100).into(), ("冊", 100).into()]
/// )]);
///
/// let dict = LayeredDictionary::new(vec![Box::new(sys_dict)], Box::new(user_dict));
/// assert_eq!(
///     [
///         ("側", 1, 0).into(),
///         ("冊", 100, 0).into(),
///         ("測", 1, 0).into(),
///         ("策", 100, 0).into(),
///     ]
///     .into_iter()
///     .collect::<Vec<Phrase>>(),
///     dict.lookup_all_phrases(&[
///         syl![Bopomofo::C, Bopomofo::E, Bopomofo::TONE4]
///     ]),
/// );
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
pub struct LayeredDictionary {
    sys_dict: Vec<Box<dyn Dictionary>>,
    user_dict: Box<dyn Dictionary>,
}

impl LayeredDictionary {
    /// Creates a new `LayeredDictionary` with the list of dictionaries.
    pub fn new(
        sys_dict: Vec<Box<dyn Dictionary>>,
        user_dict: Box<dyn Dictionary>,
    ) -> LayeredDictionary {
        LayeredDictionary {
            sys_dict,
            user_dict,
        }
    }
    pub fn user_dict(&mut self) -> &mut dyn Dictionary {
        self.user_dict.as_mut()
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
    /// Foreach d in d_layers
    ///   Foreach phrase, freq in d.lookup_syllables()
    ///     If phrase in phrases
    ///       Set phrases[phrase].freq = max(phrases[phrase].freq, freq)
    ///     Else
    ///       Add phrases <- (phrase, freq)
    /// ```
    fn lookup_first_n_phrases(&self, syllables: &dyn SyllableSlice, first: usize) -> Vec<Phrase> {
        let mut sort_map: BTreeMap<String, usize> = BTreeMap::new();
        let mut phrases: Vec<Phrase> = Vec::new();

        self.sys_dict
            .iter()
            .chain(iter::once(&self.user_dict))
            .for_each(|d| {
                for phrase in d.lookup_all_phrases(syllables) {
                    debug_assert!(!phrase.as_str().is_empty());
                    match sort_map.entry(phrase.to_string()) {
                        Entry::Occupied(entry) => {
                            let index = *entry.get();
                            phrases[index] = cmp::max(&phrase, &phrases[index]).clone();
                        }
                        Entry::Vacant(entry) => {
                            entry.insert(phrases.len());
                            phrases.push(phrase);
                        }
                    }
                }
            });
        phrases.truncate(first);
        phrases
    }

    /// Returns all entries from all dictionaries.
    ///
    /// **NOTE**: Duplicate entries are not removed.
    fn entries(&self) -> Entries<'_> {
        Box::new(
            self.sys_dict
                .iter()
                .chain(iter::once(&self.user_dict))
                .flat_map(|dict| dict.entries()),
        )
    }

    fn about(&self) -> DictionaryInfo {
        DictionaryInfo {
            name: "Built-in LayeredDictionary".to_string(),
            ..Default::default()
        }
    }

    fn reopen(&mut self) -> Result<(), UpdateDictionaryError> {
        self.user_dict.reopen()
    }

    fn flush(&mut self) -> Result<(), UpdateDictionaryError> {
        self.user_dict.flush()
    }

    fn add_phrase(
        &mut self,
        syllables: &dyn SyllableSlice,
        phrase: Phrase,
    ) -> Result<(), UpdateDictionaryError> {
        if phrase.as_str().is_empty() {
            error!("BUG! added phrase is empty");
            return Ok(());
        }
        self.user_dict.add_phrase(syllables, phrase)
    }

    fn update_phrase(
        &mut self,
        syllables: &dyn SyllableSlice,
        phrase: Phrase,
        user_freq: u32,
        time: u64,
    ) -> Result<(), UpdateDictionaryError> {
        if phrase.as_str().is_empty() {
            error!("BUG! added phrase is empty");
            return Ok(());
        }
        self.user_dict
            .update_phrase(syllables, phrase, user_freq, time)
    }

    fn remove_phrase(
        &mut self,
        syllables: &dyn SyllableSlice,
        phrase_str: &str,
    ) -> Result<(), UpdateDictionaryError> {
        self.user_dict.remove_phrase(syllables, phrase_str)
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use crate::{
        dictionary::{Dictionary, TrieBufDictionary},
        syl,
        zhuyin::Bopomofo,
    };

    use super::LayeredDictionary;

    #[test]
    fn test_entries() -> Result<(), Box<dyn Error>> {
        let sys_dict = TrieBufDictionary::from([(
            vec![syl![Bopomofo::C, Bopomofo::E, Bopomofo::TONE4]],
            vec![("測", 1).into(), ("冊", 1).into(), ("側", 1).into()],
        )]);
        let user_dict = TrieBufDictionary::from([(
            vec![syl![Bopomofo::C, Bopomofo::E, Bopomofo::TONE4]],
            vec![("策", 100).into(), ("冊", 100).into()],
        )]);

        let dict = LayeredDictionary::new(vec![Box::new(sys_dict)], Box::new(user_dict));
        assert_eq!(
            [
                (
                    vec![syl![Bopomofo::C, Bopomofo::E, Bopomofo::TONE4]],
                    ("側", 1, 0).into()
                ),
                (
                    vec![syl![Bopomofo::C, Bopomofo::E, Bopomofo::TONE4]],
                    ("冊", 1, 0).into()
                ),
                (
                    vec![syl![Bopomofo::C, Bopomofo::E, Bopomofo::TONE4]],
                    ("測", 1, 0).into()
                ),
                (
                    vec![syl![Bopomofo::C, Bopomofo::E, Bopomofo::TONE4]],
                    ("冊", 100, 0).into()
                ),
                (
                    vec![syl![Bopomofo::C, Bopomofo::E, Bopomofo::TONE4]],
                    ("策", 100, 0).into()
                ),
            ]
            .into_iter()
            .collect::<Vec<_>>(),
            dict.entries().collect::<Vec<_>>(),
        );
        Ok(())
    }
}
