use std::{
    cmp::min,
    collections::BTreeMap,
    ffi::{CStr, CString, c_char, c_int, c_uint, c_ushort, c_void},
    mem,
    ptr::{null, null_mut},
    slice, str,
    sync::RwLock,
};

use chewing::{
    conversion::{ChewingEngine, FuzzyChewingEngine, Interval, SimpleEngine, Symbol},
    dictionary::{
        Dictionary, Layered, LookupStrategy, SystemDictionaryLoader, Trie, UserDictionaryLoader,
    },
    editor::{
        AbbrevTable, BasicEditor, CharacterForm, ConversionEngineKind, Editor, EditorKeyBehavior,
        LanguageMode, LaxUserFreqEstimate, SymbolSelector, UserPhraseAddDirection,
        keyboard::{AnyKeyboardLayout, KeyCode, KeyboardLayout, Modifiers, Qwerty},
        zhuyin_layout::{
            DaiChien26, Et, Et26, GinYieh, Hsu, Ibm, KeyboardLayoutCompat, Pinyin, Standard,
            SyllableEditor,
        },
    },
    zhuyin::Syllable,
};
use log::{debug, error, info};

use crate::public::{
    CHEWING_CONVERSION_ENGINE, CHINESE_MODE, ChewingConfigData, ChewingContext, FULLSHAPE_MODE,
    FUZZY_CHEWING_CONVERSION_ENGINE, HALFSHAPE_MODE, IntervalType, MAX_SELKEY,
    SIMPLE_CONVERSION_ENGINE, SYMBOL_MODE, SelKeys,
};

use super::logger::ChewingLogger;

const TRUE: c_int = 1;
const FALSE: c_int = 0;
const OK: c_int = 0;
const ERROR: c_int = -1;

static LOGGER: ChewingLogger = ChewingLogger::new();

enum Owned {
    CString,
    CUShortSlice(usize),
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

#[unsafe(no_mangle)]
pub extern "C" fn chewing_new() -> *mut ChewingContext {
    unsafe { chewing_new2(null(), null(), None, null_mut()) }
}

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
    LOGGER.init();
    let _ = log::set_logger(&LOGGER);
    log::set_max_level(log::LevelFilter::Trace);
    if let Some(logger) = logger {
        LOGGER.set(Some((logger, loggerdata)));
    }
    let mut sys_loader = SystemDictionaryLoader::new();
    if !syspath.is_null() {
        if let Ok(search_path) = unsafe { CStr::from_ptr(syspath).to_str() } {
            sys_loader = sys_loader.sys_path(search_path);
        }
    }
    let dictionaries = match sys_loader.load() {
        Ok(d) => d,
        Err(e) => {
            let builtin = Trie::new(&include_bytes!("../data/mini.dat")[..]);
            error!("Failed to load system dict: {e}");
            error!("Loading builtin minimum dictionary...");
            // NB: we can unwrap because the built-in dictionary should always
            // be valid.
            vec![Box::new(builtin.unwrap()) as Box<dyn Dictionary>]
        }
    };
    let drop_in_dicts = sys_loader.load_drop_in().unwrap_or_default();
    let abbrev = sys_loader.load_abbrev();
    let abbrev = match abbrev {
        Ok(abbr) => abbr,
        Err(e) => {
            error!("Failed to load abbrev table: {e}");
            error!("Loading empty table...");
            AbbrevTable::new()
        }
    };
    let sym_sel = sys_loader.load_symbol_selector();
    let sym_sel = match sym_sel {
        Ok(sym_sel) => sym_sel,
        Err(e) => {
            error!("Failed to load symbol table: {e}");
            error!("Loading empty table...");
            // NB: we can unwrap here because empty table is always valid.
            SymbolSelector::new(b"".as_slice()).unwrap()
        }
    };
    let mut user_dictionary = UserDictionaryLoader::new();
    if !userpath.is_null() {
        if let Ok(data_path) = unsafe { CStr::from_ptr(userpath).to_str() } {
            user_dictionary = user_dictionary.userphrase_path(data_path);
        }
    }
    let user_dictionary = match user_dictionary.load() {
        Ok(d) => d,
        Err(e) => {
            error!("Failed to load user dict: {e}");
            UserDictionaryLoader::in_memory()
        }
    };

    let estimate = LaxUserFreqEstimate::max_from(user_dictionary.as_ref());

    let system_dicts = Vec::from_iter(dictionaries.into_iter().chain(drop_in_dicts));
    let dict = Layered::new(system_dicts, user_dictionary);
    let conversion_engine = Box::new(ChewingEngine::new());
    let kb_compat = KeyboardLayoutCompat::Default;
    let keyboard = AnyKeyboardLayout::Qwerty(Qwerty);
    let editor = Editor::new(conversion_engine, dict, estimate, abbrev, sym_sel);
    let context = Box::new(ChewingContext {
        kb_compat,
        keyboard,
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
    });
    let ptr = Box::into_raw(context);
    info!("Initialized context {ptr:?}");
    ptr
}

