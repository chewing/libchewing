use std::{
    cmp::min,
    collections::BTreeMap,
    ffi::{CStr, CString, c_char, c_int, c_uint, c_ushort, c_void},
    mem,
    ops::Not,
    ptr::{null, null_mut},
    slice,
    str::{self},
    sync::RwLock,
};

use chewing::{
    conversion::{ChewingEngine, FuzzyChewingEngine, Interval, SimpleEngine, Symbol},
    dictionary::{DEFAULT_DICT_NAMES, LookupStrategy},
    editor::{
        BasicEditor, CharacterForm, ConversionEngineKind, Editor, EditorKeyBehavior, LanguageMode,
        UserPhraseAddDirection,
        zhuyin_layout::{
            DaiChien26, Et, Et26, GinYieh, Hsu, Ibm, KeyboardLayoutCompat, Pinyin, Standard,
            SyllableEditor,
        },
    },
    input::{
        KeyboardEvent,
        keycode::*,
        keymap::{
            DVORAK_MAP, INVERTED_COLEMAK_DH_ANSI_MAP, INVERTED_COLEMAK_DH_ORTH_MAP,
            INVERTED_COLEMAK_MAP, INVERTED_DVORAK_MAP, INVERTED_WORKMAN_MAP, Keymap, QWERTY_MAP,
            map_ascii,
        },
        keysym::*,
    },
    zhuyin::Syllable,
};
use log::{debug, info};

use crate::{
    logger::init_scoped_logging,
    public::{
        CHEWING_CONVERSION_ENGINE, CHINESE_MODE, ChewingConfigData, ChewingContext, FULLSHAPE_MODE,
        FUZZY_CHEWING_CONVERSION_ENGINE, HALFSHAPE_MODE, IntervalType, MAX_SELKEY,
        SIMPLE_CONVERSION_ENGINE, SYMBOL_MODE, SelKeys,
    },
};

const TRUE: c_int = 1;
const FALSE: c_int = 0;
const OK: c_int = 0;
const ERROR: c_int = -1;

enum Owned {
    CString,
    CUShortSlice(usize),
    CIntSlice(usize),
}

static OWNED: RwLock<BTreeMap<usize, Owned>> = RwLock::new(BTreeMap::new());

fn owned_into_raw<T>(owned: Owned, ptr: *mut T) -> *mut T {
    match OWNED.write() {
        Ok(mut map) => {
            map.insert(ptr as usize, owned);
            ptr
        }
        Err(_) => null_mut(),
    }
}

static EMPTY_STRING_BUFFER: [u8; 1] = [0; 1];

fn copy_cstr(buf: &mut [u8], buffer: &str) -> *const c_char {
    let n = min(buf.len(), buffer.len());
    buf.fill(0);
    buf[..n].copy_from_slice(&buffer.as_bytes()[..n]);
    buf.as_ptr().cast()
}

fn global_empty_cstr() -> *const c_char {
    EMPTY_STRING_BUFFER.as_ptr().cast()
}

unsafe fn slice_from_ptr_with_nul<'a>(ptr: *const c_char) -> Option<&'a [c_char]> {
    if ptr.is_null() {
        return None;
    }
    let mut len = 0;
    let mut needle = ptr;
    while unsafe { needle.read() } != 0 {
        needle = unsafe { needle.add(1) };
        len += 1;
    }
    Some(unsafe { slice::from_raw_parts(ptr, len) })
}

unsafe fn str_from_ptr_with_nul<'a>(ptr: *const c_char) -> Option<&'a str> {
    unsafe { slice_from_ptr_with_nul(ptr) }
        .and_then(|data| str::from_utf8(unsafe { mem::transmute::<&[c_char], &[u8]>(data) }).ok())
}

/// Creates a new instance of the Chewing IM.
///
/// The return value is a pointer to the new Chewing IM instance.
///
/// See also the [chewing_new2], and [chewing_delete] functions.
#[unsafe(no_mangle)]
pub extern "C" fn chewing_new() -> *mut ChewingContext {
    unsafe { chewing_new2(null(), null(), None, null_mut()) }
}

/// Creates a new instance of the Chewing IM.
///
/// The `syspath` is the directory path to system dictionary. The `userpath`
/// is file path to user dictionary. User shall have enough permission to
/// update this file. The logger and loggerdata is logger function and its
/// data.
///
/// All parameters will be default if set to NULL.
///
/// The return value is a pointer to the new Chewing IM instance. See also
/// the [chewing_new], [chewing_delete] function.
///
/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_new2(
    syspath: *const c_char,
    userpath: *const c_char,
    logger: Option<unsafe extern "C" fn(data: *mut c_void, level: c_int, fmt: *const c_char, ...)>,
    loggerdata: *mut c_void,
) -> *mut ChewingContext {
    unsafe {
        chewing_new3(
            syspath,
            userpath,
            c"word.dat,tsi.dat,chewing.dat".as_ptr(),
            logger,
            loggerdata,
        )
    }
}

/// Creates a new instance of the Chewing IM.
///
/// The `syspath` is the directory path to system dictionary. The `userpath`
/// is file path to user dictionary. User shall have enough permission to
/// update this file. The `enabled_dicts` is a comma separated list of
/// dictionary file names.
///
/// The logger and loggerdata is logger function and its data.
///
/// All parameters will be default if set to NULL.
///
/// The return value is a pointer to the new Chewing IM instance. See also
/// the [chewing_new], [chewing_delete] function.
///
/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_new3(
    syspath: *const c_char,
    userpath: *const c_char,
    enabled_dicts: *const c_char,
    logger_fn: Option<
        unsafe extern "C" fn(data: *mut c_void, level: c_int, fmt: *const c_char, ...),
    >,
    logger_data: *mut c_void,
) -> *mut ChewingContext {
    let _ = crate::logger::init();
    let _logger_guard = init_scoped_logging(logger_fn, logger_data);
    let mut dict_names: Vec<String> = DEFAULT_DICT_NAMES.iter().map(|&n| n.to_owned()).collect();
    if !enabled_dicts.is_null() {
        if let Ok(enabled_dicts) = unsafe { CStr::from_ptr(enabled_dicts).to_str() } {
            dict_names = enabled_dicts
                .split(",")
                .map(|n| n.trim().to_owned())
                .collect();
        }
    }
    let syspath = if syspath.is_null() {
        None
    } else {
        unsafe { CStr::from_ptr(syspath).to_str() }
            .ok()
            .map(|p| p.to_owned())
    };
    let userpath = if userpath.is_null() {
        None
    } else {
        unsafe { CStr::from_ptr(userpath).to_str() }
            .ok()
            .map(|p| p.to_owned())
    };
    let kb_compat = KeyboardLayoutCompat::Default;
    let editor = Editor::chewing(syspath, userpath, &dict_names);
    let context = Box::new(ChewingContext {
        kb_compat,
        keymap: &QWERTY_MAP,
        editor,
        kbcompat_iter: None,
        cand_iter: None,
        interval_iter: None,
        userphrase_iter: None,
        sel_keys: SelKeys([
            b'1' as i32,
            b'2' as i32,
            b'3' as i32,
            b'4' as i32,
            b'5' as i32,
            b'6' as i32,
            b'7' as i32,
            b'8' as i32,
            b'9' as i32,
            b'0' as i32,
        ]),
        commit_buf: [0; 256],
        preedit_buf: [0; 256],
        bopomofo_buf: [0; 16],
        cand_buf: [0; 256],
        aux_buf: [0; 256],
        kbtype_buf: [0; 32],
        logger_fn,
        logger_data,
    });
    let ptr = Box::into_raw(context);
    info!("Initialized context {ptr:?}");
    ptr
}

/// Returns the default comma separated dictionary names.
///
/// This function should be used with [`chewing_new3`].
///
/// The return value is a const pointer to a character string. The pointer
/// don't need to be freed.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_get_defaultDictionaryNames() -> *const c_char {
    c"word.dat,tsi.dat,chewing.dat".as_ptr()
}

/// Releases the resources used by the given Chewing IM instance.
///
/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_delete(ctx: *mut ChewingContext) {
    if !ctx.is_null() {
        info!("Destroying context {ctx:?}");
        drop(unsafe { Box::from_raw(ctx) })
    }
}

/// Releases the memory allocated by the Chewing IM and returned to the
/// caller.
///
/// There are functions returning pointers of strings or other data
/// structures that are allocated on the heap. These memory must be freed to
/// avoid memory leak. To avoid memory allocator mismatch between the
/// library and the caller, use this function to free the resources.
///
/// Do nothing if ptr is NULL.
///
/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_free(ptr: *mut c_void) {
    if !ptr.is_null() {
        if let Ok(map) = OWNED.write() {
            if let Some(owned) = map.get(&(ptr as usize)) {
                match owned {
                    Owned::CString => drop(unsafe { CString::from_raw(ptr.cast()) }),
                    Owned::CUShortSlice(len) => {
                        drop(unsafe { Vec::<c_ushort>::from_raw_parts(ptr.cast(), *len, *len) })
                    }
                    Owned::CIntSlice(len) => {
                        drop(unsafe { Vec::<c_int>::from_raw_parts(ptr.cast(), *len, *len) })
                    }
                }
            }
        };
    }
}

macro_rules! as_mut_or_return {
    ($ctx:expr) => {
        match unsafe { $ctx.as_mut() } {
            Some(ctx) => ctx,
            None => return,
        }
    };
    ($ctx:expr, $ret:expr) => {
        match unsafe { $ctx.as_mut() } {
            Some(ctx) => ctx,
            None => return $ret,
        }
    };
}

macro_rules! as_ref_or_return {
    ($ctx:expr, $ret:expr) => {
        match unsafe { $ctx.as_ref() } {
            Some(ctx) => ctx,
            None => return $ret,
        }
    };
}

/// Reset the context but keep all settings.
///
/// All preedit buffers are reset to empty.
///
/// The return value is 0 on success and -1 on failure.
///
/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_Reset(ctx: *mut ChewingContext) -> c_int {
    let ctx = as_mut_or_return!(ctx, ERROR);
    let _logger_guard = init_scoped_logging(ctx.logger_fn, ctx.logger_data);
    ctx.editor.clear();
    OK
}

/// Acknowledge the commit buffer and aux output buffer.
///
/// Chewing automatically acknowledges and clear the output buffers after
/// processing new input events.
///
/// After handling the ephemeral output buffer like the commit buffer and
/// the aux output buffer, IM wrappers can proactively acknowledge and clear
/// the buffers. This can be used so that IM wrappers don't have to remember
/// whether an output has been handled or not.
///
/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_ack(ctx: *mut ChewingContext) -> c_int {
    let ctx = as_mut_or_return!(ctx, ERROR);
    let _logger_guard = init_scoped_logging(ctx.logger_fn, ctx.logger_data);
    ctx.editor.ack();
    OK
}

/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_config_has_option(
    ctx: *const ChewingContext,
    name: *const c_char,
) -> c_int {
    let ctx = as_ref_or_return!(ctx, ERROR);
    let _logger_guard = init_scoped_logging(ctx.logger_fn, ctx.logger_data);
    let cstr = unsafe { CStr::from_ptr(name) };
    let name = cstr.to_string_lossy();

    let ret = matches!(
        name.as_ref(),
        "chewing.user_phrase_add_direction"
            | "chewing.disable_auto_learn_phrase"
            | "chewing.auto_shift_cursor"
            | "chewing.candidates_per_page"
            | "chewing.language_mode"
            | "chewing.easy_symbol_input"
            | "chewing.esc_clear_all_buffer"
            | "chewing.keyboard_type"
            | "chewing.auto_commit_threshold"
            | "chewing.phrase_choice_rearward"
            | "chewing.selection_keys"
            | "chewing.character_form"
            | "chewing.space_is_select_key"
            | "chewing.conversion_engine"
            | "chewing.enable_fullwidth_toggle_key"
            | "chewing.sort_candidates_by_frequency"
    );

    ret as c_int
}

