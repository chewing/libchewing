use std::ffi::{c_char, c_int};

#[no_mangle]
pub extern "C" fn chewing_version() -> *const c_char {
    c"0.9.0-rc.1".as_ptr()
}

#[no_mangle]
pub extern "C" fn chewing_version_major() -> c_int {
    0
}

#[no_mangle]
pub extern "C" fn chewing_version_minor() -> c_int {
    9
}

#[no_mangle]
pub extern "C" fn chewing_version_patch() -> c_int {
    0
}

#[no_mangle]
pub extern "C" fn chewing_version_extra() -> *const c_char {
    c"-rc.1".as_ptr()
}
