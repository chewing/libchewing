use std::path::{Path, PathBuf};

use crate::{
    editor::SqliteUserFreqEstimate,
    path::{find_path_by_files, sys_path_from_env_var, userphrase_path},
};

use super::{CdbDictionary, Dictionary, SqliteDictionary, TrieDictionary};

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
        let tsi_db = if let Ok(db) = SqliteDictionary::open_read_only(&tsi_db_path) {
            Box::new(db) as Box<dyn Dictionary>
        } else if let Ok(db) = TrieDictionary::open(&tsi_db_path) {
            Box::new(db) as Box<dyn Dictionary>
        } else if let Ok(db) = CdbDictionary::open(&tsi_db_path) {
            Box::new(db) as Box<dyn Dictionary>
        } else {
            return None;
        };

        let mut word_db_path = sys_path;
        word_db_path.push("word.dat");
        let word_db = if let Ok(db) = SqliteDictionary::open_read_only(&word_db_path) {
            Box::new(db) as Box<dyn Dictionary>
        } else if let Ok(db) = TrieDictionary::open(&word_db_path) {
            Box::new(db) as Box<dyn Dictionary>
        } else if let Ok(db) = CdbDictionary::open(&word_db_path) {
            Box::new(db) as Box<dyn Dictionary>
        } else {
            return None;
        };

        Some(vec![word_db, tsi_db])
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
        let mut data_path = if let Some(data_path) = self.data_path {
            data_path
        } else {
            userphrase_path()?
        };
        data_path.set_extension("cdb");

        let dict = /*if let Ok(db) = SqliteDictionary::open(&data_path) {
            Box::new(db) as Box<dyn Dictionary>
        } else if let Ok(db) = TrieDictionary::open(&data_path) {
            Box::new(db) as Box<dyn Dictionary>
        } else*/ if let Ok(db) = CdbDictionary::open(&data_path) {
            Box::new(db) as Box<dyn Dictionary>
        } else {
            return None;
        };

        Some(dict)
    }
}

#[derive(Debug)]
pub struct UserFreqEstimateLoader {
    data_path: Option<PathBuf>,
}

impl UserFreqEstimateLoader {
    pub fn new() -> UserFreqEstimateLoader {
        UserFreqEstimateLoader { data_path: None }
    }
    pub fn userphrase_path(mut self, path: impl AsRef<Path>) -> UserFreqEstimateLoader {
        self.data_path = Some(path.as_ref().to_path_buf());
        self
    }
    pub fn load(self) -> Option<SqliteUserFreqEstimate> {
        let data_path = if let Some(data_path) = self.data_path {
            data_path
        } else {
            userphrase_path()?
        };

        let estimate = if let Ok(db) = SqliteUserFreqEstimate::open_in_memory() {
            db.into()
        } else {
            return None;
        };

        Some(estimate)
    }
}