/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_config_get_int(
    ctx: *const ChewingContext,
    name: *const c_char,
) -> c_int {
    let ctx = as_ref_or_return!(ctx, ERROR);
    let _logger_guard = init_scoped_logging(ctx.logger_fn, ctx.logger_data);

    if unsafe { chewing_config_has_option(ctx, name) } != 1 {
        return ERROR;
    }

    let cstr = unsafe { CStr::from_ptr(name) };
    let name = cstr.to_string_lossy();

    let option = &ctx.editor.editor_options();

    match name.as_ref() {
        "chewing.user_phrase_add_direction" => match option.user_phrase_add_dir {
            UserPhraseAddDirection::Forward => 0,
            UserPhraseAddDirection::Backward => 1,
        },
        "chewing.disable_auto_learn_phrase" => option.disable_auto_learn_phrase as c_int,
        "chewing.auto_shift_cursor" => option.auto_shift_cursor as c_int,
        "chewing.candidates_per_page" => option.candidates_per_page as c_int,
        "chewing.language_mode" => match option.language_mode {
            LanguageMode::Chinese => CHINESE_MODE,
            LanguageMode::English => SYMBOL_MODE,
        },
        "chewing.easy_symbol_input" => option.easy_symbol_input as c_int,
        "chewing.esc_clear_all_buffer" => option.esc_clear_all_buffer as c_int,
        "chewing.auto_commit_threshold" => option.auto_commit_threshold as c_int,
        "chewing.phrase_choice_rearward" => option.phrase_choice_rearward as c_int,
        "chewing.character_form" => match option.character_form {
            CharacterForm::Halfwidth => HALFSHAPE_MODE,
            CharacterForm::Fullwidth => FULLSHAPE_MODE,
        },
        "chewing.space_is_select_key" => option.space_is_select_key as c_int,
        "chewing.conversion_engine" => match option.conversion_engine {
            ConversionEngineKind::SimpleEngine => SIMPLE_CONVERSION_ENGINE,
            ConversionEngineKind::ChewingEngine => CHEWING_CONVERSION_ENGINE,
            ConversionEngineKind::FuzzyChewingEngine => FUZZY_CHEWING_CONVERSION_ENGINE,
        },
        "chewing.enable_fullwidth_toggle_key" => option.enable_fullwidth_toggle_key as c_int,
        "chewing.sort_candidates_by_frequency" => option.sort_candidates_by_frequency as c_int,
        _ => ERROR,
    }
}

/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_config_set_int(
    ctx: *mut ChewingContext,
    name: *const c_char,
    value: c_int,
) -> c_int {
    let ctx = as_mut_or_return!(ctx, ERROR);
    let _logger_guard = init_scoped_logging(ctx.logger_fn, ctx.logger_data);

    if unsafe { chewing_config_has_option(ctx, name) } != 1 {
        return ERROR;
    }

    let cstr = unsafe { CStr::from_ptr(name) };
    let name = cstr.to_string_lossy();

    if value < 0 {
        return ERROR;
    }

    let mut options = ctx.editor.editor_options();

    macro_rules! ensure_bool {
        ($expr:expr) => {
            match $expr {
                0 | 1 => {}
                _ => return ERROR,
            };
        };
    }

    match name.as_ref() {
        "chewing.user_phrase_add_direction" => match value {
            0 => options.user_phrase_add_dir = UserPhraseAddDirection::Forward,
            1 => options.user_phrase_add_dir = UserPhraseAddDirection::Backward,
            _ => return ERROR,
        },
        "chewing.disable_auto_learn_phrase" => {
            ensure_bool!(value);
            options.disable_auto_learn_phrase = value > 0;
        }
        "chewing.auto_shift_cursor" => {
            ensure_bool!(value);
            options.auto_shift_cursor = value > 0;
        }
        "chewing.candidates_per_page" => {
            if value == 0 || value > 10 {
                return ERROR;
            }
            options.candidates_per_page = value as usize
        }
        "chewing.language_mode" => {
            options.language_mode = match value {
                CHINESE_MODE => LanguageMode::Chinese,
                SYMBOL_MODE => LanguageMode::English,
                _ => return ERROR,
            }
        }
        "chewing.easy_symbol_input" => {
            ensure_bool!(value);
            options.easy_symbol_input = value > 0;
        }
        "chewing.esc_clear_all_buffer" => {
            ensure_bool!(value);
            options.esc_clear_all_buffer = value > 0;
        }
        "chewing.auto_commit_threshold" => {
            if !(0..=39).contains(&value) {
                return ERROR;
            }
            options.auto_commit_threshold = value as usize;
        }
        "chewing.phrase_choice_rearward" => {
            ensure_bool!(value);
            options.phrase_choice_rearward = value > 0;
        }
        "chewing.character_form" => {
            options.character_form = match value {
                HALFSHAPE_MODE => CharacterForm::Halfwidth,
                FULLSHAPE_MODE => CharacterForm::Fullwidth,
                _ => return ERROR,
            }
        }
        "chewing.space_is_select_key" => {
            ensure_bool!(value);
            options.space_is_select_key = value > 0;
        }
        "chewing.conversion_engine" => {
            options.conversion_engine = match value {
                SIMPLE_CONVERSION_ENGINE => {
                    ctx.editor
                        .set_conversion_engine(Box::new(SimpleEngine::new()));
                    options.lookup_strategy = LookupStrategy::Standard;
                    ConversionEngineKind::SimpleEngine
                }
                CHEWING_CONVERSION_ENGINE => {
                    ctx.editor
                        .set_conversion_engine(Box::new(ChewingEngine::new()));
                    options.lookup_strategy = LookupStrategy::Standard;
                    ConversionEngineKind::ChewingEngine
                }
                FUZZY_CHEWING_CONVERSION_ENGINE => {
                    ctx.editor
                        .set_conversion_engine(Box::new(FuzzyChewingEngine::new()));
                    options.lookup_strategy = LookupStrategy::FuzzyPartialPrefix;
                    ConversionEngineKind::FuzzyChewingEngine
                }
                _ => return ERROR,
            }
        }
        "chewing.enable_fullwidth_toggle_key" => {
            ensure_bool!(value);
            options.enable_fullwidth_toggle_key = value > 0;
        }
        "chewing.sort_candidates_by_frequency" => {
            ensure_bool!(value);
            options.sort_candidates_by_frequency = value > 0;
        }
        _ => return ERROR,
    };

    ctx.editor.set_editor_options(|opt| *opt = options);

    OK
}

/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_config_get_str(
    ctx: *const ChewingContext,
    name: *const c_char,
    value: *mut *mut c_char,
) -> c_int {
    let ctx = as_ref_or_return!(ctx, ERROR);
    let _logger_guard = init_scoped_logging(ctx.logger_fn, ctx.logger_data);

    if unsafe { chewing_config_has_option(ctx, name) } != 1 {
        return ERROR;
    }

    let cstr = unsafe { CStr::from_ptr(name) };
    let name = cstr.to_string_lossy();

    let _option = &ctx.editor.editor_options();

    let string = match name.as_ref() {
        "chewing.keyboard_type" => ctx.kb_compat.to_string(),
        "chewing.selection_keys" => ctx
            .sel_keys
            .0
            .iter()
            .map(|&key| char::from(key as u8))
            .collect(),
        _ => return ERROR,
    };

    match unsafe { value.as_mut() } {
        Some(place) => {
            *place = owned_into_raw(
                Owned::CString,
                CString::new(string)
                    .expect("should have valid string")
                    .into_raw(),
            )
        }
        None => return ERROR,
    }

    OK
}

/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_config_set_str(
    ctx: *mut ChewingContext,
    name: *const c_char,
    value: *const c_char,
) -> c_int {
    let ctx = as_mut_or_return!(ctx, ERROR);
    let _logger_guard = init_scoped_logging(ctx.logger_fn, ctx.logger_data);

    if unsafe { chewing_config_has_option(ctx, name) } != 1 {
        return ERROR;
    }

    let cstr = unsafe { CStr::from_ptr(name) };
    let name = cstr.to_string_lossy();
    let cstr = unsafe { CStr::from_ptr(value) };
    let value = cstr.to_string_lossy();

    let _option = &mut ctx.editor.editor_options();

    match name.as_ref() {
        "chewing.keyboard_type" => {
            use KeyboardLayoutCompat as KB;
            let kb_compat = match value.parse() {
                Ok(kbtype) => kbtype,
                Err(_) => return ERROR,
            };
            let (keymap, syl): (&'static Keymap, Box<dyn SyllableEditor>) = match kb_compat {
                KB::Default => (&QWERTY_MAP, Box::new(Standard::new())),
                KB::Hsu => (&QWERTY_MAP, Box::new(Hsu::new())),
                KB::Ibm => (&QWERTY_MAP, Box::new(Ibm::new())),
                KB::GinYieh => (&QWERTY_MAP, Box::new(GinYieh::new())),
                KB::Et => (&QWERTY_MAP, Box::new(Et::new())),
                KB::Et26 => (&QWERTY_MAP, Box::new(Et26::new())),
                KB::Dvorak => (&INVERTED_DVORAK_MAP, Box::new(Standard::new())),
                KB::DvorakHsu => (&DVORAK_MAP, Box::new(Hsu::new())),
                KB::DachenCp26 => (&QWERTY_MAP, Box::new(DaiChien26::new())),
                KB::HanyuPinyin => (&QWERTY_MAP, Box::new(Pinyin::hanyu())),
                KB::ThlPinyin => (&QWERTY_MAP, Box::new(Pinyin::thl())),
                KB::Mps2Pinyin => (&QWERTY_MAP, Box::new(Pinyin::mps2())),
                KB::Carpalx => (&QWERTY_MAP, Box::new(Standard::new())),
                KB::Colemak => (&INVERTED_COLEMAK_MAP, Box::new(Standard::new())),
                KB::ColemakDhAnsi => (&INVERTED_COLEMAK_DH_ANSI_MAP, Box::new(Standard::new())),
                KB::ColemakDhOrth => (&INVERTED_COLEMAK_DH_ORTH_MAP, Box::new(Standard::new())),
                KB::Workman => (&INVERTED_WORKMAN_MAP, Box::new(Standard::new())),
            };
            ctx.kb_compat = kb_compat;
            ctx.keymap = keymap;
            ctx.editor.set_syllable_editor(syl);
        }
        "chewing.selection_keys" => {
            if value.len() != 10 {
                return ERROR;
            }
            let mut sel_keys = [0_i32; MAX_SELKEY];
            value
                .chars()
                .enumerate()
                .for_each(|(i, key)| sel_keys[i] = key as i32);
            ctx.sel_keys = SelKeys(sel_keys)
        }
        _ => return ERROR,
    };

    OK
}

