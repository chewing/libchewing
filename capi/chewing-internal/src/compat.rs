use std::ffi::c_char;

use crate::types::ChewingData;

#[no_mangle]
pub extern "C" fn InitPinyin(_pgdata: &mut ChewingData, _path: *mut c_char) -> bool {
    true
}

#[no_mangle]
pub extern "C" fn TerminatePinyin(_pgdata: &mut ChewingData) {}
