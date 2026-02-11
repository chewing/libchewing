//! Types and functions related to file system path operations.

use std::{
    env,
    ffi::OsStr,
    fs,
    io::{self, ErrorKind},
    path::{Path, PathBuf},
};

use log::{info, warn};

#[cfg(target_family = "windows")]
const DEFAULT_SYS_PATH: &str = "C:\\Program Files\\ChewingTextService\\Dictionary";
#[cfg(target_family = "unix")]
const DEFAULT_SYS_PATH: &str = "/usr/share/libchewing";
const SYS_PATH: Option<&str> = option_env!("CHEWING_DATADIR");

#[cfg(target_family = "windows")]
const SEARCH_PATH_SEP: char = ';';
#[cfg(target_family = "unix")]
const SEARCH_PATH_SEP: char = ':';

const DICT_FOLDER: &str = "dictionary.d";

// On Windows if a low integrity process tries to write to a higher integrity
// process, it fails with PermissionDenied error. Current `fs::exists()` in Rust
// happens to use CreateFile to check if a file exists that triggers this error.
fn file_exists(path: &Path) -> bool {
    match fs::exists(path) {
        Ok(true) => true,
        Ok(false) => false,
        Err(error) => matches!(error.kind(), ErrorKind::PermissionDenied),
    }
}

pub fn search_path_from_env_var() -> String {
    let mut paths = vec![];
    if let Some(user_datadir) = data_dir() {
        paths.push(
            user_datadir
                .join(DICT_FOLDER)
                .to_string_lossy()
                .into_owned(),
        );
        paths.push(user_datadir.to_string_lossy().into_owned());
    }
    let chewing_path = env::var("CHEWING_PATH");
    if let Ok(chewing_path) = chewing_path {
        info!("Add path from CHEWING_PATH: {}", chewing_path);
        paths.push(chewing_path);
    } else {
        let sys_datadir = PathBuf::from(SYS_PATH.unwrap_or(DEFAULT_SYS_PATH));
        paths.push(sys_datadir.join(DICT_FOLDER).to_string_lossy().into_owned());
        paths.push(sys_datadir.to_string_lossy().into_owned());
    }
    let chewing_path = paths.join(&SEARCH_PATH_SEP.to_string());
    info!("Using search path: {}", chewing_path);
    chewing_path
}

pub(crate) fn find_path_by_files(search_path: &str, files: &[&str]) -> Result<PathBuf, io::Error> {
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
            .all(|it| file_exists(&it))
        {
            info!("Found {:?} in {}", files, prefix.display());
            return Ok(prefix);
        }
    }
    Err(ErrorKind::NotFound.into())
}

pub fn find_files_by_ext(search_path: &str, exts: &[&str]) -> Vec<PathBuf> {
    let mut files = vec![];
    for path in search_path.split(SEARCH_PATH_SEP) {
        let prefix = Path::new(path).to_path_buf();
        info!(
            "Search files with extension {:?} in {}",
            exts,
            prefix.display()
        );
        if let Ok(read_dir) = prefix.read_dir() {
            for entry in read_dir.flatten() {
                let file_path = entry.path();
                if file_path.is_file()
                    && file_path
                        .extension()
                        .and_then(OsStr::to_str)
                        .is_some_and(|ext| exts.contains(&ext))
                {
                    info!("Found {}", file_path.display());
                    files.push(file_path.to_path_buf());
                }
            }
        }
    }
    files
}

pub fn find_files_by_names<T>(search_path: &str, names: &[T]) -> Vec<PathBuf>
where
    T: AsRef<str>,
{
    let mut files = vec![];
    for path in search_path.split(SEARCH_PATH_SEP) {
        let prefix = Path::new(path).to_path_buf();
        info!("Search files in {}", prefix.display());
        if let Ok(read_dir) = prefix.read_dir() {
            for entry in read_dir.flatten() {
                let file_path = entry.path();
                if file_path.is_file()
                    && names.iter().any(|name| file_path.ends_with(name.as_ref()))
                {
                    info!("Found {}", file_path.display());
                    files.push(file_path.to_path_buf());
                }
            }
        }
    }
    files
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
        if file_exists(&path) && path.is_dir() {
            info!("Using legacy userpath: {}", path.display());
            return Some(path);
        }
    }
    let data_dir = project_data_dir();
    if let Some(path) = &data_dir {
        info!("Using default userpath: {}", path.display());
    } else {
        warn!("No valid home directory path could be retrieved from the operating system.");
    }
    data_dir
}

