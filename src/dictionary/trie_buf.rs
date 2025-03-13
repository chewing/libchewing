use std::{
    borrow::Cow,
    cmp,
    collections::{BTreeMap, BTreeSet, btree_map::Entry},
    io,
    path::{Path, PathBuf},
    thread::{self, JoinHandle},
};

use log::{error, info};

use crate::zhuyin::{Syllable, SyllableSlice};

use super::{
    BuildDictionaryError, Dictionary, DictionaryBuilder, DictionaryInfo, DictionaryMut, Entries,
    LookupStrategy, Phrase, Trie, TrieBuilder, UpdateDictionaryError,
};

/// A mutable dictionary backed by a Trie and a BTreeMap.
#[derive(Debug)]
pub struct TrieBuf {
    trie: Option<Trie>,
    btree: BTreeMap<PhraseKey, (u32, u64)>,
    graveyard: BTreeSet<PhraseKey>,
    join_handle: Option<JoinHandle<Result<Trie, UpdateDictionaryError>>>,
    dirty: bool,
}

type PhraseKey = (Cow<'static, [Syllable]>, Cow<'static, str>);

const MIN_PHRASE: &str = "";
const MAX_PHRASE: &str = "\u{10FFFF}";

fn software_version() -> String {
    format!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"))
}

impl TrieBuf {
    /// Open the target Trie dictionary and wrap it create a TrieBuf.
    pub fn open<P: Into<PathBuf>>(path: P) -> io::Result<TrieBuf> {
        let path = path.into();
        if !path.exists() {
            let info = DictionaryInfo {
                name: "我的詞庫".to_string(),
                copyright: "Unknown".to_string(),
                license: "Unknown".to_string(),
                version: "0.0.0".to_string(),
                software: software_version(),
            };
            let mut builder = TrieBuilder::new();
            builder
                .set_info(info)
                .map_err(|_| io::Error::from(io::ErrorKind::Other))?;
            builder
                .build(&path)
                .map_err(|_| io::Error::from(io::ErrorKind::Other))?;
        }
        let trie = Trie::open(&path)?;
        Ok(TrieBuf {
            trie: Some(trie),
            btree: BTreeMap::new(),
            graveyard: BTreeSet::new(),
            join_handle: None,
            dirty: false,
        })
    }

    /// Creates a pure in memory dictionary.
    pub fn new_in_memory() -> TrieBuf {
        TrieBuf {
            trie: None,
            btree: BTreeMap::new(),
            graveyard: BTreeSet::new(),
            join_handle: None,
            dirty: false,
        }
    }

