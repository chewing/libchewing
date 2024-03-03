use std::{
    ffi::OsStr,
    marker::PhantomData,
    path::{Path, PathBuf},
};

use crate::path::{find_path_by_files, sys_path_from_env_var, userphrase_path};

#[cfg(feature = "sqlite")]
use super::SqliteDictionary;
use super::{CdbDictionary, Dictionary, TrieDictionary};

const SD_WORD_FILE_NAME: &str = "word.dat";
const SD_TSI_FILE_NAME: &str = "tsi.dat";
const UD_UHASH_FILE_NAME: &str = "uhash.dat";
const UD_SQLITE_FILE_NAME: &str = "chewing.sqlite";
const UD_CDB_FILE_NAME: &str = "chewing.cdb";

#[derive(Debug)]
pub struct SystemDictionaryLoader {
    sys_path: Option<String>,
}

impl SystemDictionaryLoader {
    pub fn new() -> SystemDictionaryLoader {
        SystemDictionaryLoader { sys_path: None }
    }
    pub fn sys_path(mut self, path: impl Into<String>) -> SystemDictionaryLoader {
        self.sys_path = Some(path.into());
        self
    }
    pub fn load(self) -> Result<Vec<Box<dyn Dictionary>>, &'static str> {
        let mut db_loaders: Vec<Box<dyn DictionaryLoader>> = vec![];
        #[cfg(feature = "sqlite")]
        {
            db_loaders.push(LoaderWrapper::<SqliteDictionary>::new());
        }
        db_loaders.push(LoaderWrapper::<TrieDictionary>::new());

        let search_path = if let Some(sys_path) = self.sys_path {
            sys_path
        } else {
            sys_path_from_env_var()
        };
        let sys_path = find_path_by_files(&search_path, &[SD_WORD_FILE_NAME, SD_TSI_FILE_NAME])
            .ok_or("SystemDictionaryNotFound")?;

        let tsi_dict_path = sys_path.join(SD_TSI_FILE_NAME);
        let tsi_dict = db_loaders
            .iter()
            .find_map(|loader| loader.open_read_only(&tsi_dict_path));

        let word_dict_path = sys_path.join(SD_WORD_FILE_NAME);
        let word_dict = db_loaders
            .iter()
            .find_map(|loader| loader.open_read_only(&word_dict_path));

        Ok(vec![word_dict.unwrap(), tsi_dict.unwrap()])
    }
}

#[derive(Debug)]
pub struct UserDictionaryLoader {
    data_path: Option<PathBuf>,
}

impl UserDictionaryLoader {
    pub fn new() -> UserDictionaryLoader {
        UserDictionaryLoader { data_path: None }
    }
    pub fn userphrase_path(mut self, path: impl AsRef<Path>) -> UserDictionaryLoader {
        self.data_path = Some(path.as_ref().to_path_buf());
        self
    }
    pub fn load(self) -> Result<Box<dyn Dictionary>, &'static str> {
        let data_path = self
            .data_path
            .or_else(userphrase_path)
            .ok_or("UserDictionaryNotFound")?;
        if data_path.exists() {
            return guess_format_and_load(data_path);
        }
        init_user_dictionary(data_path)
    }
}

fn guess_format_and_load(dict_path: PathBuf) -> Result<Box<dyn Dictionary>, &'static str> {
    let metadata = dict_path
        .metadata()
        .map_err(|_| "ReadUserDictionaryError")?;
    if metadata.permissions().readonly() {
        return Err("ReadonlyUserDictionaryError");
    }

    init_user_dictionary(dict_path)
}

fn init_user_dictionary(dict_path: PathBuf) -> Result<Box<dyn Dictionary>, &'static str> {
    let ext = dict_path.extension().unwrap_or(OsStr::new("unknown"));
    if ext.eq_ignore_ascii_case("sqlite3") {
        #[cfg(feature = "sqlite")]
        {
            SqliteDictionary::open(dict_path)
                .map(|dict| Box::new(dict) as Box<dyn Dictionary>)
                .map_err(|_| "ReadUserDictionaryError")
        }
        #[cfg(not(feature = "sqlite"))]
        {
            Err("UnsupportedUserDictionaryFormat")
        }
    } else if ext.eq_ignore_ascii_case("cdb") {
        CdbDictionary::open(dict_path)
            .map(|dict| Box::new(dict) as Box<dyn Dictionary>)
            .map_err(|_| "ReadUserDictionaryError")
    } else {
        Err("UnknownUserDictionaryError")
    }
}

trait DictionaryLoader {
    fn open(&self, path: &PathBuf) -> Option<Box<dyn Dictionary>>;
    fn open_read_only(&self, path: &PathBuf) -> Option<Box<dyn Dictionary>>;
}

struct LoaderWrapper<T> {
    _marker: PhantomData<T>,
}

impl<T> LoaderWrapper<T> {
    fn new() -> Box<LoaderWrapper<T>> {
        Box::new(LoaderWrapper {
            _marker: PhantomData,
        })
    }
}

#[cfg(feature = "sqlite")]
impl DictionaryLoader for LoaderWrapper<SqliteDictionary> {
    fn open(&self, path: &PathBuf) -> Option<Box<dyn Dictionary>> {
        SqliteDictionary::open(path)
            .map(|dict| Box::new(dict) as Box<dyn Dictionary>)
            .ok()
    }

    fn open_read_only(&self, path: &PathBuf) -> Option<Box<dyn Dictionary>> {
        SqliteDictionary::open_read_only(path)
            .map(|dict| Box::new(dict) as Box<dyn Dictionary>)
            .ok()
    }
}

impl DictionaryLoader for LoaderWrapper<TrieDictionary> {
    fn open(&self, path: &PathBuf) -> Option<Box<dyn Dictionary>> {
        TrieDictionary::open(path)
            .map(|dict| Box::new(dict) as Box<dyn Dictionary>)
            .ok()
    }

    fn open_read_only(&self, path: &PathBuf) -> Option<Box<dyn Dictionary>> {
        self.open(path)
    }
}

impl DictionaryLoader for LoaderWrapper<CdbDictionary> {
    fn open(&self, path: &PathBuf) -> Option<Box<dyn Dictionary>> {
        CdbDictionary::open(path)
            .map(|dict| Box::new(dict) as Box<dyn Dictionary>)
            .ok()
    }

    fn open_read_only(&self, path: &PathBuf) -> Option<Box<dyn Dictionary>> {
        self.open(path)
    }
}
