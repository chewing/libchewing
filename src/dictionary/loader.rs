use std::{
    error::Error,
    ffi::OsStr,
    fmt::Display,
    fs::{self, File},
    io::{self, Seek},
    path::{Path, PathBuf},
};

use log::{error, info};

use crate::{
    editor::{AbbrevTable, SymbolSelector},
    path::{find_files_by_names, find_path_by_files, sys_path_from_env_var, userphrase_path},
};

#[cfg(feature = "sqlite")]
use super::SqliteDictionary;
use super::{Dictionary, TrieBuf, uhash};

const UD_UHASH_FILE_NAME: &str = "uhash.dat";
// const UD_TRIE_FILE_NAME: &str = "chewing.dat";
const UD_SQLITE_FILE_NAME: &str = "chewing.sqlite3";
const UD_MEM_FILE_NAME: &str = ":memory:";
const ABBREV_FILE_NAME: &str = "swkb.dat";
const SYMBOLS_FILE_NAME: &str = "symbols.dat";

pub const DEFAULT_DICT_NAMES: &[&str] = &["word.dat", "tsi.dat"];

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
        write!(f, "Unable to load system dictionary: {self:?}")
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
    pub fn sys_path(mut self, search_path: impl Into<String>) -> SystemDictionaryLoader {
        self.sys_path = Some(search_path.into());
        self
    }
    /// Searches and loads the specified dictionaries.
    ///
    /// Search path can be changed using [`sys_path`][SystemDictionaryLoader::sys_path].
    pub fn load<T>(&self, names: &[T]) -> Result<Vec<Box<dyn Dictionary>>, LoadDictionaryError>
    where
        T: AsRef<str>,
    {
        let search_path = if let Some(sys_path) = &self.sys_path {
            sys_path.to_owned()
        } else {
            sys_path_from_env_var()
        };
        let loader = SingleDictionaryLoader::new();
        let files = find_files_by_names(&search_path, names);
        let mut results = vec![];
        'next: for target_name in names {
            for file in files.iter() {
                if let Some(file_name) = file.file_name()
                    && target_name.as_ref() == file_name.to_string_lossy()
                    && let Ok(dict) = loader.guess_format_and_load(file)
                {
                    info!("Load dictionary {}", file.display());
                    results.push(dict);
                    continue 'next;
                }
            }
            error!("Dictionary file not found: {}", target_name.as_ref());
            return Err(LoadDictionaryError::NotFound);
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
        let mut loader = SingleDictionaryLoader::new();
        loader.migrate_sqlite(true);
        let data_path = self
            .data_path
            .or_else(userphrase_path)
            .ok_or(io::Error::from(io::ErrorKind::NotFound))?;
        if data_path.ends_with(UD_MEM_FILE_NAME) {
            return Ok(Self::in_memory());
        }
        if data_path.exists() {
            info!("Loading {}", data_path.display());
            return loader.guess_format_and_load(&data_path);
        }
        let userdata_dir = data_path.parent().expect("path should contain a filename");
        if !userdata_dir.exists() {
            info!("Creating userdata_dir: {}", userdata_dir.display());
            fs::create_dir_all(userdata_dir)?;
        }
        info!("Loading {}", data_path.display());
        let mut fresh_dict = loader.guess_format_and_load(&data_path)?;

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
                            .map_err(|e| io::Error::other(Box::new(e)))?;
                    }
                    fresh_dict
                        .as_dict_mut()
                        .unwrap()
                        .flush()
                        .map_err(|e| io::Error::other(Box::new(e)))?;
                }
            }
        }

        Ok(fresh_dict)
    }
    /// Load a in-memory user dictionary.
    pub fn in_memory() -> Box<dyn Dictionary> {
        info!("Use in memory trie dictionary");
        Box::new(TrieBuf::new_in_memory())
    }
}

#[derive(Debug)]
pub struct SingleDictionaryLoader {
    migrate_sqlite: bool,
}

impl SingleDictionaryLoader {
    pub fn new() -> SingleDictionaryLoader {
        SingleDictionaryLoader {
            migrate_sqlite: false,
        }
    }
    pub fn migrate_sqlite(&mut self, migrate: bool) {
        self.migrate_sqlite = migrate;
    }
    pub fn guess_format_and_load(&self, dict_path: &PathBuf) -> io::Result<Box<dyn Dictionary>> {
        if self.migrate_sqlite && dict_path.is_file() {
            let metadata = dict_path.metadata()?;
            if metadata.permissions().readonly() {
                return Err(io::Error::from(io::ErrorKind::PermissionDenied));
            }
        }

        let ext = dict_path.extension().unwrap_or(OsStr::new("unknown"));
        if ext.eq_ignore_ascii_case("sqlite3") {
            #[cfg(feature = "sqlite")]
            {
                if self.migrate_sqlite {
                    SqliteDictionary::open(dict_path)
                        .map(|dict| Box::new(dict) as Box<dyn Dictionary>)
                        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, Box::new(e)))
                } else {
                    SqliteDictionary::open_readonly(dict_path)
                        .map(|dict| Box::new(dict) as Box<dyn Dictionary>)
                        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, Box::new(e)))
                }
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
}
