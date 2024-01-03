use std::{
    borrow::Cow,
    collections::{HashMap, HashSet},
    fmt::Debug,
    fs::File,
    io::{self, Write},
    mem,
    path::{Path, PathBuf},
};

use cdb::{CDBMake, CDBWriter, CDB};
use thiserror::Error;

use crate::zhuyin::{Syllable, SyllableSlice};

use super::{
    BuildDictionaryError, DictEntries, Dictionary, DictionaryBuilder, DictionaryInfo,
    DictionaryUpdateError, Phrase,
};

mod serde {
    use std::str;

    use bytemuck;

    use super::Phrase;

    pub(crate) struct PhraseData<T>(T);

    impl<'a> PhraseData<&'a [u8]> {
        pub(crate) fn frequency(&self) -> u32 {
            bytemuck::pod_read_unaligned(&self.0[..4])
        }
        pub(crate) fn last_used(&self) -> u64 {
            bytemuck::pod_read_unaligned(&self.0[4..12])
        }
        pub(crate) fn phrase_str(&self) -> &'a str {
            let len = self.0[12] as usize;
            let data = &self.0[13..];
            str::from_utf8(&data[..len]).expect("should be utf8 encoded string")
        }
        pub(crate) fn len(&self) -> usize {
            13 + self.0[12] as usize
        }
    }

    pub(crate) struct PhrasesIter<'a> {
        bytes: &'a [u8],
    }

    impl<'a> PhrasesIter<'a> {
        pub(crate) fn new(bytes: &'a [u8]) -> PhrasesIter<'a> {
            PhrasesIter { bytes }
        }

        pub(crate) fn empty() -> PhrasesIter<'static> {
            PhrasesIter { bytes: &[] }
        }
    }

    impl Iterator for PhrasesIter<'_> {
        type Item = Phrase;

        #[inline(always)]
        fn next(&mut self) -> Option<Self::Item> {
            if self.bytes.is_empty() {
                return None;
            }
            let phrase_data = PhraseData(self.bytes);
            self.bytes = &self.bytes[phrase_data.len()..];
            Some(
                Phrase::new(phrase_data.phrase_str(), phrase_data.frequency())
                    .with_time(phrase_data.last_used()),
            )
        }
    }
}

use serde::PhrasesIter;

pub struct CdbDictionary {
    path: PathBuf,
    base: CDB,
    added: HashMap<Vec<u8>, Vec<Phrase>>,
    updated: HashMap<PhraseKey, (u32, u64)>,
    graveyard: HashSet<PhraseKey>,
}

type PhraseKey = (Cow<'static, [u8]>, Cow<'static, str>);

#[derive(Debug, Error)]
#[error("cdb error")]
pub struct CdbDictionaryError {
    #[from]
    source: io::Error,
}

type Error = CdbDictionaryError;

impl Debug for CdbDictionary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CdbDictionary")
            .field("base", &"CDB { /* opaque */ }")
            .field("added", &self.added)
            .field("updated", &self.updated)
            .field("graveyard", &self.graveyard)
            .finish()
    }
}

impl From<CdbDictionaryError> for DictionaryUpdateError {
    fn from(value: CdbDictionaryError) -> Self {
        DictionaryUpdateError {
            source: Some(value.into()),
        }
    }
}

impl From<BuildDictionaryError> for CdbDictionaryError {
    fn from(value: BuildDictionaryError) -> Self {
        CdbDictionaryError {
            source: io::Error::other(value),
        }
    }
}

impl CdbDictionary {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<CdbDictionary, CdbDictionaryError> {
        match path.as_ref().try_exists() {
            Ok(exists) => {
                if !exists {
                    let mut builder = CdbDictionaryBuilder::new();
                    builder
                        .set_info(DictionaryInfo::default())
                        .map_err(Error::from)?;
                    builder.build(path.as_ref())?;
                }
            }
            Err(_) => todo!(),
        }
        let base = CDB::open(&path)?;
        let path = path.as_ref().to_path_buf();
        Ok(CdbDictionary {
            path,
            base,
            added: Default::default(),
            updated: Default::default(),
            graveyard: Default::default(),
        })
    }
}

impl Dictionary for CdbDictionary {
    fn lookup_first_n_phrases(&self, syllables: &dyn SyllableSlice, first: usize) -> Vec<Phrase> {
        let syllable_bytes = syllables.get_bytes();
        let base_bytes = self.base.get(&syllable_bytes);
        let base_phrases = match &base_bytes {
            Some(record) => PhrasesIter::new(record.as_deref().unwrap_or(&[])),
            None => PhrasesIter::empty(),
        };
        let added_phrases = match self.added.get(&syllable_bytes) {
            Some(phrases) => phrases.clone().into_iter(),
            None => vec![].into_iter(),
        };
        base_phrases
            .chain(added_phrases)
            .filter(|it| {
                let phrase_key = (syllable_bytes.as_slice().into(), it.as_str().into());
                !self.graveyard.contains(&phrase_key)
            })
            .map(|it| {
                let phrase_key = (syllable_bytes.as_slice().into(), it.as_str().into());
                match self.updated.get(&phrase_key) {
                    Some(value) => Phrase::new(it.as_str(), value.0).with_time(value.1),
                    None => it,
                }
            })
            .take(first)
            .collect()
    }

    fn entries(&self) -> Option<DictEntries> {
        None
    }

    fn about(&self) -> DictionaryInfo {
        todo!()
    }

    fn reopen(&mut self) -> Result<(), DictionaryUpdateError> {
        self.base = CDB::open(&self.path).map_err(Error::from)?;
        Ok(())
    }

