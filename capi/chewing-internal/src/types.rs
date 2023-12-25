use std::{fmt::Debug, iter::Peekable, rc::Rc};

use chewing::{
    conversion::{ChewingConversionEngine, Interval},
    dictionary::{AnyDictionary, LayeredDictionary},
    editor::{keyboard::AnyKeyboardLayout, Editor},
};
use chewing_public::types::IntervalType;
use libc::{c_char, c_int, c_uint, c_void};

// use crate::userphrase::UserphraseDbAndEstimate;

// use super::bopomofo::SyllableEditorWithKeymap;

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

/// cbindgen:prefix-with-name
#[repr(C)]
pub enum BOPOMOFO {
    Ignore,
    Absorb,
    Commit,
    KeyError,
    Error,
    NoWord,
    OpenSymbolTable,
}

/// cbindgen:prefix-with-name
#[repr(u8)]
pub enum UserUpdate {
    Insert = 1,
    Modify = 2,
    Fail = 4,
}

#[repr(C)]
pub struct PhrasingOutput {
    pub disp_interval: [IntervalType; MAX_INTERVAL],
    pub n_disp_interval: c_int,
    pub n_num_cut: c_int,
}

// #[repr(C)]
// pub struct BopomofoData {
//     pub editor_with_keymap: Box<SyllableEditorWithKeymap>,
// }

#[repr(C)]
pub struct PinYinData {
    pub r#type: c_int,
    pub key_seq: [c_char; PINYIN_SIZE],
}

/// Information of available phrase or character choices.
#[repr(C)]
pub struct AvailInfo {
    /// All kinds of lengths of available phrases.
    pub avail: [AvailInfoAvail; MAX_PHRASE_LEN],
    /// Total number of available lengths.
    pub n_avail: c_int,
    /// The current choosing available length.
    pub current_avail: c_int,
}

#[repr(C)]
pub struct AvailInfoAvail {
    pub len: c_int,
    pub id: *const c_void,
}

#[repr(C)]
pub struct ChoiceInfo {
    /// Total page number.
    pub n_page: c_int,
    /// Current page number.
    pub page_no: c_int,
    /// Number of choices per page.
    pub n_choice_per_page: c_int,
    /// Store possible phrases for being chosen.
    pub total_choice_str: [[c_char; MAX_PHRASE_UTF8_BUF]; MAX_CHOICE],
    /// Number of phrases to choose.
    pub n_total_choice: c_int,
    pub old_chi_symbol_cursor: c_int,
    pub is_symbol: c_int,
}

#[repr(C)]
pub struct SymbolEntry {
    /// Total number of symbols in the category.
    ///
    /// If n_symbols = 0, the category is treat as a symbol,
    /// which is a zero-terminated utf-8 string.
    ///
    /// In that case, `symbols` is unused and isn't allocated at all.
    pub n_symbols: c_int,
    /// Category name of the symbols.
    pub category: [c_char; MAX_PHRASE_UTF8_BUF],
    pub symbols: [[c_char; MAX_UTF8_BUF]; 0],
}

#[repr(C)]
pub struct ChewingStaticData {
    pub n_symbol_entry: c_uint,
    pub symbol_table: *mut *mut SymbolEntry,
    pub g_easy_symbol_value: [*mut c_char; EASY_SYMBOL_KEY_TAB_LEN],
    pub g_easy_symbol_num: [c_int; EASY_SYMBOL_KEY_TAB_LEN],
}

#[derive(Debug)]
#[repr(C)]
pub enum Category {
    ChewingNone,
    ChewingChinese,
    ChewingSymbol,
}

#[derive(Debug)]
#[repr(C)]
pub struct PreeditBuf {
    pub category: Category,
    pub char_: [u8; MAX_UTF8_BUF],
}

#[repr(C)]
pub struct UserPhraseData {
    pub phone_seq: [u16; MAX_PHONE_SEQ_LEN],
    pub word_seq: [c_char; MAX_PHRASE_UTF8_BUF],
    pub userfreq: c_int,
    pub recent_time: c_int,
    pub origfreq: c_int,
    pub maxfreq: c_int,
}

pub struct ChewingData;

