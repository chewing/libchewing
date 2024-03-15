use std::{
    cmp::min,
    collections::BTreeMap,
    ffi::{c_char, c_int, c_uint, c_ushort, c_void, CStr, CString},
    mem,
    ptr::{null, null_mut},
    slice, str,
    sync::OnceLock,
    u8,
};

use log::{debug, warn};

use crate::{
    capi::public::{
        ChewingConfigData, ChewingContext, IntervalType, SelKeys, CHINESE_MODE, FULLSHAPE_MODE,
        HALFSHAPE_MODE, MAX_SELKEY, SYMBOL_MODE,
    },
    conversion::{ChewingEngine, Interval, Symbol},
    dictionary::{LayeredDictionary, SystemDictionaryLoader, UserDictionaryLoader},
    editor::{
        keyboard::{AnyKeyboardLayout, KeyCode, KeyboardLayout, Modifiers, Qwerty},
        syllable::{
            DaiChien26, Et, Et26, GinYieh, Hsu, Ibm, KeyboardLayoutCompat, Pinyin, Standard,
            SyllableEditor,
        },
        BasicEditor, CharacterForm, Editor, EditorKeyBehavior, EditorOptions, LanguageMode,
        LaxUserFreqEstimate, UserPhraseAddDirection,
    },
    zhuyin::Syllable,
};

const TRUE: c_int = 1;
const FALSE: c_int = 0;
const OK: c_int = 0;
const ERROR: c_int = -1;

enum Owned {
    CString,
    CUShortSlice(usize),
}

static mut OWNED: OnceLock<BTreeMap<*mut c_void, Owned>> = OnceLock::new();

fn owned_into_raw<T>(owned: Owned, ptr: *mut T) -> *mut T {
    unsafe {
        if OWNED.get().is_none() {
            let _ = OWNED.set(BTreeMap::new());
        }
    }
    let void_ptr: *mut c_void = ptr.cast();
    match unsafe { OWNED.get_mut() } {
        Some(map) => {
            map.insert(void_ptr, owned);
            ptr
        }
        None => null_mut(),
    }
}

fn drop_owned(ptr: *mut c_void) {
    match unsafe { OWNED.get() } {
        Some(map) => {
            if let Some(owned) = map.get(&ptr) {
                match owned {
                    Owned::CString => drop(unsafe { CString::from_raw(ptr.cast()) }),
                    Owned::CUShortSlice(len) => {
                        drop(unsafe { Vec::from_raw_parts(ptr, *len, *len) })
                    }
                }
            }
        }
        None => (),
    }
}

static mut GLOBAL_STRING_BUFFER: [u8; 256] = [0; 256];
static mut EMPTY_STRING_BUFFER: [u8; 1] = [0; 1];

unsafe fn global_cstr(buffer: &str) -> *const c_char {
    unsafe {
        let n = min(GLOBAL_STRING_BUFFER.len(), buffer.len());
        GLOBAL_STRING_BUFFER.fill(0);
        GLOBAL_STRING_BUFFER[..n].copy_from_slice(&buffer.as_bytes()[..n]);
        GLOBAL_STRING_BUFFER.as_ptr().cast()
    }
}

unsafe fn global_empty_cstr() -> *mut c_char {
    unsafe { EMPTY_STRING_BUFFER.as_mut_ptr().cast() }
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
        .and_then(|data| str::from_utf8(unsafe { mem::transmute(data) }).ok())
}

#[no_mangle]
pub extern "C" fn chewing_new() -> *mut ChewingContext {
    chewing_new2(null(), null(), None, null_mut())
}

#[no_mangle]
pub extern "C" fn chewing_new2(
    syspath: *const c_char,
    userpath: *const c_char,
    _logger: Option<
        unsafe extern "C" fn(data: *mut c_void, level: c_int, fmt: *const c_char, arg: ...),
    >,
    _loggerdata: *mut c_void,
) -> *mut ChewingContext {
    unsafe {
        if OWNED.get().is_none() {
            let _ = OWNED.set(BTreeMap::new());
        }
    }
    let dictionaries = if syspath.is_null() {
        SystemDictionaryLoader::new().load()
    } else {
        let search_path = unsafe { CStr::from_ptr(syspath) }
            .to_str()
            .expect("invalid syspath string");
        SystemDictionaryLoader::new().sys_path(search_path).load()
    };
    let dictionaries = match dictionaries {
        Ok(d) => d,
        Err(_) => return null_mut(),
    };
    let user_dictionary = if userpath.is_null() {
        UserDictionaryLoader::new().load()
    } else {
        let data_path = unsafe { CStr::from_ptr(userpath) }
            .to_str()
            .expect("invalid syspath string");
        UserDictionaryLoader::new()
            .userphrase_path(data_path)
            .load()
    };
    let user_dictionary = match user_dictionary {
        Ok(d) => d,
        Err(_) => return null_mut(),
    };

    let estimate = LaxUserFreqEstimate::open(user_dictionary.as_ref());
    let estimate = match estimate {
        Ok(d) => d,
        Err(_) => return null_mut(),
    };

    let dict = LayeredDictionary::new(dictionaries, user_dictionary);
    let conversion_engine = ChewingEngine::new();
    let kb_compat = KeyboardLayoutCompat::Default;
    let keyboard = AnyKeyboardLayout::Qwerty(Qwerty);
    let editor = Editor::new(conversion_engine, dict, estimate);
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
    });
    Box::into_raw(context)
}

#[no_mangle]
pub extern "C" fn chewing_delete(ctx: *mut ChewingContext) {
    if !ctx.is_null() {
        unsafe {
            if OWNED.get().is_none() {
                let _ = OWNED.take();
            }
        }
        drop(unsafe { Box::from_raw(ctx) })
    }
}

#[no_mangle]
pub extern "C" fn chewing_free(ptr: *mut c_void) {
    if !ptr.is_null() {
        drop_owned(ptr);
    }
}

#[no_mangle]
pub extern "C" fn chewing_Reset(ctx: *mut ChewingContext) -> c_int {
    let ctx = match unsafe { ctx.as_mut() } {
        Some(ctx) => ctx,
        None => return -1,
    };
    ctx.editor.clear();
    0
}