    fn flush(&mut self) -> Result<(), DictionaryUpdateError> {
        #[inline(always)]
        fn write_phrase(data_buf: &mut Vec<u8>, phrase: &Phrase) -> Result<(), io::Error> {
            data_buf.write_all(&phrase.freq().to_le_bytes())?;
            data_buf.write_all(&phrase.last_used().unwrap_or_default().to_le_bytes())?;
            data_buf.write_all(&[phrase.as_str().len() as u8])?;
            data_buf.write_all(phrase.as_str().as_bytes())
        }
        // FIXME fix in CDB crate to use only PathBuf
        let mut writer =
            CDBWriter::create(dbg!(&self.path.display().to_string())).map_err(Error::from)?;
        // FIXME reuse entries()
        // FIXME fix CDB to provide key iter
        for entry in self.base.iter() {
            // FIXME skip info entry
            let (key, value) = entry.map_err(Error::from)?;
            let syllable_bytes = key;
            let base_bytes = value;
            let base_phrases = PhrasesIter::new(&base_bytes);
            let added_phrases = match self.added.get(&syllable_bytes) {
                Some(phrases) => phrases.clone().into_iter(),
                None => vec![].into_iter(),
            };
            let mut data_buf = vec![];
            for phrase in base_phrases
                .chain(added_phrases)
                .filter(|it| {
                    let phrase_key = (syllable_bytes.as_slice().into(), it.as_str().into());
                    !self.graveyard.contains(&phrase_key)
                })
                .map(|it| {
                    let phrase_key = (syllable_bytes.as_slice().into(), it.as_str().into());
                    match self.updated.get(&phrase_key) {
                        Some(value) => Phrase::new(it.as_str(), value.0).with_time(value.1),
                        None => it,
                    }
                })
            {
                write_phrase(&mut data_buf, &phrase).map_err(Error::from)?;
            }
            self.added.remove(&syllable_bytes);
            writer
                .add(&syllable_bytes, &data_buf)
                .map_err(Error::from)?;
        }
        for (syllable_bytes, phrases) in &self.added {
            let mut data_buf = vec![];
            for phrase in phrases {
                write_phrase(&mut data_buf, &phrase).map_err(Error::from)?;
            }
            writer
                .add(&syllable_bytes, &data_buf)
                .map_err(Error::from)?;
        }
        writer.finish().map_err(Error::from)?;
        self.added.clear();
        self.updated.clear();
        self.graveyard.clear();
        dbg!(self.reopen())
    }

    fn add_phrase(
        &mut self,
        syllables: &dyn SyllableSlice,
        phrase: Phrase,
    ) -> Result<(), DictionaryUpdateError> {
        let syllable_bytes = syllables.get_bytes();
        let phrase_key = (syllable_bytes.into(), phrase.to_string().into());
        if self.updated.contains_key(&phrase_key) {
            return Err(DictionaryUpdateError { source: None });
        }
        self.graveyard.remove(&phrase_key);
        self.added
            .entry(phrase_key.0.into_owned())
            .or_default()
            .push(phrase);
        Ok(())
    }

    fn update_phrase(
        &mut self,
        syllables: &dyn SyllableSlice,
        phrase: Phrase,
        user_freq: u32,
        time: u64,
    ) -> Result<(), DictionaryUpdateError> {
        let syllable_bytes = syllables.get_bytes();
        let phrase_key = (syllable_bytes.into(), String::from(phrase).into());
        self.graveyard.remove(&phrase_key);
        self.updated.insert(phrase_key, (user_freq, time));
        Ok(())
    }

    fn remove_phrase(
        &mut self,
        syllables: &dyn SyllableSlice,
        phrase_str: &str,
    ) -> Result<(), DictionaryUpdateError> {
        let syllable_bytes = syllables.get_bytes();
        let phrase_key = (syllable_bytes.into(), phrase_str.to_owned().into());
        self.graveyard.insert(phrase_key);
        Ok(())
    }
}

#[derive(Debug)]
pub struct CdbDictionaryBuilder {
    added: HashMap<Vec<u8>, Vec<Phrase>>,
    info: DictionaryInfo,
}

impl CdbDictionaryBuilder {
    pub fn new() -> CdbDictionaryBuilder {
        CdbDictionaryBuilder {
            added: Default::default(),
            info: Default::default(),
        }
    }
}

impl From<CdbDictionaryError> for BuildDictionaryError {
    fn from(value: CdbDictionaryError) -> Self {
        BuildDictionaryError {
            source: Box::new(value),
        }
    }
}

impl From<DictionaryUpdateError> for BuildDictionaryError {
    fn from(value: DictionaryUpdateError) -> Self {
        BuildDictionaryError {
            source: Box::new(value),
        }
    }
}

impl DictionaryBuilder for CdbDictionaryBuilder {
    fn set_info(&mut self, info: DictionaryInfo) -> Result<(), BuildDictionaryError> {
        // TODO
        Ok(())
    }

    fn insert(
        &mut self,
        syllables: &[Syllable],
        phrase: Phrase,
    ) -> Result<(), BuildDictionaryError> {
        self.added
            .entry(syllables.get_bytes())
            .or_default()
            .push(phrase);
        Ok(())
    }

    fn build(&mut self, path: &Path) -> Result<(), BuildDictionaryError> {
        let mut maker = CDBMake::new(File::create(path)?)?;
        // FIXME cannot create empty db. Insert info?
        maker.add(b"INFO", &[])?;
        maker.finish()?;
        let mut dict = CdbDictionary::open(path)?;
        mem::swap(&mut dict.added, &mut self.added);
        dict.flush()?;
        Ok(())
    }
}
