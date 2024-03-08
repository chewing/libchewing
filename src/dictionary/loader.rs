use std::{
    ffi::OsStr,
    fs::{self, File},
    io::{self, Seek},
    marker::PhantomData,
    path::{Path, PathBuf},
};

use crate::path::{find_path_by_files, sys_path_from_env_var, userphrase_path};

#[cfg(feature = "sqlite")]
use super::SqliteDictionary;
use super::{uhash, CdbDictionary, Dictionary, TrieDictionary};

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
    pub fn load(self) -> io::Result<Box<dyn Dictionary>> {
        let data_path = self
            .data_path
            .or_else(userphrase_path)
            .ok_or(io::Error::from(io::ErrorKind::NotFound))?;
        if data_path.exists() {
            return guess_format_and_load(&data_path);
        }
        let userdata_dir = data_path.parent().expect("path should contain a filename");
        if !userdata_dir.exists() {
            fs::create_dir_all(&userdata_dir)?;
        }
        let mut fresh_dict = init_user_dictionary(&data_path)?;

        // FIXME: should use dict.update_phrase to also migrate user_freq
        let cdb_path = userdata_dir.join(UD_CDB_FILE_NAME);
        if data_path != cdb_path && cdb_path.exists() {
            let cdb_dict = CdbDictionary::open(cdb_path)
                .map_err(|e| io::Error::new(io::ErrorKind::Other, Box::new(e)))?;
            for (syllables, phrase) in cdb_dict
                .entries()
                .expect("CDB dictionary should support entries()")
            {
                fresh_dict
                    .add_phrase(&syllables, phrase)
                    .map_err(|e| io::Error::new(io::ErrorKind::Other, Box::new(e)))?;
            }
            fresh_dict
                .flush()
                .map_err(|e| io::Error::new(io::ErrorKind::Other, Box::new(e)))?;
        } else {
            let uhash_path = userdata_dir.join(UD_UHASH_FILE_NAME);
            if uhash_path.exists() {
                let mut input = File::open(uhash_path)?;
                if let Ok(phrases) = uhash::try_load_bin(&input).or_else(|_| {
                    input.rewind()?;
                    uhash::try_load_text(&input)
                }) {
                    for (syllables, phrase) in phrases {
                        fresh_dict
                            .add_phrase(&syllables, phrase)
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

    init_user_dictionary(&dict_path)
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
    } else if ext.eq_ignore_ascii_case("cdb") {
        CdbDictionary::open(dict_path)
            .map(|dict| Box::new(dict) as Box<dyn Dictionary>)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, Box::new(e)))
    } else {
        Err(io::Error::from(io::ErrorKind::Other))
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
