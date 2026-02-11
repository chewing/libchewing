use std::{
    error::Error,
    ffi::OsStr,
    fmt::Display,
    fs::{self, File},
    io::{self, Seek},
    path::{Path, PathBuf},
};

use log::{error, info};

#[cfg(feature = "sqlite")]
use super::SqliteDictionary;
use super::{Dictionary, TrieBuf, uhash};
use crate::exn::{Exn, ResultExt};
use crate::{
    dictionary::DictionaryUsage,
    editor::{AbbrevTable, SymbolSelector},
    path::{find_files_by_names, find_path_by_files, search_path_from_env_var, userphrase_path},
};

const UD_UHASH_FILE_NAME: &str = "uhash.dat";
// const UD_TRIE_FILE_NAME: &str = "chewing.dat";
const UD_SQLITE_FILE_NAME: &str = "chewing.sqlite3";
const UD_MEM_FILE_NAME: &str = ":memory:";
const ABBREV_FILE_NAME: &str = "swkb.dat";
const SYMBOLS_FILE_NAME: &str = "symbols.dat";

pub const DEFAULT_DICT_NAMES: &[&str] = &["word.dat", "tsi.dat", "chewing.dat"];

/// Automatically searchs and loads dictionaries.
#[derive(Debug, Default)]
pub struct AssetLoader {
    search_path: Option<String>,
}

impl AssetLoader {
    /// Creates a new dictionary loader.
    pub fn new() -> AssetLoader {
        AssetLoader::default()
    }
    /// Override the default dictionary search path.
    pub fn search_path(mut self, search_path: impl Into<String>) -> AssetLoader {
        self.search_path = Some(search_path.into());
        self
    }
    /// Searches and loads the specified dictionaries.
    ///
    /// Search path can be changed using [`search_path`][SystemDictionaryLoader::search_path].
    ///
    /// Any dictionary that is not in the search paths or cannot be load is skipped.
    pub fn load<T>(&self, names: &[T]) -> Vec<Box<dyn Dictionary>>
    where
        T: AsRef<str>,
    {
        let search_path = if let Some(path) = &self.search_path {
            path.to_owned()
        } else {
            search_path_from_env_var()
        };
        let loader = SingleDictionaryLoader::new();
        let files = find_files_by_names(&search_path, names);
        let mut results = vec![];
        'next: for target_name in names {
            for file in files.iter() {
                if let Some(file_name) = file.file_name()
                    && target_name.as_ref() == file_name.to_string_lossy()
                    && let Ok(mut dict) = loader.guess_format_and_load(file)
                {
                    match target_name.as_ref() {
                        "tsi.dat" | "word.dat" => {
                            dict.set_usage(DictionaryUsage::BuiltIn);
                        }
                        "chewing.dat" => {
                            dict.set_usage(DictionaryUsage::User);
                        }
                        "chewing-deleted.dat" => {
                            dict.set_usage(DictionaryUsage::ExcludeList);
                        }
                        _ => {
                            dict.set_usage(DictionaryUsage::Unknown);
                        }
                    }
                    results.push(dict);
                    continue 'next;
                }
            }
            error!("Dictionary file not found: {}", target_name.as_ref());
            continue;
        }
        results
    }
    /// Loads the abbrev table.
    pub fn load_abbrev(&self) -> Result<AbbrevTable, LoadDictionaryError> {
        let error = || LoadDictionaryError::new("failed to load abbrev table");
        let not_found = || error().with_source(io::Error::from(io::ErrorKind::NotFound));
        let search_path = if let Some(path) = &self.search_path {
            path.to_owned()
        } else {
            search_path_from_env_var()
        };
        let parent_path =
            find_path_by_files(&search_path, &[ABBREV_FILE_NAME]).or_raise(not_found)?;
        let abbrev_path = parent_path.join(ABBREV_FILE_NAME);
        info!("Loading {ABBREV_FILE_NAME}");
        AbbrevTable::open(abbrev_path).or_raise(error)
    }
    /// Loads the symbol table.
    pub fn load_symbol_selector(&self) -> Result<SymbolSelector, LoadDictionaryError> {
        let error = || LoadDictionaryError::new("failed to load symbol table");
        let not_found = || error().with_source(io::Error::from(io::ErrorKind::NotFound));
        let search_path = if let Some(path) = &self.search_path {
            path.to_owned()
        } else {
            search_path_from_env_var()
        };
        let parent_path =
            find_path_by_files(&search_path, &[SYMBOLS_FILE_NAME]).or_raise(not_found)?;
        let symbol_path = parent_path.join(SYMBOLS_FILE_NAME);
        info!("Loading {SYMBOLS_FILE_NAME}");
        SymbolSelector::open(symbol_path).or_raise(error)
    }
}

