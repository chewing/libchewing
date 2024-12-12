//! Types and functions related to file system path operations.

use std::{
    env,
    ffi::OsStr,
    path::{Path, PathBuf},
};

use directories::{BaseDirs, ProjectDirs};
use log::{info, warn};

const DEFAULT_UNIX_SYS_PATH: &str = "/usr/share/libchewing";
const UNIX_SYS_PATH: Option<&str> = option_env!("CHEWING_DATADIR");

#[cfg(target_family = "windows")]
const SEARCH_PATH_SEP: char = ';';

#[cfg(target_family = "unix")]
const SEARCH_PATH_SEP: char = ':';

const DICT_FOLDER: &str = "dictionary.d";

pub(crate) fn sys_path_from_env_var() -> String {
    let chewing_path = env::var("CHEWING_PATH");
    if let Ok(chewing_path) = chewing_path {
        info!("Using syspath from env CHEWING_PATH: {}", chewing_path);
        chewing_path
    } else {
        let user_datadir = data_dir();
        let sys_datadir = UNIX_SYS_PATH.unwrap_or(DEFAULT_UNIX_SYS_PATH);
        let chewing_path = if let Some(datadir) = user_datadir.as_ref().and_then(|p| p.to_str()) {
            format!("{datadir}:{sys_datadir}")
        } else {
            sys_datadir.into()
        };
        info!("Using default syspath: {}", chewing_path);
        chewing_path
    }
}

pub(crate) fn find_path_by_files(search_path: &str, files: &[&str]) -> Option<PathBuf> {
    for path in search_path.split(SEARCH_PATH_SEP) {
        let prefix = Path::new(path).to_path_buf();
        info!("Search files {:?} in {}", files, prefix.display());
        if files
            .iter()
            .map(|it| {
                let mut path = prefix.clone();
                path.push(it);
                path
            })
            .all(|it| it.exists())
        {
            info!("Found {:?} in {}", files, prefix.display());
            return Some(prefix);
        }
    }
    None
}

pub(crate) fn find_extra_dat_by_path(search_path: &str) -> Vec<PathBuf> {
    let mut results = vec![];
    for path in search_path.split(SEARCH_PATH_SEP) {
        let prefix = Path::new(path).join(DICT_FOLDER);
        info!("Search dictionary files in {}", prefix.display());
        if let Ok(read_dir) = prefix.read_dir() {
            let mut files = vec![];
            for entry in read_dir.flatten() {
                let path = entry.path();
                let is_dat = path
                    .extension()
                    .and_then(OsStr::to_str)
                    .map_or(false, |ext| ext == "dat");
                if path.is_file() && is_dat {
                    info!("Found {}", path.display());
                    files.push(path);
                }
            }
            files.sort();
            results.extend(files);
        }
    }
    results
}

/// Returns the path to the user's default chewing data directory.
///
/// The returned value depends on the operating system and is either a
/// Some, containing a value from the following table, or a None.
///
/// |Platform | Base                                     | Example                                                     |
/// | ------- | ---------------------------------------- | ------------------------------------------------------------|
/// | Linux   | `$XDG_DATA_HOME` or `$HOME`/.local/share | /home/alice/.local/share/chewing                            |
/// | macOS   | `$HOME`/Library/Application Support      | /Users/Alice/Library/Application Support/im.chewing.Chewing |
/// | Windows | `{FOLDERID_RoamingAppData}`              | C:\Users\Alice\AppData\Roaming\chewing\Chewing\data         |
///
/// Legacy path is automatically detected and used
///
/// |Platform | Base           | Example                            |
/// | ------- | -------------- | --------------------- ------------ |
/// | Linux   | `$HOME`        | /home/alice/.chewing               |
/// | macOS   | /Library       | /Library/ChewingOSX                |
/// | Windows | `$USERPROFILE` | C:\Users\Alice\ChewingTextService  |
///
/// Users can set the `CHEWING_USER_PATH` environment variable to
/// override the default path.
pub fn data_dir() -> Option<PathBuf> {
    if let Ok(path) = env::var("CHEWING_USER_PATH") {
        info!("Using userpath from env CHEWING_USER_PATH: {}", path);
        return Some(path.into());
    }
    if let Some(path) = legacy_data_dir() {
        if path.exists() && path.is_dir() {
            info!("Using legacy userpath: {}", path.display());
            return Some(path);
        }
    }
    let data_dir = ProjectDirs::from("im", "chewing", "Chewing")
        .as_ref()
        .map(ProjectDirs::data_dir)
        .map(Path::to_owned);
    if let Some(path) = &data_dir {
        info!("Using default userpath: {}", path.display());
    } else {
        warn!("No valid home directory path could be retrieved from the operating system.");
    }
    data_dir
}

fn legacy_data_dir() -> Option<PathBuf> {
    if cfg!(target_os = "windows") {
        return BaseDirs::new()
            .as_ref()
            .map(BaseDirs::home_dir)
            .map(|path| path.join("ChewingTextService"));
    }

    if cfg!(any(target_os = "macos", target_os = "ios")) {
        return Some("/Library/ChewingOSX".into());
    }

    BaseDirs::new()
        .as_ref()
        .map(BaseDirs::home_dir)
        .map(|path| path.join(".chewing"))
}

/// Returns the path to the user's default userphrase database file.
///
/// This function uses the default path from the [`data_dir()`] method
/// and also respects the `CHEWING_USER_PATH` environment variable.
pub fn userphrase_path() -> Option<PathBuf> {
    data_dir().map(|path| path.join("chewing.dat"))
}

#[cfg(test)]
mod tests {
    use directories::BaseDirs;

    use super::data_dir;

    #[test]
    fn resolve_data_dir() {
        if BaseDirs::new().is_some() {
            let data_dir = data_dir();
            assert!(data_dir.is_some());
        }
    }
}
