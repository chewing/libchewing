use core::slice;
use std::{
    borrow::Cow,
    env,
    ffi::{CStr, CString, c_char, c_int},
    path::Path,
};

use chewing::path;

#[cfg(target_family = "windows")]
const SEARCH_PATH_SEP: char = ';';

#[cfg(target_family = "unix")]
const SEARCH_PATH_SEP: char = ':';

/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn get_search_path(path: *mut c_char, path_len: usize) -> c_int {
    let chewing_path = env::var("CHEWING_PATH");
    if let Ok(chewing_path) = chewing_path {
        let path_cstring = CString::new(chewing_path).expect("string should not have internal nul");
        let bytes = path_cstring.as_bytes_with_nul();
        if bytes.len() > path_len {
            return 1;
        }
        let out = unsafe { slice::from_raw_parts_mut(path as *mut u8, bytes.len()) };
        out.copy_from_slice(bytes);
    } else {
        let user_datadir = path::data_dir();
        if let Some(datadir) = user_datadir.as_ref().and_then(|p| p.to_str()) {
            let path_cstring = CString::new(format!("{datadir}:/usr/share/libchewing")).unwrap();
            let bytes = path_cstring.as_bytes_with_nul();
            if bytes.len() > path_len {
                return 1;
            }
            let out = unsafe { slice::from_raw_parts_mut(path as *mut u8, bytes.len()) };
            out.copy_from_slice(bytes);
        }
    }
    0
}

/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn find_path_by_files(
    search_path: *const c_char,
    files: *const *const c_char,
    output: *mut c_char,
    output_len: usize,
) -> c_int {
    let search_path = unsafe { CStr::from_ptr(search_path) };
    let search_path = search_path.to_str();
    let files = unsafe { files_ptr_to_slice(files) };
    if let Ok(search_path) = search_path {
        for path in search_path.split(SEARCH_PATH_SEP) {
            let prefix = Path::new(path).to_path_buf();
            if files
                .iter()
                .map(|it| {
                    let mut path = prefix.clone();
                    path.push(it.as_ref());
                    path
                })
                .all(|it| it.exists())
            {
                let path = CString::new(path).expect("no internal null");
                let path = path.as_bytes_with_nul();
                if path.len() > output_len {
                    return -1;
                }
                let output = unsafe { slice::from_raw_parts_mut(output.cast(), path.len()) };
                output.copy_from_slice(path);

                return 0;
            }
        }
    }
    -1
}

unsafe fn files_ptr_to_slice(files: *const *const c_char) -> Vec<Cow<'static, str>> {
    let len = {
        let mut i = 0;
        while unsafe { !files.add(i).read().is_null() } {
            i += 1;
        }
        i
    };
    let files = unsafe { slice::from_raw_parts(files, len) };
    files
        .iter()
        .map(|&it| unsafe { CStr::from_ptr(it) })
        .map(|it| it.to_string_lossy())
        .collect()
}