    pub(crate) fn entries_iter_for<'a>(
        &'a self,
        syllables: &'a dyn SyllableSlice,
        strategy: LookupStrategy,
    ) -> impl Iterator<Item = Phrase> + 'a {
        let syllable_key = Cow::from(syllables.to_slice().into_owned());
        let min_key = (syllable_key.clone(), Cow::from(MIN_PHRASE));
        let max_key = (syllable_key.clone(), Cow::from(MAX_PHRASE));
        let store_iter = self
            .trie
            .iter()
            .flat_map(move |trie| trie.lookup_all_phrases(syllables, strategy));
        let btree_iter = self
            .btree
            .range(min_key..max_key)
            .map(|(key, value)| Phrase {
                phrase: key.1.clone().into(),
                freq: value.0,
                last_used: Some(value.1),
            });

        store_iter.chain(btree_iter).filter(move |it| {
            !self
                .graveyard
                .contains(&(syllable_key.clone(), Cow::from(it.as_str())))
        })
    }

    pub(crate) fn entries_iter(&self) -> impl Iterator<Item = (Vec<Syllable>, Phrase)> + '_ {
        let trie_iter = self.trie.iter().flat_map(|trie| trie.entries()).peekable();
        let btree_iter = self
            .btree
            .iter()
            .map(|(key, value)| {
                (
                    key.0.clone().into_owned(),
                    Phrase {
                        phrase: key.1.clone().into(),
                        freq: value.0,
                        last_used: Some(value.1),
                    },
                )
            })
            .peekable();
        trie_iter.chain(btree_iter).filter(|it| {
            !self
                .graveyard
                .contains(&(Cow::from(it.0.as_slice()), Cow::from(it.1.as_str())))
        })
    }

    pub(crate) fn lookup_first_n_phrases(
        &self,
        syllables: &dyn SyllableSlice,
        first: usize,
        strategy: LookupStrategy,
    ) -> Vec<Phrase> {
        let mut sort_map = BTreeMap::new();
        let mut phrases: Vec<Phrase> = Vec::new();

        for phrase in self.entries_iter_for(syllables, strategy) {
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

    pub(crate) fn entries(&self) -> Entries<'_> {
        Box::new(self.entries_iter())
    }

    pub(crate) fn add_phrase(
        &mut self,
        syllables: &dyn SyllableSlice,
        phrase: Phrase,
    ) -> Result<(), UpdateDictionaryError> {
        let syllable_slice = syllables.to_slice();
        if self
            .entries_iter_for(&syllable_slice.as_ref(), LookupStrategy::Standard)
            .any(|ph| ph.as_str() == phrase.as_str())
        {
            return Err(UpdateDictionaryError { source: None });
        }

        self.btree.insert(
            (
                Cow::from(syllable_slice.into_owned()),
                Cow::from(phrase.phrase.into_string()),
            ),
            (phrase.freq, phrase.last_used.unwrap_or_default()),
        );
        self.dirty = true;

        Ok(())
    }

    pub(crate) fn update_phrase(
        &mut self,
        syllables: &dyn SyllableSlice,
        phrase: Phrase,
        user_freq: u32,
        time: u64,
    ) -> Result<(), UpdateDictionaryError> {
        self.btree.insert(
            (
                Cow::from(syllables.to_slice().into_owned()),
                Cow::from(phrase.phrase.into_string()),
            ),
            (user_freq, time),
        );
        self.dirty = true;

        Ok(())
    }

    pub(crate) fn remove_phrase(
        &mut self,
        syllables: &dyn SyllableSlice,
        phrase_str: &str,
    ) -> Result<(), UpdateDictionaryError> {
        let syllable_slice = Cow::from(syllables.to_slice().into_owned());
        self.btree
            .remove(&(syllable_slice.clone(), Cow::from(phrase_str.to_owned())));
        self.graveyard
            .insert((syllable_slice, phrase_str.to_owned().into()));
        self.dirty = true;

        Ok(())
    }

    pub(crate) fn sync(&mut self) -> Result<(), UpdateDictionaryError> {
        info!("Synchronize dictionary from disk...");
        if let Some(join_handle) = self.join_handle.take() {
            if !join_handle.is_finished() {
                info!("Aborted. Wait until previous sync is finished.");
                self.join_handle = Some(join_handle);
                return Ok(());
            }
            match join_handle.join() {
                Ok(Ok(trie)) => {
                    if self.dirty {
                        info!("Aborted. The in memory dictionary is already dirty.");
                        return Ok(());
                    }
                    self.trie = Some(trie);
                    self.btree.clear();
                    self.graveyard.clear();
                }
                Ok(Err(e)) => {
                    error!("Failed to flush dictionary due to error: {e}");
                }
                Err(_) => {
                    error!("Failed to join thread.");
                }
            }
        } else {
            // TODO: reduce reading
            if self.path().is_some() {
                info!("Reloading...");
                self.trie = Some(Trie::open(self.path().unwrap())?);
            }
        }
        Ok(())
    }

    pub(crate) fn checkpoint(&mut self) {
        info!("Check pointing...");
        if self.join_handle.is_some() {
            info!("Aborted. Wait until previous checkpoint result is handled.");
            return;
        }
        if self.trie.is_none() || self.trie.as_ref().unwrap().path().is_none() || !self.dirty {
            info!("Aborted. Don't need to checkpoint in memory or clean dictionary.");
            return;
        }
        let snapshot = TrieBuf {
            trie: self.trie.clone(),
            btree: self.btree.clone(),
            graveyard: self.graveyard.clone(),
            join_handle: None,
            dirty: false,
        };
        self.join_handle = Some(thread::spawn(move || {
            let mut builder = TrieBuilder::new();
            info!("Saving snapshot...");
            builder.set_info(DictionaryInfo {
                software: software_version(),
                ..snapshot.about()
            })?;
            for (syllables, phrase) in snapshot.entries() {
                builder.insert(&syllables, phrase)?;
            }
            info!("Flushing snapshot...");
            builder.build(snapshot.path().unwrap())?;
            let trie = Trie::open(snapshot.path().unwrap()).map_err(|err| UpdateDictionaryError {
                source: Some(Box::new(err)),
            });
            info!("    Done");
            trie
        }));
        self.dirty = false;
    }
}

impl From<BuildDictionaryError> for UpdateDictionaryError {
    fn from(value: BuildDictionaryError) -> Self {
        UpdateDictionaryError {
            source: Some(Box::new(value)),
        }
    }
}

impl Dictionary for TrieBuf {
    fn lookup_first_n_phrases(
        &self,
        syllables: &dyn SyllableSlice,
        first: usize,
        strategy: LookupStrategy,
    ) -> Vec<Phrase> {
        TrieBuf::lookup_first_n_phrases(self, syllables, first, strategy)
    }