/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_delete(ctx: *mut ChewingContext) {
    if !ctx.is_null() {
        LOGGER.set(None);
        info!("Destroying context {ctx:?}");
        drop(unsafe { Box::from_raw(ctx) })
    }
}

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
                        drop(unsafe { Vec::from_raw_parts(ptr, *len, *len) })
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

/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_Reset(ctx: *mut ChewingContext) -> c_int {
    let ctx = as_mut_or_return!(ctx, ERROR);
    ctx.editor.clear();
    OK
}

/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_ack(ctx: *mut ChewingContext) -> c_int {
    let ctx = as_mut_or_return!(ctx, ERROR);
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
    let _ctx = as_ref_or_return!(ctx, ERROR);
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
        _ => return ERROR,
    };

    ctx.editor.set_editor_options(options);

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
    let cstr = unsafe { CStr::from_ptr(name) };
    let name = cstr.to_string_lossy();
    let cstr = unsafe { CStr::from_ptr(value) };
    let string = cstr.to_string_lossy();

    let _option = &mut ctx.editor.editor_options();

    match name.as_ref() {
        "chewing.keyboard_type" => {
            use KeyboardLayoutCompat as KB;
            ctx.kb_compat = match string.parse() {
                Ok(kbtype) => kbtype,
                Err(_) => return ERROR,
            };
            let (keyboard, syl): (AnyKeyboardLayout, Box<dyn SyllableEditor>) = match ctx.kb_compat
            {
                KB::Default => (AnyKeyboardLayout::qwerty(), Box::new(Standard::new())),
                KB::Hsu => (AnyKeyboardLayout::qwerty(), Box::new(Hsu::new())),
                KB::Ibm => (AnyKeyboardLayout::qwerty(), Box::new(Ibm::new())),
                KB::GinYieh => (AnyKeyboardLayout::qwerty(), Box::new(GinYieh::new())),
                KB::Et => (AnyKeyboardLayout::qwerty(), Box::new(Et::new())),
                KB::Et26 => (AnyKeyboardLayout::qwerty(), Box::new(Et26::new())),
                KB::Dvorak => (AnyKeyboardLayout::qwerty(), Box::new(Standard::new())),
                KB::DvorakHsu => (AnyKeyboardLayout::qwerty(), Box::new(Hsu::new())),
                KB::DachenCp26 => (AnyKeyboardLayout::qwerty(), Box::new(DaiChien26::new())),
                KB::HanyuPinyin => (AnyKeyboardLayout::qwerty(), Box::new(Pinyin::hanyu())),
                KB::ThlPinyin => (AnyKeyboardLayout::qwerty(), Box::new(Pinyin::thl())),
                KB::Mps2Pinyin => (AnyKeyboardLayout::qwerty(), Box::new(Pinyin::mps2())),
                KB::Carpalx => (AnyKeyboardLayout::qwerty(), Box::new(Standard::new())),
                KB::Colemak => (AnyKeyboardLayout::colemak(), Box::new(Standard::new())),
                KB::ColemakDhAnsi => (
                    AnyKeyboardLayout::colemak_dh_ansi(),
                    Box::new(Standard::new()),
                ),
                KB::ColemakDhOrth => (
                    AnyKeyboardLayout::colemak_dh_orth(),
                    Box::new(Standard::new()),
                ),
                KB::Workman => (AnyKeyboardLayout::workman(), Box::new(Standard::new())),
            };
            ctx.keyboard = keyboard;
            ctx.editor.set_syllable_editor(syl);
        }
        "chewing.selection_keys" => {
            if string.len() != 10 {
                return ERROR;
            }
            let mut sel_keys = [0_i32; MAX_SELKEY];
            string
                .chars()
                .enumerate()
                .for_each(|(i, key)| sel_keys[i] = key as i32);
            ctx.sel_keys = SelKeys(sel_keys)
        }
        _ => return ERROR,
    };

    OK
}

