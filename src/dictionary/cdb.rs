use std::{
    any::Any,
    fmt::{Debug, Display},
    io::{self, Write},
    mem,
    path::{Path, PathBuf},
    str,
};

use cdb2::{CDBKeyValueIter, CDBValueIter, CDBWriter, CDB};

use crate::zhuyin::{Syllable, SyllableSlice};

use super::{
    kv::{KVDictionary, KVStore},
    BuildDictionaryError, DictEntries, Dictionary, DictionaryBuilder, DictionaryInfo,
    DictionaryUpdateError, Phrase,
};

#[derive(Debug)]
pub struct CdbDictionary {
    path: PathBuf,
    inner: KVDictionary<CDB>,
    info: DictionaryInfo,
}

#[derive(Debug)]
pub struct CdbDictionaryError {
    source: io::Error,
}

impl Display for CdbDictionaryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "cdb error")
    }
}

impl std::error::Error for CdbDictionaryError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.source)
    }
}

type Error = CdbDictionaryError;

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
            source: io::Error::new(io::ErrorKind::Other, value),
        }
    }
}

impl From<io::Error> for CdbDictionaryError {
    fn from(value: io::Error) -> Self {
        CdbDictionaryError { source: value }
    }
}

pub(crate) struct OkCDBValueIter<'a>(CDBValueIter<'a>);
pub(crate) struct OkCDBKeyValueIter<'a>(CDBKeyValueIter<'a>);

impl Iterator for OkCDBValueIter<'_> {
    type Item = Vec<u8>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(it) = self.0.next() {
            it.ok()
        } else {
            None
        }
    }
}

impl Iterator for OkCDBKeyValueIter<'_> {
    type Item = (Vec<u8>, Vec<u8>);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(it) = self.0.next() {
            it.ok()
        } else {
            None
        }
    }
}

impl<'a> KVStore<'a> for CDB {
    type ValueIter = OkCDBValueIter<'a>;
    type KeyValueIter = OkCDBKeyValueIter<'a>;

    fn find(&'a self, key: &[u8]) -> Self::ValueIter {
        OkCDBValueIter(self.find(key))
    }

    fn iter(&'a self) -> Self::KeyValueIter {
        OkCDBKeyValueIter(self.iter())
    }
}

impl CdbDictionary {
    pub fn open<P: Into<PathBuf>>(path: P) -> Result<CdbDictionary, CdbDictionaryError> {
        let path = path.into();
        match path.try_exists() {
            Ok(exists) => {
                if !exists {
                    let mut maker = CDBWriter::create(&path)?;
                    write_info(&mut maker, &DictionaryInfo::default())?;
                    maker.finish()?;
                }
            }
            Err(_) => todo!(),
        }
        let base = CDB::open(&path)?;
        let info = read_info(&base)?;
        Ok(CdbDictionary {
            path,
            inner: KVDictionary::new(base),
            info,
        })
    }
}

impl Dictionary for CdbDictionary {
    fn lookup_first_n_phrases(&self, syllables: &dyn SyllableSlice, first: usize) -> Vec<Phrase> {
        self.inner.lookup_first_n_phrases(syllables, first)
    }

    fn entries(&self) -> DictEntries<'_> {
        self.inner.entries()
    }

    fn about(&self) -> DictionaryInfo {
        self.info.clone()
    }

    fn reopen(&mut self) -> Result<(), DictionaryUpdateError> {
        self.inner.set(CDB::open(&self.path).map_err(Error::from)?);
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
        let mut writer = CDBWriter::create(&self.path)?;
        write_info(&mut writer, &self.info)?;
        for entry in self.entries() {
            let mut data_buf = vec![];
            write_phrase(&mut data_buf, &entry.1)?;
            writer.add(&entry.0.get_bytes(), &data_buf)?;
        }
        drop(self.inner.take());
        writer.finish()?;
        self.reopen()
    }

    fn add_phrase(
        &mut self,
        syllables: &dyn SyllableSlice,
        phrase: Phrase,
    ) -> Result<(), DictionaryUpdateError> {
        self.inner.add_phrase(syllables, phrase)
    }

    fn update_phrase(
        &mut self,
        syllables: &dyn SyllableSlice,
        phrase: Phrase,
        user_freq: u32,
        time: u64,
    ) -> Result<(), DictionaryUpdateError> {
        self.inner.update_phrase(syllables, phrase, user_freq, time)
    }

    fn remove_phrase(
        &mut self,
        syllables: &dyn SyllableSlice,
        phrase_str: &str,
    ) -> Result<(), DictionaryUpdateError> {
        self.inner.remove_phrase(syllables, phrase_str)
    }
}

impl Drop for CdbDictionary {
    fn drop(&mut self) {
        let _ = self.flush();
    }
}

#[derive(Debug)]
pub struct CdbDictionaryBuilder {
    inner: KVDictionary<()>,
    info: DictionaryInfo,
}

