use std::{
    cmp,
    collections::{BTreeMap, btree_map::Entry},
    iter,
};

use log::error;

use crate::zhuyin::SyllableSlice;

use super::{
    Dictionary, DictionaryInfo, DictionaryMut, Entries, LookupStrategy, Phrase,
    UpdateDictionaryError,
};

/// A collection of dictionaries that returns the union of the lookup results.
/// # Examples
///
/// ```
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
///
/// use chewing::{dictionary::{Layered, TrieBuf, Dictionary, LookupStrategy, Phrase}, syl, zhuyin::Bopomofo};
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
///         ("冊", 100, 0).into(),
///         ("測", 1, 0).into(),
///         ("策", 100, 0).into(),
///     ]
///     .into_iter()
///     .collect::<Vec<Phrase>>(),
///     dict.lookup_all_phrases(&[
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
    fn lookup_first_n_phrases(
        &self,
        syllables: &dyn SyllableSlice,
        first: usize,
        strategy: LookupStrategy,
    ) -> Vec<Phrase> {
        let mut sort_map: BTreeMap<String, usize> = BTreeMap::new();
        let mut phrases: Vec<Phrase> = Vec::new();

        self.sys_dict
            .iter()
            .chain(iter::once(&self.user_dict))
            .for_each(|d| {
                for phrase in d.lookup_all_phrases(syllables, strategy) {
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
            name: "Built-in Layered".to_string(),
            ..Default::default()
        }
    }

    fn path(&self) -> Option<&std::path::Path> {
        None
    }

    fn as_dict_mut(&mut self) -> Option<&mut dyn DictionaryMut> {
        self.user_dict.as_dict_mut()
    }
}

impl DictionaryMut for Layered {
    fn reopen(&mut self) -> Result<(), UpdateDictionaryError> {
        if let Some(writer) = self.user_dict.as_dict_mut() {
            writer.reopen()
        } else {
            Ok(())
        }
    }

    fn flush(&mut self) -> Result<(), UpdateDictionaryError> {
        if let Some(writer) = self.user_dict.as_dict_mut() {
            writer.flush()
        } else {
            Ok(())
        }
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
        if let Some(writer) = self.user_dict.as_dict_mut() {
            writer.add_phrase(syllables, phrase)
        } else {
            Ok(())
        }
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
        if let Some(writer) = self.user_dict.as_dict_mut() {
            writer.update_phrase(syllables, phrase, user_freq, time)
        } else {
            Ok(())
        }
    }

    fn remove_phrase(
        &mut self,
        syllables: &dyn SyllableSlice,
        phrase_str: &str,
    ) -> Result<(), UpdateDictionaryError> {
        if let Some(writer) = self.user_dict.as_dict_mut() {
            writer.remove_phrase(syllables, phrase_str)
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{
        error::Error,
        io::{Cursor, Seek},
    };

    use crate::{
        dictionary::{
            Dictionary, DictionaryBuilder, DictionaryMut, LookupStrategy, Phrase, Trie, TrieBuf,
            TrieBuilder,
        },
        syl,
        zhuyin::Bopomofo,
    };

    use super::Layered;

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
            dict.lookup_first_phrase(
                &vec![syl![Bopomofo::C, Bopomofo::E, Bopomofo::TONE4]],
                LookupStrategy::Standard
            ),
        );
        assert_eq!(
            [
                ("側", 1, 0).into(),
                ("冊", 100, 0).into(),
                ("測", 1, 0).into(),
                ("策", 100, 0).into(),
            ]
            .into_iter()
            .collect::<Vec<Phrase>>(),
            dict.lookup_all_phrases(
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
            dict.lookup_first_phrase(
                &vec![syl![Bopomofo::C, Bopomofo::E, Bopomofo::TONE4]],
                LookupStrategy::Standard
            ),
        );
        assert_eq!(
            [
                ("側", 1, 0).into(),
                ("冊", 100, 0).into(),
                ("測", 1, 0).into(),
                ("策", 100, 0).into(),
            ]
            .into_iter()
            .collect::<Vec<Phrase>>(),
            dict.lookup_all_phrases(
                &vec![syl![Bopomofo::C, Bopomofo::E, Bopomofo::TONE4]],
                LookupStrategy::Standard
            ),
        );
        let _ = dict.about();
        assert!(dict.as_dict_mut().is_none());
        assert!(dict.reopen().is_ok());
        assert!(dict.flush().is_ok());
        assert!(
            dict.add_phrase(
                &[syl![Bopomofo::C, Bopomofo::E, Bopomofo::TONE4]],
                ("冊", 100).into()
            )
            .is_ok()
        );
        assert!(
            dict.update_phrase(
                &[syl![Bopomofo::C, Bopomofo::E, Bopomofo::TONE4]],
                ("冊", 100).into(),
                0,
                0,
            )
            .is_ok()
        );
        assert!(
            dict.remove_phrase(&[syl![Bopomofo::C, Bopomofo::E, Bopomofo::TONE4]], "冊")
                .is_ok()
        );
        Ok(())
    }
}
