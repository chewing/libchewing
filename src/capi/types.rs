use std::{ffi::c_int, iter::Peekable};

use crate::{
    capi::public::MAX_SELKEY,
    conversion::{ChewingEngine, Interval},
    dictionary::DictEntries,
    editor::{keyboard::AnyKeyboardLayout, syllable::KeyboardLayoutCompat, Editor},
};

/// cbindgen:prefix-with-name
/// cbindgen:enum-trailing-values=[TypeNum]
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(C)]
pub enum KB {
    Default,
    Hsu,
    Ibm,
    GinYieh,
    Et,
    Et26,
    Dvorak,
    DvorakHsu,
    DachenCp26,
    HanyuPinyin,
    ThlPinyin,
    Mps2Pinyin,
    Carpalx,
}

/// cbindgen:rename-all=None
pub struct ChewingContext {
    pub(crate) kb_compat: KeyboardLayoutCompat,
    pub(crate) keyboard: AnyKeyboardLayout,
    pub(crate) editor: Editor<ChewingEngine>,
    pub(crate) kbcompat_iter: Option<Peekable<Box<dyn Iterator<Item = KeyboardLayoutCompat>>>>,
    pub(crate) cand_iter: Option<Peekable<Box<dyn Iterator<Item = String>>>>,
    pub(crate) interval_iter: Option<Peekable<Box<dyn Iterator<Item = Interval>>>>,
    pub(crate) userphrase_iter: Option<Peekable<DictEntries>>,
    pub(crate) sel_keys: SelKeys,
}

#[repr(C)]
pub(crate) struct SelKeys(pub(crate) [c_int; MAX_SELKEY]);