#[no_mangle]
pub extern "C" fn chewing_set_KBType(ctx: *mut ChewingContext, kbtype: c_int) -> c_int {
    let ctx = match unsafe { ctx.as_mut() } {
        Some(ctx) => ctx,
        None => return -1,
    };
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
        KB::Dvorak => (AnyKeyboardLayout::qwerty(), Box::new(Standard::new())),
        KB::DvorakHsu => (AnyKeyboardLayout::qwerty(), Box::new(Hsu::new())),
        KB::DachenCp26 => (AnyKeyboardLayout::qwerty(), Box::new(DaiChien26::new())),
        KB::HanyuPinyin => (AnyKeyboardLayout::qwerty(), Box::new(Pinyin::hanyu())),
        KB::ThlPinyin => (AnyKeyboardLayout::qwerty(), Box::new(Pinyin::thl())),
        KB::Mps2Pinyin => (AnyKeyboardLayout::qwerty(), Box::new(Pinyin::mps2())),
        KB::Carpalx => (AnyKeyboardLayout::qwerty(), Box::new(Standard::new())),
        KB::ColemakDhAnsi => (
            AnyKeyboardLayout::colemak_dh_ansi(),
            Box::new(Standard::new()),
        ),
        KB::ColemakDhOrth => (
            AnyKeyboardLayout::colemak_dh_orth(),
            Box::new(Standard::new()),
        ),
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

#[no_mangle]
pub extern "C" fn chewing_get_KBType(ctx: *const ChewingContext) -> c_int {
    let ctx = match unsafe { ctx.as_ref() } {
        Some(ctx) => ctx,
        None => return -1,
    };
    ctx.kb_compat as c_int
}

#[no_mangle]
pub extern "C" fn chewing_get_KBString(ctx: *const ChewingContext) -> *mut c_char {
    let ctx = match unsafe { ctx.as_ref() } {
        Some(ctx) => ctx,
        None => return owned_into_raw(Owned::CString, CString::default().into_raw()),
    };
    let kb_string = ctx.kb_compat.to_string();
    owned_into_raw(
        Owned::CString,
        CString::new(kb_string)
            .expect("should have valid kb_string")
            .into_raw(),
    )
}

#[no_mangle]
pub extern "C" fn chewing_KBStr2Num(str: *const c_char) -> c_int {
    let cstr = unsafe { CStr::from_ptr(str) };
    let utf8str = cstr.to_string_lossy();
    let layout: KeyboardLayoutCompat = utf8str.parse().unwrap_or(KeyboardLayoutCompat::Default);
    layout as c_int
}

#[no_mangle]
pub extern "C" fn chewing_set_ChiEngMode(ctx: *mut ChewingContext, mode: c_int) {
    let ctx = match unsafe { ctx.as_mut() } {
        Some(ctx) => ctx,
        None => return,
    };

    match mode {
        CHINESE_MODE => ctx.editor.set_language_mode(LanguageMode::Chinese),
        SYMBOL_MODE => ctx.editor.set_language_mode(LanguageMode::English),
        _ => warn!("invalid language mode {}", mode),
    }
}

#[no_mangle]
pub extern "C" fn chewing_get_ChiEngMode(ctx: *const ChewingContext) -> c_int {
    let ctx = match unsafe { ctx.as_ref() } {
        Some(ctx) => ctx,
        None => return -1,
    };

    match ctx.editor.language_mode() {
        LanguageMode::Chinese => CHINESE_MODE,
        LanguageMode::English => SYMBOL_MODE,
    }
}

#[no_mangle]
pub extern "C" fn chewing_set_ShapeMode(ctx: *mut ChewingContext, mode: c_int) {
    let ctx = match unsafe { ctx.as_mut() } {
        Some(ctx) => ctx,
        None => return,
    };

    match mode {
        HALFSHAPE_MODE => ctx.editor.set_character_form(CharacterForm::Halfwidth),
        FULLSHAPE_MODE => ctx.editor.set_character_form(CharacterForm::Fullwidth),
        _ => warn!("invalid shape mode {}", mode),
    }
}

#[no_mangle]
pub extern "C" fn chewing_get_ShapeMode(ctx: *const ChewingContext) -> c_int {
    let ctx = match unsafe { ctx.as_ref() } {
        Some(ctx) => ctx,
        None => return -1,
    };

    match ctx.editor.character_form() {
        CharacterForm::Halfwidth => HALFSHAPE_MODE,
        CharacterForm::Fullwidth => FULLSHAPE_MODE,
    }
}

#[no_mangle]
pub extern "C" fn chewing_set_candPerPage(ctx: *mut ChewingContext, n: c_int) {
    let ctx = match unsafe { ctx.as_mut() } {
        Some(ctx) => ctx,
        None => return,
    };

    if n == 0 || n > 10 {
        return;
    }

    ctx.editor.set_editor_options(EditorOptions {
        candidates_per_page: n as usize,
        ..ctx.editor.editor_options()
    });
}

#[no_mangle]
pub extern "C" fn chewing_get_candPerPage(ctx: *const ChewingContext) -> c_int {
    let ctx = match unsafe { ctx.as_ref() } {
        Some(ctx) => ctx,
        None => return -1,
    };

    ctx.editor.editor_options().candidates_per_page as c_int
}

#[no_mangle]
pub extern "C" fn chewing_set_maxChiSymbolLen(ctx: *mut ChewingContext, n: c_int) {
    let ctx = match unsafe { ctx.as_mut() } {
        Some(ctx) => ctx,
        None => return,
    };

    if n < 0 || n > 39 {
        return;
    }

    ctx.editor.set_editor_options(EditorOptions {
        auto_commit_threshold: n as usize,
        ..ctx.editor.editor_options()
    });
}

#[no_mangle]
pub extern "C" fn chewing_get_maxChiSymbolLen(ctx: *const ChewingContext) -> c_int {
    let ctx = match unsafe { ctx.as_ref() } {
        Some(ctx) => ctx,
        None => return -1,
    };

    ctx.editor.editor_options().auto_commit_threshold as c_int
}

#[no_mangle]
pub extern "C" fn chewing_set_selKey(ctx: *mut ChewingContext, sel_keys: *const c_int, len: c_int) {
    let ctx = match unsafe { ctx.as_mut() } {
        Some(ctx) => ctx,
        None => return,
    };

    if sel_keys.is_null() || len != 10 {
        return;
    }

    let sel_keys = unsafe { slice::from_raw_parts(sel_keys, len as usize) };
    ctx.sel_keys.0.copy_from_slice(sel_keys);
}

#[no_mangle]
pub extern "C" fn chewing_get_selKey(ctx: *const ChewingContext) -> *mut c_int {
    let ctx = match unsafe { ctx.as_ref() } {
        Some(ctx) => ctx,
        None => return null_mut(),
    };

    ctx.sel_keys.0.as_ptr().cast_mut()
}

#[no_mangle]
pub extern "C" fn chewing_set_addPhraseDirection(ctx: *mut ChewingContext, direction: c_int) {
    let ctx = match unsafe { ctx.as_mut() } {
        Some(ctx) => ctx,
        None => return,
    };

    if direction != 0 && direction != 1 {
        return;
    }

    ctx.editor.set_editor_options(EditorOptions {
        user_phrase_add_dir: match direction {
            0 => UserPhraseAddDirection::Forward,
            _ => UserPhraseAddDirection::Backward,
        },
        ..ctx.editor.editor_options()
    });
}

#[no_mangle]
pub extern "C" fn chewing_get_addPhraseDirection(ctx: *const ChewingContext) -> c_int {
    let ctx = match unsafe { ctx.as_ref() } {
        Some(ctx) => ctx,
        None => return -1,
    };

    match ctx.editor.editor_options().user_phrase_add_dir {
        UserPhraseAddDirection::Forward => 0,
        UserPhraseAddDirection::Backward => 1,
    }
}

#[no_mangle]
pub extern "C" fn chewing_set_spaceAsSelection(ctx: *mut ChewingContext, mode: c_int) {
    let ctx = match unsafe { ctx.as_mut() } {
        Some(ctx) => ctx,
        None => return,
    };

    if mode != 0 && mode != 1 {
        return;
    }

    ctx.editor.set_editor_options(EditorOptions {
        space_is_select_key: match mode {
            0 => false,
            _ => true,
        },
        ..ctx.editor.editor_options()
    });
}

#[no_mangle]
pub extern "C" fn chewing_get_spaceAsSelection(ctx: *const ChewingContext) -> c_int {
    let ctx = match unsafe { ctx.as_ref() } {
        Some(ctx) => ctx,
        None => return -1,
    };

    match ctx.editor.editor_options().space_is_select_key {
        true => 1,
        false => 0,
    }
}

#[no_mangle]
pub extern "C" fn chewing_set_escCleanAllBuf(ctx: *mut ChewingContext, mode: c_int) {
    let ctx = match unsafe { ctx.as_mut() } {
        Some(ctx) => ctx,
        None => return,
    };

    if mode != 0 && mode != 1 {
        return;
    }

    ctx.editor.set_editor_options(EditorOptions {
        esc_clear_all_buffer: match mode {
            0 => false,
            _ => true,
        },
        ..ctx.editor.editor_options()
    });
}

#[no_mangle]
pub extern "C" fn chewing_get_escCleanAllBuf(ctx: *const ChewingContext) -> c_int {
    let ctx = match unsafe { ctx.as_ref() } {
        Some(ctx) => ctx,
        None => return -1,
    };

    match ctx.editor.editor_options().esc_clear_all_buffer {
        true => 1,
        false => 0,
    }
}

#[no_mangle]
pub extern "C" fn chewing_set_autoShiftCur(ctx: *mut ChewingContext, mode: c_int) {
    let ctx = match unsafe { ctx.as_mut() } {
        Some(ctx) => ctx,
        None => return,
    };

    if mode != 0 && mode != 1 {
        return;
    }

    ctx.editor.set_editor_options(EditorOptions {
        auto_shift_cursor: match mode {
            0 => false,
            _ => true,
        },
        ..ctx.editor.editor_options()
    });
}

#[no_mangle]
pub extern "C" fn chewing_get_autoShiftCur(ctx: *const ChewingContext) -> c_int {
    let ctx = match unsafe { ctx.as_ref() } {
        Some(ctx) => ctx,
        None => return -1,
    };

    ctx.editor.editor_options().auto_shift_cursor as c_int
}

#[no_mangle]
pub extern "C" fn chewing_set_easySymbolInput(ctx: *mut ChewingContext, mode: c_int) {
    let ctx = match unsafe { ctx.as_mut() } {
        Some(ctx) => ctx,
        None => return,
    };

    if mode != 0 && mode != 1 {
        return;
    }

    ctx.editor.set_editor_options(EditorOptions {
        easy_symbol_input: match mode {
            0 => false,
            _ => true,
        },
        ..ctx.editor.editor_options()
    });
}

#[no_mangle]
pub extern "C" fn chewing_get_easySymbolInput(ctx: *const ChewingContext) -> c_int {
    let ctx = match unsafe { ctx.as_ref() } {
        Some(ctx) => ctx,
        None => return -1,
    };

    ctx.editor.editor_options().easy_symbol_input as c_int
}

#[no_mangle]
pub extern "C" fn chewing_set_phraseChoiceRearward(ctx: *mut ChewingContext, mode: c_int) {
    let ctx = match unsafe { ctx.as_mut() } {
        Some(ctx) => ctx,
        None => return,
    };

    if mode != 0 && mode != 1 {
        return;
    }

    ctx.editor.set_editor_options(EditorOptions {
        phrase_choice_rearward: match mode {
            0 => false,
            _ => true,
        },
        ..Default::default()
    });
}

#[no_mangle]
pub extern "C" fn chewing_get_phraseChoiceRearward(ctx: *const ChewingContext) -> c_int {
    let ctx = match unsafe { ctx.as_ref() } {
        Some(ctx) => ctx,
        None => return -1,
    };

    ctx.editor.editor_options().phrase_choice_rearward as c_int
}

#[no_mangle]
pub extern "C" fn chewing_set_autoLearn(ctx: *mut ChewingContext, mode: c_int) {
    let ctx = match unsafe { ctx.as_mut() } {
        Some(ctx) => ctx,
        None => return,
    };

    if mode != 0 && mode != 1 {
        return;
    }

    ctx.editor.set_editor_options(EditorOptions {
        disable_auto_learn_phrase: match mode {
            0 => false,
            _ => true,
        },
        ..Default::default()
    });
}

#[no_mangle]
pub extern "C" fn chewing_get_autoLearn(ctx: *const ChewingContext) -> c_int {
    let ctx = match unsafe { ctx.as_ref() } {
        Some(ctx) => ctx,
        None => return -1,
    };

    ctx.editor.editor_options().disable_auto_learn_phrase as c_int
}

#[no_mangle]
pub extern "C" fn chewing_get_phoneSeq(ctx: *const ChewingContext) -> *mut c_ushort {
    let ctx = match unsafe { ctx.as_ref() } {
        Some(ctx) => ctx,
        None => return null_mut(),
    };

    let syllables: Vec<_> = ctx
        .editor
        .symbols()
        .into_iter()
        .cloned()
        .filter(Symbol::is_syllable)
        .map(|sym| sym.as_syllable().to_u16())
        .collect();
    let len = syllables.len();
    let ptr = Box::into_raw(syllables.into_boxed_slice());
    owned_into_raw(Owned::CUShortSlice(len), ptr.cast())
}

#[no_mangle]
pub extern "C" fn chewing_get_phoneSeqLen(ctx: *const ChewingContext) -> c_int {
    let ctx = match unsafe { ctx.as_ref() } {
        Some(ctx) => ctx,
        None => return -1,
    };

    ctx.editor
        .symbols()
        .into_iter()
        .cloned()
        .filter(Symbol::is_syllable)
        .count() as c_int
}

#[no_mangle]
pub extern "C" fn chewing_set_logger(
    _ctx: *mut ChewingContext,
    logger: extern "C" fn(data: *mut c_void, level: c_int, fmt: *const c_char, arg: ...),
    data: *mut c_void,
) {
}

#[no_mangle]
pub extern "C" fn chewing_userphrase_enumerate(ctx: *mut ChewingContext) -> c_int {
    let ctx = match unsafe { ctx.as_mut() } {
        Some(ctx) => ctx,
        None => return -1,
    };

    ctx.userphrase_iter = Some(ctx.editor.user_dict().entries().peekable());
    0
}

#[no_mangle]
pub extern "C" fn chewing_userphrase_has_next(
    ctx: *mut ChewingContext,
    phrase_len: *mut c_uint,
    bopomofo_len: *mut c_uint,
) -> c_int {
    let ctx = match unsafe { ctx.as_mut() } {
        Some(ctx) => ctx,
        None => return 0,
    };

    if ctx.userphrase_iter.is_none() {
        return 0;
    }

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

#[no_mangle]
pub extern "C" fn chewing_userphrase_get(
    ctx: *mut ChewingContext,
    phrase_buf: *mut c_char,
    phrase_len: c_uint,
    bopomofo_buf: *mut c_char,
    bopomofo_len: c_uint,
) -> c_int {
    let ctx = match unsafe { ctx.as_mut() } {
        Some(ctx) => ctx,
        None => return -1,
    };

    if ctx.userphrase_iter.is_none() {
        return -1;
    }

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

#[no_mangle]
pub extern "C" fn chewing_userphrase_add(
    ctx: *mut ChewingContext,
    phrase_buf: *const c_char,
    bopomofo_buf: *const c_char,
) -> c_int {
    let ctx = match unsafe { ctx.as_mut() } {
        Some(ctx) => ctx,
        None => return -1,
    };
    let syllables = match unsafe { str_from_ptr_with_nul(bopomofo_buf) } {
        Some(bopomofo) => bopomofo
            .split_ascii_whitespace()
            .into_iter()
            .map_while(|it| it.parse::<Syllable>().ok())
            .collect::<Vec<_>>(),
        None => return 0,
    };

    if syllables.len() > 11 {
        return 0;
    }

    match unsafe { str_from_ptr_with_nul(phrase_buf) } {
        Some(phrase) => match ctx.editor.learn_phrase(&syllables, &phrase) {
            Ok(_) => TRUE,
            Err(_) => FALSE,
        },
        None => ERROR,
    }
}

#[no_mangle]
pub extern "C" fn chewing_userphrase_remove(
    ctx: *mut ChewingContext,
    phrase_buf: *const c_char,
    bopomofo_buf: *const c_char,
) -> c_int {
    let ctx = match unsafe { ctx.as_mut() } {
        Some(ctx) => ctx,
        None => return ERROR,
    };

    // return FALSE when phrase does not exist is C API only behavior
    if chewing_userphrase_lookup(ctx, phrase_buf, bopomofo_buf) != TRUE {
        return FALSE;
    }

    let syllables = match unsafe { str_from_ptr_with_nul(bopomofo_buf) } {
        Some(bopomofo) => bopomofo
            .split_ascii_whitespace()
            .into_iter()
            .map_while(|it| it.parse::<Syllable>().ok())
            .collect::<Vec<_>>(),
        None => return ERROR,
    };

    match unsafe { str_from_ptr_with_nul(phrase_buf) } {
        Some(phrase) => match ctx.editor.unlearn_phrase(&syllables, &phrase) {
            Err(_) => FALSE,
            Ok(_) => TRUE,
        },
        None => -1,
    }
}

#[no_mangle]
pub extern "C" fn chewing_userphrase_lookup(
    ctx: *mut ChewingContext,
    phrase_buf: *const c_char,
    bopomofo_buf: *const c_char,
) -> c_int {
    let ctx = match unsafe { ctx.as_mut() } {
        Some(ctx) => ctx,
        None => return 0,
    };
    let syllables = match unsafe { str_from_ptr_with_nul(bopomofo_buf) } {
        Some(bopomofo) => bopomofo
            .split_ascii_whitespace()
            .into_iter()
            .map_while(|it| it.parse::<Syllable>().ok())
            .collect::<Vec<_>>(),
        None => return 0,
    };

    match unsafe { str_from_ptr_with_nul(phrase_buf) } {
        Some(phrase) => ctx
            .editor
            .user_dict()
            .lookup_all_phrases(&syllables)
            .iter()
            .any(|ph| ph.as_str() == phrase) as c_int,
        None => ctx
            .editor
            .user_dict()
            .lookup_first_phrase(&syllables)
            .is_some() as c_int,
    }
}

#[no_mangle]
pub extern "C" fn chewing_cand_list_first(ctx: *mut ChewingContext) -> c_int {
    let ctx = match unsafe { ctx.as_mut() } {
        Some(ctx) => ctx,
        None => return -1,
    };

    if !ctx.editor.is_selecting() {
        return -1;
    }

    ctx.editor.jump_to_first_selection_point();
    0
}

#[no_mangle]
pub extern "C" fn chewing_cand_list_last(ctx: *mut ChewingContext) -> c_int {
    let ctx = match unsafe { ctx.as_mut() } {
        Some(ctx) => ctx,
        None => return -1,
    };

    if !ctx.editor.is_selecting() {
        return -1;
    }

    ctx.editor.jump_to_last_selection_point();
    0
}

#[no_mangle]
pub extern "C" fn chewing_cand_list_has_next(ctx: *mut ChewingContext) -> c_int {
    let ctx = match unsafe { ctx.as_ref() } {
        Some(ctx) => ctx,
        None => return 0,
    };

    if !ctx.editor.is_selecting() {
        return 0;
    }

    match ctx.editor.has_next_selection_point() {
        true => 1,
        false => 0,
    }
}

#[no_mangle]
pub extern "C" fn chewing_cand_list_has_prev(ctx: *mut ChewingContext) -> c_int {
    let ctx = match unsafe { ctx.as_ref() } {
        Some(ctx) => ctx,
        None => return 0,
    };

    if !ctx.editor.is_selecting() {
        return 0;
    }

    match ctx.editor.has_prev_selection_point() {
        true => 1,
        false => 0,
    }
}

#[no_mangle]
pub extern "C" fn chewing_cand_list_next(ctx: *mut ChewingContext) -> c_int {
    let ctx = match unsafe { ctx.as_mut() } {
        Some(ctx) => ctx,
        None => return -1,
    };
    if !ctx.editor.is_selecting() {
        return -1;
    }
    match ctx.editor.jump_to_next_selection_point() {
        Ok(_) => 0,
        Err(_) => -1,
    }
}

#[no_mangle]
pub extern "C" fn chewing_cand_list_prev(ctx: *mut ChewingContext) -> c_int {
    let ctx = match unsafe { ctx.as_mut() } {
        Some(ctx) => ctx,
        None => return -1,
    };
    if !ctx.editor.is_selecting() {
        return -1;
    }
    match ctx.editor.jump_to_prev_selection_point() {
        Ok(_) => 0,
        Err(_) => -1,
    }
}

#[no_mangle]
pub extern "C" fn chewing_commit_preedit_buf(ctx: *mut ChewingContext) -> c_int {
    let ctx = match unsafe { ctx.as_mut() } {
        Some(ctx) => ctx,
        None => return ERROR,
    };

    match ctx.editor.commit() {
        Ok(_) => OK,
        Err(_) => ERROR,
    }
}

#[no_mangle]
pub extern "C" fn chewing_clean_preedit_buf(ctx: *mut ChewingContext) -> c_int {
    let ctx = match unsafe { ctx.as_mut() } {
        Some(ctx) => ctx,
        None => return -1,
    };

    if !ctx.editor.is_entering() {
        -1
    } else {
        ctx.editor.clear();
        0
    }
}

#[no_mangle]
pub extern "C" fn chewing_clean_bopomofo_buf(ctx: *mut ChewingContext) -> c_int {
    let ctx = match unsafe { ctx.as_mut() } {
        Some(ctx) => ctx,
        None => return -1,
    };

    ctx.editor.clear_syllable_editor();
    0
}

#[no_mangle]
pub extern "C" fn chewing_phone_to_bopomofo(
    phone: c_ushort,
    buf: *mut c_char,
    len: c_ushort,
) -> c_int {
    let syl_str = match Syllable::try_from(phone) {
        Ok(s) => s.to_string(),
        Err(_) => return -1,
    };
    if !buf.is_null() && len as usize >= (syl_str.len() + 1) {
        let buf = unsafe { slice::from_raw_parts_mut(buf.cast(), len as usize) };
        buf[0..syl_str.len()].copy_from_slice(syl_str.as_bytes());
        buf[syl_str.len()] = 0;
    }
    (syl_str.len() + 1) as c_int
}

#[no_mangle]
pub extern "C" fn chewing_handle_Space(ctx: *mut ChewingContext) -> c_int {
    let ctx = match unsafe { ctx.as_mut() } {
        Some(ctx) => ctx,
        None => return -1,
    };

    ctx.editor
        .process_keyevent(ctx.keyboard.map(KeyCode::Space));
    0
}

#[no_mangle]
pub extern "C" fn chewing_handle_Esc(ctx: *mut ChewingContext) -> c_int {
    let ctx = match unsafe { ctx.as_mut() } {
        Some(ctx) => ctx,
        None => return -1,
    };

    ctx.editor.process_keyevent(ctx.keyboard.map(KeyCode::Esc));
    0
}

#[no_mangle]
pub extern "C" fn chewing_handle_Enter(ctx: *mut ChewingContext) -> c_int {
    let ctx = match unsafe { ctx.as_mut() } {
        Some(ctx) => ctx,
        None => return -1,
    };

    ctx.editor
        .process_keyevent(ctx.keyboard.map(KeyCode::Enter));
    0
}

#[no_mangle]
pub extern "C" fn chewing_handle_Del(ctx: *mut ChewingContext) -> c_int {
    let ctx = match unsafe { ctx.as_mut() } {
        Some(ctx) => ctx,
        None => return -1,
    };

    ctx.editor.process_keyevent(ctx.keyboard.map(KeyCode::Del));
    0
}

#[no_mangle]
pub extern "C" fn chewing_handle_Backspace(ctx: *mut ChewingContext) -> c_int {
    let ctx = match unsafe { ctx.as_mut() } {
        Some(ctx) => ctx,
        None => return -1,
    };

    ctx.editor
        .process_keyevent(ctx.keyboard.map(KeyCode::Backspace));
    0
}

#[no_mangle]
pub extern "C" fn chewing_handle_Tab(ctx: *mut ChewingContext) -> c_int {
    let ctx = match unsafe { ctx.as_mut() } {
        Some(ctx) => ctx,
        None => return -1,
    };

    ctx.editor.process_keyevent(ctx.keyboard.map(KeyCode::Tab));
    0
}

#[no_mangle]
pub extern "C" fn chewing_handle_ShiftLeft(ctx: *mut ChewingContext) -> c_int {
    let ctx = match unsafe { ctx.as_mut() } {
        Some(ctx) => ctx,
        None => return -1,
    };

    ctx.editor
        .process_keyevent(ctx.keyboard.map_with_mod(KeyCode::Left, Modifiers::shift()));
    0
}

#[no_mangle]
pub extern "C" fn chewing_handle_Left(ctx: *mut ChewingContext) -> c_int {
    let ctx = match unsafe { ctx.as_mut() } {
        Some(ctx) => ctx,
        None => return -1,
    };

    let key_event = ctx.keyboard.map(KeyCode::Left);
    ctx.editor.process_keyevent(key_event);
    0
}

#[no_mangle]
pub extern "C" fn chewing_handle_ShiftRight(ctx: *mut ChewingContext) -> c_int {
    let ctx = match unsafe { ctx.as_mut() } {
        Some(ctx) => ctx,
        None => return -1,
    };

    ctx.editor.process_keyevent(
        ctx.keyboard
            .map_with_mod(KeyCode::Right, Modifiers::shift()),
    );
    0
}

#[no_mangle]
pub extern "C" fn chewing_handle_Right(ctx: *mut ChewingContext) -> c_int {
    let ctx = match unsafe { ctx.as_mut() } {
        Some(ctx) => ctx,
        None => return -1,
    };

    ctx.editor
        .process_keyevent(ctx.keyboard.map(KeyCode::Right));
    0
}

#[no_mangle]
pub extern "C" fn chewing_handle_Up(ctx: *mut ChewingContext) -> c_int {
    let ctx = match unsafe { ctx.as_mut() } {
        Some(ctx) => ctx,
        None => return -1,
    };

    ctx.editor.process_keyevent(ctx.keyboard.map(KeyCode::Up));
    0
}

#[no_mangle]
pub extern "C" fn chewing_handle_Home(ctx: *mut ChewingContext) -> c_int {
    let ctx = match unsafe { ctx.as_mut() } {
        Some(ctx) => ctx,
        None => return -1,
    };

    ctx.editor.process_keyevent(ctx.keyboard.map(KeyCode::Home));
    0
}

#[no_mangle]
pub extern "C" fn chewing_handle_End(ctx: *mut ChewingContext) -> c_int {
    let ctx = match unsafe { ctx.as_mut() } {
        Some(ctx) => ctx,
        None => return -1,
    };

    ctx.editor.process_keyevent(ctx.keyboard.map(KeyCode::End));
    0
}

#[no_mangle]
pub extern "C" fn chewing_handle_PageUp(ctx: *mut ChewingContext) -> c_int {
    let ctx = match unsafe { ctx.as_mut() } {
        Some(ctx) => ctx,
        None => return -1,
    };

    ctx.editor
        .process_keyevent(ctx.keyboard.map(KeyCode::PageUp));
    0
}

#[no_mangle]
pub extern "C" fn chewing_handle_PageDown(ctx: *mut ChewingContext) -> c_int {
    let ctx = match unsafe { ctx.as_mut() } {
        Some(ctx) => ctx,
        None => return -1,
    };

    ctx.editor
        .process_keyevent(ctx.keyboard.map(KeyCode::PageDown));
    0
}

#[no_mangle]
pub extern "C" fn chewing_handle_Down(ctx: *mut ChewingContext) -> c_int {
    let ctx = match unsafe { ctx.as_mut() } {
        Some(ctx) => ctx,
        None => return -1,
    };

    ctx.editor.process_keyevent(ctx.keyboard.map(KeyCode::Down));
    0
}

#[no_mangle]
pub extern "C" fn chewing_handle_Capslock(ctx: *mut ChewingContext) -> c_int {
    let ctx = match unsafe { ctx.as_mut() } {
        Some(ctx) => ctx,
        None => return -1,
    };

    ctx.editor.process_keyevent(
        ctx.keyboard
            .map_with_mod(KeyCode::Unknown, Modifiers::capslock()),
    );
    0
}

#[no_mangle]
pub extern "C" fn chewing_handle_Default(ctx: *mut ChewingContext, key: c_int) -> c_int {
    let ctx = match unsafe { ctx.as_mut() } {
        Some(ctx) => ctx,
        None => return -1,
    };

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
    0
}

#[no_mangle]
pub extern "C" fn chewing_handle_CtrlNum(ctx: *mut ChewingContext, key: c_int) -> c_int {
    let ctx = match unsafe { ctx.as_mut() } {
        Some(ctx) => ctx,
        None => return -1,
    };

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
    0
}

#[no_mangle]
pub extern "C" fn chewing_handle_ShiftSpace(ctx: *mut ChewingContext) -> c_int {
    let ctx = match unsafe { ctx.as_mut() } {
        Some(ctx) => ctx,
        None => return -1,
    };

    ctx.editor.process_keyevent(
        ctx.keyboard
            .map_with_mod(KeyCode::Space, Modifiers::shift()),
    );
    0
}

#[no_mangle]
pub extern "C" fn chewing_handle_DblTab(ctx: *mut ChewingContext) -> c_int {
    let _ctx = match unsafe { ctx.as_mut() } {
        Some(ctx) => ctx,
        None => return -1,
    };

    // todo!()
    0
}

#[no_mangle]
pub extern "C" fn chewing_handle_Numlock(ctx: *mut ChewingContext, key: c_int) -> c_int {
    let ctx = match unsafe { ctx.as_mut() } {
        Some(ctx) => ctx,
        None => return -1,
    };

    ctx.editor
        .process_keyevent(ctx.keyboard.map_ascii_numlock(key as u8));
    0
}

#[no_mangle]
pub extern "C" fn chewing_commit_Check(ctx: *const ChewingContext) -> c_int {
    let ctx = match unsafe { ctx.as_ref() } {
        Some(ctx) => ctx,
        None => return -1,
    };

    if ctx.editor.display_commit().is_empty() {
        0
    } else {
        1
    }
}

#[no_mangle]
pub extern "C" fn chewing_commit_String(ctx: *const ChewingContext) -> *mut c_char {
    let ctx = match unsafe { ctx.as_ref() } {
        Some(ctx) => ctx,
        None => return owned_into_raw(Owned::CString, CString::default().into_raw()),
    };

    let buffer = ctx.editor.display_commit();
    let cstr = match CString::new(buffer) {
        Ok(cstr) => cstr,
        Err(_) => return null_mut(),
    };
    owned_into_raw(Owned::CString, cstr.into_raw())
}

#[no_mangle]
pub extern "C" fn chewing_commit_String_static(ctx: *const ChewingContext) -> *const c_char {
    let ctx = match unsafe { ctx.as_ref() } {
        Some(ctx) => ctx,
        None => return unsafe { global_empty_cstr() },
    };

    let buffer = ctx.editor.display_commit();
    unsafe { global_cstr(&buffer) }
}

#[no_mangle]
pub extern "C" fn chewing_buffer_String(ctx: *const ChewingContext) -> *mut c_char {
    let ctx = match unsafe { ctx.as_ref() } {
        Some(ctx) => ctx,
        None => return owned_into_raw(Owned::CString, CString::default().into_raw()),
    };

    let buffer = ctx.editor.display();
    let cstr = match CString::new(buffer) {
        Ok(cstr) => cstr,
        Err(_) => return null_mut(),
    };
    owned_into_raw(Owned::CString, cstr.into_raw())
}

#[no_mangle]
pub extern "C" fn chewing_buffer_String_static(ctx: *const ChewingContext) -> *const c_char {
    let ctx = match unsafe { ctx.as_ref() } {
        Some(ctx) => ctx,
        None => return unsafe { global_empty_cstr() },
    };

    let buffer = ctx.editor.display();
    unsafe { global_cstr(&buffer) }
}

#[no_mangle]
pub extern "C" fn chewing_buffer_Check(ctx: *const ChewingContext) -> c_int {
    let ctx = match unsafe { ctx.as_ref() } {
        Some(ctx) => ctx,
        None => return -1,
    };

    if ctx.editor.display().len() > 0 {
        1
    } else {
        0
    }
}

#[no_mangle]
pub extern "C" fn chewing_buffer_Len(ctx: *const ChewingContext) -> c_int {
    let ctx = match unsafe { ctx.as_ref() } {
        Some(ctx) => ctx,
        None => return -1,
    };

    ctx.editor.display().chars().count() as c_int
}

#[no_mangle]
pub extern "C" fn chewing_bopomofo_String_static(ctx: *const ChewingContext) -> *const c_char {
    let ctx = match unsafe { ctx.as_ref() } {
        Some(ctx) => ctx,
        None => return unsafe { global_empty_cstr() },
    };

    let syllable = ctx.editor.syllable_buffer().to_string();
    unsafe { global_cstr(&syllable) }
}

#[no_mangle]
pub extern "C" fn chewing_bopomofo_Check(ctx: *const ChewingContext) -> c_int {
    let ctx = match unsafe { ctx.as_ref() } {
        Some(ctx) => ctx,
        None => return -1,
    };

    if ctx.editor.entering_syllable() {
        1
    } else {
        0
    }
}

#[no_mangle]
pub extern "C" fn chewing_cursor_Current(ctx: *const ChewingContext) -> c_int {
    let ctx = match unsafe { ctx.as_ref() } {
        Some(ctx) => ctx,
        None => return -1,
    };

    ctx.editor.cursor() as c_int
}

#[deprecated(note = "The chewing_cand_TotalPage function could achieve the same effect.")]
#[no_mangle]
pub extern "C" fn chewing_cand_CheckDone(ctx: *const ChewingContext) -> c_int {
    let _ctx = match unsafe { ctx.as_ref() } {
        Some(ctx) => ctx,
        None => return -1,
    };

    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_cand_TotalPage(ctx: *const ChewingContext) -> c_int {
    let ctx = match unsafe { ctx.as_ref() } {
        Some(ctx) => ctx,
        None => return -1,
    };

    ctx.editor.total_page().unwrap_or_default() as c_int
}

#[no_mangle]
pub extern "C" fn chewing_cand_ChoicePerPage(ctx: *const ChewingContext) -> c_int {
    let ctx = match unsafe { ctx.as_ref() } {
        Some(ctx) => ctx,
        None => return -1,
    };

    ctx.editor.editor_options().candidates_per_page as c_int
}

#[no_mangle]
pub extern "C" fn chewing_cand_TotalChoice(ctx: *const ChewingContext) -> c_int {
    let ctx = match unsafe { ctx.as_ref() } {
        Some(ctx) => ctx,
        None => return -1,
    };

    match ctx.editor.all_candidates() {
        Ok(candidates) => candidates.len() as c_int,
        Err(_) => 0,
    }
}

#[no_mangle]
pub extern "C" fn chewing_cand_CurrentPage(ctx: *const ChewingContext) -> c_int {
    let ctx = match unsafe { ctx.as_ref() } {
        Some(ctx) => ctx,
        None => return -1,
    };

    ctx.editor.current_page_no().unwrap_or_default() as c_int
}

#[no_mangle]
pub extern "C" fn chewing_cand_Enumerate(ctx: *mut ChewingContext) {
    let ctx = match unsafe { ctx.as_mut() } {
        Some(ctx) => ctx,
        None => return,
    };

    match ctx.editor.paginated_candidates() {
        Ok(candidates) => {
            debug!("candidates: {candidates:?}");
            let phrases = Box::new(candidates.into_iter()) as Box<dyn Iterator<Item = String>>;
            ctx.cand_iter = Some(phrases.peekable());
        }
        Err(_) => (),
    }
}

#[no_mangle]
pub extern "C" fn chewing_cand_hasNext(ctx: *mut ChewingContext) -> c_int {
    let ctx = match unsafe { ctx.as_mut() } {
        Some(ctx) => ctx,
        None => return -1,
    };

    ctx.cand_iter
        .as_mut()
        .and_then(|it| it.peek())
        .map_or(0, |_| 1)
}

#[no_mangle]
pub extern "C" fn chewing_cand_String(ctx: *mut ChewingContext) -> *mut c_char {
    let ctx = match unsafe { ctx.as_mut() } {
        Some(ctx) => ctx,
        None => return owned_into_raw(Owned::CString, CString::default().into_raw()),
    };

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

#[no_mangle]
pub extern "C" fn chewing_cand_String_static(ctx: *mut ChewingContext) -> *const c_char {
    let ctx = match unsafe { ctx.as_mut() } {
        Some(ctx) => ctx,
        None => return unsafe { global_empty_cstr() },
    };

    match ctx.cand_iter.as_mut().and_then(|it| it.next()) {
        Some(phrase) => unsafe { global_cstr(&phrase) },
        None => unsafe { global_empty_cstr() },
    }
}

#[no_mangle]
pub extern "C" fn chewing_cand_string_by_index(
    ctx: *mut ChewingContext,
    index: c_int,
) -> *mut c_char {
    let ctx = match unsafe { ctx.as_mut() } {
        Some(ctx) => ctx,
        None => return owned_into_raw(Owned::CString, CString::default().into_raw()),
    };

    if let Ok(phrases) = ctx.editor.all_candidates() {
        if let Some(phrase) = phrases.get(index as usize) {
            return owned_into_raw(
                Owned::CString,
                CString::new(phrase.to_owned()).unwrap().into_raw(),
            );
        }
    }
    owned_into_raw(Owned::CString, CString::default().into_raw())
}

#[no_mangle]
pub extern "C" fn chewing_cand_string_by_index_static(
    ctx: *mut ChewingContext,
    index: c_int,
) -> *const c_char {
    let ctx = match unsafe { ctx.as_mut() } {
        Some(ctx) => ctx,
        None => return unsafe { global_empty_cstr() },
    };

    if let Ok(phrases) = ctx.editor.all_candidates() {
        if let Some(phrase) = phrases.get(index as usize) {
            return unsafe { global_cstr(&phrase) };
        }
    }
    unsafe { global_empty_cstr() }
}

#[no_mangle]
pub extern "C" fn chewing_cand_choose_by_index(ctx: *mut ChewingContext, index: c_int) -> c_int {
    let ctx = match unsafe { ctx.as_mut() } {
        Some(ctx) => ctx,
        None => return -1,
    };

    match ctx.editor.select(index as usize) {
        Ok(_) => 0,
        Err(_) => -1,
    }
}

#[no_mangle]
pub extern "C" fn chewing_cand_open(ctx: *mut ChewingContext) -> c_int {
    let ctx = match unsafe { ctx.as_mut() } {
        Some(ctx) => ctx,
        None => return -1,
    };

    match ctx.editor.start_selecting() {
        Ok(_) => 0,
        Err(_) => -1,
    }
}

#[no_mangle]
pub extern "C" fn chewing_cand_close(ctx: *mut ChewingContext) -> c_int {
    // FIXME exit selecting mode
    chewing_handle_Up(ctx)
}

#[no_mangle]
pub extern "C" fn chewing_interval_Enumerate(ctx: *mut ChewingContext) {
    let ctx = match unsafe { ctx.as_mut() } {
        Some(ctx) => ctx,
        None => return,
    };

    ctx.interval_iter = Some(
        (Box::new(ctx.editor.intervals().filter(|it| it.is_phrase))
            as Box<dyn Iterator<Item = Interval>>)
            .peekable(),
    );
}

#[no_mangle]
pub extern "C" fn chewing_interval_hasNext(ctx: *mut ChewingContext) -> c_int {
    let ctx = match unsafe { ctx.as_mut() } {
        Some(ctx) => ctx,
        None => return -1,
    };

    ctx.interval_iter.as_mut().map_or(0, |it| match it.peek() {
        Some(_) => 1,
        None => 0,
    })
}

#[no_mangle]
pub extern "C" fn chewing_interval_Get(ctx: *mut ChewingContext, it: *mut IntervalType) {
    let ctx = match unsafe { ctx.as_mut() } {
        Some(ctx) => ctx,
        None => return,
    };

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

#[no_mangle]
pub extern "C" fn chewing_aux_Check(ctx: *const ChewingContext) -> c_int {
    let ctx = match unsafe { ctx.as_ref() } {
        Some(ctx) => ctx,
        None => return -1,
    };

    match !ctx.editor.notification().is_empty() {
        true => 1,
        false => 0,
    }
}

#[no_mangle]
pub extern "C" fn chewing_aux_Length(ctx: *const ChewingContext) -> c_int {
    let ctx = match unsafe { ctx.as_ref() } {
        Some(ctx) => ctx,
        None => return -1,
    };

    ctx.editor.notification().chars().count() as c_int
}

#[no_mangle]
pub extern "C" fn chewing_aux_String(ctx: *const ChewingContext) -> *mut c_char {
    let ctx = match unsafe { ctx.as_ref() } {
        Some(ctx) => ctx,
        None => return owned_into_raw(Owned::CString, CString::default().into_raw()),
    };

    let cstring = CString::new(ctx.editor.notification()).unwrap();
    owned_into_raw(Owned::CString, cstring.into_raw())
}

#[no_mangle]
pub extern "C" fn chewing_aux_String_static(ctx: *const ChewingContext) -> *const c_char {
    let ctx = match unsafe { ctx.as_ref() } {
        Some(ctx) => ctx,
        None => return unsafe { global_empty_cstr() },
    };

    unsafe { global_cstr(&ctx.editor.notification()) }
}

#[no_mangle]
pub extern "C" fn chewing_keystroke_CheckIgnore(ctx: *const ChewingContext) -> c_int {
    let ctx = match unsafe { ctx.as_ref() } {
        Some(ctx) => ctx,
        None => return -1,
    };

    match ctx.editor.last_key_behavior() {
        EditorKeyBehavior::Ignore => 1,
        _ => 0,
    }
}

#[no_mangle]
pub extern "C" fn chewing_keystroke_CheckAbsorb(ctx: *const ChewingContext) -> c_int {
    let ctx = match unsafe { ctx.as_ref() } {
        Some(ctx) => ctx,
        None => return -1,
    };

    match ctx.editor.last_key_behavior() {
        EditorKeyBehavior::Absorb => 1,
        _ => 0,
    }
}

#[no_mangle]
pub extern "C" fn chewing_kbtype_Total(_ctx: *const ChewingContext) -> c_int {
    (0..)
        .into_iter()
        .map_while(|id| KeyboardLayoutCompat::try_from(id).ok())
        .count() as c_int
}

#[no_mangle]
pub extern "C" fn chewing_kbtype_Enumerate(ctx: *mut ChewingContext) {
    let ctx = match unsafe { ctx.as_mut() } {
        Some(ctx) => ctx,
        None => return,
    };

    ctx.kbcompat_iter = Some(
        (Box::new(
            (0..)
                .into_iter()
                .map_while(|id| KeyboardLayoutCompat::try_from(id).ok()),
        ) as Box<dyn Iterator<Item = KeyboardLayoutCompat>>)
            .peekable(),
    )
}

#[no_mangle]
pub extern "C" fn chewing_kbtype_hasNext(ctx: *mut ChewingContext) -> c_int {
    let ctx = match unsafe { ctx.as_mut() } {
        Some(ctx) => ctx,
        None => return -1,
    };

    ctx.kbcompat_iter
        .as_mut()
        .and_then(|it| it.peek())
        .map_or(0, |_| 1)
}

#[no_mangle]
pub extern "C" fn chewing_kbtype_String(ctx: *mut ChewingContext) -> *mut c_char {
    let ctx = match unsafe { ctx.as_mut() } {
        Some(ctx) => ctx,
        None => return owned_into_raw(Owned::CString, CString::default().into_raw()),
    };

    match ctx.kbcompat_iter.as_mut().and_then(|it| it.next()) {
        Some(kb_compat) => {
            let cstr = match CString::new(String::from(kb_compat.to_string())) {
                Ok(cstr) => cstr,
                Err(_) => return null_mut(),
            };
            owned_into_raw(Owned::CString, cstr.into_raw())
        }
        None => owned_into_raw(Owned::CString, CString::default().into_raw()),
    }
}

#[no_mangle]
pub extern "C" fn chewing_kbtype_String_static(ctx: *mut ChewingContext) -> *const c_char {
    let ctx = match unsafe { ctx.as_mut() } {
        Some(ctx) => ctx,
        None => return unsafe { global_empty_cstr() },
    };

    match ctx.kbcompat_iter.as_mut().and_then(|it| it.next()) {
        Some(kb_compat) => unsafe { global_cstr(&kb_compat.to_string()) },
        None => unsafe { global_empty_cstr() },
    }
}

#[no_mangle]
#[deprecated]
pub extern "C" fn chewing_zuin_Check(ctx: *const ChewingContext) -> c_int {
    chewing_bopomofo_Check(ctx) ^ 1
}

#[no_mangle]
#[deprecated]
pub extern "C" fn chewing_zuin_String(
    ctx: *const ChewingContext,
    zuin_count: *mut c_int,
) -> *mut c_char {
    let ctx = match unsafe { ctx.as_ref() } {
        Some(ctx) => ctx,
        None => return owned_into_raw(Owned::CString, CString::default().into_raw()),
    };

    let syllable = ctx.editor.syllable_buffer().to_string();
    unsafe {
        *zuin_count = syllable.chars().count() as c_int;
    }
    let cstr = match CString::new(syllable) {
        Ok(cstr) => cstr,
        Err(_) => return null_mut(),
    };
    owned_into_raw(Owned::CString, cstr.into_raw())
}

#[no_mangle]
#[deprecated]
pub extern "C" fn chewing_Init(data_path: *const c_char, hash_path: *const c_char) -> c_int {
    0
}

#[no_mangle]
#[deprecated]
pub extern "C" fn chewing_Terminate() {}

#[no_mangle]
#[deprecated]
pub extern "C" fn chewing_Configure(
    ctx: *mut ChewingContext,
    pcd: *mut ChewingConfigData,
) -> c_int {
    let pcd = match unsafe { pcd.as_ref() } {
        Some(pcd) => pcd,
        None => return -1,
    };

    chewing_set_candPerPage(ctx, pcd.cand_per_page);
    chewing_set_maxChiSymbolLen(ctx, pcd.max_chi_symbol_len);
    chewing_set_selKey(ctx, pcd.sel_key.as_ptr(), MAX_SELKEY as i32);
    chewing_set_addPhraseDirection(ctx, pcd.b_add_phrase_forward);
    chewing_set_spaceAsSelection(ctx, pcd.b_space_as_selection);
    chewing_set_escCleanAllBuf(ctx, pcd.b_esc_clean_all_buf);
    chewing_set_autoShiftCur(ctx, pcd.b_auto_shift_cur);
    chewing_set_easySymbolInput(ctx, pcd.b_easy_symbol_input);
    chewing_set_phraseChoiceRearward(ctx, pcd.b_phrase_choice_rearward);

    0
}

#[no_mangle]
#[deprecated]
pub extern "C" fn chewing_set_hsuSelKeyType(_ctx: *mut ChewingContext, mode: c_int) {}

#[no_mangle]
#[deprecated]
pub extern "C" fn chewing_get_hsuSelKeyType(_ctx: *mut ChewingContext) -> c_int {
    0
}