impl CdbDictionaryBuilder {
    pub fn new() -> CdbDictionaryBuilder {
        CdbDictionaryBuilder {
            inner: KVDictionary::<()>::new_in_memory(),
            info: Default::default(),
        }
    }
}

impl Default for CdbDictionaryBuilder {
    fn default() -> Self {
        Self::new()
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
        self.info = info;
        Ok(())
    }

    fn insert(
        &mut self,
        syllables: &[Syllable],
        phrase: Phrase,
    ) -> Result<(), BuildDictionaryError> {
        self.inner.add_phrase(&syllables, phrase)?;
        Ok(())
    }

    fn build(&mut self, path: &Path) -> Result<(), BuildDictionaryError> {
        let mut maker = CDBWriter::create(path)?;
        write_info(&mut maker, &self.info)?;
        maker.finish()?;
        let cdb = CDB::open(path)?;
        let inner = mem::replace(&mut self.inner, KVDictionary::<()>::new_in_memory());
        let mut dict = CdbDictionary {
            path: path.to_path_buf(),
            inner: KVDictionary::from_raw_parts(cdb, inner),
            info: self.info.clone(),
        };
        dict.flush()?;
        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

fn bytes_from<'a>(opt: &'a Option<String>, s: &'a str) -> &'a [u8] {
    opt.as_ref().map(|s| s.as_str()).unwrap_or(s).as_bytes()
}

fn write_info(maker: &mut CDBWriter, info: &DictionaryInfo) -> Result<(), io::Error> {
    maker.add(b"INAM", bytes_from(&info.name, "我的詞庫"))?;
    maker.add(b"ICOP", bytes_from(&info.copyright, "Unknown"))?;
    maker.add(b"ILIC", bytes_from(&info.license, "Unknown"))?;
    maker.add(b"IREV", bytes_from(&info.version, "0.0.0"))?;
    maker.add(
        b"ISFT",
        bytes_from(
            &info.software,
            &format!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION")),
        ),
    )?;

    Ok(())
}

fn optstr_from(reader: &CDB, key: &str) -> Result<Option<String>, CdbDictionaryError> {
    Ok(reader
        .get(key.as_bytes())
        .transpose()?
        .map(|b| str::from_utf8(&b).unwrap_or("INVALID").to_string()))
}

fn read_info(reader: &CDB) -> Result<DictionaryInfo, CdbDictionaryError> {
    let name = optstr_from(reader, "INAM")?;
    let copyright = optstr_from(reader, "ICOP")?;
    let license = optstr_from(reader, "ILIC")?;
    let version = optstr_from(reader, "IREV")?;
    let software = optstr_from(reader, "ISFT")?;
    Ok(DictionaryInfo {
        name,
        copyright,
        license,
        version,
        software,
    })
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use crate::{dictionary::Phrase, syl, zhuyin::Bopomofo::*};

    use super::{CdbDictionary, Dictionary};

    #[test]
    fn create_new_dictionary_in_memory_and_query() -> Result<(), Box<dyn Error>> {
        let tmp_dir = tempfile::tempdir()?;
        let file_path = tmp_dir.path().join("chewing.cdb");
        let mut dict = CdbDictionary::open(file_path)?;
        let info = dict.about();
        dict.add_phrase(
            &[syl![Z, TONE4], syl![D, I, AN, TONE3]],
            ("dict", 1, 2).into(),
        )?;
        assert_eq!(Some("Unknown".to_string()), info.copyright);
        assert_eq!(
            Some(("dict", 1, 2).into()),
            dict.lookup_first_phrase(&[syl![Z, TONE4], syl![D, I, AN, TONE3]])
        );
        Ok(())
    }

    #[test]
    fn create_new_dictionary_and_query() -> Result<(), Box<dyn Error>> {
        let tmp_dir = tempfile::tempdir()?;
        let file_path = tmp_dir.path().join("chewing.cdb");
        let mut dict = CdbDictionary::open(&file_path)?;
        dict.add_phrase(
            &[syl![Z, TONE4], syl![D, I, AN, TONE3]],
            ("dict", 1, 2).into(),
        )?;
        dict.flush()?;
        let dict = CdbDictionary::open(file_path)?;
        let info = dict.about();
        assert_eq!(Some("Unknown".to_string()), info.copyright);
        assert_eq!(
            Some(("dict", 1, 2).into()),
            dict.lookup_first_phrase(&[syl![Z, TONE4], syl![D, I, AN, TONE3]])
        );
        Ok(())
    }

    #[test]
    fn create_new_dictionary_and_enumerate() -> Result<(), Box<dyn Error>> {
        let tmp_dir = tempfile::tempdir()?;
        let file_path = tmp_dir.path().join("chewing.cdb");
        let mut dict = CdbDictionary::open(file_path)?;
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
