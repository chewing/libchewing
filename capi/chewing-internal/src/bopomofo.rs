use std::{ffi::CString, slice};

use chewing::editor::{
    keymap::{
        IdentityKeymap, KeyCode, KeyCodeFromQwerty, Keymap, RemappingKeymap, CARPALX, DVORAK,
        QWERTY,
    },
    layout::{
        DaiChien26, Et, Et26, GinYieh, Hsu, Ibm, KeyBehavior, KeyboardLayoutCompat, Pinyin,
        Standard,
    },
    SyllableEditor,
};
use libc::{c_char, c_int};

use super::{
    binding::HaninSymbolInput,
    types::{BopomofoData, ChewingData},
};

#[repr(C)]
pub struct SyllableEditorWithKeymap {
    kb_type: KeyboardLayoutCompat,
    keymap: Box<dyn Keymap>,
    editor: Box<dyn SyllableEditor>,
}

#[no_mangle]
pub extern "C" fn NewPhoneticEditor(
    kb_type: KeyboardLayoutCompat,
) -> Box<SyllableEditorWithKeymap> {
    use KeyboardLayoutCompat as KB;
    match kb_type {
        KB::Default => Box::new(SyllableEditorWithKeymap {
            kb_type,
            keymap: Box::new(IdentityKeymap::new(QWERTY)),
            editor: Box::new(Standard::new()),
        }),
        KB::Hsu => Box::new(SyllableEditorWithKeymap {
            kb_type,
            keymap: Box::new(IdentityKeymap::new(QWERTY)),
            editor: Box::new(Hsu::new()),
        }),
        KB::Ibm => Box::new(SyllableEditorWithKeymap {
            kb_type,
            keymap: Box::new(IdentityKeymap::new(QWERTY)),
            editor: Box::new(Ibm::new()),
        }),
        KB::GinYieh => Box::new(SyllableEditorWithKeymap {
            kb_type,
            keymap: Box::new(IdentityKeymap::new(QWERTY)),
            editor: Box::new(GinYieh::new()),
        }),
        KB::Et => Box::new(SyllableEditorWithKeymap {
            kb_type,
            keymap: Box::new(IdentityKeymap::new(QWERTY)),
            editor: Box::new(Et::new()),
        }),
        KB::Et26 => Box::new(SyllableEditorWithKeymap {
            kb_type,
            keymap: Box::new(IdentityKeymap::new(QWERTY)),
            editor: Box::new(Et26::new()),
        }),
        KB::Dvorak => Box::new(SyllableEditorWithKeymap {
            kb_type,
            keymap: Box::new(RemappingKeymap::new(DVORAK, QWERTY)),
            editor: Box::new(Standard::new()),
        }),
        KB::DvorakHsu => Box::new(SyllableEditorWithKeymap {
            kb_type,
            keymap: Box::new(RemappingKeymap::new(DVORAK, QWERTY)),
            editor: Box::new(Hsu::new()),
        }),
        KB::DachenCp26 => Box::new(SyllableEditorWithKeymap {
            kb_type,
            keymap: Box::new(IdentityKeymap::new(QWERTY)),
            editor: Box::new(DaiChien26::new()),
        }),
        KB::HanyuPinyin => Box::new(SyllableEditorWithKeymap {
            kb_type,
            keymap: Box::new(IdentityKeymap::new(QWERTY)),
            editor: Box::new(Pinyin::hanyu()),
        }),
        KB::ThlPinyin => Box::new(SyllableEditorWithKeymap {
            kb_type,
            keymap: Box::new(IdentityKeymap::new(QWERTY)),
            editor: Box::new(Pinyin::thl()),
        }),
        KB::Mps2Pinyin => Box::new(SyllableEditorWithKeymap {
            kb_type,
            keymap: Box::new(IdentityKeymap::new(QWERTY)),
            editor: Box::new(Pinyin::mps2()),
        }),
        KB::Carpalx => Box::new(SyllableEditorWithKeymap {
            kb_type,
            keymap: Box::new(RemappingKeymap::new(CARPALX, QWERTY)),
            editor: Box::new(Standard::new()),
        }),
    }
}

#[no_mangle]
pub extern "C" fn FreePhoneticEditor(editor_keymap: Option<Box<SyllableEditorWithKeymap>>) {
    drop(editor_keymap);
}

