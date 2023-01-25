//! Types and functions related to file system path operations.

use std::{env, path::PathBuf};

use dirs_next::{data_dir as user_data_dir, home_dir};

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
    user_data_dir().map(|path| path.join("chewing"))
}

fn legacy_data_dir() -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    return home_dir().map(|path| path.join("ChewingTextService"));

    #[cfg(any(target_os = "macos", target_os = "ios"))]
    return Some("/Library/ChewingOSX".into());

    home_dir().map(|path| path.join(".chewing"))
}

/// Returns the path to the user's default userphrase database file.
///
/// This function uses the default path from the [`data_dir()`] method
/// and also respects the `CHEWING_USER_PATH` environment variable.
pub fn userphrase_path() -> Option<PathBuf> {
    // TODO support uhash.dat
    data_dir().map(|path| path.join("chewing.sqlite3"))
}