/// Sets the current keyboard layout for ctx.
///
/// The kbtype argument must be a value defined in [KB][super::public::KB].
///
/// The return value is 0 on success and -1 on failure. The keyboard type
/// will set to KB_DEFAULT if return value is -1.
///
/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_set_KBType(ctx: *mut ChewingContext, kbtype: c_int) -> c_int {
    let ctx = as_mut_or_return!(ctx, ERROR);
    let _logger_guard = init_scoped_logging(ctx.logger_fn, ctx.logger_data);

    use KeyboardLayoutCompat as KB;
    let kb_compat = match KB::try_from(kbtype as u8) {
        Ok(kb) => kb,
        Err(()) => KB::Default,
    };
    let (keymap, syl): (&'static Keymap, Box<dyn SyllableEditor>) = match kb_compat {
        KB::Default => (&QWERTY_MAP, Box::new(Standard::new())),
        KB::Hsu => (&QWERTY_MAP, Box::new(Hsu::new())),
        KB::Ibm => (&QWERTY_MAP, Box::new(Ibm::new())),
        KB::GinYieh => (&QWERTY_MAP, Box::new(GinYieh::new())),
        KB::Et => (&QWERTY_MAP, Box::new(Et::new())),
        KB::Et26 => (&QWERTY_MAP, Box::new(Et26::new())),
        KB::Dvorak => (&INVERTED_DVORAK_MAP, Box::new(Standard::new())),
        KB::DvorakHsu => (&DVORAK_MAP, Box::new(Hsu::new())),
        KB::DachenCp26 => (&QWERTY_MAP, Box::new(DaiChien26::new())),
        KB::HanyuPinyin => (&QWERTY_MAP, Box::new(Pinyin::hanyu())),
        KB::ThlPinyin => (&QWERTY_MAP, Box::new(Pinyin::thl())),
        KB::Mps2Pinyin => (&QWERTY_MAP, Box::new(Pinyin::mps2())),
        KB::Carpalx => (&QWERTY_MAP, Box::new(Standard::new())),
        KB::Colemak => (&INVERTED_COLEMAK_MAP, Box::new(Standard::new())),
        KB::ColemakDhAnsi => (&INVERTED_COLEMAK_DH_ANSI_MAP, Box::new(Standard::new())),
        KB::ColemakDhOrth => (&INVERTED_COLEMAK_DH_ORTH_MAP, Box::new(Standard::new())),
        KB::Workman => (&INVERTED_WORKMAN_MAP, Box::new(Standard::new())),
    };
    ctx.kb_compat = kb_compat;
    ctx.keymap = keymap;
    ctx.editor.set_syllable_editor(syl);
    if kb_compat == KB::Default && kb_compat as c_int != kbtype {
        -1
    } else {
        0
    }
}

/// Returns the current keyboard layout index for ctx.
///
/// The return value is the layout index defined in [KB][super::public::KB].
///
/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_get_KBType(ctx: *const ChewingContext) -> c_int {
    let ctx = as_ref_or_return!(ctx, ERROR);
    let _logger_guard = init_scoped_logging(ctx.logger_fn, ctx.logger_data);

    ctx.kb_compat as c_int
}

/// Returns the the current layout name string of ctx.
///
/// The return value is the name of the current layout, see also function
/// [chewing_KBStr2Num].
///
/// The returned pointer must be freed by
/// [chewing_free][super::setup::chewing_free].
///
/// # Failures
///
/// This function returns NULL when memory allocation fails.
///
/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_get_KBString(ctx: *const ChewingContext) -> *mut c_char {
    let ctx = as_ref_or_return!(
        ctx,
        owned_into_raw(Owned::CString, CString::default().into_raw())
    );
    let _logger_guard = init_scoped_logging(ctx.logger_fn, ctx.logger_data);

    let kb_string = ctx.kb_compat.to_string();
    owned_into_raw(
        Owned::CString,
        CString::new(kb_string)
            .expect("should have valid kb_string")
            .into_raw(),
    )
}

/// Converts the keyboard layout name from string to corresponding layout
/// index.
///
/// If the string does not match any layout, this function returns
/// KB_DEFAULT.
///
/// The string str might be one of the following layouts:
/// * KB_DEFAULT
/// * KB_HSU
/// * KB_IBM
/// * KB_GIN_YIEH
/// * KB_ET
/// * KB_ET26
/// * KB_DVORAK
/// * KB_DVORAK_HSU
/// * KB_DVORAK_CP26
/// * KB_HANYU_PINYIN
/// * KB_THL_PINYIN
/// * KB_MPS2_PINYIN
/// * KB_CARPALX
/// * KB_COLEMAK
/// * KB_COLEMAK_DH_ANSI
/// * KB_COLEMAK_DH_ORTH
/// * KB_WORKMAN
///
/// See also [chewing_kbtype_Enumerate] for getting the list of supported
/// layouts programmatically.
///
/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_KBStr2Num(str: *const c_char) -> c_int {
    let cstr = unsafe { CStr::from_ptr(str) };
    let utf8str = cstr.to_string_lossy();
    let layout: KeyboardLayoutCompat = utf8str.parse().unwrap_or(KeyboardLayoutCompat::Default);
    layout as c_int
}

/// Sets the input mode to Chinese or English.
///
/// The *mode* argument is one of the [CHINESE_MODE] and [SYMBOL_MODE]
/// constants.
///
/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_set_ChiEngMode(ctx: *mut ChewingContext, mode: c_int) {
    unsafe { chewing_config_set_int(ctx, c"chewing.language_mode".as_ptr().cast(), mode) };
}

/// Returns the current Chinese/English mode setting.
///
/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_get_ChiEngMode(ctx: *const ChewingContext) -> c_int {
    unsafe { chewing_config_get_int(ctx, c"chewing.language_mode".as_ptr().cast()) }
}

/// Sets the current punctuation input mode.
///
/// The *mode* argument is one of the [FULLSHAPE_MODE] and [HALFSHAPE_MODE]
/// constants.
///
/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_set_ShapeMode(ctx: *mut ChewingContext, mode: c_int) {
    unsafe { chewing_config_set_int(ctx, c"chewing.character_form".as_ptr().cast(), mode) };
}

/// Returns the current punctuation mode.
///
/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_get_ShapeMode(ctx: *const ChewingContext) -> c_int {
    unsafe { chewing_config_get_int(ctx, c"chewing.character_form".as_ptr().cast()) }
}

/// Sets the number of candidates returned per page.
///
/// The setting is ignored if *n* is not between [MIN_SELKEY][super::public::MIN_SELKEY] and
/// [MAX_SELKEY][super::public::MAX_SELKEY] inclusive.
///
/// The default value is MAX_SELKEY.
///
/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_set_candPerPage(ctx: *mut ChewingContext, n: c_int) {
    unsafe { chewing_config_set_int(ctx, c"chewing.candidates_per_page".as_ptr().cast(), n) };
}

/// Gets the number of candidates returned per page.
///
/// The default value is MAX_SELKEY.
///
/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_get_candPerPage(ctx: *const ChewingContext) -> c_int {
    unsafe { chewing_config_get_int(ctx, c"chewing.candidates_per_page".as_ptr().cast()) }
}

/// Sets the maximum number of the Chinese characters allowed in the
/// pre-edit buffer.
///
/// If the pre-edit string is longer than this number then the leading part
/// will be committed automatically. The range of n shall between
/// [MIN_CHI_SYMBOL_LEN][super::public::MIN_CHI_SYMBOL_LEN] and [MAX_CHI_SYMBOL_LEN][super::public::MAX_CHI_SYMBOL_LEN].
///
/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_set_maxChiSymbolLen(ctx: *mut ChewingContext, n: c_int) {
    unsafe { chewing_config_set_int(ctx, c"chewing.auto_commit_threshold".as_ptr().cast(), n) };
}

/// Returns the maximum number of the Chinese characters allowed in the
/// pre-edit buffer.
///
/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_get_maxChiSymbolLen(ctx: *const ChewingContext) -> c_int {
    unsafe { chewing_config_get_int(ctx, c"chewing.auto_commit_threshold".as_ptr().cast()) }
}

/// Sets the key codes for candidate selection.
///
/// *selkeys* is an ASCII code integer array of length [MAX_SELKEY]. The
/// second argument is unused.
///
/// The default selection key is `1234567890`.
///
/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_set_selKey(
    ctx: *mut ChewingContext,
    sel_keys: *const c_int,
    len: c_int,
) {
    let ctx = as_mut_or_return!(ctx);
    let _logger_guard = init_scoped_logging(ctx.logger_fn, ctx.logger_data);

    if sel_keys.is_null() || len != 10 {
        return;
    }

    let sel_keys = unsafe { slice::from_raw_parts(sel_keys, len as usize) };
    ctx.sel_keys.0.copy_from_slice(sel_keys);
}

/// Returns the current selection key setting.
///
/// The returned value is a pointer to an integer array. The memory must
/// be freed by the caller using function
/// [chewing_free][super::setup::chewing_free].
///
/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_get_selKey(ctx: *const ChewingContext) -> *mut c_int {
    let ctx = as_ref_or_return!(ctx, null_mut());
    let _logger_guard = init_scoped_logging(ctx.logger_fn, ctx.logger_data);

    let len = ctx.sel_keys.0.len();
    let ptr = Box::into_raw(ctx.sel_keys.0.to_vec().into_boxed_slice());
    owned_into_raw(Owned::CIntSlice(len), ptr.cast())
}

/// Sets the direction to add new phrases when using CtrlNum.
///
/// The direction argument is 0 when the direction is backward and 1 when
/// the direction is forward.
///
/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_set_addPhraseDirection(
    ctx: *mut ChewingContext,
    direction: c_int,
) {
    unsafe {
        chewing_config_set_int(
            ctx,
            c"chewing.user_phrase_add_direction".as_ptr().cast(),
            direction,
        )
    };
}

/// Returns the direction to add new phrases when using CtrlNum.
///
/// The direction argument is 0 when the direction is backward and 1 when
/// the direction is forward.
///
/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_get_addPhraseDirection(ctx: *const ChewingContext) -> c_int {
    unsafe { chewing_config_get_int(ctx, c"chewing.user_phrase_add_direction".as_ptr().cast()) }
}

/// Sets whether the Space key is treated as a selection key.
///
/// When the mode argument is 1, the Space key will initiate the candidates
/// selection mode.
///
/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_set_spaceAsSelection(ctx: *mut ChewingContext, mode: c_int) {
    unsafe { chewing_config_set_int(ctx, c"chewing.space_is_select_key".as_ptr().cast(), mode) };
}

/// Returns whether the Space key is treated as a selection key.
///
/// Returns 1 when the Space key will initiate the candidates selection
/// mode.
///
/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_get_spaceAsSelection(ctx: *const ChewingContext) -> c_int {
    unsafe { chewing_config_get_int(ctx, c"chewing.space_is_select_key".as_ptr().cast()) }
}

/// Sets whether the Esc key will flush the current pre-edit buffer.
///
/// When the mode argument is 1, the Esc key will flush the pre-edit buffer.
///
/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_set_escCleanAllBuf(ctx: *mut ChewingContext, mode: c_int) {
    unsafe { chewing_config_set_int(ctx, c"chewing.esc_clear_all_buffer".as_ptr().cast(), mode) };
}

/// Returns whether the Esc key will flush the current pre-edit buffer.
///
/// Returns 1 when the Esc key will flush the pre-edit buffer.
///
/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_get_escCleanAllBuf(ctx: *const ChewingContext) -> c_int {
    unsafe { chewing_config_get_int(ctx, c"chewing.esc_clear_all_buffer".as_ptr().cast()) }
}

/// Sets whether the Chewing IM will automatically shift cursor after
/// selection.
///
/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_set_autoShiftCur(ctx: *mut ChewingContext, mode: c_int) {
    unsafe { chewing_config_set_int(ctx, c"chewing.auto_shift_cursor".as_ptr().cast(), mode) };
}

/// Returns whether the Chewing IM will automatically shift cursor after
/// selection.
///
/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_get_autoShiftCur(ctx: *const ChewingContext) -> c_int {
    unsafe { chewing_config_get_int(ctx, c"chewing.auto_shift_cursor".as_ptr().cast()) }
}

/// Sets the current normal/easy symbol mode.
///
/// In easy symbol mode, the key be will changed to its related easy symbol
/// in swkb.dat. The format of swkb.dat is key symbol pair per line. The
/// valid value of key is [0-9A-Z]. The lower case character in key will be
/// changed to upper case when loading swkb.dat. However, in easy symbol
/// mode, only [0-9A-Z] are accepted.
///
/// The mode argument is 0 for normal mode or other for easy symbol mode.
///
/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_set_easySymbolInput(ctx: *mut ChewingContext, mode: c_int) {
    unsafe { chewing_config_set_int(ctx, c"chewing.easy_symbol_input".as_ptr().cast(), mode) };
}

/// Gets the current normal/easy symbol mode.
///
/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_get_easySymbolInput(ctx: *const ChewingContext) -> c_int {
    unsafe { chewing_config_get_int(ctx, c"chewing.easy_symbol_input".as_ptr().cast()) }
}

