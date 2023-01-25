use std::{
    ffi::{c_void, CStr, CString},
    iter::Peekable,
    os::raw::{c_char, c_int, c_uint},
    path::Path,
    slice,
};

use chewing::{
    dictionary::{DictEntries, Dictionary, SqliteDictionary},
    editor::{SqliteUserFreqEstimate, UserFreqEstimate},
    path::userphrase_path,
    zhuyin::Syllable,
};

use crate::{
    ffi::CopyToCString,
    types::{ChewingData, UserPhraseData, UserUpdate},
};

pub struct UserphraseDbAndEstimate {
    db: SqliteDictionary,
    estimate: SqliteUserFreqEstimate,
}

#[no_mangle]
pub extern "C" fn GetDefaultUserPhrasePath(_data: *mut c_void) -> *mut c_char {
    match userphrase_path() {
        Some(path) => CString::new(
            path.as_os_str()
                .to_str()
                .expect("path should be valid utf-8"),
        )
        .expect("path should be vaild C string")
        .into_raw(),
        None => std::ptr::null_mut(),
    }
}

#[no_mangle]
pub unsafe extern "C" fn FreeDefaultUserPhrasePath(path: *mut c_char) {
    unsafe { CString::from_raw(path) };
}

#[no_mangle]
pub unsafe extern "C" fn InitUserphrase(pgdata: &mut ChewingData, path: *mut c_char) -> c_int {
    let path = unsafe { CStr::from_ptr(path) }
        .to_str()
        .expect("Invalid prefix string");
    let path: &Path = path.as_ref();

    let chewing_db = match SqliteDictionary::open(path) {
        Ok(db) => db,
        Err(_) => return -1,
    };

    let estimate = match SqliteUserFreqEstimate::open(path) {
        Ok(db) => db,
        Err(_) => return -1,
    };

    pgdata.ue = Some(Box::new(UserphraseDbAndEstimate {
        db: chewing_db,
        estimate,
    }));
    0
}

#[no_mangle]
pub extern "C" fn TerminateUserphrase(pgdata: &mut ChewingData) {
    pgdata.ue = None;
}

#[no_mangle]
pub unsafe extern "C" fn UserGetPhraseFirst<'a>(
    pgdata: &'a mut ChewingData,
    syllables_u16_ptr: *mut u16,
) -> Option<&'a UserPhraseData> {
    let ue = pgdata.ue.as_ref().expect("nonnull ue");
    let syllables_u16 = unsafe { slice::from_raw_parts(syllables_u16_ptr, 50) };
    let syllables = syllables_u16
        .iter()
        .take_while(|&&syl_u16| syl_u16 != 0)
        .map(|&syl_u16| Syllable::try_from(syl_u16).unwrap())
        .collect::<Vec<_>>();
    let iter = Box::new(
        ue.db
            .lookup_phrase(syllables.as_slice())
            .collect::<Vec<_>>()
            .into_iter(),
    ) as Box<dyn Iterator<Item = chewing::dictionary::Phrase>>;
    let mut iter = Box::new(iter);

    if let Some(phrase) = iter.next() {
        pgdata.phrase_iter = Box::into_raw(iter).cast();
        pgdata.userphrase_data.userfreq = phrase.freq() as c_int;
        phrase.copy_to(&mut pgdata.userphrase_data.word_seq);
        return Some(&pgdata.userphrase_data);
    }

    pgdata.phrase_iter = std::ptr::null_mut();
    None
}

#[no_mangle]
pub extern "C" fn UserGetPhraseNext<'a>(
    pgdata: &'a mut ChewingData,
    _syllables_u16_ptr: *mut u16,
) -> Option<&'a UserPhraseData> {
    let iter_ptr: *mut Box<dyn Iterator<Item = chewing::dictionary::Phrase>> =
        pgdata.phrase_iter.cast();
    let iter = unsafe { iter_ptr.as_mut().expect("nonnull iter") };
    if let Some(phrase) = iter.next() {
        pgdata.userphrase_data.userfreq = phrase.freq() as c_int;
        phrase.copy_to(&mut pgdata.userphrase_data.word_seq);
        return Some(&pgdata.userphrase_data);
    }
    None
}

#[no_mangle]
pub extern "C" fn UserGetPhraseEnd(pgdata: &mut ChewingData, _syllables_u16_ptr: *mut u16) {
    if !pgdata.phrase_iter.is_null() {
        let iter_ptr: *mut Box<dyn Iterator<Item = chewing::dictionary::Phrase>> =
            pgdata.phrase_iter.cast();
        let _ = unsafe { Box::from_raw(iter_ptr) };
        pgdata.phrase_iter = std::ptr::null_mut();
    }
}

const C_API_MAX_USER_PHRASE_LEN: usize = 11;

