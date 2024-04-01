use std::{
    error::Error,
    ffi::OsStr,
    fs::{self, File},
    io::{self, Seek},
    path::{Path, PathBuf},
};

use crate::{
    editor::{abbrev::AbbrevTable, SymbolSelector},
    path::{find_path_by_files, sys_path_from_env_var, userphrase_path},
};

#[cfg(feature = "sqlite")]
use super::SqliteDictionary;
use super::{uhash, Dictionary, TrieBufDictionary, TrieDictionary};

const SD_WORD_FILE_NAME: &str = "word.dat";
const SD_TSI_FILE_NAME: &str = "tsi.dat";
const UD_UHASH_FILE_NAME: &str = "uhash.dat";
// const UD_TRIE_FILE_NAME: &str = "chewing.dat";
const UD_SQLITE_FILE_NAME: &str = "chewing.sqlite3";
const UD_MEM_FILE_NAME: &str = ":memory:";
const ABBREV_FILE_NAME: &str = "swkb.dat";
const SYMBOLS_FILE_NAME: &str = "symbols.dat";

#[derive(Debug, Default)]
pub struct SystemDictionaryLoader {
    sys_path: Option<String>,
}

fn load_err(_: impl Error) -> &'static str {
    "LoadSystemDictionaryError"
}

impl SystemDictionaryLoader {
    pub fn new() -> SystemDictionaryLoader {
        SystemDictionaryLoader::default()
    }
    pub fn sys_path(mut self, path: impl Into<String>) -> SystemDictionaryLoader {
        self.sys_path = Some(path.into());
        self
    }
    pub fn load(&self) -> Result<Vec<Box<dyn Dictionary>>, &'static str> {
        let search_path = if let Some(sys_path) = &self.sys_path {
            sys_path.to_owned()
        } else {
            sys_path_from_env_var()
        };
        let sys_path = find_path_by_files(&search_path, &[SD_WORD_FILE_NAME, SD_TSI_FILE_NAME])
            .ok_or("SystemDictionaryNotFound")?;

        let tsi_dict_path = sys_path.join(SD_TSI_FILE_NAME);
        let tsi_dict = TrieDictionary::open(tsi_dict_path).map_err(load_err)?;
        let word_dict_path = sys_path.join(SD_WORD_FILE_NAME);
        let word_dict = TrieDictionary::open(word_dict_path).map_err(load_err)?;
        Ok(vec![Box::new(word_dict), Box::new(tsi_dict)])
    }
    pub fn load_abbrev(&self) -> Result<AbbrevTable, &'static str> {
        let search_path = if let Some(sys_path) = &self.sys_path {
            sys_path.to_owned()
        } else {
            sys_path_from_env_var()
        };
        let sys_path = find_path_by_files(&search_path, &[ABBREV_FILE_NAME])
            .ok_or("SystemDictionaryNotFound")?;
        AbbrevTable::open(sys_path.join(ABBREV_FILE_NAME)).map_err(|_| "error loading abbrev table")
    }
    pub fn load_symbol_selector(&self) -> Result<SymbolSelector, &'static str> {
        let search_path = if let Some(sys_path) = &self.sys_path {
            sys_path.to_owned()
        } else {
            sys_path_from_env_var()
        };
        let sys_path = find_path_by_files(&search_path, &[SYMBOLS_FILE_NAME])
            .ok_or("SystemDictionaryNotFound")?;
        SymbolSelector::open(sys_path.join(SYMBOLS_FILE_NAME))
            .map_err(|_| "error loading abbrev table")
    }
}

#[derive(Debug, Default)]
pub struct UserDictionaryLoader {
    data_path: Option<PathBuf>,
}

impl UserDictionaryLoader {
    pub fn new() -> UserDictionaryLoader {
        UserDictionaryLoader::default()
    }
    pub fn userphrase_path(mut self, path: impl AsRef<Path>) -> UserDictionaryLoader {
        self.data_path = Some(path.as_ref().to_path_buf());
        self
    }
    pub fn load(self) -> io::Result<Box<dyn Dictionary>> {
        let data_path = self
            .data_path
            .or_else(userphrase_path)
            .ok_or(io::Error::from(io::ErrorKind::NotFound))?;
        if data_path.ends_with(UD_MEM_FILE_NAME) {
            return Ok(Box::new(TrieBufDictionary::new_in_memory()));
        }
        if data_path.exists() {
            return guess_format_and_load(&data_path);
        }
        let userdata_dir = data_path.parent().expect("path should contain a filename");
        if !userdata_dir.exists() {
            fs::create_dir_all(userdata_dir)?;
        }
        let mut fresh_dict = init_user_dictionary(&data_path)?;

        let user_dict_path = userdata_dir.join(UD_SQLITE_FILE_NAME);
        if cfg!(feature = "sqlite") && user_dict_path.exists() {
            #[cfg(feature = "sqlite")]
            {
                let trie_dict = SqliteDictionary::open(user_dict_path)
                    .map_err(|e| io::Error::new(io::ErrorKind::Other, Box::new(e)))?;
                for (syllables, phrase) in trie_dict.entries() {
                    let freq = phrase.freq();
                    let last_used = phrase.last_used().unwrap_or_default();
                    fresh_dict
                        .update_phrase(&syllables, phrase, freq, last_used)
                        .map_err(|e| io::Error::new(io::ErrorKind::Other, Box::new(e)))?;
                }
                fresh_dict
                    .flush()
                    .map_err(|e| io::Error::new(io::ErrorKind::Other, Box::new(e)))?;
            }
        } else {
            let uhash_path = userdata_dir.join(UD_UHASH_FILE_NAME);
            if uhash_path.exists() {
                let mut input = File::open(uhash_path)?;
                if let Ok(phrases) = uhash::try_load_bin(&input).or_else(|_| {
                    input.rewind()?;
                    uhash::try_load_text(&input)
                }) {
                    for (syllables, phrase) in phrases {
                        let freq = phrase.freq();
                        let last_used = phrase.last_used().unwrap_or_default();
                        fresh_dict
                            .update_phrase(&syllables, phrase, freq, last_used)
                            .map_err(|e| io::Error::new(io::ErrorKind::Other, Box::new(e)))?;
                    }
                    fresh_dict
                        .flush()
                        .map_err(|e| io::Error::new(io::ErrorKind::Other, Box::new(e)))?;
                }
            }
        }

        Ok(fresh_dict)
    }
}

fn guess_format_and_load(dict_path: &PathBuf) -> io::Result<Box<dyn Dictionary>> {
    let metadata = dict_path.metadata()?;
    if metadata.permissions().readonly() {
        return Err(io::Error::from(io::ErrorKind::PermissionDenied));
    }

    init_user_dictionary(dict_path)
}

fn init_user_dictionary(dict_path: &PathBuf) -> io::Result<Box<dyn Dictionary>> {
    let ext = dict_path.extension().unwrap_or(OsStr::new("unknown"));
    if ext.eq_ignore_ascii_case("sqlite3") {
        #[cfg(feature = "sqlite")]
        {
            SqliteDictionary::open(dict_path)
                .map(|dict| Box::new(dict) as Box<dyn Dictionary>)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, Box::new(e)))
        }
        #[cfg(not(feature = "sqlite"))]
        {
            Err(io::Error::from(io::ErrorKind::Unsupported))
        }
    } else if ext.eq_ignore_ascii_case("dat") {
        TrieBufDictionary::open(dict_path)
            .map(|dict| Box::new(dict) as Box<dyn Dictionary>)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, Box::new(e)))
    } else {
        Err(io::Error::from(io::ErrorKind::Other))
    }
}