/// Sets whether the phrase for candidates selection is before the cursor or
/// after the cursor.
///
/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_set_phraseChoiceRearward(ctx: *mut ChewingContext, mode: c_int) {
    unsafe { chewing_config_set_int(ctx, c"chewing.phrase_choice_rearward".as_ptr().cast(), mode) };
}

/// Returns the phrase choice rearward setting.
///
/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_get_phraseChoiceRearward(ctx: *const ChewingContext) -> c_int {
    unsafe { chewing_config_get_int(ctx, c"chewing.phrase_choice_rearward".as_ptr().cast()) }
}

/// Sets enable or disable the automatic learning.
///
/// The mode argument is be one of the [AUTOLEARN_ENABLED][super::public::AUTOLEARN_ENABLED] and
/// [AUTOLEARN_DISABLED][super::public::AUTOLEARN_DISABLED] constants.
///
/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_set_autoLearn(ctx: *mut ChewingContext, mode: c_int) {
    unsafe {
        chewing_config_set_int(
            ctx,
            c"chewing.disable_auto_learn_phrase".as_ptr().cast(),
            mode,
        )
    };
}

/// Returns whether the automatic learning is enabled or disabled.
///
/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_get_autoLearn(ctx: *const ChewingContext) -> c_int {
    unsafe { chewing_config_get_int(ctx, c"chewing.disable_auto_learn_phrase".as_ptr().cast()) }
}

/// Returns the phonetic sequence in the Chewing IM internal state machine.
///
/// The return value is a pointer to a unsigned short array. The values in
/// the array is encoded Bopomofo phone. The memory must be freed by the
/// caller using function [chewing_free][super::setup::chewing_free].
///
/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_get_phoneSeq(ctx: *const ChewingContext) -> *mut c_ushort {
    let ctx = as_ref_or_return!(ctx, null_mut());
    let _logger_guard = init_scoped_logging(ctx.logger_fn, ctx.logger_data);

    let syllables: Vec<_> = ctx
        .editor
        .symbols()
        .iter()
        .cloned()
        .filter(Symbol::is_syllable)
        // NB: we just checked symbol is valid syllable
        .map(|sym| sym.to_syllable().unwrap().to_u16())
        .collect();
    let len = syllables.len();
    let ptr = Box::into_raw(syllables.into_boxed_slice());
    owned_into_raw(Owned::CUShortSlice(len), ptr.cast())
}

/// Returns the length of the phonetic sequence in the Chewing IM internal
/// state machine.
///
/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_get_phoneSeqLen(ctx: *const ChewingContext) -> c_int {
    let ctx = as_ref_or_return!(ctx, ERROR);
    let _logger_guard = init_scoped_logging(ctx.logger_fn, ctx.logger_data);

    ctx.editor
        .symbols()
        .iter()
        .cloned()
        .filter(Symbol::is_syllable)
        .count() as c_int
}

/// Sets the external logger callback.
///
/// The logger function is used to provide log inside Chewing IM for debugging.
/// The user_data pointer is passed directly to the logger when logging.
///
/// # Examples
///
/// The following example shows how to use user_data:
///
/// ```c
/// void logger( void *data, int level, const char *fmt, ... )
/// {
///     FILE *fd = (FILE *) data;
///     ...
/// }
///
/// int main()
/// {
///     ChewingContext *ctx;
///     FILE *fd;
///     ...
///     chewing_set_logger(ctx, logger, fd);
///     ...
/// }
/// ```
///
/// The level is log level.
///
/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_set_logger(
    ctx: *mut ChewingContext,
    logger_fn: Option<
        unsafe extern "C" fn(data: *mut c_void, level: c_int, fmt: *const c_char, ...),
    >,
    logger_data: *mut c_void,
) {
    let ctx = as_mut_or_return!(ctx);
    ctx.logger_fn = logger_fn;
    ctx.logger_data = logger_data;
}

/// Starts a userphrase enumeration.
///
/// Caller shall call this function prior [chewing_userphrase_has_next] and
/// [chewing_userphrase_get] in order to enumerate userphrase correctly.
///
/// This function stores an iterator in the context. The iterator is only
/// destroyed after enumerate all userphrases using
/// [chewing_userphrase_has_next].
///
/// Returns 0 on success, -1 on failure.
///
/// # Examples
///
/// ```c
/// chewing_userphrase_enumerate(ctx);
/// while (chewing_userphrase_has_next(ctx, &phrase_len, &bopomofo_len)) {
///     phrase = malloc(phrase_len);
///     if (!phrase) goto error;
///     bopomofo = malloc(bopomofo_len);
///     if (!bopomofo) goto error;
///
///     chewing_userphrase_get(ctx, phrase, phrase_len, bopomofo, bopomofo_len);
///     // ...
/// }
/// ```
///
/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_userphrase_enumerate(ctx: *mut ChewingContext) -> c_int {
    let ctx = as_mut_or_return!(ctx, ERROR);
    let _logger_guard = init_scoped_logging(ctx.logger_fn, ctx.logger_data);

    ctx.userphrase_iter = Some(ctx.editor.user_dict().entries().peekable());
    OK
}

/// Checks if there is another userphrase in current enumeration.
///
/// The *phrase_len* and *bopomofo_len* are output buffer length needed by the userphrase and its bopomofo string.
///
/// Returns 1 when true, 0 when false.
///
/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_userphrase_has_next(
    ctx: *mut ChewingContext,
    phrase_len: *mut c_uint,
    bopomofo_len: *mut c_uint,
) -> c_int {
    let ctx = as_mut_or_return!(ctx, FALSE);
    let _logger_guard = init_scoped_logging(ctx.logger_fn, ctx.logger_data);

    if ctx.userphrase_iter.is_none() {
        return 0;
    }

    // NB: we just checked the iter is Some
    match ctx.userphrase_iter.as_mut().unwrap().peek() {
        Some(entry) => {
            if !phrase_len.is_null() {
                let phrase = entry.1.as_str().as_bytes();
                unsafe { phrase_len.write((phrase.len() + 1) as u32) }
            }
            if !bopomofo_len.is_null() {
                let bopomofo = entry
                    .0
                    .iter()
                    .map(|it| it.to_string())
                    .collect::<Vec<_>>()
                    .join(" ");
                unsafe { bopomofo_len.write((bopomofo.len() + 1) as u32) }
            }
            1
        }
        None => {
            ctx.userphrase_iter = None;
            0
        }
    }
}

/// Gets the current enumerated userphrase.
///
/// The *phrase_buf* and *bopomofo_buf* are userphrase and its bopomofo
/// buffer provided by caller. The length of the buffers can be retrived
/// from [chewing_userphrase_has_next].
///
/// Returns 0 on success, -1 on failure.
///
/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_userphrase_get(
    ctx: *mut ChewingContext,
    phrase_buf: *mut c_char,
    phrase_len: c_uint,
    bopomofo_buf: *mut c_char,
    bopomofo_len: c_uint,
) -> c_int {
    let ctx = as_mut_or_return!(ctx, ERROR);
    let _logger_guard = init_scoped_logging(ctx.logger_fn, ctx.logger_data);

    if ctx.userphrase_iter.is_none() {
        return -1;
    }

    // NB: we just checked the iter is Some
    match ctx.userphrase_iter.as_mut().unwrap().next() {
        Some(entry) => {
            if !phrase_buf.is_null() {
                let phrase = entry.1.as_str().as_bytes();
                let phrase_buf =
                    unsafe { slice::from_raw_parts_mut(phrase_buf.cast(), phrase_len as usize) };
                phrase_buf[..phrase.len()].copy_from_slice(phrase);
                phrase_buf[phrase.len()] = 0;
            }
            if !bopomofo_buf.is_null() {
                let bopomofo = entry
                    .0
                    .iter()
                    .map(|it| it.to_string())
                    .collect::<Vec<_>>()
                    .join(" ");
                let bopomofo = bopomofo.as_bytes();
                let bopomofo_buf = unsafe {
                    slice::from_raw_parts_mut(bopomofo_buf.cast(), bopomofo_len as usize)
                };
                bopomofo_buf[..bopomofo.len()].copy_from_slice(bopomofo);
                bopomofo_buf[bopomofo.len()] = 0;
            }
            0
        }
        None => -1,
    }
}

/// Adds new userphrase to the user dictionary.
///
/// Returns how many phrases are added, -1 on failure.
///
/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_userphrase_add(
    ctx: *mut ChewingContext,
    phrase_buf: *const c_char,
    bopomofo_buf: *const c_char,
) -> c_int {
    let ctx = as_mut_or_return!(ctx, ERROR);
    let _logger_guard = init_scoped_logging(ctx.logger_fn, ctx.logger_data);

    let syllables = match unsafe { str_from_ptr_with_nul(bopomofo_buf) } {
        Some(bopomofo) => bopomofo
            .split_ascii_whitespace()
            .map_while(|it| it.parse::<Syllable>().ok())
            .collect::<Vec<_>>(),
        None => return 0,
    };

    if syllables.len() > 11 {
        return 0;
    }

    match unsafe { str_from_ptr_with_nul(phrase_buf) } {
        Some(phrase) => match ctx.editor.learn_phrase(&syllables, phrase) {
            Ok(_) => TRUE,
            Err(_) => FALSE,
        },
        None => ERROR,
    }
}

/// Removes a userphrase from the user dictionary.
///
/// Returns how many phrases are removed, -1 on failure.
///
/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_userphrase_remove(
    ctx: *mut ChewingContext,
    phrase_buf: *const c_char,
    bopomofo_buf: *const c_char,
) -> c_int {
    let ctx = as_mut_or_return!(ctx, ERROR);
    let _logger_guard = init_scoped_logging(ctx.logger_fn, ctx.logger_data);

    // return FALSE when phrase does not exist is C API only behavior
    if unsafe { chewing_userphrase_lookup(ctx, phrase_buf, bopomofo_buf) } != TRUE {
        return FALSE;
    }

    let syllables = match unsafe { str_from_ptr_with_nul(bopomofo_buf) } {
        Some(bopomofo) => bopomofo
            .split_ascii_whitespace()
            .map_while(|it| it.parse::<Syllable>().ok())
            .collect::<Vec<_>>(),
        None => return ERROR,
    };

    match unsafe { str_from_ptr_with_nul(phrase_buf) } {
        Some(phrase) => match ctx.editor.unlearn_phrase(&syllables, phrase) {
            Err(_) => FALSE,
            Ok(_) => TRUE,
        },
        None => ERROR,
    }
}

/// Searchs if a userphrase is in the user dictionary.
///
/// Returns 1 when true, 0 when false.
///
/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_userphrase_lookup(
    ctx: *mut ChewingContext,
    phrase_buf: *const c_char,
    bopomofo_buf: *const c_char,
) -> c_int {
    let ctx = as_mut_or_return!(ctx, FALSE);
    let _logger_guard = init_scoped_logging(ctx.logger_fn, ctx.logger_data);

    let syllables = match unsafe { str_from_ptr_with_nul(bopomofo_buf) } {
        Some(bopomofo) => bopomofo
            .split_ascii_whitespace()
            .map_while(|it| it.parse::<Syllable>().ok())
            .collect::<Vec<_>>(),
        None => return 0,
    };

    match unsafe { str_from_ptr_with_nul(phrase_buf) } {
        Some(phrase) => ctx
            .editor
            .user_dict()
            .lookup(&syllables, LookupStrategy::Standard)
            .iter()
            .any(|ph| ph.as_str() == phrase) as c_int,
        None => ctx
            .editor
            .user_dict()
            .lookup(&syllables, LookupStrategy::Standard)
            .is_empty()
            .not() as c_int,
    }
}

/// Sets the candidate list to the first (longest) candidate list.
///
/// Returns 0 when success, -1 otherwise.
///
/// # Errors
///
/// This function fails if the candidate selection window is not currently
/// open.
///
/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_cand_list_first(ctx: *mut ChewingContext) -> c_int {
    let ctx = as_mut_or_return!(ctx, ERROR);
    let _logger_guard = init_scoped_logging(ctx.logger_fn, ctx.logger_data);

    if !ctx.editor.is_selecting() {
        return -1;
    }

    let _ = ctx.editor.jump_to_first_selection_point();
    OK
}

