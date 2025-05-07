use std::{
    ffi::{CStr, c_char, c_int, c_uchar},
    ptr,
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

unsafe fn ue_str_nbytes(str: *const c_char, n: c_int) -> c_int {
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
    let bytes = unsafe { ue_str_nbytes(src, n as c_int) } as usize;
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
    let bytes = unsafe { ue_str_nbytes(src, n as c_int) };
    unsafe { src.offset(bytes as isize) }
}