    fn entries(&self) -> Entries<'_> {
        TrieBuf::entries(self)
    }

    fn about(&self) -> DictionaryInfo {
        self.trie
            .as_ref()
            .map_or(DictionaryInfo::default(), |trie| trie.about())
    }

    fn path(&self) -> Option<&Path> {
        self.trie.as_ref()?.path()
    }

    fn as_dict_mut(&mut self) -> Option<&mut dyn DictionaryMut> {
        Some(self)
    }
}

impl DictionaryMut for TrieBuf {
    fn reopen(&mut self) -> Result<(), UpdateDictionaryError> {
        self.sync()?;
        Ok(())
    }

    fn flush(&mut self) -> Result<(), UpdateDictionaryError> {
        self.checkpoint();
        Ok(())
    }

    fn add_phrase(
        &mut self,
        syllables: &dyn SyllableSlice,
        phrase: Phrase,
    ) -> Result<(), UpdateDictionaryError> {
        TrieBuf::add_phrase(self, syllables, phrase)
    }

    fn update_phrase(
        &mut self,
        syllables: &dyn SyllableSlice,
        phrase: Phrase,
        user_freq: u32,
        time: u64,
    ) -> Result<(), UpdateDictionaryError> {
        TrieBuf::update_phrase(self, syllables, phrase, user_freq, time)
    }

    fn remove_phrase(
        &mut self,
        syllables: &dyn SyllableSlice,
        phrase_str: &str,
    ) -> Result<(), UpdateDictionaryError> {
        TrieBuf::remove_phrase(self, syllables, phrase_str)
    }
}

impl<P: Into<Phrase>, const N: usize> From<[(Vec<Syllable>, Vec<P>); N]> for TrieBuf {
    fn from(value: [(Vec<Syllable>, Vec<P>); N]) -> Self {
        let mut dict = TrieBuf::new_in_memory();
        for (syllables, phrases) in value {
            for phrase in phrases {
                dict.add_phrase(&syllables, phrase.into()).unwrap();
            }
        }
        dict
    }
}

impl Drop for TrieBuf {
    fn drop(&mut self) {
        let _ = self.sync();
        let _ = self.flush();
        if let Some(join_handle) = self.join_handle.take() {
            let _ = join_handle.join();
        }
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use crate::{
        dictionary::{DictionaryMut, LookupStrategy, Phrase},
        syl,
        zhuyin::Bopomofo::*,
    };

    use super::{Dictionary, TrieBuf};

    #[test]
    fn create_new_dictionary_in_memory_and_query() -> Result<(), Box<dyn Error>> {
        let tmp_dir = tempfile::tempdir()?;
        let file_path = tmp_dir.path().join("user.dat");
        let mut dict = TrieBuf::open(file_path)?;
        let info = dict.about();
        dict.add_phrase(
            &[syl![Z, TONE4], syl![D, I, AN, TONE3]],
            ("dict", 1, 2).into(),
        )?;
        assert_eq!("Unknown", info.copyright);
        assert_eq!(
            Some(("dict", 1, 2).into()),
            dict.lookup_first_phrase(
                &[syl![Z, TONE4], syl![D, I, AN, TONE3]],
                LookupStrategy::Standard
            )
        );
        Ok(())
    }

    #[test]
    fn create_new_dictionary_and_query() -> Result<(), Box<dyn Error>> {
        let tmp_dir = tempfile::tempdir()?;
        let file_path = tmp_dir.path().join("user.dat");
        // Force dict to drop to sync async write
        {
            let mut dict = TrieBuf::open(&file_path)?;
            dict.add_phrase(
                &[syl![Z, TONE4], syl![D, I, AN, TONE3]],
                ("dict", 1, 2).into(),
            )?;
            dict.flush()?;
        }
        let dict = TrieBuf::open(file_path)?;
        let info = dict.about();
        assert_eq!("Unknown", info.copyright);
        assert_eq!(
            Some(("dict", 1, 2).into()),
            dict.lookup_first_phrase(
                &[syl![Z, TONE4], syl![D, I, AN, TONE3]],
                LookupStrategy::Standard
            )
        );
        Ok(())
    }

    #[test]
    fn create_new_dictionary_and_enumerate() -> Result<(), Box<dyn Error>> {
        let tmp_dir = tempfile::tempdir()?;
        let file_path = tmp_dir.path().join("user.dat");
        let mut dict = TrieBuf::open(file_path)?;
        dict.add_phrase(
            &[syl![Z, TONE4], syl![D, I, AN, TONE3]],
            ("dict", 1, 2).into(),
        )?;
        dict.flush()?;
        assert_eq!(
            vec![(
                vec![syl![Z, TONE4], syl![D, I, AN, TONE3]],
                Phrase::from(("dict", 1, 2))
            )],
            dict.entries().collect::<Vec<_>>()
        );
        Ok(())
    }
}
