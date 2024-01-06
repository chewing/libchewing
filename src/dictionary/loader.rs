use std::path::{Path, PathBuf};

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
    pub fn load(self) -> Option<Vec<Box<dyn Dictionary>>> {
        let search_path = if let Some(sys_path) = self.sys_path {
            sys_path
        } else {
            sys_path_from_env_var()
        };
        let sys_path = find_path_by_files(&search_path, &["tsi.dat", "word.dat"])?;

        let mut tsi_db_path = sys_path.clone();
        tsi_db_path.push("tsi.dat");
        let mut tsi_db = None;
        #[cfg(feature = "sqlite")]
        {
            tsi_db = SqliteDictionary::open_read_only(&tsi_db_path)
                .map(|db| Box::new(db) as Box<dyn Dictionary>)
                .ok();
        }
        if tsi_db.is_none() {
            tsi_db = TrieDictionary::open(&tsi_db_path)
                .map(|db| Box::new(db) as Box<dyn Dictionary>)
                .ok();
        }

        let mut word_db_path = sys_path;
        word_db_path.push("word.dat");
        let mut word_db = None;
        #[cfg(feature = "sqlite")]
        {
            word_db = SqliteDictionary::open_read_only(&word_db_path)
                .map(|db| Box::new(db) as Box<dyn Dictionary>)
                .ok();
        }
        if word_db.is_none() {
            word_db = TrieDictionary::open(&word_db_path)
                .map(|db| Box::new(db) as Box<dyn Dictionary>)
                .ok();
        }

        Some(vec![word_db.unwrap(), tsi_db.unwrap()])
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
    pub fn load(self) -> Option<Box<dyn Dictionary>> {
        let data_path = if let Some(data_path) = self.data_path {
            data_path
        } else {
            userphrase_path()?
        };

        let mut dict = None;
        #[cfg(feature = "sqlite")]
        {
            dict = dbg!(SqliteDictionary::open(&data_path))
                .map(|db| Box::new(db) as Box<dyn Dictionary>)
                .ok();
        }
        if dict.is_none() {
            dict = CdbDictionary::open(&data_path)
                .map(|db| Box::new(db) as Box<dyn Dictionary>)
                .ok();
        }

        dict
    }
}