/// Sets the candidate list to the last (shortest) candidate list.
///
/// Returns 0 when success, -1 otherwise.
///
/// # Errors
///
/// This function fails if the candidate selection window is not currently
/// open.
///
/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_cand_list_last(ctx: *mut ChewingContext) -> c_int {
    let ctx = as_mut_or_return!(ctx, ERROR);
    let _logger_guard = init_scoped_logging(ctx.logger_fn, ctx.logger_data);

    if !ctx.editor.is_selecting() {
        return -1;
    }

    let _ = ctx.editor.jump_to_last_selection_point();
    OK
}

/// Checks whether there is a next (shorter) candidate list.
///
/// Returns 1 (true) when there is a next candidate list, 0 otherwise.
///
/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_cand_list_has_next(ctx: *mut ChewingContext) -> c_int {
    let ctx = as_ref_or_return!(ctx, FALSE);
    let _logger_guard = init_scoped_logging(ctx.logger_fn, ctx.logger_data);

    if !ctx.editor.is_selecting() {
        return 0;
    }

    ctx.editor.has_next_selection_point() as c_int
}

/// Checks whether there is a previous (longer) candidate list.
///
/// Returns 1 (true) when there is a previous candidate list, 0 otherwise.
///
/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_cand_list_has_prev(ctx: *mut ChewingContext) -> c_int {
    let ctx = as_ref_or_return!(ctx, FALSE);
    let _logger_guard = init_scoped_logging(ctx.logger_fn, ctx.logger_data);

    if !ctx.editor.is_selecting() {
        return 0;
    }

    ctx.editor.has_prev_selection_point() as c_int
}

/// Changes current candidate list to next candidate list.
///
/// Returns 0 when success, -1 otherwise.
///
/// # Errors
///
/// This function fails if the candidate selection window is not currently
/// open.
///
/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_cand_list_next(ctx: *mut ChewingContext) -> c_int {
    let ctx = as_mut_or_return!(ctx, ERROR);
    let _logger_guard = init_scoped_logging(ctx.logger_fn, ctx.logger_data);

    if !ctx.editor.is_selecting() {
        return -1;
    }
    match ctx.editor.jump_to_next_selection_point() {
        Ok(_) => OK,
        Err(_) => ERROR,
    }
}

/// Changes current candidate list to previous candidate list.
///
/// Returns 0 when success, -1 otherwise.
///
/// # Errors
///
/// This function fails if the candidate selection window is not currently
/// open.
///
/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_cand_list_prev(ctx: *mut ChewingContext) -> c_int {
    let ctx = as_mut_or_return!(ctx, ERROR);
    let _logger_guard = init_scoped_logging(ctx.logger_fn, ctx.logger_data);

    if !ctx.editor.is_selecting() {
        return -1;
    }
    match ctx.editor.jump_to_prev_selection_point() {
        Ok(_) => OK,
        Err(_) => ERROR,
    }
}

/// Commits the current preedit buffer content to the commit buffer.
///
/// Returns 0 when success, -1 otherwise.
///
/// # Errors
///
/// This function fails if the IM editor is not in entering state.
///
/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_commit_preedit_buf(ctx: *mut ChewingContext) -> c_int {
    let ctx = as_mut_or_return!(ctx, ERROR);
    let _logger_guard = init_scoped_logging(ctx.logger_fn, ctx.logger_data);

    if ctx.editor.is_selecting() {
        return ERROR;
    }

    match ctx.editor.commit() {
        Ok(_) => OK,
        Err(_) => ERROR,
    }
}

/// Clears the current preedit buffer content.
///
/// Returns 0 when success, -1 otherwise.
///
/// # Errors
///
/// This function fails if the IM editor is not in entering state.
///
/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_clean_preedit_buf(ctx: *mut ChewingContext) -> c_int {
    let ctx = as_mut_or_return!(ctx, ERROR);
    let _logger_guard = init_scoped_logging(ctx.logger_fn, ctx.logger_data);

    if ctx.editor.is_selecting() {
        return ERROR;
    }

    ctx.editor.clear_composition_editor();
    OK
}

/// Clears the current bopomofo buffer content.
///
/// Returns 0 when success, -1 otherwise.
///
/// # Errors
///
/// This function fails if the IM editor is not in entering state.
///
/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_clean_bopomofo_buf(ctx: *mut ChewingContext) -> c_int {
    let ctx = as_mut_or_return!(ctx, ERROR);
    let _logger_guard = init_scoped_logging(ctx.logger_fn, ctx.logger_data);

    ctx.editor.clear_syllable_editor();
    OK
}

/// Converts the u16 encoded syllables to a bopomofo string.
///
/// If both of the buf and the len are 0, this function will return buf
/// length for bopomofo including the null character so that caller can
/// prepare enough buffer for it.
///
/// Returns 0 on success, -1 on failure.
///
/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_phone_to_bopomofo(
    phone: c_ushort,
    buf: *mut c_char,
    len: c_ushort,
) -> c_int {
    let syl_str = match Syllable::try_from(phone) {
        Ok(s) => s.to_string(),
        Err(_) => return ERROR,
    };
    if !buf.is_null() && len as usize >= (syl_str.len() + 1) {
        let buf = unsafe { slice::from_raw_parts_mut(buf.cast(), len as usize) };
        buf[0..syl_str.len()].copy_from_slice(syl_str.as_bytes());
        buf[syl_str.len()] = 0;
    }
    (syl_str.len() + 1) as c_int
}

/// Handles all possible key events.
///
/// **code**
///
/// Code that identifies a physical key on a keyboard.
///
/// Keycodes are the result of the low-level processing of the data that
/// keyboards send to a computer. For instance 36 may represent the return
/// key.
///
/// Symbolic names are assigned to raw keycodes in order to facilitate
/// their mapping to symbols. By convention keycode names are based on US
/// QWERTY layout. For example the keycode for the return key is
/// RETURN.
///
/// Chewing keycodes have same numeric encoding as X11 or xkbcommon
/// keycodes.
///
/// **ksym**
///
/// The symbol on the cap of a key.
///
/// Keysyms (short for "key symbol") are translated from keycodes via a
/// keymap. On different layout (qwerty, dvorak, etc.) all keyboards emit
/// the same keycodes but produce different keysyms after translation.
/// The key press / release state and state of modifier keys.
///
/// **state**
///
/// Use the state mask to read whether a modifier key is active and
/// whether the key is pressed.
///
/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_handle_KeyboardEvent(
    ctx: *mut ChewingContext,
    code: u8,
    ksym: u32,
    state: u32,
) -> c_int {
    let ctx = as_mut_or_return!(ctx, ERROR);
    let _logger_guard = init_scoped_logging(ctx.logger_fn, ctx.logger_data);

    // XXX hack for selkey
    let key = if ctx.editor.is_selecting() {
        match ctx
            .sel_keys
            .0
            .iter()
            .position(|&it| it == Keysym(ksym).to_unicode() as i32)
        {
            Some(idx) => {
                let key = match idx {
                    0 => b'1',
                    1 => b'2',
                    2 => b'3',
                    3 => b'4',
                    4 => b'5',
                    5 => b'6',
                    6 => b'7',
                    7 => b'8',
                    8 => b'9',
                    9 => b'0',
                    _ => b'0',
                };
                Some(key as c_int)
            }
            None => None,
        }
    } else {
        None
    };

    let evt = if let Some(key) = key {
        let mut evt = map_ascii(&ctx.keymap, key as u8);
        evt.state = state;
        evt
    } else {
        KeyboardEvent {
            code: Keycode(code),
            ksym: Keysym(ksym),
            state,
        }
    };

    ctx.editor.process_keyevent(evt);
    OK
}

/// Handles the Space key.
///
/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_handle_Space(ctx: *mut ChewingContext) -> c_int {
    let ctx = as_mut_or_return!(ctx, ERROR);
    let _logger_guard = init_scoped_logging(ctx.logger_fn, ctx.logger_data);

    ctx.editor.process_keyevent(KeyboardEvent {
        code: KEY_SPACE,
        ksym: SYM_SPACE,
        state: 0,
    });
    OK
}

/// Handles the Esc key.
///
/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_handle_Esc(ctx: *mut ChewingContext) -> c_int {
    let ctx = as_mut_or_return!(ctx, ERROR);
    let _logger_guard = init_scoped_logging(ctx.logger_fn, ctx.logger_data);

    ctx.editor.process_keyevent(KeyboardEvent {
        code: KEY_ESC,
        ksym: SYM_ESC,
        state: 0,
    });
    OK
}

/// Handles the Enter or Return key.
///
/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_handle_Enter(ctx: *mut ChewingContext) -> c_int {
    let ctx = as_mut_or_return!(ctx, ERROR);
    let _logger_guard = init_scoped_logging(ctx.logger_fn, ctx.logger_data);

    ctx.editor.process_keyevent(KeyboardEvent {
        code: KEY_ENTER,
        ksym: SYM_RETURN,
        state: 0,
    });
    OK
}

/// Handles the Delete key.
///
/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_handle_Del(ctx: *mut ChewingContext) -> c_int {
    let ctx = as_mut_or_return!(ctx, ERROR);
    let _logger_guard = init_scoped_logging(ctx.logger_fn, ctx.logger_data);

    ctx.editor.process_keyevent(KeyboardEvent {
        code: KEY_DELETE,
        ksym: SYM_DELETE,
        state: 0,
    });
    OK
}

/// Handles the Backspace key.
///
/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_handle_Backspace(ctx: *mut ChewingContext) -> c_int {
    let ctx = as_mut_or_return!(ctx, ERROR);
    let _logger_guard = init_scoped_logging(ctx.logger_fn, ctx.logger_data);

    ctx.editor.process_keyevent(KeyboardEvent {
        code: KEY_BACKSPACE,
        ksym: SYM_BACKSPACE,
        state: 0,
    });
    OK
}

/// Handles the Tab key.
///
/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_handle_Tab(ctx: *mut ChewingContext) -> c_int {
    let ctx = as_mut_or_return!(ctx, ERROR);
    let _logger_guard = init_scoped_logging(ctx.logger_fn, ctx.logger_data);

    ctx.editor.process_keyevent(KeyboardEvent {
        code: KEY_TAB,
        ksym: SYM_TAB,
        state: 0,
    });
    OK
}

/// Handles the Left key with the Shift modifier.
///
/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_handle_ShiftLeft(ctx: *mut ChewingContext) -> c_int {
    let ctx = as_mut_or_return!(ctx, ERROR);
    let _logger_guard = init_scoped_logging(ctx.logger_fn, ctx.logger_data);

    ctx.editor.process_keyevent(
        KeyboardEvent::builder()
            .code(KEY_LEFT)
            .ksym(SYM_LEFT)
            .shift()
            .build(),
    );
    OK
}

/// Handles the Left key.
///
/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_handle_Left(ctx: *mut ChewingContext) -> c_int {
    let ctx = as_mut_or_return!(ctx, ERROR);
    let _logger_guard = init_scoped_logging(ctx.logger_fn, ctx.logger_data);

    ctx.editor.process_keyevent(KeyboardEvent {
        code: KEY_LEFT,
        ksym: SYM_LEFT,
        state: 0,
    });
    OK
}

/// Handles the Right key with the Shift modifier.
///
/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_handle_ShiftRight(ctx: *mut ChewingContext) -> c_int {
    let ctx = as_mut_or_return!(ctx, ERROR);
    let _logger_guard = init_scoped_logging(ctx.logger_fn, ctx.logger_data);

    ctx.editor.process_keyevent(
        KeyboardEvent::builder()
            .code(KEY_RIGHT)
            .ksym(SYM_RIGHT)
            .shift()
            .build(),
    );
    OK
}

