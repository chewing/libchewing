//! Types and functions related to file system path operations.

use std::{
    env,
    path::{Path, PathBuf},
};

use directories::{BaseDirs, ProjectDirs};

const UNIX_SYS_PATH: &str = "/usr/share/libchewing";

#[cfg(target_family = "windows")]
const SEARCH_PATH_SEP: char = ';';

#[cfg(target_family = "unix")]
const SEARCH_PATH_SEP: char = ':';

pub(crate) fn sys_path_from_env_var() -> String {
    let chewing_path = env::var("CHEWING_PATH");
    if let Ok(chewing_path) = chewing_path {
        chewing_path
    } else {
        let user_datadir = data_dir();
        if let Some(datadir) = user_datadir.as_ref().and_then(|p| p.to_str()) {
            format!("{datadir}:{UNIX_SYS_PATH}")
        } else {
            UNIX_SYS_PATH.into()
        }
    }
}

pub(crate) fn find_path_by_files(search_path: &str, files: &[&str]) -> Option<PathBuf> {
    for path in search_path.split(SEARCH_PATH_SEP) {
        let prefix = Path::new(path).to_path_buf();
        if files
            .iter()
            .map(|it| {
                let mut path = prefix.clone();
                path.push(it);
                path
            })
            .all(|it| it.exists())
        {
            return Some(prefix);
        }
    }
    None
}

/// Returns the path to the user's default chewing data directory.
///
/// The returned value depends on the operating system and is either a
/// Some, containing a value from the following table, or a None.
///
/// |Platform | Base                                     | Example                                          |
/// | ------- | ---------------------------------------- | ------------------------------------------------ |
/// | Linux   | `$XDG_DATA_HOME` or `$HOME`/.local/share | /home/alice/.local/share/chewing                 |
/// | macOS   | `$HOME`/Library/Application Support      | /Users/Alice/Library/Application Support/chewing |
/// | Windows | `{FOLDERID_RoamingAppData}`              | C:\Users\Alice\AppData\Roaming\chewing           |
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
    // TODO per-OS integration test
    if let Ok(path) = env::var("CHEWING_USER_PATH") {
        return Some(path.into());
    }
    if let Some(path) = legacy_data_dir() {
        if path.exists() && path.is_dir() {
            return Some(path);
        }
    }
    ProjectDirs::from("im", "chewing", "chewing")
        .as_ref()
        .map(ProjectDirs::data_dir)
        .map(Path::to_owned)
}

fn legacy_data_dir() -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    return BaseDirs::new()
        .as_ref()
        .map(BaseDirs::home_dir)
        .map(|path| path.join("ChewingTextService"));

    #[cfg(any(target_os = "macos", target_os = "ios"))]
    return Some("/Library/ChewingOSX".into());

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
    // TODO support uhash.dat
    data_dir().map(|path| path.join("chewing.sqlite3"))
}