/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_set_KBType(ctx: *mut ChewingContext, kbtype: c_int) -> c_int {
    let ctx = as_mut_or_return!(ctx, ERROR);
    use KeyboardLayoutCompat as KB;
    let kb_compat = match KB::try_from(kbtype as u8) {
        Ok(kb) => kb,
        Err(()) => KB::Default,
    };
    let (keyboard, syl): (AnyKeyboardLayout, Box<dyn SyllableEditor>) = match kb_compat {
        KB::Default => (AnyKeyboardLayout::qwerty(), Box::new(Standard::new())),
        KB::Hsu => (AnyKeyboardLayout::qwerty(), Box::new(Hsu::new())),
        KB::Ibm => (AnyKeyboardLayout::qwerty(), Box::new(Ibm::new())),
        KB::GinYieh => (AnyKeyboardLayout::qwerty(), Box::new(GinYieh::new())),
        KB::Et => (AnyKeyboardLayout::qwerty(), Box::new(Et::new())),
        KB::Et26 => (AnyKeyboardLayout::qwerty(), Box::new(Et26::new())),
        KB::Dvorak => (AnyKeyboardLayout::dvorak(), Box::new(Standard::new())),
        KB::DvorakHsu => (AnyKeyboardLayout::dvorak_on_qwerty(), Box::new(Hsu::new())),
        KB::DachenCp26 => (AnyKeyboardLayout::qwerty(), Box::new(DaiChien26::new())),
        KB::HanyuPinyin => (AnyKeyboardLayout::qwerty(), Box::new(Pinyin::hanyu())),
        KB::ThlPinyin => (AnyKeyboardLayout::qwerty(), Box::new(Pinyin::thl())),
        KB::Mps2Pinyin => (AnyKeyboardLayout::qwerty(), Box::new(Pinyin::mps2())),
        KB::Carpalx => (AnyKeyboardLayout::qwerty(), Box::new(Standard::new())),
        KB::Colemak => (AnyKeyboardLayout::colemak(), Box::new(Standard::new())),
        KB::ColemakDhAnsi => (
            AnyKeyboardLayout::colemak_dh_ansi(),
            Box::new(Standard::new()),
        ),
        KB::ColemakDhOrth => (
            AnyKeyboardLayout::colemak_dh_orth(),
            Box::new(Standard::new()),
        ),
        KB::Workman => (AnyKeyboardLayout::workman(), Box::new(Standard::new())),
    };
    ctx.kb_compat = kb_compat;
    ctx.keyboard = keyboard;
    ctx.editor.set_syllable_editor(syl);
    if kb_compat == KB::Default && kb_compat as c_int != kbtype {
        -1
    } else {
        0
    }
}

/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_get_KBType(ctx: *const ChewingContext) -> c_int {
    let ctx = as_ref_or_return!(ctx, ERROR);
    ctx.kb_compat as c_int
}

/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_get_KBString(ctx: *const ChewingContext) -> *mut c_char {
    let ctx = as_ref_or_return!(
        ctx,
        owned_into_raw(Owned::CString, CString::default().into_raw())
    );

    let kb_string = ctx.kb_compat.to_string();
    owned_into_raw(
        Owned::CString,
        CString::new(kb_string)
            .expect("should have valid kb_string")
            .into_raw(),
    )
}

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

/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_set_ChiEngMode(ctx: *mut ChewingContext, mode: c_int) {
    unsafe { chewing_config_set_int(ctx, c"chewing.language_mode".as_ptr().cast(), mode) };
}

/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_get_ChiEngMode(ctx: *const ChewingContext) -> c_int {
    unsafe { chewing_config_get_int(ctx, c"chewing.language_mode".as_ptr().cast()) }
}

/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_set_ShapeMode(ctx: *mut ChewingContext, mode: c_int) {
    unsafe { chewing_config_set_int(ctx, c"chewing.character_form".as_ptr().cast(), mode) };
}

/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_get_ShapeMode(ctx: *const ChewingContext) -> c_int {
    unsafe { chewing_config_get_int(ctx, c"chewing.character_form".as_ptr().cast()) }
}

/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_set_candPerPage(ctx: *mut ChewingContext, n: c_int) {
    unsafe { chewing_config_set_int(ctx, c"chewing.candidates_per_page".as_ptr().cast(), n) };
}

/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_get_candPerPage(ctx: *const ChewingContext) -> c_int {
    unsafe { chewing_config_get_int(ctx, c"chewing.candidates_per_page".as_ptr().cast()) }
}

/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_set_maxChiSymbolLen(ctx: *mut ChewingContext, n: c_int) {
    unsafe { chewing_config_set_int(ctx, c"chewing.auto_commit_threshold".as_ptr().cast(), n) };
}

/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_get_maxChiSymbolLen(ctx: *const ChewingContext) -> c_int {
    unsafe { chewing_config_get_int(ctx, c"chewing.auto_commit_threshold".as_ptr().cast()) }
}

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

    if sel_keys.is_null() || len != 10 {
        return;
    }

    let sel_keys = unsafe { slice::from_raw_parts(sel_keys, len as usize) };
    ctx.sel_keys.0.copy_from_slice(sel_keys);
}

/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_get_selKey(ctx: *const ChewingContext) -> *mut c_int {
    let ctx = as_ref_or_return!(ctx, null_mut());

    ctx.sel_keys.0.as_ptr().cast_mut()
}

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

/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_get_addPhraseDirection(ctx: *const ChewingContext) -> c_int {
    unsafe { chewing_config_get_int(ctx, c"chewing.user_phrase_add_direction".as_ptr().cast()) }
}

/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_set_spaceAsSelection(ctx: *mut ChewingContext, mode: c_int) {
    unsafe { chewing_config_set_int(ctx, c"chewing.space_is_select_key".as_ptr().cast(), mode) };
}

/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_get_spaceAsSelection(ctx: *const ChewingContext) -> c_int {
    unsafe { chewing_config_get_int(ctx, c"chewing.space_is_select_key".as_ptr().cast()) }
}

