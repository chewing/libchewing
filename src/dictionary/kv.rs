use std::{
    borrow::Cow,
    cmp,
    collections::{btree_map::Entry, BTreeMap, BTreeSet},
    fmt::Debug,
    io,
    iter::{self, Empty},
    path::Path,
    str,
};

use crate::zhuyin::{Syllable, SyllableSlice};

use super::{DictEntries, DictionaryUpdateError, Phrase};

pub(crate) trait KVStore<'a> {
    type ValueIter: Iterator<Item = Vec<u8>>;
    type KeyValueIter: Iterator<Item = (Vec<u8>, Vec<u8>)>;

    fn find(&'a self, key: &[u8]) -> Self::ValueIter;
    fn iter(&'a self) -> Self::KeyValueIter;
}

pub(crate) trait KVStoreBuilder {
    fn create<P: AsRef<Path>>(filename: P) -> io::Result<Self>
    where
        Self: Sized;
    fn add(&mut self, key: &[u8], data: &[u8]) -> io::Result<()>;
    fn finish(self) -> io::Result<()>;
}

type PhraseKey = (Cow<'static, [u8]>, Cow<'static, str>);

pub(crate) struct KVDictionary<T> {
    store: T,
    btree: BTreeMap<PhraseKey, (u32, u64)>,
    graveyard: BTreeSet<PhraseKey>,
}

impl<T> Debug for KVDictionary<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("KVDictionary")
            .field("store", &"/* private fields */")
            .field("btree", &self.btree)
            .finish()
    }
}

const MIN_PHRASE: &str = "";
const MAX_PHRASE: &str = "\u{10FFFF}";

fn phrase_from_bytes(bytes: &[u8]) -> Vec<Syllable> {
    bytes
        .chunks_exact(2)
        .map(|bytes| {
            let mut u16_bytes = [0; 2];
            u16_bytes.copy_from_slice(bytes);
            let syl_u16 = u16::from_le_bytes(u16_bytes);
            Syllable::try_from(syl_u16).unwrap()
        })
        .collect::<Vec<_>>()
}