#[no_mangle]
pub unsafe extern "C" fn UserUpdatePhrase(
    pgdata: &mut ChewingData,
    syllables_u16_ptr: *mut u16,
    phrase_str_ptr: *mut c_char,
) -> u8 {
    let ue = pgdata.ue.as_mut().expect("nonnull ue");
    let syllables_u16 = unsafe { slice::from_raw_parts(syllables_u16_ptr, 50) };
    let syllables = syllables_u16
        .iter()
        .take_while(|&&syl_u16| syl_u16 != 0)
        .map(|&syl_u16| Syllable::try_from(syl_u16).unwrap())
        .collect::<Vec<_>>();
    let phrase_str = unsafe { CStr::from_ptr(phrase_str_ptr) }
        .to_str()
        .expect("Invalid UTF-8 str");
    if syllables.len() > C_API_MAX_USER_PHRASE_LEN {
        return UserUpdate::Fail as u8;
    }
    let phrases = ue.db.lookup_phrase(&syllables).collect::<Vec<_>>();
    if phrases.is_empty() {
        // FIXME provide max_freq, orig_freq
        ue.db
            .as_mut_dict()
            .unwrap()
            .insert(&syllables, (phrase_str, 1).into())
            .expect("SQL error");
        return UserUpdate::Insert as u8;
    }
    let phrase_freq = phrases
        .iter()
        .find(|p| p.as_str() == phrase_str)
        .map(|p| p.freq())
        .unwrap_or(1);
    let phrase = (phrase_str, phrase_freq).into();
    let max_freq = phrases.iter().map(|p| p.freq()).max().unwrap();
    let user_freq = ue.estimate.estimate(&phrase, phrase.freq(), max_freq);
    let time = ue.estimate.now().unwrap();
    ue.db
        .as_mut_dict()
        .unwrap()
        .update(&syllables, phrase, user_freq, time)
        .expect("SQL error");
    UserUpdate::Modify as u8
}

#[no_mangle]
pub unsafe extern "C" fn UserRemovePhrase(
    pgdata: &mut ChewingData,
    syllables_u16_ptr: *mut u16,
    phrase_str_ptr: *mut c_char,
) -> bool {
    let ue = pgdata.ue.as_mut().expect("nonnull ue");
    let syllables_u16 = unsafe { slice::from_raw_parts(syllables_u16_ptr, 50) };
    let syllables = syllables_u16
        .iter()
        .take_while(|&&syl_u16| syl_u16 != 0)
        .map(|&syl_u16| Syllable::try_from(syl_u16).unwrap())
        .collect::<Vec<_>>();
    let phrase_str = unsafe { CStr::from_ptr(phrase_str_ptr) }
        .to_str()
        .expect("Invalid UTF-8 str");
    let has_phrase = ue
        .db
        .lookup_phrase(&syllables)
        .any(|p| p.as_str() == phrase_str);
    ue.db
        .as_mut_dict()
        .unwrap()
        .remove(&syllables, phrase_str)
        .expect("SQL error");
    has_phrase
}

#[no_mangle]
pub extern "C" fn IncreaseLifeTime(pgdata: &mut ChewingData) {
    pgdata
        .ue
        .as_mut()
        .expect("nonnull ue")
        .estimate
        .tick()
        .expect("SQL error");
}

#[no_mangle]
pub extern "C" fn UserUpdatePhraseBegin(_pgdata: &mut ChewingData) {}

#[no_mangle]
pub extern "C" fn UserUpdatePhraseEnd(_pgdata: &mut ChewingData) {}

#[no_mangle]
pub extern "C" fn UserEnumeratePhrase(ue: &UserphraseDbAndEstimate) -> *mut c_void {
    Box::into_raw(Box::new(ue.db.entries().peekable()) as Box<Peekable<DictEntries>>).cast()
}

#[no_mangle]
pub unsafe extern "C" fn UserEnumerateHasNext(
    iter_ptr: *mut c_void,
    phrase_len_ptr: *mut c_uint,
    bopomofo_len: *mut c_uint,
) -> bool {
    let iter_ptr: *mut Peekable<DictEntries> = iter_ptr.cast();
    let iter = unsafe { iter_ptr.as_mut() }.expect("Null ptr");
    match iter.peek() {
        Some(entry) => {
            unsafe {
                phrase_len_ptr.write((entry.1.as_str().len() + 1) as u32);
                bopomofo_len.write(
                    (entry
                        .0
                        .iter()
                        .map(|syl| syl.to_string().len() + 1)
                        .sum::<usize>()
                        + 1) as u32,
                );
            }
            true
        }
        None => false,
    }
}

#[no_mangle]
pub unsafe extern "C" fn UserEnumerateGet(
    iter_ptr: *mut c_void,
    phrase_buf: *mut c_char,
    _phrase_len_ptr: *const c_uint,
    bopomofo_buf: *mut c_char,
    _bopomofo_len: *const c_uint,
) -> c_int {
    let iter_ptr: *mut Peekable<DictEntries> = iter_ptr.cast();
    let iter = unsafe { iter_ptr.as_mut() }.expect("Null ptr");
    match iter.next() {
        Some(entry) => {
            unsafe {
                let phrase_str = CString::new(entry.1.as_str()).unwrap();
                let phrase_str_bytes = phrase_str.as_bytes_with_nul();
                phrase_buf.copy_from(
                    phrase_str_bytes.as_ptr() as *const i8,
                    phrase_str_bytes.len(),
                );
                // phrase_len_ptr.write((entry.1.as_str().len() + 1) as u32);
                let bopomofo_str = CString::new(
                    entry
                        .0
                        .iter()
                        .map(|syl| syl.to_string())
                        .collect::<Vec<_>>()
                        .join(" "),
                )
                .unwrap();
                let bopomofo_str_bytes = bopomofo_str.as_bytes_with_nul();
                bopomofo_buf.copy_from(
                    bopomofo_str_bytes.as_ptr() as *const i8,
                    bopomofo_str_bytes.len(),
                );
                // bopomofo_len.write(bopomofo_str_bytes.len() as u32);
            }
            0
        }
        None => {
            let _ = unsafe { Box::from_raw(iter_ptr) };
            1
        }
    }
}