/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_set_escCleanAllBuf(ctx: *mut ChewingContext, mode: c_int) {
    unsafe { chewing_config_set_int(ctx, c"chewing.esc_clear_all_buffer".as_ptr().cast(), mode) };
}

/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_get_escCleanAllBuf(ctx: *const ChewingContext) -> c_int {
    unsafe { chewing_config_get_int(ctx, c"chewing.esc_clear_all_buffer".as_ptr().cast()) }
}

/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_set_autoShiftCur(ctx: *mut ChewingContext, mode: c_int) {
    unsafe { chewing_config_set_int(ctx, c"chewing.auto_shift_cursor".as_ptr().cast(), mode) };
}

/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_get_autoShiftCur(ctx: *const ChewingContext) -> c_int {
    unsafe { chewing_config_get_int(ctx, c"chewing.auto_shift_cursor".as_ptr().cast()) }
}

/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_set_easySymbolInput(ctx: *mut ChewingContext, mode: c_int) {
    unsafe { chewing_config_set_int(ctx, c"chewing.easy_symbol_input".as_ptr().cast(), mode) };
}

/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_get_easySymbolInput(ctx: *const ChewingContext) -> c_int {
    unsafe { chewing_config_get_int(ctx, c"chewing.easy_symbol_input".as_ptr().cast()) }
}

/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_set_phraseChoiceRearward(ctx: *mut ChewingContext, mode: c_int) {
    unsafe { chewing_config_set_int(ctx, c"chewing.phrase_choice_rearward".as_ptr().cast(), mode) };
}

/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_get_phraseChoiceRearward(ctx: *const ChewingContext) -> c_int {
    unsafe { chewing_config_get_int(ctx, c"chewing.phrase_choice_rearward".as_ptr().cast()) }
}

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

/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_get_autoLearn(ctx: *const ChewingContext) -> c_int {
    unsafe { chewing_config_get_int(ctx, c"chewing.disable_auto_learn_phrase".as_ptr().cast()) }
}

/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_get_phoneSeq(ctx: *const ChewingContext) -> *mut c_ushort {
    let ctx = as_ref_or_return!(ctx, null_mut());

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

/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_get_phoneSeqLen(ctx: *const ChewingContext) -> c_int {
    let ctx = as_ref_or_return!(ctx, ERROR);

    ctx.editor
        .symbols()
        .iter()
        .cloned()
        .filter(Symbol::is_syllable)
        .count() as c_int
}

/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_set_logger(
    ctx: *mut ChewingContext,
    logger: Option<extern "C" fn(data: *mut c_void, level: c_int, fmt: *const c_char, ...)>,
    data: *mut c_void,
) {
    as_mut_or_return!(ctx);
    if let Some(logger) = logger {
        log::set_max_level(log::LevelFilter::Trace);
        LOGGER.set(Some((logger, data)));
    } else {
        log::set_max_level(log::LevelFilter::Off);
        LOGGER.set(None);
    }
}

/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_userphrase_enumerate(ctx: *mut ChewingContext) -> c_int {
    let ctx = as_mut_or_return!(ctx, ERROR);

    ctx.userphrase_iter = Some(ctx.editor.user_dict().entries().peekable());
    OK
}

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

/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_userphrase_remove(
    ctx: *mut ChewingContext,
    phrase_buf: *const c_char,
    bopomofo_buf: *const c_char,
) -> c_int {
    let ctx = match unsafe { ctx.as_mut() } {
        Some(ctx) => ctx,
        None => return ERROR,
    };

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
            .lookup_all_phrases(&syllables, LookupStrategy::Standard)
            .iter()
            .any(|ph| ph.as_str() == phrase) as c_int,
        None => ctx
            .editor
            .user_dict()
            .lookup_first_phrase(&syllables, LookupStrategy::Standard)
            .is_some() as c_int,
    }
}

/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_cand_list_first(ctx: *mut ChewingContext) -> c_int {
    let ctx = as_mut_or_return!(ctx, ERROR);

    if !ctx.editor.is_selecting() {
        return -1;
    }

    let _ = ctx.editor.jump_to_first_selection_point();
    OK
}

/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_cand_list_last(ctx: *mut ChewingContext) -> c_int {
    let ctx = as_mut_or_return!(ctx, ERROR);

    if !ctx.editor.is_selecting() {
        return -1;
    }

    let _ = ctx.editor.jump_to_last_selection_point();
    OK
}

/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_cand_list_has_next(ctx: *mut ChewingContext) -> c_int {
    let ctx = as_ref_or_return!(ctx, FALSE);

    if !ctx.editor.is_selecting() {
        return 0;
    }

    ctx.editor.has_next_selection_point() as c_int
}

/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_cand_list_has_prev(ctx: *mut ChewingContext) -> c_int {
    let ctx = as_ref_or_return!(ctx, FALSE);

    if !ctx.editor.is_selecting() {
        return 0;
    }

    ctx.editor.has_prev_selection_point() as c_int
}