/// Handles the Right key.
///
/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_handle_Right(ctx: *mut ChewingContext) -> c_int {
    let ctx = as_mut_or_return!(ctx, ERROR);
    let _logger_guard = init_scoped_logging(ctx.logger_fn, ctx.logger_data);

    ctx.editor.process_keyevent(KeyboardEvent {
        code: KEY_RIGHT,
        ksym: SYM_RIGHT,
        state: 0,
    });
    OK
}

/// Handles the Up key.
///
/// See also [chewing_cand_close][super::candidates::chewing_cand_close] keyboardless API to close candidate
/// window.
///
/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_handle_Up(ctx: *mut ChewingContext) -> c_int {
    let ctx = as_mut_or_return!(ctx, ERROR);
    let _logger_guard = init_scoped_logging(ctx.logger_fn, ctx.logger_data);

    ctx.editor.process_keyevent(KeyboardEvent {
        code: KEY_UP,
        ksym: SYM_UP,
        state: 0,
    });
    OK
}

/// Handles the Home key.
///
/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_handle_Home(ctx: *mut ChewingContext) -> c_int {
    let ctx = as_mut_or_return!(ctx, ERROR);
    let _logger_guard = init_scoped_logging(ctx.logger_fn, ctx.logger_data);

    ctx.editor.process_keyevent(KeyboardEvent {
        code: KEY_HOME,
        ksym: SYM_HOME,
        state: 0,
    });
    OK
}

/// Handles the End key.
///
/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_handle_End(ctx: *mut ChewingContext) -> c_int {
    let ctx = as_mut_or_return!(ctx, ERROR);
    let _logger_guard = init_scoped_logging(ctx.logger_fn, ctx.logger_data);

    ctx.editor.process_keyevent(KeyboardEvent {
        code: KEY_END,
        ksym: SYM_END,
        state: 0,
    });
    OK
}

/// Handles the PageUp key.
///
/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_handle_PageUp(ctx: *mut ChewingContext) -> c_int {
    let ctx = as_mut_or_return!(ctx, ERROR);
    let _logger_guard = init_scoped_logging(ctx.logger_fn, ctx.logger_data);

    ctx.editor.process_keyevent(KeyboardEvent {
        code: KEY_PAGEUP,
        ksym: SYM_PAGEUP,
        state: 0,
    });
    OK
}

/// Handles the PageDown key.
///
/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_handle_PageDown(ctx: *mut ChewingContext) -> c_int {
    let ctx = as_mut_or_return!(ctx, ERROR);
    let _logger_guard = init_scoped_logging(ctx.logger_fn, ctx.logger_data);

    ctx.editor.process_keyevent(KeyboardEvent {
        code: KEY_PAGEDOWN,
        ksym: SYM_PAGEDOWN,
        state: 0,
    });
    OK
}

/// Handles the Down key.
///
/// See also [super::io::chewing_cand_open] keyboardless API to open candidate window.
///
/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_handle_Down(ctx: *mut ChewingContext) -> c_int {
    let ctx = as_mut_or_return!(ctx, ERROR);
    let _logger_guard = init_scoped_logging(ctx.logger_fn, ctx.logger_data);

    ctx.editor.process_keyevent(KeyboardEvent {
        code: KEY_DOWN,
        ksym: SYM_DOWN,
        state: 0,
    });
    OK
}

/// Handles the Capslock key.
///
/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_handle_Capslock(ctx: *mut ChewingContext) -> c_int {
    let ctx = as_mut_or_return!(ctx, ERROR);
    let _logger_guard = init_scoped_logging(ctx.logger_fn, ctx.logger_data);

    ctx.editor.process_keyevent(
        KeyboardEvent::builder()
            .code(KEY_CAPSLOCK)
            .ksym(SYM_CAPSLOCK)
            .caps_lock()
            .build(),
    );
    OK
}

/// Handles all keys that do not have dedicated methods.
///
/// The value of of key can be any printable ASCII characters.
///
/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_handle_Default(ctx: *mut ChewingContext, key: c_int) -> c_int {
    let ctx = as_mut_or_return!(ctx, ERROR);
    let _logger_guard = init_scoped_logging(ctx.logger_fn, ctx.logger_data);

    let evt = map_ascii(&ctx.keymap, key as u8);

    unsafe { chewing_handle_KeyboardEvent(ctx, evt.code.0, evt.ksym.0, evt.state) }
}

/// Handles any number key with the Ctrl modifier.
///
/// The value of key should be in the range between ASCII character code
/// from 0 to 9.
///
/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_handle_CtrlNum(ctx: *mut ChewingContext, key: c_int) -> c_int {
    let ctx = as_mut_or_return!(ctx, ERROR);
    let _logger_guard = init_scoped_logging(ctx.logger_fn, ctx.logger_data);

    let (code, ksym) = match key as u8 {
        b'0' => (KEY_0, SYM_0),
        b'1' => (KEY_1, SYM_1),
        b'2' => (KEY_2, SYM_2),
        b'3' => (KEY_3, SYM_3),
        b'4' => (KEY_4, SYM_4),
        b'5' => (KEY_5, SYM_5),
        b'6' => (KEY_6, SYM_6),
        b'7' => (KEY_7, SYM_7),
        b'8' => (KEY_8, SYM_8),
        b'9' => (KEY_9, SYM_9),
        _ => return -1,
    };

    ctx.editor.process_keyevent(
        KeyboardEvent::builder()
            .code(code)
            .ksym(ksym)
            .control()
            .build(),
    );
    OK
}

/// Handles the Space key with the Shift modifier.
///
/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_handle_ShiftSpace(ctx: *mut ChewingContext) -> c_int {
    let ctx = as_mut_or_return!(ctx, ERROR);
    let _logger_guard = init_scoped_logging(ctx.logger_fn, ctx.logger_data);

    ctx.editor.process_keyevent(
        KeyboardEvent::builder()
            .code(KEY_SPACE)
            .ksym(SYM_SPACE)
            .shift()
            .build(),
    );
    OK
}

/// Handles tapping the Tab key twice quickly.
///
/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_handle_DblTab(ctx: *mut ChewingContext) -> c_int {
    let ctx = as_mut_or_return!(ctx, ERROR);
    let _logger_guard = init_scoped_logging(ctx.logger_fn, ctx.logger_data);

    // todo!()
    OK
}

/// Handles any numeric key from the keypad.
///
/// The value of key should be in the range between ASCII character code
/// from 0 to 9.
///
/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_handle_Numlock(ctx: *mut ChewingContext, key: c_int) -> c_int {
    let ctx = as_mut_or_return!(ctx, ERROR);
    let _logger_guard = init_scoped_logging(ctx.logger_fn, ctx.logger_data);

    let (code, ksym) = match key as u8 {
        b'0' => (KEY_KP0, SYM_KP0),
        b'1' => (KEY_KP1, SYM_KP1),
        b'2' => (KEY_KP2, SYM_KP2),
        b'3' => (KEY_KP3, SYM_KP3),
        b'4' => (KEY_KP4, SYM_KP4),
        b'5' => (KEY_KP5, SYM_KP5),
        b'6' => (KEY_KP6, SYM_KP6),
        b'7' => (KEY_KP7, SYM_KP7),
        b'8' => (KEY_KP8, SYM_KP8),
        b'9' => (KEY_KP9, SYM_KP9),
        b'+' => (KEY_KPPLUS, SYM_KPADD),
        b'-' => (KEY_KPMINUS, SYM_KPSUBTRACT),
        b'*' => (KEY_KPASTERISK, SYM_KPMULTIPLY),
        b'/' => (KEY_KPSLASH, SYM_KPDIVIDE),
        b'.' => (KEY_KPDOT, SYM_KPDECIMAL),
        _ => return -1,
    };

    ctx.editor.process_keyevent(
        KeyboardEvent::builder()
            .code(code)
            .ksym(ksym)
            .num_lock_if(true)
            .build(),
    );
    OK
}

/// Checks whether the commit buffer has something to read.
///
/// Returns 1 when true, 0 when false.
///
/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_commit_Check(ctx: *const ChewingContext) -> c_int {
    let ctx = as_ref_or_return!(ctx, ERROR);
    let _logger_guard = init_scoped_logging(ctx.logger_fn, ctx.logger_data);

    !ctx.editor.display_commit().is_empty() as c_int
}

/// Returns the string in the commit buffer.
///
/// The returned value is a pointer to a character string. The memory must
/// be freed by the caller using function
/// [chewing_free][super::setup::chewing_free].
///
/// # Failures
///
/// This function returns NULL when memory allocation fails.
///
/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_commit_String(ctx: *const ChewingContext) -> *mut c_char {
    let ctx = as_ref_or_return!(
        ctx,
        owned_into_raw(Owned::CString, CString::default().into_raw())
    );
    let _logger_guard = init_scoped_logging(ctx.logger_fn, ctx.logger_data);

    let buffer = ctx.editor.display_commit();
    let cstr = match CString::new(buffer) {
        Ok(cstr) => cstr,
        Err(_) => return null_mut(),
    };
    owned_into_raw(Owned::CString, cstr.into_raw())
}

/// Returns the string in the commit buffer.
///
/// The return value is a const pointer to a character string. The pointer
/// is only valid immediately after checking the [chewing_commit_Check]
/// condition.
///
/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_commit_String_static(ctx: *const ChewingContext) -> *const c_char {
    let ctx = as_mut_or_return!(ctx.cast_mut(), global_empty_cstr());
    let _logger_guard = init_scoped_logging(ctx.logger_fn, ctx.logger_data);

    copy_cstr(&mut ctx.commit_buf, ctx.editor.display_commit())
}

/// Returns the current output in the pre-edit buffer.
///
/// The returned value is a pointer to a character string. The memory must
/// be freed by the caller using function
/// [chewing_free][super::setup::chewing_free].
///
/// # Failures
///
/// This function returns NULL when memory allocation fails.
///
/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_buffer_String(ctx: *const ChewingContext) -> *mut c_char {
    let ctx = as_ref_or_return!(
        ctx,
        owned_into_raw(Owned::CString, CString::default().into_raw())
    );
    let _logger_guard = init_scoped_logging(ctx.logger_fn, ctx.logger_data);

    let buffer = ctx.editor.display();
    let cstr = match CString::new(buffer) {
        Ok(cstr) => cstr,
        Err(_) => return null_mut(),
    };
    owned_into_raw(Owned::CString, cstr.into_raw())
}

/// Returns the current output in the pre-edit buffer.
///
/// The return value is a const pointer to a character string. The pointer
/// is only valid immediately after checking the [chewing_buffer_Check]
/// condition.
///
/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_buffer_String_static(ctx: *const ChewingContext) -> *const c_char {
    let ctx = as_mut_or_return!(ctx.cast_mut(), global_empty_cstr());
    let _logger_guard = init_scoped_logging(ctx.logger_fn, ctx.logger_data);

    copy_cstr(&mut ctx.preedit_buf, &ctx.editor.display())
}

/// Checks whether there is output in the pre-edit buffer.
///
/// Returns 1 when true, 0 when false.
///
/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_buffer_Check(ctx: *const ChewingContext) -> c_int {
    let ctx = as_ref_or_return!(ctx, ERROR);
    let _logger_guard = init_scoped_logging(ctx.logger_fn, ctx.logger_data);

    !ctx.editor.is_empty() as c_int
}

/// Returns the length of the string in current pre-edit buffer.
///
/// <p style="background:rgba(255,181,77,0.16);padding:0.75em;">
/// <strong>⚠ Warning:</strong> The length is calculated in terms of
/// unicode characters. One character might occupy multiple bytes.
/// </p>
///
/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_buffer_Len(ctx: *const ChewingContext) -> c_int {
    let ctx = as_ref_or_return!(ctx, ERROR);
    let _logger_guard = init_scoped_logging(ctx.logger_fn, ctx.logger_data);

    ctx.editor.len() as c_int
}

/// Returns the phonetic characters in the pre-edit buffer.
///
/// The return value is a const pointer to a character string. The pointer
/// is only valid immediately after checking the [chewing_bopomofo_Check]
/// condition.
///
/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_bopomofo_String_static(
    ctx: *const ChewingContext,
) -> *const c_char {
    let ctx = as_mut_or_return!(ctx.cast_mut(), global_empty_cstr());
    let _logger_guard = init_scoped_logging(ctx.logger_fn, ctx.logger_data);

    copy_cstr(&mut ctx.bopomofo_buf, &ctx.editor.syllable_buffer_display())
}

