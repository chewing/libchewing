use std::{ffi::CString, slice};

use chewing::dictionary::Phrase;
use libc::c_char;

pub trait CopyToCString {
    fn copy_to(&self, buf: &mut [c_char]);
}

impl CopyToCString for Phrase {
    fn copy_to(&self, buf: &mut [c_char]) {
        let phrase_str = CString::new(self.as_str()).expect("Unable to convert to CString");
        let phrase_bytes = phrase_str.as_bytes_with_nul();
        let phrase_bytes = unsafe {
            slice::from_raw_parts(phrase_bytes.as_ptr() as *const i8, phrase_bytes.len())
        };
        buf[0..phrase_bytes.len()].copy_from_slice(phrase_bytes);
    }
}