/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_cand_list_next(ctx: *mut ChewingContext) -> c_int {
    let ctx = as_mut_or_return!(ctx, ERROR);
    if !ctx.editor.is_selecting() {
        return -1;
    }
    match ctx.editor.jump_to_next_selection_point() {
        Ok(_) => OK,
        Err(_) => ERROR,
    }
}

/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_cand_list_prev(ctx: *mut ChewingContext) -> c_int {
    let ctx = as_mut_or_return!(ctx, ERROR);
    if !ctx.editor.is_selecting() {
        return -1;
    }
    match ctx.editor.jump_to_prev_selection_point() {
        Ok(_) => OK,
        Err(_) => ERROR,
    }
}

/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_commit_preedit_buf(ctx: *mut ChewingContext) -> c_int {
    let ctx = as_mut_or_return!(ctx, ERROR);

    match ctx.editor.commit() {
        Ok(_) => OK,
        Err(_) => ERROR,
    }
}

/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_clean_preedit_buf(ctx: *mut ChewingContext) -> c_int {
    let ctx = as_mut_or_return!(ctx, ERROR);

    if !ctx.editor.is_entering() {
        ERROR
    } else {
        ctx.editor.clear();
        OK
    }
}

/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_clean_bopomofo_buf(ctx: *mut ChewingContext) -> c_int {
    let ctx = as_mut_or_return!(ctx, ERROR);

    ctx.editor.clear_syllable_editor();
    OK
}

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

/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_handle_Space(ctx: *mut ChewingContext) -> c_int {
    let ctx = as_mut_or_return!(ctx, ERROR);

    ctx.editor
        .process_keyevent(ctx.keyboard.map(KeyCode::Space));
    OK
}

/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_handle_Esc(ctx: *mut ChewingContext) -> c_int {
    let ctx = as_mut_or_return!(ctx, ERROR);

    ctx.editor.process_keyevent(ctx.keyboard.map(KeyCode::Esc));
    OK
}

/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_handle_Enter(ctx: *mut ChewingContext) -> c_int {
    let ctx = as_mut_or_return!(ctx, ERROR);

    ctx.editor
        .process_keyevent(ctx.keyboard.map(KeyCode::Enter));
    OK
}

/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_handle_Del(ctx: *mut ChewingContext) -> c_int {
    let ctx = as_mut_or_return!(ctx, ERROR);

    ctx.editor.process_keyevent(ctx.keyboard.map(KeyCode::Del));
    OK
}

/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_handle_Backspace(ctx: *mut ChewingContext) -> c_int {
    let ctx = as_mut_or_return!(ctx, ERROR);

    ctx.editor
        .process_keyevent(ctx.keyboard.map(KeyCode::Backspace));
    OK
}

/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_handle_Tab(ctx: *mut ChewingContext) -> c_int {
    let ctx = as_mut_or_return!(ctx, ERROR);

    ctx.editor.process_keyevent(ctx.keyboard.map(KeyCode::Tab));
    OK
}

/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_handle_ShiftLeft(ctx: *mut ChewingContext) -> c_int {
    let ctx = as_mut_or_return!(ctx, ERROR);

    ctx.editor
        .process_keyevent(ctx.keyboard.map_with_mod(KeyCode::Left, Modifiers::shift()));
    OK
}

/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_handle_Left(ctx: *mut ChewingContext) -> c_int {
    let ctx = as_mut_or_return!(ctx, ERROR);

    let key_event = ctx.keyboard.map(KeyCode::Left);
    ctx.editor.process_keyevent(key_event);
    OK
}

/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_handle_ShiftRight(ctx: *mut ChewingContext) -> c_int {
    let ctx = as_mut_or_return!(ctx, ERROR);

    ctx.editor.process_keyevent(
        ctx.keyboard
            .map_with_mod(KeyCode::Right, Modifiers::shift()),
    );
    OK
}

/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_handle_Right(ctx: *mut ChewingContext) -> c_int {
    let ctx = as_mut_or_return!(ctx, ERROR);

    ctx.editor
        .process_keyevent(ctx.keyboard.map(KeyCode::Right));
    OK
}

/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_handle_Up(ctx: *mut ChewingContext) -> c_int {
    let ctx = as_mut_or_return!(ctx, ERROR);

    ctx.editor.process_keyevent(ctx.keyboard.map(KeyCode::Up));
    OK
}

/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_handle_Home(ctx: *mut ChewingContext) -> c_int {
    let ctx = as_mut_or_return!(ctx, ERROR);

    ctx.editor.process_keyevent(ctx.keyboard.map(KeyCode::Home));
    OK
}

/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_handle_End(ctx: *mut ChewingContext) -> c_int {
    let ctx = as_mut_or_return!(ctx, ERROR);

    ctx.editor.process_keyevent(ctx.keyboard.map(KeyCode::End));
    OK
}

/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_handle_PageUp(ctx: *mut ChewingContext) -> c_int {
    let ctx = as_mut_or_return!(ctx, ERROR);

    ctx.editor
        .process_keyevent(ctx.keyboard.map(KeyCode::PageUp));
    OK
}

