use std::ffi::{c_char, c_int};

pub const CHEWING_VERSION_MAJOR: c_int = 0;
pub const CHEWING_VERSION_MINOR: c_int = 9;
pub const CHEWING_VERSION_PATCH: c_int = 1;

#[no_mangle]
pub extern "C" fn chewing_version() -> *const c_char {
    c"0.9.1".as_ptr()
}

#[no_mangle]
pub extern "C" fn chewing_version_major() -> c_int {
    CHEWING_VERSION_MAJOR
}

#[no_mangle]
pub extern "C" fn chewing_version_minor() -> c_int {
    CHEWING_VERSION_MINOR
}

#[no_mangle]
pub extern "C" fn chewing_version_patch() -> c_int {
    CHEWING_VERSION_PATCH
}

#[no_mangle]
pub extern "C" fn chewing_version_extra() -> *const c_char {
    c"".as_ptr()
}