/// Returns the phonetic characters in the pre-edit buffer.
///
/// The returned value is a pointer to a character string. The memory must
/// be freed by the caller using function
/// [chewing_free][super::setup::chewing_free].
///
/// # Failures
///
/// This function returns NULL when memory allocation fails.
///
/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_bopomofo_String(ctx: *const ChewingContext) -> *mut c_char {
    let ctx = as_ref_or_return!(
        ctx,
        owned_into_raw(Owned::CString, CString::default().into_raw())
    );
    let _logger_guard = init_scoped_logging(ctx.logger_fn, ctx.logger_data);

    let buffer = ctx.editor.syllable_buffer_display();
    let cstr = match CString::new(buffer) {
        Ok(cstr) => cstr,
        Err(_) => return null_mut(),
    };
    owned_into_raw(Owned::CString, cstr.into_raw())
}

/// Returns whether there are phonetic pre-edit string in the buffer.
///
/// Returns 1 when true, 0 when false.
///
/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_bopomofo_Check(ctx: *const ChewingContext) -> c_int {
    let ctx = as_ref_or_return!(ctx, ERROR);
    let _logger_guard = init_scoped_logging(ctx.logger_fn, ctx.logger_data);

    ctx.editor.entering_syllable() as c_int
}

/// Returns the current cursor position in the pre-edit buffer.
///
/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_cursor_Current(ctx: *const ChewingContext) -> c_int {
    let ctx = as_ref_or_return!(ctx, ERROR);
    let _logger_guard = init_scoped_logging(ctx.logger_fn, ctx.logger_data);

    ctx.editor.cursor() as c_int
}

/// Checks if the candidates selection has finished.
///
/// <p style="background:rgba(255,181,77,0.16);padding:0.75em;">
/// <strong>⚠ Warning:</strong> Not implemented.
/// </p>
///
/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
#[deprecated(note = "The chewing_cand_TotalPage function could achieve the same effect.")]
pub unsafe extern "C" fn chewing_cand_CheckDone(ctx: *const ChewingContext) -> c_int {
    let ctx = as_ref_or_return!(ctx, ERROR);
    let _logger_guard = init_scoped_logging(ctx.logger_fn, ctx.logger_data);

    if ctx.editor.is_selecting() {
        FALSE
    } else {
        TRUE
    }
}

/// Returns the number of pages of the candidates.
///
/// If the return value is greater than zero, then the IM interface should
/// display a selection window of the candidates for the user to choose a
/// candidate. Otherwise hide the selection window.
///
/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_cand_TotalPage(ctx: *const ChewingContext) -> c_int {
    let ctx = as_ref_or_return!(ctx, ERROR);
    let _logger_guard = init_scoped_logging(ctx.logger_fn, ctx.logger_data);

    ctx.editor.total_page().unwrap_or_default() as c_int
}

/// Returns the number of the coices per page.
///
/// See also the [chewing_set_candPerPage] function.
///
/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_cand_ChoicePerPage(ctx: *const ChewingContext) -> c_int {
    let ctx = as_ref_or_return!(ctx, ERROR);
    let _logger_guard = init_scoped_logging(ctx.logger_fn, ctx.logger_data);

    ctx.editor.editor_options().candidates_per_page as c_int
}

/// Returns the total number of the available choices.
///
/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_cand_TotalChoice(ctx: *const ChewingContext) -> c_int {
    let ctx = as_ref_or_return!(ctx, ERROR);
    let _logger_guard = init_scoped_logging(ctx.logger_fn, ctx.logger_data);

    match ctx.editor.all_candidates() {
        Ok(candidates) => candidates.len() as c_int,
        Err(_) => 0,
    }
}

/// Returns the current candidate page number.
///
/// # Examples
///
/// The candidates pagination could be displayed as:
///
/// ```c
/// sprintf(buf, "[%d / %d]",
///     chewing_cand_CurrentPage(ctx),
///     chewing_cand_TotalPage(ctx));
/// ```
///
/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_cand_CurrentPage(ctx: *const ChewingContext) -> c_int {
    let ctx = as_ref_or_return!(ctx, ERROR);
    let _logger_guard = init_scoped_logging(ctx.logger_fn, ctx.logger_data);

    ctx.editor.current_page_no().unwrap_or_default() as c_int
}

/// Starts the enumeration of the candidates starting from the first one in
/// the current page.
///
/// This function stores an iterator in the context. The iterator is only
/// destroyed after enumerate candidates using
/// [chewing_cand_hasNext].
///
/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_cand_Enumerate(ctx: *mut ChewingContext) {
    let ctx = as_mut_or_return!(ctx);
    let _logger_guard = init_scoped_logging(ctx.logger_fn, ctx.logger_data);

    if let Ok(candidates) = ctx.editor.paginated_candidates() {
        debug!("candidates: {candidates:?}");
        let phrases = Box::new(candidates.into_iter()) as Box<dyn Iterator<Item = String>>;
        ctx.cand_iter = Some(phrases.peekable());
    }
}

/// Checks if there are more candidates to enumerate.
///
/// <p style="background:rgba(255,181,77,0.16);padding:0.75em;">
/// <strong>⚠ Warning:</strong> This function checks the end of total choices
/// instead of the end of current page.
/// </p>
///
/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_cand_hasNext(ctx: *mut ChewingContext) -> c_int {
    let ctx = as_mut_or_return!(ctx, ERROR);
    let _logger_guard = init_scoped_logging(ctx.logger_fn, ctx.logger_data);

    if !ctx.editor.is_selecting() {
        return FALSE;
    }

    ctx.cand_iter
        .as_mut()
        .and_then(|it| it.peek())
        .map_or(0, |_| 1)
}

/// Returns the current enumerated candidate string.
///
/// The returned value is a pointer to a character string. The memory must
/// be freed by the caller using function
/// [chewing_free][super::setup::chewing_free].
///
/// # Failures
///
/// This function returns NULL when memory allocation fails.
///
/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_cand_String(ctx: *mut ChewingContext) -> *mut c_char {
    let ctx = as_mut_or_return!(
        ctx,
        owned_into_raw(Owned::CString, CString::default().into_raw())
    );
    let _logger_guard = init_scoped_logging(ctx.logger_fn, ctx.logger_data);

    match ctx.cand_iter.as_mut().and_then(|it| it.next()) {
        Some(phrase) => {
            let cstr = match CString::new(phrase.clone()) {
                Ok(cstr) => cstr,
                Err(_) => return owned_into_raw(Owned::CString, CString::default().into_raw()),
            };
            owned_into_raw(Owned::CString, cstr.into_raw())
        }
        None => owned_into_raw(Owned::CString, CString::default().into_raw()),
    }
}

/// Returns the current enumerated candidate string.
///
/// The returned string is emtpy string when enumeration is over.
///
/// The return value is a const pointer to a character string. The pointer
/// is only valid immediately after checking the [chewing_cand_hasNext]
/// condition.
///
/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_cand_String_static(ctx: *mut ChewingContext) -> *const c_char {
    let ctx = as_mut_or_return!(ctx, global_empty_cstr());
    let _logger_guard = init_scoped_logging(ctx.logger_fn, ctx.logger_data);

    match ctx.cand_iter.as_mut().and_then(|it| it.next()) {
        Some(phrase) => copy_cstr(&mut ctx.cand_buf, &phrase),
        None => global_empty_cstr(),
    }
}

/// Returns the candidate string by its index.
///
/// The *index* must be between 0 and [chewing_cand_TotalChoice] inclusive.
///
/// The returned value is a pointer to a character string. The memory must
/// be freed by the caller using function
/// [chewing_free][super::setup::chewing_free].
///
/// # Failures
///
/// This function returns NULL when memory allocation fails.
///
/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_cand_string_by_index(
    ctx: *mut ChewingContext,
    index: c_int,
) -> *mut c_char {
    let ctx = as_mut_or_return!(
        ctx,
        owned_into_raw(Owned::CString, CString::default().into_raw())
    );
    let _logger_guard = init_scoped_logging(ctx.logger_fn, ctx.logger_data);

    if let Ok(phrases) = ctx.editor.all_candidates() {
        if let Some(phrase) = phrases.get(index as usize) {
            return owned_into_raw(
                Owned::CString,
                CString::new(phrase.to_owned())
                    .expect("phrase should be valid UTF-8")
                    .into_raw(),
            );
        }
    }
    owned_into_raw(Owned::CString, CString::default().into_raw())
}

/// Returns the candidate string by its index.
///
/// The *index* must be between 0 and [chewing_cand_TotalChoice] inclusive.
///
/// The return value is a const pointer to a character string. The pointer
/// is only valid immediately after calling this function.
///
/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_cand_string_by_index_static(
    ctx: *mut ChewingContext,
    index: c_int,
) -> *const c_char {
    let ctx = as_mut_or_return!(ctx, global_empty_cstr());
    let _logger_guard = init_scoped_logging(ctx.logger_fn, ctx.logger_data);

    if let Ok(phrases) = ctx.editor.all_candidates() {
        if let Some(phrase) = phrases.get(index as usize) {
            return copy_cstr(&mut ctx.cand_buf, phrase);
        }
    }
    global_empty_cstr()
}

/// Selects the candidate by its index.
///
/// The *index* must be between 0 and [chewing_cand_TotalChoice] inclusive.
///
/// Returns 0 when success, -1 otherwise.
///
/// # Errors
///
/// This function fails if the *index* is out of range or the candidate
/// selection window is not currently open.
///
/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_cand_choose_by_index(
    ctx: *mut ChewingContext,
    index: c_int,
) -> c_int {
    let ctx = as_mut_or_return!(ctx, ERROR);
    let _logger_guard = init_scoped_logging(ctx.logger_fn, ctx.logger_data);

    match ctx.editor.select(index as usize) {
        Ok(_) => OK,
        Err(_) => ERROR,
    }
}

/// Opens the candidate selection window.
///
/// This operation is only allowed when the IM editor is in entering state.
///
/// Returns 0 when success, -1 otherwise.
///
/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_cand_open(ctx: *mut ChewingContext) -> c_int {
    let ctx = as_mut_or_return!(ctx, ERROR);
    let _logger_guard = init_scoped_logging(ctx.logger_fn, ctx.logger_data);

    match ctx.editor.start_selecting() {
        Ok(_) => OK,
        Err(_) => ERROR,
    }
}

/// Closes the candidate selection window.
///
/// Returns 0 when success, -1 otherwise.
///
/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_cand_close(ctx: *mut ChewingContext) -> c_int {
    let ctx = as_mut_or_return!(ctx, ERROR);
    let _logger_guard = init_scoped_logging(ctx.logger_fn, ctx.logger_data);

    match ctx.editor.cancel_selecting() {
        Ok(_) => OK,
        // For backward compatible reason this method never errors
        Err(_) => OK,
    }
}

/// Starts the enumeration of intervals of recognized phrases.
///
/// This function stores an iterator in the context. The iterator is only
/// destroyed after enumerate all intervals using
/// [chewing_interval_hasNext].
///
/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_interval_Enumerate(ctx: *mut ChewingContext) {
    let ctx = as_mut_or_return!(ctx);
    let _logger_guard = init_scoped_logging(ctx.logger_fn, ctx.logger_data);

    ctx.interval_iter = Some(
        (Box::new(ctx.editor.intervals().filter(|it| it.is_phrase))
            as Box<dyn Iterator<Item = Interval>>)
            .peekable(),
    );
}

/// Checks whether there are more intervals or not.
///
/// Returns 1 when true, 0 when false.
///
/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_interval_hasNext(ctx: *mut ChewingContext) -> c_int {
    let ctx = as_mut_or_return!(ctx, ERROR);
    let _logger_guard = init_scoped_logging(ctx.logger_fn, ctx.logger_data);

    ctx.interval_iter
        .as_mut()
        .map_or(FALSE, |it| match it.peek() {
            Some(_) => TRUE,
            None => FALSE,
        })
}