/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_handle_PageDown(ctx: *mut ChewingContext) -> c_int {
    let ctx = as_mut_or_return!(ctx, ERROR);

    ctx.editor
        .process_keyevent(ctx.keyboard.map(KeyCode::PageDown));
    OK
}

/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_handle_Down(ctx: *mut ChewingContext) -> c_int {
    let ctx = as_mut_or_return!(ctx, ERROR);

    ctx.editor.process_keyevent(ctx.keyboard.map(KeyCode::Down));
    OK
}

/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_handle_Capslock(ctx: *mut ChewingContext) -> c_int {
    let ctx = as_mut_or_return!(ctx, ERROR);

    ctx.editor.process_keyevent(
        ctx.keyboard
            .map_with_mod(KeyCode::Unknown, Modifiers::capslock()),
    );
    OK
}

/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_handle_Default(ctx: *mut ChewingContext, key: c_int) -> c_int {
    let ctx = as_mut_or_return!(ctx, ERROR);

    // XXX hack for selkey
    let key = if ctx.editor.is_selecting() {
        match ctx.sel_keys.0.iter().position(|&it| it == key) {
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
                key as c_int
            }
            None => key,
        }
    } else {
        key
    };

    ctx.editor
        .process_keyevent(ctx.keyboard.map_ascii(key as u8));
    OK
}

/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_handle_CtrlNum(ctx: *mut ChewingContext, key: c_int) -> c_int {
    let ctx = as_mut_or_return!(ctx, ERROR);

    let keycode = match key as u8 {
        b'0' => KeyCode::N0,
        b'1' => KeyCode::N1,
        b'2' => KeyCode::N2,
        b'3' => KeyCode::N3,
        b'4' => KeyCode::N4,
        b'5' => KeyCode::N5,
        b'6' => KeyCode::N6,
        b'7' => KeyCode::N7,
        b'8' => KeyCode::N8,
        b'9' => KeyCode::N9,
        _ => return -1,
    };

    ctx.editor
        .process_keyevent(ctx.keyboard.map_with_mod(keycode, Modifiers::control()));
    OK
}

/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_handle_ShiftSpace(ctx: *mut ChewingContext) -> c_int {
    let ctx = as_mut_or_return!(ctx, ERROR);

    ctx.editor.process_keyevent(
        ctx.keyboard
            .map_with_mod(KeyCode::Space, Modifiers::shift()),
    );
    OK
}

/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_handle_DblTab(ctx: *mut ChewingContext) -> c_int {
    let _ctx = as_mut_or_return!(ctx, ERROR);

    // todo!()
    OK
}

/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_handle_Numlock(ctx: *mut ChewingContext, key: c_int) -> c_int {
    let ctx = as_mut_or_return!(ctx, ERROR);

    ctx.editor
        .process_keyevent(ctx.keyboard.map_ascii_numlock(key as u8));
    OK
}

/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_commit_Check(ctx: *const ChewingContext) -> c_int {
    let ctx = as_ref_or_return!(ctx, ERROR);

    !ctx.editor.display_commit().is_empty() as c_int
}

/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_commit_String(ctx: *const ChewingContext) -> *mut c_char {
    let ctx = as_ref_or_return!(
        ctx,
        owned_into_raw(Owned::CString, CString::default().into_raw())
    );

    let buffer = ctx.editor.display_commit();
    let cstr = match CString::new(buffer) {
        Ok(cstr) => cstr,
        Err(_) => return null_mut(),
    };
    owned_into_raw(Owned::CString, cstr.into_raw())
}

/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_commit_String_static(ctx: *const ChewingContext) -> *const c_char {
    let ctx = as_mut_or_return!(ctx.cast_mut(), global_empty_cstr());

    copy_cstr(&mut ctx.commit_buf, ctx.editor.display_commit())
}

/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_buffer_String(ctx: *const ChewingContext) -> *mut c_char {
    let ctx = as_ref_or_return!(
        ctx,
        owned_into_raw(Owned::CString, CString::default().into_raw())
    );

    let buffer = ctx.editor.display();
    let cstr = match CString::new(buffer) {
        Ok(cstr) => cstr,
        Err(_) => return null_mut(),
    };
    owned_into_raw(Owned::CString, cstr.into_raw())
}

/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_buffer_String_static(ctx: *const ChewingContext) -> *const c_char {
    let ctx = as_mut_or_return!(ctx.cast_mut(), global_empty_cstr());

    copy_cstr(&mut ctx.preedit_buf, &ctx.editor.display())
}

/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_buffer_Check(ctx: *const ChewingContext) -> c_int {
    let ctx = as_ref_or_return!(ctx, ERROR);

    !ctx.editor.is_empty() as c_int
}

/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_buffer_Len(ctx: *const ChewingContext) -> c_int {
    let ctx = as_ref_or_return!(ctx, ERROR);

    ctx.editor.len() as c_int
}

