use std::{
    error::Error,
    ffi::OsStr,
    fmt::Display,
    fs::{self, File},
    io::{self, Seek},
    path::{Path, PathBuf},
};

use log::{info, warn};

use crate::{
    editor::{AbbrevTable, SymbolSelector},
    path::{find_extra_dat_by_path, find_path_by_files, sys_path_from_env_var, userphrase_path},
};

#[cfg(feature = "sqlite")]
use super::SqliteDictionary;
use super::{Dictionary, Trie, TrieBuf, uhash};

const SD_WORD_FILE_NAME: &str = "word.dat";
const SD_TSI_FILE_NAME: &str = "tsi.dat";
const UD_UHASH_FILE_NAME: &str = "uhash.dat";
// const UD_TRIE_FILE_NAME: &str = "chewing.dat";
const UD_SQLITE_FILE_NAME: &str = "chewing.sqlite3";
const UD_MEM_FILE_NAME: &str = ":memory:";
const ABBREV_FILE_NAME: &str = "swkb.dat";
const SYMBOLS_FILE_NAME: &str = "symbols.dat";

/// Automatically searchs and loads system dictionaries.
#[derive(Debug, Default)]
pub struct SystemDictionaryLoader {
    sys_path: Option<String>,
}

/// Errors during loading system or user dictionaries.
#[derive(Debug)]
pub enum LoadDictionaryError {
    /// Cannot find any system or user dictionary.
    NotFound,
    /// IO Error.
    IoError(io::Error),
}

impl Display for LoadDictionaryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Unable to load system dictionary: {:?}", self)
    }
}

impl Error for LoadDictionaryError {}

fn io_err(err: io::Error) -> LoadDictionaryError {
    LoadDictionaryError::IoError(err)
}

impl SystemDictionaryLoader {
    /// Creates a new system dictionary loader.
    pub fn new() -> SystemDictionaryLoader {
        SystemDictionaryLoader::default()
    }
    /// Override the default system dictionary search path.
    pub fn sys_path(mut self, path: impl Into<String>) -> SystemDictionaryLoader {
        self.sys_path = Some(path.into());
        self
    }
    /// Searches and loads the system dictionaries and extra dictionaries.
    ///
    /// If no dictionary were found, a builtn minimum dictionary will be loaded.
    pub fn load(&self) -> Result<Vec<Box<dyn Dictionary>>, LoadDictionaryError> {
        let search_path = if let Some(sys_path) = &self.sys_path {
            sys_path.to_owned()
        } else {
            sys_path_from_env_var()
        };
        let sys_path = find_path_by_files(&search_path, &[SD_WORD_FILE_NAME, SD_TSI_FILE_NAME])
            .ok_or(LoadDictionaryError::NotFound)?;

        let mut results: Vec<Box<dyn Dictionary>> = vec![];

        let word_dict_path = sys_path.join(SD_WORD_FILE_NAME);
        info!("Loading {SD_WORD_FILE_NAME}");
        let word_dict = Trie::open(word_dict_path).map_err(io_err)?;
        results.push(Box::new(word_dict));

        let tsi_dict_path = sys_path.join(SD_TSI_FILE_NAME);
        info!("Loading {SD_TSI_FILE_NAME}");
        let tsi_dict = Trie::open(tsi_dict_path).map_err(io_err)?;
        results.push(Box::new(tsi_dict));

        let extra_files = find_extra_dat_by_path(&search_path);
        for path in extra_files {
            info!("Loading {}", path.display());
            match Trie::open(&path) {
                Ok(dict) => results.push(Box::new(dict)),
                Err(e) => warn!("Failed to load {}: {e}", path.display()),
            }
        }

        Ok(results)
    }
    /// Loads the abbrev table.
    pub fn load_abbrev(&self) -> Result<AbbrevTable, LoadDictionaryError> {
        let search_path = if let Some(sys_path) = &self.sys_path {
            sys_path.to_owned()
        } else {
            sys_path_from_env_var()
        };
        let sys_path = find_path_by_files(&search_path, &[ABBREV_FILE_NAME])
            .ok_or(LoadDictionaryError::NotFound)?;
        let abbrev_path = sys_path.join(ABBREV_FILE_NAME);
        info!("Loading {ABBREV_FILE_NAME}");
        AbbrevTable::open(abbrev_path).map_err(io_err)
    }
    /// Loads the symbol table.
    pub fn load_symbol_selector(&self) -> Result<SymbolSelector, LoadDictionaryError> {
        let search_path = if let Some(sys_path) = &self.sys_path {
            sys_path.to_owned()
        } else {
            sys_path_from_env_var()
        };
        let sys_path = find_path_by_files(&search_path, &[SYMBOLS_FILE_NAME])
            .ok_or(LoadDictionaryError::NotFound)?;
        let symbol_path = sys_path.join(SYMBOLS_FILE_NAME);
        info!("Loading {SYMBOLS_FILE_NAME}");
        SymbolSelector::open(symbol_path).map_err(io_err)
    }
}

/// Automatically searches and loads the user dictionary.
#[derive(Debug, Default)]
pub struct UserDictionaryLoader {
    data_path: Option<PathBuf>,
}

impl UserDictionaryLoader {
    /// Creates a user dictionary loader.
    pub fn new() -> UserDictionaryLoader {
        UserDictionaryLoader::default()
    }
    /// Override the default user dictionary search path.
    pub fn userphrase_path(mut self, path: impl AsRef<Path>) -> UserDictionaryLoader {
        self.data_path = Some(path.as_ref().to_path_buf());
        self
    }
    /// Searches and loads the user dictionary.
    ///
    /// If no user dictionary were found, a new dictionary will be created at
    /// the default path.
    pub fn load(self) -> io::Result<Box<dyn Dictionary>> {
        let data_path = self
            .data_path
            .or_else(userphrase_path)
            .ok_or(io::Error::from(io::ErrorKind::NotFound))?;
        if data_path.ends_with(UD_MEM_FILE_NAME) {
            info!("Use in memory trie dictionary");
            return Ok(Box::new(TrieBuf::new_in_memory()));
        }
        if data_path.exists() {
            info!("Loading {}", data_path.display());
            return guess_format_and_load(&data_path);
        }
        let userdata_dir = data_path.parent().expect("path should contain a filename");
        if !userdata_dir.exists() {
            info!("Creating userdata_dir: {}", userdata_dir.display());
            fs::create_dir_all(userdata_dir)?;
        }
        info!("Loading {}", data_path.display());
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
                        .as_dict_mut()
                        .unwrap()
                        .update_phrase(&syllables, phrase, freq, last_used)
                        .map_err(|e| io::Error::new(io::ErrorKind::Other, Box::new(e)))?;
                }
                fresh_dict
                    .as_dict_mut()
                    .unwrap()
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
                            .as_dict_mut()
                            .unwrap()
                            .update_phrase(&syllables, phrase, freq, last_used)
                            .map_err(|e| io::Error::new(io::ErrorKind::Other, Box::new(e)))?;
                    }
                    fresh_dict
                        .as_dict_mut()
                        .unwrap()
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
        TrieBuf::open(dict_path)
            .map(|dict| Box::new(dict) as Box<dyn Dictionary>)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, Box::new(e)))
    } else {
        Err(io::Error::from(io::ErrorKind::Other))
    }
}
