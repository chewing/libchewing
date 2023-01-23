use std::{
    ffi::{c_char, c_int, CStr},
    ptr, slice,
};

use chewing::zhuyin::{Bopomofo, Syllable};

#[no_mangle]
pub extern "C" fn UintFromPhone(phone: *const c_char) -> u16 {
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

#[no_mangle]
pub extern "C" fn UintFromPhoneInx(ph_inx: *const c_int) -> u16 {
    let ph_inx = unsafe { slice::from_raw_parts(ph_inx, 4) };
    let mut builder = Syllable::builder();
    if ph_inx[0] > 0 {
        let bopomofo = match Bopomofo::from_initial(ph_inx[0] as u16) {
            Ok(bopomofo) => bopomofo,
            Err(_) => return 0,
        };
        builder = match builder.insert(bopomofo) {
            Ok(builder) => builder,
            Err(_) => return 0,
        };
    }
    if ph_inx[1] > 0 {
        let bopomofo = match Bopomofo::from_medial(ph_inx[1] as u16) {
            Ok(bopomofo) => bopomofo,
            Err(_) => return 0,
        };
        builder = match builder.insert(bopomofo) {
            Ok(builder) => builder,
            Err(_) => return 0,
        };
    }
    if ph_inx[2] > 0 {
        let bopomofo = match Bopomofo::from_rime(ph_inx[2] as u16) {
            Ok(bopomofo) => bopomofo,
            Err(_) => return 0,
        };
        builder = match builder.insert(bopomofo) {
            Ok(builder) => builder,
            Err(_) => return 0,
        };
    }
    if ph_inx[3] > 0 {
        let bopomofo = match Bopomofo::from_tone(ph_inx[3] as u16) {
            Ok(bopomofo) => bopomofo,
            Err(_) => return 0,
        };
        builder = match builder.insert(bopomofo) {
            Ok(builder) => builder,
            Err(_) => return 0,
        };
    }
    let syl = builder.build();
    if syl.is_empty() {
        return 0;
    }
    syl.to_u16()
}

#[no_mangle]
pub extern "C" fn PhoneFromUint(phone: *mut c_char, phone_len: usize, phone_num: u16) -> c_int {
    let syl = match Syllable::try_from(phone_num) {
        Ok(syl) => syl,
        Err(_) => return 1,
    };
    let str = syl.to_string();
    if phone_len < str.len() + 1 {
        return 1;
    }
    unsafe {
        phone.copy_from(str.as_ptr() as *const c_char, str.len());
        ptr::write(phone.offset(str.len() as isize), 0);
    }
    0
}

#[no_mangle]
pub extern "C" fn UintArrayFromBopomofo(
    phone_seq: *mut u16,
    phone_len: usize,
    bopomofo_buf: *const c_char,
) -> isize {
    let syllables_str = match unsafe { CStr::from_ptr(bopomofo_buf) }.to_str() {
        Ok(str) => str,
        Err(_) => return -1,
    };
    let syllables: Vec<_> = syllables_str
        .split_ascii_whitespace()
        .map(|it| it.parse::<Syllable>().map(|syl| syl.to_u16()))
        .collect();
    let len = syllables.len();
    if syllables.iter().any(|it| it.is_err()) {
        return -1;
    }
    if len > phone_len || phone_seq.is_null() {
        return len as isize;
    }
    for (i, syl) in syllables.into_iter().enumerate() {
        let syl_u16 = syl.unwrap();
        unsafe { ptr::write(phone_seq.offset(i as isize), syl_u16) };
    }
    len as isize
}

#[no_mangle]
pub extern "C" fn GetPhoneLenFromUint(phone_num: u16) -> c_int {
    let syl = match Syllable::try_from(phone_num) {
        Ok(syl) => syl,
        Err(_) => return -1,
    };
    if syl.is_empty() {
        return -1;
    }
    (syl.to_string().len() + 1) as c_int
}