fn project_data_dir() -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        if let Ok(path) = env::var("AppData") {
            return Some(PathBuf::from(path).join("Chewing"));
        }
    }
    #[cfg(target_os = "macos")]
    {
        return env::home_dir().map(|path| {
            path.join("Library")
                .join("Application Support")
                .join("im.chewing.Chewing")
        });
    }
    #[cfg(not(target_family = "unix"))]
    {
        return None;
    }

    #[cfg(target_family = "unix")]
    {
        if let Ok(path) = env::var("XDG_DATA_HOME") {
            return Some(PathBuf::from(path).join("chewing"));
        }
        env::home_dir().map(|path| path.join(".local").join("share").join("chewing"))
    }
}

fn legacy_data_dir() -> Option<PathBuf> {
    if cfg!(target_os = "windows") {
        return env::home_dir().map(|path| path.join("ChewingTextService"));
    }

    if cfg!(any(target_os = "macos", target_os = "ios")) {
        return Some("/Library/ChewingOSX".into());
    }

    env::home_dir().map(|path| path.join(".chewing"))
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
    use std::{error::Error, fs};

    use tempfile::TempDir;

    use super::{
        SEARCH_PATH_SEP, data_dir, find_files_by_ext, find_files_by_names, project_data_dir,
    };

    #[test]
    fn support_project_data_dir() {
        assert!(project_data_dir().is_some());
    }

    #[test]
    fn resolve_data_dir() {
        if project_data_dir().is_some() {
            let data_dir = data_dir();
            assert!(data_dir.is_some());
        }
    }

    #[test]
    fn find_files_by_ext_from_places_two_exts() -> Result<(), Box<dyn Error>> {
        let project_data_dir = TempDir::new()?;
        let user_data_dir = TempDir::new()?;
        let user_drop_in_dir = TempDir::new()?;

        let project_tsi_dat = project_data_dir.path().join("tsi.dat");
        let user_tsi_dat = user_data_dir.path().join("tsi.dat");
        let user_sqlite3 = user_drop_in_dir.path().join("chewing.sqlite3");

        fs::write(&project_tsi_dat, "")?;
        fs::write(&user_tsi_dat, "")?;
        fs::write(&user_sqlite3, "")?;

        let search_path = [
            project_data_dir.path().to_string_lossy().as_ref(),
            user_data_dir.path().to_string_lossy().as_ref(),
            user_drop_in_dir.path().to_string_lossy().as_ref(),
        ]
        .join(&SEARCH_PATH_SEP.to_string());

        assert_eq!(
            [project_tsi_dat, user_tsi_dat, user_sqlite3].as_slice(),
            find_files_by_ext(&search_path, &["dat", "sqlite3"])
        );

        Ok(())
    }

    #[test]
    fn find_files_by_names_from_places_two_names() -> Result<(), Box<dyn Error>> {
        let project_data_dir = TempDir::new()?;
        let user_data_dir = TempDir::new()?;
        let user_drop_in_dir = TempDir::new()?;

        let project_tsi_dat = project_data_dir.path().join("tsi.dat");
        let user_tsi_dat = user_data_dir.path().join("tsi.dat");
        let user_alt_dat = user_drop_in_dir.path().join("alt.dat");

        fs::write(&project_tsi_dat, "")?;
        fs::write(&user_tsi_dat, "")?;
        fs::write(&user_alt_dat, "")?;

        let search_path = [
            project_data_dir.path().to_string_lossy().as_ref(),
            user_data_dir.path().to_string_lossy().as_ref(),
            user_drop_in_dir.path().to_string_lossy().as_ref(),
        ]
        .join(&SEARCH_PATH_SEP.to_string());

        assert_eq!(
            [project_tsi_dat, user_tsi_dat, user_alt_dat].as_slice(),
            find_files_by_names(&search_path, &["tsi.dat", "alt.dat"])
        );

        Ok(())
    }
}