/// Returns the current enumerated interval.
///
/// The *it* argument is an output argument.
///
/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_interval_Get(ctx: *mut ChewingContext, it: *mut IntervalType) {
    let ctx = as_mut_or_return!(ctx);
    let _logger_guard = init_scoped_logging(ctx.logger_fn, ctx.logger_data);

    let it = unsafe {
        match it.as_mut() {
            Some(it) => it,
            None => return,
        }
    };
    if let Some(iter) = &mut ctx.interval_iter {
        if let Some(interval) = iter.next() {
            it.from = interval.start as i32;
            it.to = interval.end as i32;
        }
    }
}

/// Returns whether there is auxiliary string in the auxiliary buffer.
///
/// Returns 1 when true, 0 when false.
///
/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_aux_Check(ctx: *const ChewingContext) -> c_int {
    let ctx = as_ref_or_return!(ctx, ERROR);
    let _logger_guard = init_scoped_logging(ctx.logger_fn, ctx.logger_data);

    !ctx.editor.notification().is_empty() as c_int
}

/// Returns the length of the auxiliary string in the auxiliary buffer.
///
/// <p style="background:rgba(255,181,77,0.16);padding:0.75em;">
/// <strong>⚠ Warning:</strong> The length is calculated in terms of
/// unicode characters. One character might occupy multiple bytes.
/// </p>
///
/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_aux_Length(ctx: *const ChewingContext) -> c_int {
    let ctx = as_ref_or_return!(ctx, ERROR);
    let _logger_guard = init_scoped_logging(ctx.logger_fn, ctx.logger_data);

    ctx.editor.notification().chars().count() as c_int
}

/// Returns the current auxiliary string.
///
/// The returned value is a pointer to a character string. The memory must
/// be freed by the caller using function
/// [chewing_free][super::setup::chewing_free].
///
/// # Failures
///
/// This function returns NULL when memory allocation fails.
///
/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_aux_String(ctx: *const ChewingContext) -> *mut c_char {
    let ctx = match unsafe { ctx.as_ref() } {
        Some(ctx) => ctx,
        None => return owned_into_raw(Owned::CString, CString::default().into_raw()),
    };
    let _logger_guard = init_scoped_logging(ctx.logger_fn, ctx.logger_data);

    let cstring =
        CString::new(ctx.editor.notification()).expect("notification should be valid UTF-8");
    owned_into_raw(Owned::CString, cstring.into_raw())
}

/// Returns the current auxiliary string.
///
/// The return value is a const pointer to a character string. The pointer
/// is only valid immediately after checking the [chewing_aux_Check]
/// condition.
///
/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_aux_String_static(ctx: *const ChewingContext) -> *const c_char {
    let ctx = as_mut_or_return!(ctx.cast_mut(), global_empty_cstr());
    let _logger_guard = init_scoped_logging(ctx.logger_fn, ctx.logger_data);

    copy_cstr(&mut ctx.aux_buf, ctx.editor.notification())
}

/// Checks whether the previous keystroke is ignored or not.
///
/// Returns 1 when true, 0 when false.
///
/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_keystroke_CheckIgnore(ctx: *const ChewingContext) -> c_int {
    let ctx = as_ref_or_return!(ctx, ERROR);
    let _logger_guard = init_scoped_logging(ctx.logger_fn, ctx.logger_data);

    match ctx.editor.last_key_behavior() {
        EditorKeyBehavior::Ignore => TRUE,
        _ => FALSE,
    }
}

/// Checks whether the previous keystroke is absorbed or not.
///
/// Returns 1 when true, 0 when false.
///
/// Absorbed key means the Chewing IM state machine has accepted the key and
/// changed its state accordingly. Caller should check various output
/// buffers to see if they need to update the display.
///
/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_keystroke_CheckAbsorb(ctx: *const ChewingContext) -> c_int {
    let ctx = as_ref_or_return!(ctx, ERROR);
    let _logger_guard = init_scoped_logging(ctx.logger_fn, ctx.logger_data);

    match ctx.editor.last_key_behavior() {
        EditorKeyBehavior::Absorb => TRUE,
        // Historically Absorb and Bell are returned as ABSORiB | BELL so here
        // we should return true.
        EditorKeyBehavior::Bell => TRUE,
        _ => FALSE,
    }
}

/// Returns the number of keyboard layouts supported by the Chewing IM.
///
/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_kbtype_Total(_ctx: *const ChewingContext) -> c_int {
    (0..)
        .map_while(|id| KeyboardLayoutCompat::try_from(id).ok())
        .count() as c_int
}

/// Starts the enumeration of the keyboard layouts.
///
/// This function stores an iterator in the context. The iterator is only
/// destroyed after enumerate all keyboard layouts using
/// [chewing_kbtype_hasNext].
///
/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_kbtype_Enumerate(ctx: *mut ChewingContext) {
    let ctx = as_mut_or_return!(ctx);
    let _logger_guard = init_scoped_logging(ctx.logger_fn, ctx.logger_data);

    ctx.kbcompat_iter = Some(
        (Box::new((0..).map_while(|id| KeyboardLayoutCompat::try_from(id).ok()))
            as Box<dyn Iterator<Item = KeyboardLayoutCompat>>)
            .peekable(),
    )
}

/// Checks whether there are more keyboard layouts to enumerate.
///
/// Returns 1 when there are more and 0 when it's the end of the iterator.
///
/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_kbtype_hasNext(ctx: *mut ChewingContext) -> c_int {
    let ctx = as_mut_or_return!(ctx, ERROR);
    let _logger_guard = init_scoped_logging(ctx.logger_fn, ctx.logger_data);

    ctx.kbcompat_iter
        .as_mut()
        .and_then(|it| it.peek())
        .map_or(0, |_| 1)
}

/// Returns the current enumerated keyboard layout name.
///
/// The returned string is emtpy string when enumeration is over.
///
/// The returned value is a pointer to a character string. The memory must
/// be freed by the caller using function
/// [chewing_free][super::setup::chewing_free].
///
/// # Failures
///
/// This function returns NULL when memory allocation fails.
///
/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_kbtype_String(ctx: *mut ChewingContext) -> *mut c_char {
    let ctx = as_mut_or_return!(
        ctx,
        owned_into_raw(Owned::CString, CString::default().into_raw())
    );
    let _logger_guard = init_scoped_logging(ctx.logger_fn, ctx.logger_data);

    match ctx.kbcompat_iter.as_mut().and_then(|it| it.next()) {
        Some(kb_compat) => {
            let cstr = match CString::new(kb_compat.to_string()) {
                Ok(cstr) => cstr,
                Err(_) => return null_mut(),
            };
            owned_into_raw(Owned::CString, cstr.into_raw())
        }
        None => owned_into_raw(Owned::CString, CString::default().into_raw()),
    }
}

/// Returns the current enumerated keyboard layout name.
///
/// The returned string is emtpy string when enumeration is over.
///
/// The return value is a const pointer to a character string. The pointer
/// is only valid immediately after checking the [chewing_kbtype_hasNext]
/// condition.
///
/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_kbtype_String_static(ctx: *mut ChewingContext) -> *const c_char {
    let ctx = as_mut_or_return!(ctx, global_empty_cstr());
    let _logger_guard = init_scoped_logging(ctx.logger_fn, ctx.logger_data);

    match ctx.kbcompat_iter.as_mut().and_then(|it| it.next()) {
        Some(kb_compat) => copy_cstr(&mut ctx.kbtype_buf, &kb_compat.to_string()),
        None => global_empty_cstr(),
    }
}

/// Returns whether there are phonetic pre-edit string in the buffer. Here
/// “zuin” means bopomofo, a phonetic system for transcribing Chinese,
/// especially Mandarin.
///
/// Returns **0** when true, **1** when false.
///
/// <p style="background:rgba(255,181,77,0.16);padding:0.75em;">
/// <strong>⚠ Warning:</strong> The return value of this function is
/// different from other newer functions that returns boolean value.
/// </p>
///
/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
#[deprecated]
pub unsafe extern "C" fn chewing_zuin_Check(ctx: *const ChewingContext) -> c_int {
    unsafe { chewing_bopomofo_Check(ctx) ^ 1 }
}

/// Returns the phonetic characters in the pre-edit buffer.
///
/// The bopomofo_count argument is a output argument. It will contain the
/// number of phonetic characters in the returned string.
///
/// The returned value is a pointer to a character string. The memory must
/// be freed by the caller using function
/// [chewing_free][super::setup::chewing_free].
///
/// # Failures
///
/// This function returns NULL when memory allocation fails.
///
/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
#[deprecated]
pub unsafe extern "C" fn chewing_zuin_String(
    ctx: *const ChewingContext,
    zuin_count: *mut c_int,
) -> *mut c_char {
    let ctx = as_ref_or_return!(
        ctx,
        owned_into_raw(Owned::CString, CString::default().into_raw())
    );
    let _logger_guard = init_scoped_logging(ctx.logger_fn, ctx.logger_data);

    let syllable = ctx.editor.syllable_buffer_display();
    unsafe {
        *zuin_count = syllable.chars().count() as c_int;
    }
    let cstr = match CString::new(syllable) {
        Ok(cstr) => cstr,
        Err(_) => return null_mut(),
    };
    owned_into_raw(Owned::CString, cstr.into_raw())
}

/// This function exists only for backword compatibility.
///
/// The `chewing_Init` function is no-op now. The return value is always 0.
#[unsafe(no_mangle)]
#[deprecated]
pub unsafe extern "C" fn chewing_Init(data_path: *const c_char, hash_path: *const c_char) -> c_int {
    let _ = hash_path;
    let _ = data_path;
    OK
}

/// This function exists only for backword compatibility.
#[unsafe(no_mangle)]
#[deprecated]
pub unsafe extern "C" fn chewing_Terminate() {}

/// Sets the selectAreaLen, maxChiSymbolLen and selKey parameter from pcd.
///
/// The pcd argument is a pointer to a Chewing configuration data structure.
/// See also the ChewingConfigData data type.
///
/// The return value is 0 on success and -1 on failure.
///
/// **Deprecated**, use the chewing_set_* function series to set parameters
/// instead.
///
/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
#[deprecated]
pub unsafe extern "C" fn chewing_Configure(
    ctx: *mut ChewingContext,
    pcd: *mut ChewingConfigData,
) -> c_int {
    let pcd = match unsafe { pcd.as_ref() } {
        Some(pcd) => pcd,
        None => return -1,
    };

    unsafe {
        chewing_set_candPerPage(ctx, pcd.cand_per_page);
        chewing_set_maxChiSymbolLen(ctx, pcd.max_chi_symbol_len);
        chewing_set_selKey(ctx, pcd.sel_key.as_ptr(), MAX_SELKEY as i32);
        chewing_set_addPhraseDirection(ctx, pcd.b_add_phrase_forward);
        chewing_set_spaceAsSelection(ctx, pcd.b_space_as_selection);
        chewing_set_escCleanAllBuf(ctx, pcd.b_esc_clean_all_buf);
        chewing_set_autoShiftCur(ctx, pcd.b_auto_shift_cur);
        chewing_set_easySymbolInput(ctx, pcd.b_easy_symbol_input);
        chewing_set_phraseChoiceRearward(ctx, pcd.b_phrase_choice_rearward);
    }
    OK
}

/// This function is no-op now. Use [chewing_set_selKey] instead.
///
/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
#[deprecated]
pub unsafe extern "C" fn chewing_set_hsuSelKeyType(_ctx: *mut ChewingContext, mode: c_int) {
    let _ = mode;
}

/// This function is no-op now. Use [chewing_get_selKey] instead.
///
/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
#[deprecated]
pub unsafe extern "C" fn chewing_get_hsuSelKeyType(_ctx: *mut ChewingContext) -> c_int {
    OK
}