impl<T> KVDictionary<T>
where
    T: for<'a> KVStore<'a>,
{
    pub(crate) fn new(store: T) -> KVDictionary<T> {
        KVDictionary {
            store,
            btree: BTreeMap::new(),
            graveyard: BTreeSet::new(),
        }
    }

    pub(crate) fn new_in_memory() -> KVDictionary<()> {
        KVDictionary {
            store: (),
            btree: BTreeMap::new(),
            graveyard: BTreeSet::new(),
        }
    }

    pub(crate) fn take<S>(self, store: S) -> KVDictionary<S>
    where
        S: for<'a> KVStore<'a>,
    {
        KVDictionary {
            store,
            btree: self.btree,
            graveyard: self.graveyard,
        }
    }

    pub(crate) fn reopen(&mut self, store: T) {
        self.store = store;
    }

    pub(crate) fn entries_iter_for<'a>(
        &'a self,
        syllable_bytes: &'a [u8],
    ) -> impl Iterator<Item = Phrase> + 'a {
        let syllable_key = Cow::from(syllable_bytes);
        let min_key = (syllable_key.clone(), Cow::from(MIN_PHRASE));
        let max_key = (syllable_key.clone(), Cow::from(MAX_PHRASE));

        self.store
            .find(&syllable_bytes)
            .map(|bytes| Phrase::from(PhraseData(&bytes)))
            .chain(
                self.btree
                    .range(min_key..max_key)
                    .map(|(key, value)| Phrase {
                        phrase: key.1.as_ref().to_owned(),
                        freq: value.0,
                        last_used: Some(value.1),
                    }),
            )
            .filter(move |it| {
                !self
                    .graveyard
                    .contains(&(syllable_key.clone(), Cow::from(it.as_str())))
            })
    }

    pub(crate) fn entries_iter(&self) -> impl Iterator<Item = (Vec<u8>, Phrase)> + '_ {
        let mut store_iter = self
            .store
            .iter()
            .filter(|it| it.0 != b"INFO")
            .map(|(syllable_bytes, phrase_bytes)| {
                (syllable_bytes, Phrase::from(PhraseData(&phrase_bytes)))
            })
            .peekable();
        let mut btree_iter = self
            .btree
            .iter()
            .map(|(key, value)| {
                (
                    key.0.clone().into_owned(),
                    Phrase {
                        phrase: key.1.as_ref().to_owned(),
                        freq: value.0,
                        last_used: Some(value.1),
                    },
                )
            })
            .peekable();
        iter::from_fn(move || {
            let a = store_iter.peek();
            let b = btree_iter.peek();
            match (a, b) {
                (None, Some(_)) => btree_iter.next(),
                (Some(_), None) => store_iter.next(),
                (Some(x), Some(y)) => match (&x.0, x.1.as_str()).cmp(&(&y.0, y.1.as_str())) {
                    cmp::Ordering::Less => store_iter.next(),
                    cmp::Ordering::Equal => match x.1.freq.cmp(&y.1.freq) {
                        cmp::Ordering::Less | cmp::Ordering::Equal => {
                            let _ = store_iter.next();
                            btree_iter.next()
                        }
                        cmp::Ordering::Greater => {
                            let _ = btree_iter.next();
                            store_iter.next()
                        }
                    },
                    cmp::Ordering::Greater => btree_iter.next(),
                },
                (None, None) => None,
            }
        })
        .filter(|it| {
            !self
                .graveyard
                .contains(&(Cow::from(it.0.as_slice()), Cow::from(it.1.as_str())))
        })
    }

    pub(crate) fn lookup_first_n_phrases(
        &self,
        syllables: &dyn SyllableSlice,
        first: usize,
    ) -> Vec<Phrase> {
        let syllable_bytes = syllables.get_bytes();
        let mut sort_map = BTreeMap::new();
        let mut phrases: Vec<Phrase> = Vec::new();

        for phrase in self.entries_iter_for(&syllable_bytes) {
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
        phrases.truncate(first);
        phrases
    }

    pub(crate) fn entries(&self) -> Option<DictEntries> {
        Some(Box::new(
            self.entries_iter()
                .map(|it| (phrase_from_bytes(&it.0), it.1))
                .collect::<Vec<_>>()
                .into_iter(),
        ))
    }

    pub(crate) fn add_phrase(
        &mut self,
        syllables: &dyn SyllableSlice,
        phrase: Phrase,
    ) -> Result<(), DictionaryUpdateError> {
        let syllable_bytes = syllables.get_bytes();
        if self
            .entries_iter_for(&syllable_bytes)
            .any(|ph| ph.as_str() == phrase.as_str())
        {
            return Err(DictionaryUpdateError { source: None });
        }

        self.btree.insert(
            (Cow::from(syllable_bytes), Cow::from(phrase.phrase)),
            (phrase.freq, phrase.last_used.unwrap_or_default()),
        );

        Ok(())
    }

    pub(crate) fn update_phrase(
        &mut self,
        syllables: &dyn SyllableSlice,
        phrase: Phrase,
        user_freq: u32,
        time: u64,
    ) -> Result<(), DictionaryUpdateError> {
        let syllable_bytes = syllables.get_bytes();
        self.btree.insert(
            (Cow::from(syllable_bytes), Cow::from(phrase.phrase)),
            (user_freq, time),
        );

        Ok(())
    }

    pub(crate) fn remove_phrase(
        &mut self,
        syllables: &dyn SyllableSlice,
        phrase_str: &str,
    ) -> Result<(), DictionaryUpdateError> {
        let syllable_bytes = syllables.get_bytes();
        self.btree.remove(&(
            Cow::from(syllable_bytes.clone()),
            Cow::from(phrase_str.to_owned()),
        ));
        self.graveyard
            .insert((syllable_bytes.into(), phrase_str.to_owned().into()));
        Ok(())
    }
}

impl KVStore<'_> for () {
    type ValueIter = Empty<Vec<u8>>;
    type KeyValueIter = Empty<(Vec<u8>, Vec<u8>)>;

    fn find(&self, _key: &[u8]) -> Self::ValueIter {
        iter::empty()
    }

    fn iter(&self) -> Self::KeyValueIter {
        iter::empty()
    }
}