/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_bopomofo_String_static(
    ctx: *const ChewingContext,
) -> *const c_char {
    let ctx = as_mut_or_return!(ctx.cast_mut(), global_empty_cstr());

    copy_cstr(&mut ctx.bopomofo_buf, &ctx.editor.syllable_buffer_display())
}

/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_bopomofo_String(ctx: *const ChewingContext) -> *mut c_char {
    let ctx = as_ref_or_return!(
        ctx,
        owned_into_raw(Owned::CString, CString::default().into_raw())
    );

    let buffer = ctx.editor.syllable_buffer_display();
    let cstr = match CString::new(buffer) {
        Ok(cstr) => cstr,
        Err(_) => return null_mut(),
    };
    owned_into_raw(Owned::CString, cstr.into_raw())
}

/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_bopomofo_Check(ctx: *const ChewingContext) -> c_int {
    let ctx = as_ref_or_return!(ctx, ERROR);

    ctx.editor.entering_syllable() as c_int
}

/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_cursor_Current(ctx: *const ChewingContext) -> c_int {
    let ctx = as_ref_or_return!(ctx, ERROR);

    ctx.editor.cursor() as c_int
}

#[deprecated(note = "The chewing_cand_TotalPage function could achieve the same effect.")]
/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_cand_CheckDone(ctx: *const ChewingContext) -> c_int {
    let ctx = as_ref_or_return!(ctx, ERROR);

    if ctx.editor.is_selecting() {
        FALSE
    } else {
        TRUE
    }
}

/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_cand_TotalPage(ctx: *const ChewingContext) -> c_int {
    let ctx = as_ref_or_return!(ctx, ERROR);

    ctx.editor.total_page().unwrap_or_default() as c_int
}

/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_cand_ChoicePerPage(ctx: *const ChewingContext) -> c_int {
    let ctx = as_ref_or_return!(ctx, ERROR);

    ctx.editor.editor_options().candidates_per_page as c_int
}

/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_cand_TotalChoice(ctx: *const ChewingContext) -> c_int {
    let ctx = as_ref_or_return!(ctx, ERROR);

    match ctx.editor.all_candidates() {
        Ok(candidates) => candidates.len() as c_int,
        Err(_) => 0,
    }
}

/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_cand_CurrentPage(ctx: *const ChewingContext) -> c_int {
    let ctx = as_ref_or_return!(ctx, ERROR);

    ctx.editor.current_page_no().unwrap_or_default() as c_int
}

/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_cand_Enumerate(ctx: *mut ChewingContext) {
    let ctx = as_mut_or_return!(ctx);

    if let Ok(candidates) = ctx.editor.paginated_candidates() {
        debug!("candidates: {candidates:?}");
        let phrases = Box::new(candidates.into_iter()) as Box<dyn Iterator<Item = String>>;
        ctx.cand_iter = Some(phrases.peekable());
    }
}

/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_cand_hasNext(ctx: *mut ChewingContext) -> c_int {
    let ctx = as_mut_or_return!(ctx, ERROR);

    if !ctx.editor.is_selecting() {
        return FALSE;
    }

    ctx.cand_iter
        .as_mut()
        .and_then(|it| it.peek())
        .map_or(0, |_| 1)
}

