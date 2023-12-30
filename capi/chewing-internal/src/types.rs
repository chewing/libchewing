use std::{ffi::c_int, iter::Peekable};

use chewing::{
    conversion::{ChewingEngine, Interval},
    dictionary::DictEntries,
    editor::{keyboard::AnyKeyboardLayout, syllable::KeyboardLayoutCompat, Editor},
};
use chewing_public::types::MAX_SELKEY;

pub const MAX_UTF8_SIZE: usize = 4;
pub const MAX_UTF8_BUF: usize = MAX_UTF8_SIZE + 1;
pub const BOPOMOFO_SIZE: usize = 4;
pub const MAX_BOPOMOFO_UTF8_BUF: usize = BOPOMOFO_SIZE * MAX_UTF8_SIZE + 1;
pub const PINYIN_SIZE: usize = 10;
pub const MAX_PHRASE_LEN: usize = 11;
pub const MAX_PHRASE_UTF8_BUF: usize = MAX_PHRASE_LEN * MAX_UTF8_SIZE + 1;
pub const MAX_PHONE_SEQ_LEN: usize = 50;
pub const MAX_PHONE_SEQ_BUF: usize = MAX_PHONE_SEQ_LEN + 1;
pub const MAX_PHONE_SEQ_UTF8_BUF: usize = MAX_PHONE_SEQ_LEN * MAX_UTF8_SIZE + 1;
pub const MIN_CHI_SYMBOL_LEN: usize = 0;
pub const MAX_CHI_SYMBOL_LEN: usize = MAX_PHONE_SEQ_LEN - MAX_PHRASE_LEN;
pub const MAX_INTERVAL: usize = (MAX_PHONE_SEQ_LEN + 1) * MAX_PHONE_SEQ_LEN / 2;
pub const MAX_CHOICE: usize = 567;
pub const MAX_CHOICE_BUF: usize = 50;
pub const EASY_SYMBOL_KEY_TAB_LEN: usize = 36;
pub const AUX_PREFIX_LEN: usize = 3;
pub const MAX_SHOW_MSG_BUF: usize = MAX_UTF8_SIZE * (MAX_PHRASE_LEN + AUX_PREFIX_LEN) + 1;

pub const N_HASH_BIT: usize = 14;
pub const HASH_TABLE_SIZE: usize = 1 << N_HASH_BIT;

pub const WORD_CHOICE: usize = 0;
pub const SYMBOL_CATEGORY_CHOICE: usize = 1;
pub const SYMBOL_CHOICE_INSERT: usize = 2;
pub const SYMBOL_CHOICE_UPDATE: usize = 3;

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
