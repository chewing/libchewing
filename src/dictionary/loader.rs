use std::{
    marker::PhantomData,
    path::{Path, PathBuf},
};

use crate::path::{find_path_by_files, sys_path_from_env_var, userphrase_path};

#[cfg(feature = "sqlite")]
use super::SqliteDictionary;
use super::{CdbDictionary, Dictionary, TrieDictionary};

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
        let sys_path = find_path_by_files(&search_path, &["tsi.dat", "word.dat"])
            .ok_or("SystemDictionaryNotFound")?;

        let tsi_db_path = sys_path.join("tsi.dat");
        let tsi_db = db_loaders
            .iter()
            .find_map(|loader| loader.open_read_only(&tsi_db_path));

        let word_db_path = sys_path.join("word.dat");
        let word_db = db_loaders
            .iter()
            .find_map(|loader| loader.open_read_only(&word_db_path));

        Ok(vec![word_db.unwrap(), tsi_db.unwrap()])
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
        let mut db_loaders: Vec<Box<dyn DictionaryLoader>> = vec![];
        #[cfg(feature = "sqlite")]
        {
            db_loaders.push(LoaderWrapper::<SqliteDictionary>::new());
        }
        db_loaders.push(LoaderWrapper::<CdbDictionary>::new());

        let data_path = if let Some(data_path) = self.data_path {
            data_path
        } else {
            userphrase_path().ok_or("UserDictionaryNotFound")?
        };

        db_loaders
            .iter()
            .find_map(|loader| loader.open(&data_path))
            .ok_or("ErrorOpenUserDictionary")
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
