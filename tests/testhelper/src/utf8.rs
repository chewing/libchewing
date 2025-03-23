use std::{
    ffi::{c_char, c_int, c_uchar, CStr},
    ptr::{self, null},
    str,
};

/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ueStrLen(str: *const c_char) -> c_int {
    let cstr = unsafe { CStr::from_ptr(str) };
    cstr.to_str().unwrap_or("").chars().count() as c_int
}

#[unsafe(no_mangle)]
pub extern "C" fn ueBytesFromChar(b: c_uchar) -> c_int {
    const UTF8LEN_TAB: [c_int; 256] = [
        1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
        1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
        1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
        1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
        1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
        1, 1, 1, 1, 1, 1, 1, 1, 1, 1, /*bogus */
        1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
        1, 1, /*bogus */
        2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
        2, 2, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 4, 4, 4, 4, 4, 4, 4, 4, 5, 5, 5, 5,
        6, 6, 1, 1,
    ];
    UTF8LEN_TAB[b as usize]
}

/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ueStrNBytes(str: *const c_char, n: c_int) -> c_int {
    let cstr = unsafe { CStr::from_ptr(str) };
    let rstr = match cstr.to_str() {
        Ok(rstr) => rstr,
        Err(err) => str::from_utf8(&cstr.to_bytes()[..err.valid_up_to()]).unwrap(),
    };
    rstr.chars()
        .take(n as usize)
        .map(|it| it.len_utf8())
        .sum::<usize>() as c_int
}

enum StrNCpyClose {
    StrncpyClose = 1,
    // StrncpyNotClose = 0,
}

/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ueStrNCpy(
    dest: *mut c_char,
    src: *const c_char,
    n: usize,
    end: c_int,
) -> c_int {
    let bytes = unsafe { ueStrNBytes(src, n as c_int) } as usize;
    unsafe { src.copy_to(dest, bytes) };
    if end == StrNCpyClose::StrncpyClose as i32 {
        unsafe { ptr::write(dest.add(bytes), 0) };
    }
    bytes as c_int
}

/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ueStrSeek(src: *mut c_char, n: usize) -> *mut c_char {
    let bytes = unsafe { ueStrNBytes(src, n as c_int) };
    unsafe { src.offset(bytes as isize) }
}

/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ueConstStrSeek(src: *const c_char, n: usize) -> *const c_char {
    let bytes = unsafe { ueStrNBytes(src, n as c_int) };
    unsafe { src.offset(bytes as isize) }
}

/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ueStrStr(
    str: *const c_char,
    _lstr: usize,
    substr: *const c_char,
    _lsub: usize,
) -> *const c_char {
    let cstr = unsafe { CStr::from_ptr(str) }
        .to_str()
        .expect("should be valid utf8");
    let sub = unsafe { CStr::from_ptr(substr) }
        .to_str()
        .expect("should be valid utf8");
    match cstr.find(sub) {
        Some(count) => unsafe { str.add(count) },
        None => null(),
    }
}
