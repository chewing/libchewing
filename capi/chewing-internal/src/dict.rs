use std::{
    ffi::{CStr, CString},
    os::raw::{c_char, c_int},
    path::Path,
    rc::Rc,
    slice,
};

use chewing::dictionary::{
    Dictionary, LayeredDictionary, Phrases, SqliteDictionary, TrieDictionary,
};

use crate::types::ChewingData;

use super::types::Phrase;

pub struct TreeType<'a> {
    phrases: Vec<chewing::dictionary::Phrase<'a>>,
}

pub struct TreeIter<'a> {
    iter: Phrases<'a, 'a>,
}

#[no_mangle]
pub unsafe extern "C" fn InitDict(pgdata: &mut ChewingData, prefix: *const c_char) -> c_int {
    let prefix = unsafe { CStr::from_ptr(prefix) }
        .to_str()
        .expect("Invalid prefix string");
    let path: &Path = prefix.as_ref();

    let mut tsi_db_path = path.to_path_buf();
    tsi_db_path.push("tsi.dat");
    let tsi_db = if let Ok(db) = SqliteDictionary::open_read_only(&tsi_db_path) {
        Box::new(db) as Box<dyn Dictionary>
    } else if let Ok(db) = TrieDictionary::open(&tsi_db_path) {
        Box::new(db) as Box<dyn Dictionary>
    } else {
        panic!(
            "Unsupported db format for {}",
            tsi_db_path.to_string_lossy()
        );
    };

    let mut word_db_path = path.to_path_buf();
    word_db_path.push("word.dat");
    let word_db = if let Ok(db) = SqliteDictionary::open_read_only(&word_db_path) {
        Box::new(db) as Box<dyn Dictionary>
    } else if let Ok(db) = TrieDictionary::open(&word_db_path) {
        Box::new(db) as Box<dyn Dictionary>
    } else {
        panic!(
            "Unsupported db format for {}",
            word_db_path.to_string_lossy()
        );
    };

    let dict = Rc::new(LayeredDictionary::new(vec![word_db, tsi_db], vec![]));
    pgdata.dict = Rc::into_raw(dict);
    0
}

#[no_mangle]
pub extern "C" fn TerminateDict(pgdata: &mut ChewingData) {
    unsafe {
        if !pgdata.dict.is_null() {
            Rc::decrement_strong_count(pgdata.dict);
        }
    }
    pgdata.dict = std::ptr::null();
}

#[no_mangle]
pub extern "C" fn GetCharFirst(
    pgdata: &mut ChewingData,
    phrase: &mut Phrase,
    syllable_u16: u16,
) -> bool {
    let dict = unsafe { pgdata.dict.as_ref().expect("nonnull dict") };
    let syllable = syllable_u16
        .try_into()
        .expect("Unable to convert u16 to syllable");
    let mut iter = dict.lookup_word(syllable);
    if let Some(p) = iter.next() {
        let phrase_str = CString::new(p.as_str()).expect("Unable to convert to CString");
        let phrase_bytes = phrase_str.as_bytes_with_nul();
        phrase.freq = p.freq() as c_int;
        phrase.phrase[0..phrase_bytes.len()].copy_from_slice(unsafe {
            slice::from_raw_parts(phrase_bytes.as_ptr().cast(), phrase_bytes.len())
        });
        pgdata.phrase_iter = Box::into_raw(Box::new(TreeIter { iter })).cast();
        return true;
    }
    pgdata.phrase_iter = std::ptr::null_mut();
    false
}

#[no_mangle]
pub extern "C" fn GetPhraseFirst(
    pgdata: &mut ChewingData,
    phrase: &mut Phrase,
    tree_type: &TreeType,
) {
    let mut iter = Box::new(tree_type.phrases.clone().into_iter());
    pgdata.phrase_iter = std::ptr::null_mut();
    if let Some(p) = iter.next() {
        let phrase_str = CString::new(p.as_str()).expect("Unable to convert to CString");
        let phrase_bytes = phrase_str.as_bytes_with_nul();
        phrase.freq = p.freq() as c_int;
        phrase.phrase[0..phrase_bytes.len()].copy_from_slice(unsafe {
            slice::from_raw_parts(phrase_bytes.as_ptr().cast(), phrase_bytes.len())
        });
        pgdata.phrase_iter = Box::into_raw(Box::new(TreeIter { iter })).cast();
    }
}

#[no_mangle]
pub unsafe extern "C" fn TreeFindPhrase(
    pgdata: &mut ChewingData,
    begin: c_int,
    end: c_int,
    syllables_u16: *mut u16,
) -> Option<Box<TreeType>> {
    let dict = unsafe { pgdata.dict.as_ref().expect("nonnull dict") };
    let syllables_u16 = unsafe { slice::from_raw_parts(syllables_u16, 50) };
    let begin = begin as usize;
    let end = end as usize;
    let syllables = syllables_u16[begin..=end]
        .iter()
        .map(|&syl_u16| {
            syl_u16
                .try_into()
                .expect("Unable to convert u16 to syllable")
        })
        .collect::<Vec<_>>();
    let phrases = dict.lookup_phrase(syllables.as_slice()).collect::<Vec<_>>();
    if !phrases.is_empty() {
        let ptr = Some(Box::new(TreeType { phrases }));
        return ptr;
    }
    None
}

#[no_mangle]
pub extern "C" fn FreeTreePhrase(tree_type: Option<Box<TreeType>>) {
    drop(tree_type)
}

#[no_mangle]
pub extern "C" fn GetVocabNext(pgdata: &mut ChewingData, phrase: &mut Phrase) -> bool {
    let tree_iter_ptr: *mut TreeIter = pgdata.phrase_iter.cast();
    let tree_iter: &mut TreeIter = unsafe { tree_iter_ptr.as_mut().expect("nonnull iter") };
    if let Some(p) = tree_iter.iter.next() {
        let phrase_str = CString::new(p.as_str()).expect("Unable to convert to CString");
        let phrase_bytes = phrase_str.as_bytes_with_nul();
        phrase.freq = p.freq() as c_int;
        phrase.phrase[0..phrase_bytes.len()].copy_from_slice(unsafe {
            slice::from_raw_parts(phrase_bytes.as_ptr().cast(), phrase_bytes.len())
        });
        return true;
    }
    unsafe { Box::from_raw(tree_iter_ptr) };
    pgdata.phrase_iter = std::ptr::null_mut();
    false
}
