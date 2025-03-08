use std::{ffi::c_int, fmt::Debug, iter::Peekable};

use chewing::{
    conversion::Interval,
    dictionary::Entries,
    editor::{keyboard::AnyKeyboardLayout, zhuyin_layout::KeyboardLayoutCompat, Editor},
};

/// Indicates chewing will translate keystrokes to Chinese characters.
pub const CHINESE_MODE: c_int = 1;
/// Indicates the input mode is translating keystrokes to symbols.
pub const SYMBOL_MODE: c_int = 0;
/// Indicates chewing will translate latin and puctuation characters to
/// double-with characters.
pub const FULLSHAPE_MODE: c_int = 1;
/// Indicates chewing will not translate latin and puctuation characters.
pub const HALFSHAPE_MODE: c_int = 0;
/// Use conversion engine that doesn't perform intelligent phrasing.
pub const SIMPLE_CONVERSION_ENGINE: c_int = 0;
/// Use the original Chewing intelligent phrasing.
pub const CHEWING_CONVERSION_ENGINE: c_int = 1;
/// Use original Chewing intelligent phrasing with fuzzy prefix search.
pub const FUZZY_CHEWING_CONVERSION_ENGINE: c_int = 2;
/// Indicates automatic user phrase learning is disabled.
pub const AUTOLEARN_DISABLED: usize = 1;
/// Indicates automatic user phrase learning is enabled.
pub const AUTOLEARN_ENABLED: usize = 0;
/// The minimal size of pre-edit buffer.
pub const MIN_CHI_SYMBOL_LEN: usize = 0;
/// The maximum size of pre-edit buffer.
pub const MAX_CHI_SYMBOL_LEN: usize = MAX_PHONE_SEQ_LEN - MAX_PHRASE_LEN;
/// The size of internal buffer for pre-edit buffer.
pub const MAX_PHONE_SEQ_LEN: usize = 50;
/// The maximum phrase size.
pub const MAX_PHRASE_LEN: usize = 11;

/// The number of minimum candidates that are selectable via shortcut keys.
pub const MIN_SELKEY: usize = 1;
/// The number of maximum candidates that are selectable via shortcut keys.
pub const MAX_SELKEY: usize = 10;

/// Log level.
pub const CHEWING_LOG_VERBOSE: usize = 1;
/// Log level.
pub const CHEWING_LOG_DEBUG: usize = 2;
/// Log level.
pub const CHEWING_LOG_INFO: usize = 3;
/// Log level.
pub const CHEWING_LOG_WARN: usize = 4;
/// Log level.
pub const CHEWING_LOG_ERROR: usize = 5;

/// Use "asdfjkl789" as selection key.
#[deprecated]
pub const HSU_SELKEY_TYPE1: usize = 1;
/// Use "asdfzxcv89" as selection key.
#[deprecated]
pub const HSU_SELKEY_TYPE2: usize = 2;

pub const KEYSTROKE_IGNORE: usize = 1;
pub const KEYSTROKE_COMMIT: usize = 2;
pub const KEYSTROKE_BELL: usize = 4;
pub const KEYSTROKE_ABSORB: usize = 8;

/// Configuration for chewing runtime features.
///
/// Deprecated, use chewing_set_ series of functions to set parameters instead.
///
/// cbindgen:rename-all=CamelCase
#[repr(C)]
#[deprecated]
#[derive(Debug)]
pub struct ChewingConfigData {
    pub cand_per_page: c_int,
    pub max_chi_symbol_len: c_int,
    pub sel_key: [c_int; MAX_SELKEY],
    pub b_add_phrase_forward: c_int,
    pub b_space_as_selection: c_int,
    pub b_esc_clean_all_buf: c_int,
    pub b_auto_shift_cur: c_int,
    pub b_easy_symbol_input: c_int,
    pub b_phrase_choice_rearward: c_int,
    pub hsu_sel_key_type: c_int,
}

/// Specifies the interval of a phrase segment in the pre-editng area
#[repr(C)]
#[derive(Debug)]
pub struct IntervalType {
    /// Starting position of certain interval
    pub from: c_int,
    /// Ending position of certain interval (exclusive)
    pub to: c_int,
}

/// Keyboard layout index.
///
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
    ColemakDhAnsi,
    ColemakDhOrth,
    Workman,
    Colemak,
}

/// Opaque context handle used for chewing APIs.
///
/// cbindgen:rename-all=None
pub struct ChewingContext {
    pub(crate) kb_compat: KeyboardLayoutCompat,
    pub(crate) keyboard: AnyKeyboardLayout,
    pub(crate) editor: Editor,
    pub(crate) kbcompat_iter: Option<Peekable<Box<dyn Iterator<Item = KeyboardLayoutCompat>>>>,
    pub(crate) cand_iter: Option<Peekable<Box<dyn Iterator<Item = String>>>>,
    pub(crate) interval_iter: Option<Peekable<Box<dyn Iterator<Item = Interval>>>>,
    pub(crate) userphrase_iter: Option<Peekable<Entries<'static>>>,
    pub(crate) sel_keys: SelKeys,
    pub(crate) commit_buf: [u8; 256],
    pub(crate) preedit_buf: [u8; 256],
    pub(crate) bopomofo_buf: [u8; 16],
    pub(crate) cand_buf: [u8; 256],
    pub(crate) aux_buf: [u8; 256],
    pub(crate) kbtype_buf: [u8; 32],
}

impl Debug for ChewingContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ChewingContext")
            .field("kb_compat", &self.kb_compat)
            .field("keyboard", &self.keyboard)
            .field("editor", &self.editor)
            .field("kbcompat_iter.is_some()", &self.kbcompat_iter.is_some())
            .field("cand_iter.is_some()", &self.cand_iter.is_some())
            .field("interval_iter.is_some()", &self.interval_iter.is_some())
            .field("userphrase_iter.is_some()", &self.userphrase_iter.is_some())
            .field("sel_keys", &self.sel_keys)
            .finish_non_exhaustive()
    }
}

#[repr(C)]
#[derive(Debug)]
pub(crate) struct SelKeys(pub(crate) [c_int; MAX_SELKEY]);
