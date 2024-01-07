use std::ffi::{c_char, CStr};

use crate::zhuyin::Syllable;

#[no_mangle]
pub unsafe extern "C" fn UintFromPhone(phone: *const c_char) -> u16 {
    let cstr = unsafe { CStr::from_ptr(phone) };
    let rstr = match cstr.to_str() {
        Ok(rstr) => rstr,
        Err(_) => return 0,
    };
    let syl: Syllable = match rstr.parse() {
        Ok(syl) => syl,
        Err(_) => return 0,
    };
    syl.to_u16()
}