#[no_mangle]
pub extern "C" fn BopomofoPhoInput(pgdata: &mut ChewingData, key: i32) -> KeyBehavior {
    if key == b'`' as i32 {
        pgdata.b_select = 1;
        pgdata.choice_info.old_chi_symbol_cursor = pgdata.chi_symbol_cursor;
        unsafe { HaninSymbolInput((pgdata as *mut ChewingData).cast()) };
        return KeyBehavior::OpenSymbolTable;
    }

    let editor_keymap = pgdata.bopomofo_data.editor_with_keymap.as_mut();
    let key_code = match (key as u8).as_key_code() {
        Some(key_code) => key_code,
        None => return KeyBehavior::KeyError,
    };
    let key_event = editor_keymap.keymap.map_key(key_code);
    let result = editor_keymap.editor.key_press(key_event);
    let key_buf = editor_keymap.editor.read();

    if result == KeyBehavior::Commit {
        if key_buf.is_empty() {
            return if key_code == KeyCode::Space {
                KeyBehavior::KeyError
            } else {
                KeyBehavior::NoWord
            };
        }
        // FIXME make sure editors fills the tone
        // FIXME if dictionary doesn't have a word, return NO_WORD
    }

    result
}

#[no_mangle]
pub unsafe extern "C" fn BopomofoPhoInx(bopomofo_data: &BopomofoData, pho_inx: *mut i32) {
    let editor_keymap = bopomofo_data.editor_with_keymap.as_ref();
    let pho_inx = unsafe { slice::from_raw_parts_mut(pho_inx, 4) };
    let syllable = editor_keymap.editor.read();

    pho_inx[0] = match syllable.initial() {
        Some(b) => b.initial_index() as i32,
        None => 0,
    };
    pho_inx[1] = match syllable.medial() {
        Some(b) => b.medial_index() as i32,
        None => 0,
    };
    pho_inx[2] = match syllable.rime() {
        Some(b) => b.rime_index() as i32,
        None => 0,
    };
    pho_inx[3] = match syllable.tone() {
        Some(b) => b.tone_index() as i32,
        None => 0,
    };
}

#[no_mangle]
pub unsafe extern "C" fn BopomofoPhoInxAlt(bopomofo_data: &BopomofoData, pho_inx: *mut i32) {
    let editor_keymap = bopomofo_data.editor_with_keymap.as_ref();
    let pho_inx = unsafe { slice::from_raw_parts_mut(pho_inx, 4) };
    // FIXME
    let syllable = editor_keymap.editor.read();

    pho_inx[0] = match syllable.initial() {
        Some(b) => b.initial_index() as i32,
        None => 0,
    };
    pho_inx[1] = match syllable.medial() {
        Some(b) => b.medial_index() as i32,
        None => 0,
    };
    pho_inx[2] = match syllable.rime() {
        Some(b) => b.rime_index() as i32,
        None => 0,
    };
    pho_inx[3] = match syllable.tone() {
        Some(b) => b.tone_index() as i32,
        None => 0,
    };
}

#[no_mangle]
pub extern "C" fn BopomofoKeyseq(bopomofo_data: &BopomofoData, key_seq: *mut c_char) {
    let editor_keymap = bopomofo_data.editor_with_keymap.as_ref();
    let key_seq = unsafe { slice::from_raw_parts_mut(key_seq as *mut u8, 10) };
    if let Some(key_seq_str) = editor_keymap.editor.key_seq() {
        let key_seq_cstr = CString::new(key_seq_str).unwrap();
        let key_seq_bytes = key_seq_cstr.as_bytes_with_nul();
        key_seq[..key_seq_bytes.len()].copy_from_slice(key_seq_bytes);
    }
}

#[no_mangle]
pub extern "C" fn BopomofoSyllableIndex(bopomofo_data: &BopomofoData) -> u16 {
    let editor_keymap = bopomofo_data.editor_with_keymap.as_ref();
    let syllable = editor_keymap.editor.read();
    syllable.to_u16()
}

#[no_mangle]
pub extern "C" fn BopomofoSyllableIndexAlt(bopomofo_data: &BopomofoData) -> u16 {
    let editor_keymap = bopomofo_data.editor_with_keymap.as_ref();
    // FIXME
    let syllable = editor_keymap.editor.read();
    syllable.to_u16()
}

#[no_mangle]
pub extern "C" fn BopomofoRemoveLast(bopomofo_data: &mut BopomofoData) -> u16 {
    let editor_keymap = bopomofo_data.editor_with_keymap.as_mut();
    editor_keymap.editor.remove_last();
    0
}

#[no_mangle]
pub extern "C" fn BopomofoRemoveAll(bopomofo_data: &mut BopomofoData) -> u16 {
    let editor_keymap = bopomofo_data.editor_with_keymap.as_mut();
    editor_keymap.editor.clear();
    0
}

#[no_mangle]
pub extern "C" fn BopomofoKbType(bopomofo_data: &BopomofoData) -> c_int {
    let editor_keymap = bopomofo_data.editor_with_keymap.as_ref();
    editor_keymap.kb_type as c_int
}

#[no_mangle]
pub extern "C" fn BopomofoIsEntering(bopomofo_data: &BopomofoData) -> u16 {
    let editor_keymap = bopomofo_data.editor_with_keymap.as_ref();
    u16::from(!editor_keymap.editor.is_empty())
}
