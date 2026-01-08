use std::{
    collections::{BTreeMap, btree_map::Entry},
    iter,
};

use log::error;

use super::{Dictionary, DictionaryInfo, Entries, LookupStrategy, Phrase, UpdateDictionaryError};
use crate::zhuyin::Syllable;

/// A collection of dictionaries that returns the union of the lookup results.
/// # Examples
///
/// ```
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
///
/// use chewing::{
///     dictionary::{Dictionary, Layered, LookupStrategy, Phrase, TrieBuf},
///     syl,
///     zhuyin::Bopomofo,
/// };
///
/// let sys_dict = TrieBuf::from([(
///     vec![syl![Bopomofo::C, Bopomofo::E, Bopomofo::TONE4]],
///     vec![("測", 1), ("冊", 1), ("側", 1)]
/// )]);
/// let user_dict = TrieBuf::from([(
///     vec![syl![Bopomofo::C, Bopomofo::E, Bopomofo::TONE4]],
///     vec![("策", 100), ("冊", 100)]
/// )]);
///
/// let dict = Layered::new(vec![Box::new(sys_dict)], Box::new(user_dict));
/// assert_eq!(
///     [
///         ("側", 1, 0).into(),
///         ("冊", 101, 0).into(),
///         ("測", 1, 0).into(),
///         ("策", 100, 0).into(),
///     ]
///     .into_iter()
///     .collect::<Vec<Phrase>>(),
///     dict.lookup(&[
///         syl![Bopomofo::C, Bopomofo::E, Bopomofo::TONE4]
///     ], LookupStrategy::Standard),
/// );
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
pub struct Layered {
    sys_dict: Vec<Box<dyn Dictionary>>,
    user_dict: Box<dyn Dictionary>,
}

impl Layered {
    /// Creates a new `Layered` with the list of dictionaries.
    pub fn new(sys_dict: Vec<Box<dyn Dictionary>>, user_dict: Box<dyn Dictionary>) -> Layered {
        Layered {
            sys_dict,
            user_dict,
        }
    }
    pub fn user_dict(&mut self) -> &mut dyn Dictionary {
        self.user_dict.as_mut()
    }
}

impl Dictionary for Layered {
    /// Lookup phrases from all underlying dictionaries.
    ///
    /// Phrases are ordered by their first apperance in the underlying
    /// dictionaries.
    ///
    /// When a phrase appears in multiple dictionaries, the final
    /// frequency is the sum of all frequency in all dictionaries.
    ///
    /// Pseudo code
    ///
    /// ```pseudo_code
    /// Set phrases = list()
    /// Foreach d in d_layers
    ///   Foreach phrase, freq in d.lookup_syllables()
    ///     If phrase in phrases
    ///       Set phrases[phrase].freq += freq
    ///     Else
    ///       Add phrases <- (phrase, freq)
    /// ```
    fn lookup(&self, syllables: &[Syllable], strategy: LookupStrategy) -> Vec<Phrase> {
        let mut sort_map: BTreeMap<String, usize> = BTreeMap::new();
        let mut phrases: Vec<Phrase> = Vec::new();

        self.sys_dict
            .iter()
            .chain(iter::once(&self.user_dict))
            .for_each(|d| {
                for phrase in d.lookup(syllables, strategy) {
                    debug_assert!(!phrase.as_str().is_empty());
                    match sort_map.entry(phrase.to_string()) {
                        Entry::Occupied(entry) => {
                            let index = *entry.get();
                            phrases[index].freq += phrase.freq;
                            phrases[index].last_used =
                                match (phrases[index].last_used, phrase.last_used) {
                                    (Some(orig), Some(new)) => Some(u64::max(orig, new)),
                                    (Some(orig), None) => Some(orig),
                                    (None, Some(new)) => Some(new),
                                    (None, None) => None,
                                };
                        }
                        Entry::Vacant(entry) => {
                            entry.insert(phrases.len());
                            phrases.push(phrase);
                        }
                    }
                }
            });
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
            name: "Built-in Layered".to_string(),
            ..Default::default()
        }
    }

    fn path(&self) -> Option<&std::path::Path> {
        None
    }

    fn reopen(&mut self) -> Result<(), UpdateDictionaryError> {
        self.user_dict.reopen()
    }

    fn flush(&mut self) -> Result<(), UpdateDictionaryError> {
        self.user_dict.flush()
    }

    fn add_phrase(
        &mut self,
        syllables: &[Syllable],
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
        syllables: &[Syllable],
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
        syllables: &[Syllable],
        phrase_str: &str,
    ) -> Result<(), UpdateDictionaryError> {
        self.user_dict.remove_phrase(syllables, phrase_str)
    }
}