/// Automatically searches and initializes the user dictionary.
#[derive(Debug, Default)]
pub struct UserDictionaryManager {
    data_path: Option<PathBuf>,
}

impl UserDictionaryManager {
    /// Creates a user dictionary manager.
    pub fn new() -> UserDictionaryManager {
        UserDictionaryManager::default()
    }
    /// Override the default user dictionary path.
    pub fn userphrase_path(mut self, path: impl AsRef<Path>) -> UserDictionaryManager {
        self.data_path = Some(path.as_ref().to_path_buf());
        self
    }
    /// Return the resolved file name of the user dictionary file.
    pub fn file_name(&self) -> Option<String> {
        self.data_path
            .clone()
            .or_else(userphrase_path)
            .and_then(|p| {
                if p.is_file() {
                    p.file_name().map(|p| p.to_string_lossy().into_owned())
                } else {
                    None
                }
            })
    }
    /// Searches and initializes the user dictionary.
    ///
    /// If no user dictionary were found, a new dictionary will be created at
    /// the default path.
    pub fn init(&self) -> Result<Box<dyn Dictionary>, LoadDictionaryError> {
        let error = || LoadDictionaryError::new("failed to init user dictionary");
        let not_found = || error().with_source(io::Error::from(io::ErrorKind::NotFound));
        let mut loader = SingleDictionaryLoader::new();
        loader.migrate_sqlite(true);
        let data_path = self
            .data_path
            .clone()
            .or_else(userphrase_path)
            .or_raise(not_found)?;
        if data_path.ends_with(UD_MEM_FILE_NAME) {
            return Ok(Self::in_memory());
        }
        if data_path.exists() {
            info!("Use existing user dictionary {}", data_path.display());
            return loader
                .guess_format_and_load(&data_path)
                .map(|mut dict| {
                    dict.set_usage(DictionaryUsage::User);
                    dict
                })
                .or_raise(error);
        }
        let userdata_dir = data_path.parent().expect("path should contain a filename");
        if !userdata_dir.exists() {
            info!("Creating userdata_dir: {}", userdata_dir.display());
            fs::create_dir_all(userdata_dir).or_raise(error)?;
        }
        info!(
            "Creating a fresh user dictionary at {}",
            data_path.display()
        );
        let mut fresh_dict = loader.guess_format_and_load(&data_path).or_raise(error)?;

        let user_dict_path = userdata_dir.join(UD_SQLITE_FILE_NAME);
        if cfg!(feature = "sqlite") && user_dict_path.exists() {
            #[cfg(feature = "sqlite")]
            {
                info!(
                    "Importing existing sqlite dictionary at {}",
                    user_dict_path.display()
                );
                let dict = SqliteDictionary::open(user_dict_path).or_raise(error)?;
                for (syllables, phrase) in dict.entries() {
                    let freq = phrase.freq();
                    let last_used = phrase.last_used().unwrap_or_default();
                    fresh_dict
                        .update_phrase(&syllables, phrase, freq, last_used)
                        .or_raise(error)?;
                }
                fresh_dict.flush().or_raise(error)?;
            }
        } else {
            let uhash_path = userdata_dir.join(UD_UHASH_FILE_NAME);
            if uhash_path.exists() {
                info!(
                    "Importing existing uhash dictionary at {}",
                    user_dict_path.display()
                );
                let mut input = File::open(uhash_path).or_raise(error)?;
                if let Ok(phrases) = uhash::try_load_bin(&input).or_else(|_| {
                    input.rewind()?;
                    uhash::try_load_text(&input)
                }) {
                    for (syllables, phrase) in phrases {
                        let freq = phrase.freq();
                        let last_used = phrase.last_used().unwrap_or_default();
                        fresh_dict
                            .update_phrase(&syllables, phrase, freq, last_used)
                            .or_raise(error)?;
                    }
                    fresh_dict.flush().or_raise(error)?;
                }
            }
        }

        fresh_dict.set_usage(DictionaryUsage::User);
        Ok(fresh_dict)
    }
    /// Searches and initializes the user exclusion dictionary.
    ///
    /// If no user exclusion dictionary were found, a new dictionary
    /// will be created at the default path.
    pub fn init_deleted(&self) -> Result<Box<dyn Dictionary>, LoadDictionaryError> {
        let error = || LoadDictionaryError::new("failed to init user exclusion dictionary");
        let not_found = || error().with_source(io::Error::from(io::ErrorKind::NotFound));
        let loader = SingleDictionaryLoader::new();
        let data_path = self
            .data_path
            .clone()
            .or_else(userphrase_path)
            .or_raise(not_found)?;
        let userdata_dir = data_path.parent().expect("path should contain a filename");
        if !userdata_dir.exists() {
            info!("Creating userdata_dir: {}", userdata_dir.display());
            fs::create_dir_all(&userdata_dir).or_raise(error)?;
        }
        let exclude_dict_path = userdata_dir.join("chewing-deleted.dat");
        loader
            .guess_format_and_load(&exclude_dict_path)
            .or_raise(error)
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
    pub fn guess_format_and_load(
        &self,
        dict_path: &PathBuf,
    ) -> Result<Box<dyn Dictionary>, LoadDictionaryError> {
        let error = || LoadDictionaryError::new("failed to parse and load dictionary");
        info!("Loading dictionary {}", dict_path.display());
        if self.migrate_sqlite && dict_path.is_file() {
            let metadata = dict_path.metadata().or_raise(error)?;
            if metadata.permissions().readonly() {
                return Err(error().with_source(io::Error::from(io::ErrorKind::PermissionDenied)));
            }
        }

        let ext = dict_path.extension().unwrap_or(OsStr::new("unknown"));
        if ext.eq_ignore_ascii_case("sqlite3") {
            #[cfg(feature = "sqlite")]
            {
                if self.migrate_sqlite {
                    SqliteDictionary::open(dict_path)
                        .map(|dict| Box::new(dict) as Box<dyn Dictionary>)
                        .or_raise(error)
                } else {
                    SqliteDictionary::open_readonly(dict_path)
                        .map(|dict| Box::new(dict) as Box<dyn Dictionary>)
                        .or_raise(error)
                }
            }
            #[cfg(not(feature = "sqlite"))]
            {
                Err(error().with_source(io::Error::from(io::ErrorKind::Unsupported)))
            }
        } else if ext.eq_ignore_ascii_case("dat") {
            TrieBuf::open(dict_path)
                .map(|dict| Box::new(dict) as Box<dyn Dictionary>)
                .or_raise(error)
        } else {
            Err(error())
        }
    }
}

/// Errors during loading system or user dictionaries.
#[derive(Debug)]
pub struct LoadDictionaryError {
    msg: String,
    source: Option<Box<dyn Error + Send + Sync + 'static>>,
}

impl LoadDictionaryError {
    fn new(msg: &str) -> LoadDictionaryError {
        LoadDictionaryError {
            msg: msg.to_string(),
            source: None,
        }
    }
}

impl Display for LoadDictionaryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.msg)
    }
}

impl_exn!(LoadDictionaryError);