/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_cand_String(ctx: *mut ChewingContext) -> *mut c_char {
    let ctx = as_mut_or_return!(
        ctx,
        owned_into_raw(Owned::CString, CString::default().into_raw())
    );

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

/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_cand_String_static(ctx: *mut ChewingContext) -> *const c_char {
    let ctx = as_mut_or_return!(ctx, global_empty_cstr());

    match ctx.cand_iter.as_mut().and_then(|it| it.next()) {
        Some(phrase) => copy_cstr(&mut ctx.cand_buf, &phrase),
        None => global_empty_cstr(),
    }
}

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

/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_cand_string_by_index_static(
    ctx: *mut ChewingContext,
    index: c_int,
) -> *const c_char {
    let ctx = as_mut_or_return!(ctx, global_empty_cstr());

    if let Ok(phrases) = ctx.editor.all_candidates() {
        if let Some(phrase) = phrases.get(index as usize) {
            return copy_cstr(&mut ctx.cand_buf, phrase);
        }
    }
    global_empty_cstr()
}

/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_cand_choose_by_index(
    ctx: *mut ChewingContext,
    index: c_int,
) -> c_int {
    let ctx = as_mut_or_return!(ctx, ERROR);

    match ctx.editor.select(index as usize) {
        Ok(_) => OK,
        Err(_) => ERROR,
    }
}

/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_cand_open(ctx: *mut ChewingContext) -> c_int {
    let ctx = as_mut_or_return!(ctx, ERROR);

    match ctx.editor.start_selecting() {
        Ok(_) => OK,
        Err(_) => ERROR,
    }
}

/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_cand_close(ctx: *mut ChewingContext) -> c_int {
    let ctx = as_mut_or_return!(ctx, ERROR);

    match ctx.editor.cancel_selecting() {
        Ok(_) => OK,
        // For backward compatible reason this method never errors
        Err(_) => OK,
    }
}

/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_interval_Enumerate(ctx: *mut ChewingContext) {
    let ctx = as_mut_or_return!(ctx);

    ctx.interval_iter = Some(
        (Box::new(ctx.editor.intervals().filter(|it| it.is_phrase))
            as Box<dyn Iterator<Item = Interval>>)
            .peekable(),
    );
}

/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_interval_hasNext(ctx: *mut ChewingContext) -> c_int {
    let ctx = as_mut_or_return!(ctx, ERROR);

    ctx.interval_iter
        .as_mut()
        .map_or(FALSE, |it| match it.peek() {
            Some(_) => TRUE,
            None => FALSE,
        })
}

/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_interval_Get(ctx: *mut ChewingContext, it: *mut IntervalType) {
    let ctx = as_mut_or_return!(ctx);

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

/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_aux_Check(ctx: *const ChewingContext) -> c_int {
    let ctx = as_ref_or_return!(ctx, ERROR);

    !ctx.editor.notification().is_empty() as c_int
}

/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_aux_Length(ctx: *const ChewingContext) -> c_int {
    let ctx = as_ref_or_return!(ctx, ERROR);

    ctx.editor.notification().chars().count() as c_int
}

/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_aux_String(ctx: *const ChewingContext) -> *mut c_char {
    let ctx = match unsafe { ctx.as_ref() } {
        Some(ctx) => ctx,
        None => return owned_into_raw(Owned::CString, CString::default().into_raw()),
    };

    let cstring =
        CString::new(ctx.editor.notification()).expect("notification should be valid UTF-8");
    owned_into_raw(Owned::CString, cstring.into_raw())
}

/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_aux_String_static(ctx: *const ChewingContext) -> *const c_char {
    let ctx = as_mut_or_return!(ctx.cast_mut(), global_empty_cstr());

    copy_cstr(&mut ctx.aux_buf, ctx.editor.notification())
}

/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_keystroke_CheckIgnore(ctx: *const ChewingContext) -> c_int {
    let ctx = as_ref_or_return!(ctx, ERROR);

    match ctx.editor.last_key_behavior() {
        EditorKeyBehavior::Ignore => TRUE,
        _ => FALSE,
    }
}

/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_keystroke_CheckAbsorb(ctx: *const ChewingContext) -> c_int {
    let ctx = as_ref_or_return!(ctx, ERROR);

    match ctx.editor.last_key_behavior() {
        EditorKeyBehavior::Absorb => TRUE,
        _ => FALSE,
    }
}

/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_kbtype_Total(_ctx: *const ChewingContext) -> c_int {
    (0..)
        .map_while(|id| KeyboardLayoutCompat::try_from(id).ok())
        .count() as c_int
}

/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_kbtype_Enumerate(ctx: *mut ChewingContext) {
    let ctx = as_mut_or_return!(ctx);

    ctx.kbcompat_iter = Some(
        (Box::new((0..).map_while(|id| KeyboardLayoutCompat::try_from(id).ok()))
            as Box<dyn Iterator<Item = KeyboardLayoutCompat>>)
            .peekable(),
    )
}

/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_kbtype_hasNext(ctx: *mut ChewingContext) -> c_int {
    let ctx = as_mut_or_return!(ctx, ERROR);

    ctx.kbcompat_iter
        .as_mut()
        .and_then(|it| it.peek())
        .map_or(0, |_| 1)
}

/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_kbtype_String(ctx: *mut ChewingContext) -> *mut c_char {
    let ctx = as_mut_or_return!(
        ctx,
        owned_into_raw(Owned::CString, CString::default().into_raw())
    );

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

/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chewing_kbtype_String_static(ctx: *mut ChewingContext) -> *const c_char {
    let ctx = as_mut_or_return!(ctx, global_empty_cstr());

    match ctx.kbcompat_iter.as_mut().and_then(|it| it.next()) {
        Some(kb_compat) => copy_cstr(&mut ctx.kbtype_buf, &kb_compat.to_string()),
        None => global_empty_cstr(),
    }
}

/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
#[deprecated]
pub unsafe extern "C" fn chewing_zuin_Check(ctx: *const ChewingContext) -> c_int {
    unsafe { chewing_bopomofo_Check(ctx) ^ 1 }
}

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

/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
#[deprecated]
pub unsafe extern "C" fn chewing_Init(data_path: *const c_char, hash_path: *const c_char) -> c_int {
    let _ = hash_path;
    let _ = data_path;
    OK
}

/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
#[deprecated]
pub unsafe extern "C" fn chewing_Terminate() {}

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

/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
#[deprecated]
pub unsafe extern "C" fn chewing_set_hsuSelKeyType(_ctx: *mut ChewingContext, mode: c_int) {
    let _ = mode;
}

/// # Safety
///
/// This function should be called with valid pointers.
#[unsafe(no_mangle)]
#[deprecated]
pub unsafe extern "C" fn chewing_get_hsuSelKeyType(_ctx: *mut ChewingContext) -> c_int {
    OK
}