#[cfg(test)]
mod tests {
    use std::{
        error::Error,
        io::{Cursor, Seek},
    };

    use super::Layered;
    use crate::{
        dictionary::{
            Dictionary, DictionaryBuilder, LookupStrategy, Phrase, Trie, TrieBuf, TrieBuilder,
        },
        syl,
        zhuyin::Bopomofo,
    };

    #[test]
    fn test_entries() -> Result<(), Box<dyn Error>> {
        let sys_dict = TrieBuf::from([(
            vec![syl![Bopomofo::C, Bopomofo::E, Bopomofo::TONE4]],
            vec![("測", 1), ("冊", 1), ("側", 1)],
        )]);
        let user_dict = TrieBuf::from([(
            vec![syl![Bopomofo::C, Bopomofo::E, Bopomofo::TONE4]],
            vec![("策", 100), ("冊", 100)],
        )]);

        let dict = Layered::new(vec![Box::new(sys_dict)], Box::new(user_dict));
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

    #[test]
    fn test_lookup() -> Result<(), Box<dyn Error>> {
        let sys_dict = TrieBuf::from([(
            vec![syl![Bopomofo::C, Bopomofo::E, Bopomofo::TONE4]],
            vec![("測", 1), ("冊", 1), ("側", 1)],
        )]);
        let user_dict = TrieBuf::from([(
            vec![syl![Bopomofo::C, Bopomofo::E, Bopomofo::TONE4]],
            vec![("策", 100), ("冊", 100)],
        )]);

        let dict = Layered::new(vec![Box::new(sys_dict)], Box::new(user_dict));
        assert_eq!(
            Some(("側", 1, 0).into()),
            dict.lookup(
                &vec![syl![Bopomofo::C, Bopomofo::E, Bopomofo::TONE4]],
                LookupStrategy::Standard
            )
            .first()
            .cloned(),
        );
        assert_eq!(
            [
                ("側", 1, 0).into(),
                ("冊", 101, 0).into(),
                ("測", 1, 0).into(),
                ("策", 100, 0).into(),
            ]
            .into_iter()
            .collect::<Vec<Phrase>>(),
            dict.lookup(
                &vec![syl![Bopomofo::C, Bopomofo::E, Bopomofo::TONE4]],
                LookupStrategy::Standard
            ),
        );
        Ok(())
    }

    #[test]
    fn test_readonly_user_dict() -> Result<(), Box<dyn Error>> {
        let sys_dict = TrieBuf::from([(
            vec![syl![Bopomofo::C, Bopomofo::E, Bopomofo::TONE4]],
            vec![("測", 1), ("冊", 1), ("側", 1)],
        )]);
        let mut builder = TrieBuilder::new();
        builder.insert(
            &[syl![Bopomofo::C, Bopomofo::E, Bopomofo::TONE4]],
            ("策", 100, 0).into(),
        )?;
        builder.insert(
            &[syl![Bopomofo::C, Bopomofo::E, Bopomofo::TONE4]],
            ("冊", 100, 0).into(),
        )?;
        let mut cursor = Cursor::new(vec![]);
        builder.write(&mut cursor)?;
        cursor.rewind()?;
        let user_dict = Trie::new(&mut cursor)?;

        let mut dict = Layered::new(vec![Box::new(sys_dict)], Box::new(user_dict));
        assert_eq!(
            Some(("側", 1, 0).into()),
            dict.lookup(
                &vec![syl![Bopomofo::C, Bopomofo::E, Bopomofo::TONE4]],
                LookupStrategy::Standard
            )
            .first()
            .cloned(),
        );
        assert_eq!(
            [
                ("側", 1, 0).into(),
                ("冊", 101, 0).into(),
                ("測", 1, 0).into(),
                ("策", 100, 0).into(),
            ]
            .into_iter()
            .collect::<Vec<Phrase>>(),
            dict.lookup(
                &vec![syl![Bopomofo::C, Bopomofo::E, Bopomofo::TONE4]],
                LookupStrategy::Standard
            ),
        );
        let _ = dict.about();
        assert!(dict.reopen().is_err());
        assert!(dict.flush().is_err());
        assert!(
            dict.add_phrase(
                &[syl![Bopomofo::C, Bopomofo::E, Bopomofo::TONE4]],
                ("冊", 100).into()
            )
            .is_err()
        );
        assert!(
            dict.update_phrase(
                &[syl![Bopomofo::C, Bopomofo::E, Bopomofo::TONE4]],
                ("冊", 100).into(),
                0,
                0,
            )
            .is_err()
        );
        assert!(
            dict.remove_phrase(&[syl![Bopomofo::C, Bopomofo::E, Bopomofo::TONE4]], "冊")
                .is_err()
        );
        Ok(())
    }
}