// #[repr(C)]
// pub struct ChewingData {
//     pub avail_info: AvailInfo,
//     pub choice_info: ChoiceInfo,
//     pub phr_out: PhrasingOutput,
//     pub bopomofo_data: BopomofoData,
//     pub config: ChewingConfigData,
//     pub b_auto_learn: c_int,
//     /// Current input buffer, content == 0 means Chinese code
//     pub preedit_buf: [PreeditBuf; MAX_PHONE_SEQ_LEN],
//     pub chi_symbol_cursor: c_int,
//     pub chi_symbol_buf_len: c_int,
//     pub point_start: c_int,
//     pub point_end: c_int,
//     pub b_show_msg: c_int,
//     pub show_msg: [c_char; MAX_SHOW_MSG_BUF],
//     pub show_msg_len: c_int,
//     pub phone_seq: [u16; MAX_PHONE_SEQ_LEN],
//     pub phone_seq_alt: [u16; MAX_PHONE_SEQ_LEN],
//     pub n_phone_seq: c_int,
//     pub select_str: [[c_char; MAX_PHRASE_UTF8_BUF]; MAX_PHONE_SEQ_LEN],
//     pub select_interval: [IntervalType; MAX_PHONE_SEQ_LEN],
//     pub n_select: c_int,
//     pub prefer_interval: [IntervalType; MAX_INTERVAL],
//     pub n_prefer: c_int,
//     pub b_user_arr_cnnct: [c_int; MAX_PHONE_SEQ_BUF],
//     pub b_user_arr_brkpt: [c_int; MAX_PHONE_SEQ_BUF],
//     pub b_arr_brkpt: [c_int; MAX_PHONE_SEQ_BUF],
//     pub b_symbol_arr_brkpt: [c_int; MAX_PHONE_SEQ_BUF],
//     pub b_chi_sym: c_int,
//     pub b_select: c_int,
//     pub b_first_key: c_int,
//     pub b_full_shape: c_int,
//     pub symbol_key_buf: [c_char; MAX_PHONE_SEQ_LEN],
//     pub userphrase_data: UserPhraseData,
//     pub static_data: ChewingStaticData,

//     pub logger: extern "C" fn(data: *mut c_void, level: c_int, fmt: *const c_char, ...),
//     pub logger_data: *mut c_void,

//     pub dict: *const LayeredDictionary,
//     pub ce: Option<Box<ChewingConversionEngine>>,
//     pub ue: Option<Box<UserphraseDbAndEstimate>>,
//     pub phrase_iter: *mut c_void,
//     pub phrase_enum_iter: *mut c_void,
// }

// #[repr(C)]
// pub struct ChewingOutput {
//     /// The content of edit buffer
//     pub preedit_buf: [c_char; MAX_PHONE_SEQ_UTF8_BUF],
//     /// The length of edit buffer
//     pub chi_symbol_buf_len: c_int,
//     /// The current position of the cursor
//     pub chi_symbol_cursor: c_long,
//     pub point_start: c_long,
//     pub point_end: c_long,
//     pub bopomofo_buf: [c_char; MAX_BOPOMOFO_UTF8_BUF],
//     pub disp_interval: [IntervalType; MAX_INTERVAL],
//     pub n_disp_interval: c_int,
//     pub disp_brkpt: [c_int; MAX_PHONE_SEQ_BUF],
//     pub commit_buf: [c_char; MAX_PHONE_SEQ_UTF8_BUF],
//     pub commit_buf_len: c_int,
//     pub pci: *mut ChoiceInfo,
//     pub b_chi_sym: c_int,
//     pub sel_key: [c_int; MAX_SELKEY],
//     pub keystroke_rtn: c_int,
// }

/// cbindgen:rename-all=None
pub struct ChewingContext {
    pub(crate) keyboard: AnyKeyboardLayout,
    pub(crate) editor: Editor<
        ChewingConversionEngine<Rc<LayeredDictionary<AnyDictionary, ()>>>,
        Rc<LayeredDictionary<AnyDictionary, ()>>,
    >,
    pub(crate) cand_iter: Option<Peekable<Box<dyn Iterator<Item = String>>>>,
    pub(crate) interval_iter: Option<Peekable<Box<dyn Iterator<Item = Interval>>>>,
}

#[repr(C)]
pub struct Phrase {
    pub phrase: [c_char; MAX_PHONE_SEQ_BUF],
    pub freq: c_int,
}
