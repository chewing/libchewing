use std::ffi::{c_char, c_int};

pub const CHEWING_VERSION_MAJOR: c_int = 0;
pub const CHEWING_VERSION_MINOR: c_int = 10;
pub const CHEWING_VERSION_PATCH: c_int = 0;

#[unsafe(no_mangle)]
pub extern "C" fn chewing_version() -> *const c_char {
    c"0.10.0".as_ptr()
}

#[unsafe(no_mangle)]
pub extern "C" fn chewing_version_major() -> c_int {
    CHEWING_VERSION_MAJOR
}

#[unsafe(no_mangle)]
pub extern "C" fn chewing_version_minor() -> c_int {
    CHEWING_VERSION_MINOR
}

#[unsafe(no_mangle)]
pub extern "C" fn chewing_version_patch() -> c_int {
    CHEWING_VERSION_PATCH
}

#[unsafe(no_mangle)]
pub extern "C" fn chewing_version_extra() -> *const c_char {
    c"".as_ptr()
}