#[derive(Debug, Clone, Copy)]
struct PhraseData<'a>(&'a [u8]);

impl<'a> PhraseData<'a> {
    fn frequency(&self) -> u32 {
        bytemuck::pod_read_unaligned(&self.0[..4])
    }
    fn last_used(&self) -> u64 {
        bytemuck::pod_read_unaligned(&self.0[4..12])
    }
    fn phrase_str(&self) -> &'a str {
        let len = self.0[12] as usize;
        let data = &self.0[13..];
        str::from_utf8(&data[..len]).expect("should be utf8 encoded string")
    }
    fn _len(&self) -> usize {
        13 + self.0[12] as usize
    }
}

impl From<PhraseData<'_>> for Phrase {
    fn from(value: PhraseData<'_>) -> Self {
        Phrase {
            phrase: value.phrase_str().to_owned(),
            freq: value.frequency(),
            last_used: Some(value.last_used()),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use crate::{dictionary::Phrase, syl, zhuyin::Bopomofo::*};

    use super::KVDictionary;

    #[test]
    fn create_new_dictionary_in_memory_and_query() -> Result<(), Box<dyn Error>> {
        let mut dict = KVDictionary::<()>::new_in_memory();
        dict.add_phrase(
            &[syl![Z, TONE4], syl![D, I, AN, TONE3]],
            ("dict", 1, 2).into(),
        )?;
        assert_eq!(
            vec![Phrase::from(("dict", 1, 2))],
            dict.lookup_first_n_phrases(&[syl![Z, TONE4], syl![D, I, AN, TONE3]], 1)
        );
        Ok(())
    }

    #[test]
    fn create_new_dictionary_in_memory_all_entries() -> Result<(), Box<dyn Error>> {
        let mut dict = KVDictionary::<()>::new_in_memory();
        dict.add_phrase(
            &[syl![Z, TONE4], syl![D, I, AN, TONE3]],
            ("dict", 1, 2).into(),
        )?;
        dict.add_phrase(
            &[syl![Z, TONE4], syl![D, I, AN, TONE3]],
            ("dict2", 1, 2).into(),
        )?;
        dict.add_phrase(
            &[syl![Z, TONE4], syl![D, I, AN, TONE3]],
            ("dict3", 1, 2).into(),
        )?;
        assert_eq!(
            vec![
                Phrase::from(("dict", 1, 2)),
                Phrase::from(("dict2", 1, 2)),
                Phrase::from(("dict3", 1, 2))
            ],
            dict.entries_iter().map(|it| it.1).collect::<Vec<_>>()
        );
        Ok(())
    }

    #[test]
    fn create_new_dictionary_in_memory_add_remove_entries() -> Result<(), Box<dyn Error>> {
        let mut dict = KVDictionary::<()>::new_in_memory();
        dict.add_phrase(
            &[syl![Z, TONE4], syl![D, I, AN, TONE3]],
            ("dict", 1, 2).into(),
        )?;
        dict.add_phrase(
            &[syl![Z, TONE4], syl![D, I, AN, TONE3]],
            ("dict2", 1, 2).into(),
        )?;
        dict.add_phrase(
            &[syl![Z, TONE4], syl![D, I, AN, TONE3]],
            ("dict3", 1, 2).into(),
        )?;
        dict.remove_phrase(&[syl![Z, TONE4], syl![D, I, AN, TONE3]], "dict3")?;
        assert_eq!(
            vec![Phrase::from(("dict", 1, 2)), Phrase::from(("dict2", 1, 2)),],
            dict.entries_iter().map(|it| it.1).collect::<Vec<_>>()
        );
        Ok(())
    }

    #[test]
    fn create_new_dictionary_empty_and_query() -> Result<(), Box<dyn Error>> {
        let mut dict = KVDictionary::new(());
        dict.add_phrase(
            &[syl![Z, TONE4], syl![D, I, AN, TONE3]],
            ("dict", 1, 2).into(),
        )?;
        assert_eq!(
            vec![Phrase::from(("dict", 1, 2))],
            dict.lookup_first_n_phrases(&[syl![Z, TONE4], syl![D, I, AN, TONE3]], 1)
        );
        Ok(())
    }
}
