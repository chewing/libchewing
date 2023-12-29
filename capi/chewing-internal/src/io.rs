use std::{
    cmp::min,
    collections::BTreeMap,
    ffi::{c_char, c_int, c_uint, c_ushort, c_void, CStr, CString},
    ptr::{null, null_mut},
    rc::Rc,
    slice,
    sync::OnceLock,
    u8,
};

use chewing::{
    conversion::{ChewingConversionEngine, Interval, Symbol},
    dictionary::{
        LayeredDictionary, Phrase, Phrases, SystemDictionaryLoader, UserDictionaryLoader,
    },
    editor::{
        keyboard::{AnyKeyboardLayout, KeyCode, KeyboardLayout, Modifiers, Qwerty},
        syllable::{
            DaiChien26, Et, Et26, GinYieh, Hsu, Ibm, KeyBehavior, KeyboardLayoutCompat, Pinyin,
            Standard,
        },
        BasicEditor, CharacterForm, Editor, EditorKeyBehavior, EditorOptions, LanguageMode,
        SyllableEditor,
    },
    zhuyin::Syllable,
};
use chewing_public::types::{
    ChewingConfigData, IntervalType, CHINESE_MODE, FULLSHAPE_MODE, HALFSHAPE_MODE, SYMBOL_MODE,
};
use tracing::{debug, level_filters::LevelFilter, warn};

use crate::types::ChewingContext;

#[no_mangle]
pub extern "C" fn rust_link_io() {}

enum Owned {
    CString,
    CUShortSlice(usize),
    CIntSlice(usize),
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
                    Owned::CIntSlice(len) => drop(unsafe { Vec::from_raw_parts(ptr, *len, *len) }),
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

#[tracing::instrument(ret)]
#[no_mangle]
pub extern "C" fn chewing_new() -> *mut ChewingContext {
    chewing_new2(null(), null(), None, null_mut())
}

#[tracing::instrument(ret)]
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
    let mut dictionaries = if syspath.is_null() {
        SystemDictionaryLoader::new()
            .load()
            .expect("unable to find any system dictionary")
    } else {
        let search_path = unsafe { CStr::from_ptr(syspath) }
            .to_str()
            .expect("invalid syspath string");
        SystemDictionaryLoader::new()
            .sys_path(search_path)
            .load()
            .expect("unable to find any system dictionary")
    };
    let user_dictionary = if userpath.is_null() {
        UserDictionaryLoader::new()
            .load()
            .expect("unable to load user dictionary")
    } else {
        let data_path = unsafe { CStr::from_ptr(userpath) }
            .to_str()
            .expect("invalid syspath string");
        UserDictionaryLoader::new()
            .userphrase_path(data_path)
            .load()
            .expect("unable to load user dictionary")
    };
    dictionaries.insert(0, user_dictionary);

    let dict = Rc::new(LayeredDictionary::new(dictionaries, vec![]));
    let conversion_engine = ChewingConversionEngine::new(dict.clone());
    let kb_compat = KeyboardLayoutCompat::Default;
    let keyboard = AnyKeyboardLayout::Qwerty(Qwerty);
    let editor = Editor::new(conversion_engine, dict);
    let context = Box::new(ChewingContext {
        kb_compat,
        keyboard,
        editor,
        kbcompat_iter: None,
        cand_iter: None,
        interval_iter: None,
    });
    Box::into_raw(context)
}

#[tracing::instrument(skip(ctx), ret)]
#[no_mangle]
pub extern "C" fn chewing_delete(ctx: *mut ChewingContext) {
    if !ctx.is_null() {
        drop(unsafe { Box::from_raw(ctx) })
    }
}

#[tracing::instrument(ret)]
#[no_mangle]
pub extern "C" fn chewing_free(ptr: *mut c_void) {
    if !ptr.is_null() {
        drop_owned(ptr);
    }
}

#[tracing::instrument(skip(ctx), ret)]
#[no_mangle]
pub extern "C" fn chewing_Reset(ctx: *mut ChewingContext) -> c_int {
    let ctx = match unsafe { ctx.as_mut() } {
        Some(ctx) => ctx,
        None => return -1,
    };
    ctx.editor.clear();
    0
}

#[tracing::instrument(skip(ctx), ret)]
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

#[tracing::instrument(skip(ctx), ret)]
#[no_mangle]
pub extern "C" fn chewing_get_KBType(ctx: *const ChewingContext) -> c_int {
    let ctx = match unsafe { ctx.as_ref() } {
        Some(ctx) => ctx,
        None => return -1,
    };
    ctx.kb_compat as c_int
}

#[tracing::instrument(skip(ctx), ret)]
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

#[tracing::instrument(ret)]
#[no_mangle]
pub extern "C" fn chewing_KBStr2Num(str: *const c_char) -> c_int {
    let cstr = unsafe { CStr::from_ptr(str) };
    let utf8str = cstr.to_string_lossy();
    let layout: KeyboardLayoutCompat = utf8str.parse().unwrap_or(KeyboardLayoutCompat::Default);
    layout as c_int
}

#[tracing::instrument(skip(ctx), ret)]
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

#[tracing::instrument(skip(ctx), ret)]
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

#[tracing::instrument(skip(ctx), ret)]
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

#[tracing::instrument(skip(ctx), ret)]
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

#[tracing::instrument(skip(ctx), ret)]
#[no_mangle]
pub extern "C" fn chewing_set_candPerPage(ctx: *mut ChewingContext, n: c_int) {
    let ctx = match unsafe { ctx.as_mut() } {
        Some(ctx) => ctx,
        None => return,
    };

    ctx.editor.set_editor_options(EditorOptions {
        candidates_per_page: n as usize,
        ..ctx.editor.editor_options()
    });
}

#[tracing::instrument(skip(ctx), ret)]
#[no_mangle]
pub extern "C" fn chewing_get_candPerPage(ctx: *const ChewingContext) -> c_int {
    let ctx = match unsafe { ctx.as_ref() } {
        Some(ctx) => ctx,
        None => return -1,
    };

    ctx.editor.editor_options().candidates_per_page as c_int
}

#[tracing::instrument(skip(ctx), ret)]
#[no_mangle]
pub extern "C" fn chewing_set_maxChiSymbolLen(ctx: *mut ChewingContext, n: c_int) {
    let ctx = match unsafe { ctx.as_mut() } {
        Some(ctx) => ctx,
        None => return,
    };

    ctx.editor.set_editor_options(EditorOptions {
        auto_commit_threshold: n as usize,
        ..ctx.editor.editor_options()
    });
}

#[tracing::instrument(skip(ctx), ret)]
#[no_mangle]
pub extern "C" fn chewing_get_maxChiSymbolLen(ctx: *const ChewingContext) -> c_int {
    let ctx = match unsafe { ctx.as_ref() } {
        Some(ctx) => ctx,
        None => return -1,
    };

    ctx.editor.editor_options().auto_commit_threshold as c_int
}

#[tracing::instrument(skip(ctx), ret)]
#[no_mangle]
pub extern "C" fn chewing_set_selKey(ctx: *mut ChewingContext, sel_keys: *const c_int, len: c_int) {
    let ctx = match unsafe { ctx.as_mut() } {
        Some(ctx) => ctx,
        None => return,
    };
    // todo!()
}

#[tracing::instrument(skip(ctx), ret)]
#[no_mangle]
pub extern "C" fn chewing_get_selKey(ctx: *const ChewingContext) -> *mut c_int {
    let ctx = match unsafe { ctx.as_ref() } {
        Some(ctx) => ctx,
        None => return null_mut(),
    };

    todo!()
}

#[tracing::instrument(skip(ctx), ret)]
#[no_mangle]
pub extern "C" fn chewing_set_addPhraseDirection(ctx: *mut ChewingContext, direction: c_int) {
    let ctx = match unsafe { ctx.as_mut() } {
        Some(ctx) => ctx,
        None => return,
    };

    // todo!()
}

#[tracing::instrument(skip(ctx), ret)]
#[no_mangle]
pub extern "C" fn chewing_get_addPhraseDirection(ctx: *const ChewingContext) -> c_int {
    let ctx = match unsafe { ctx.as_ref() } {
        Some(ctx) => ctx,
        None => return -1,
    };

    todo!()
}

#[tracing::instrument(skip(ctx), ret)]
#[no_mangle]
pub extern "C" fn chewing_set_spaceAsSelection(ctx: *mut ChewingContext, mode: c_int) {
    let ctx = match unsafe { ctx.as_mut() } {
        Some(ctx) => ctx,
        None => return,
    };

    ctx.editor.set_editor_options(EditorOptions {
        space_is_select_key: match mode {
            0 => false,
            _ => true,
        },
        ..ctx.editor.editor_options()
    });
}

#[tracing::instrument(skip(ctx), ret)]
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

#[tracing::instrument(skip(ctx), ret)]
#[no_mangle]
pub extern "C" fn chewing_set_escCleanAllBuf(ctx: *mut ChewingContext, mode: c_int) {
    let ctx = match unsafe { ctx.as_mut() } {
        Some(ctx) => ctx,
        None => return,
    };

    ctx.editor.set_editor_options(EditorOptions {
        esc_clear_all_buffer: match mode {
            0 => false,
            _ => true,
        },
        ..ctx.editor.editor_options()
    });
}

#[tracing::instrument(skip(ctx), ret)]
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

#[tracing::instrument(skip(ctx), ret)]
#[no_mangle]
pub extern "C" fn chewing_set_autoShiftCur(ctx: *mut ChewingContext, mode: c_int) {
    let ctx = match unsafe { ctx.as_mut() } {
        Some(ctx) => ctx,
        None => return,
    };

    ctx.editor.set_editor_options(EditorOptions {
        auto_shift_cursor: match mode {
            0 => false,
            _ => true,
        },
        ..ctx.editor.editor_options()
    });
}

#[tracing::instrument(skip(ctx), ret)]
#[no_mangle]
pub extern "C" fn chewing_get_autoShiftCur(ctx: *const ChewingContext) -> c_int {
    let ctx = match unsafe { ctx.as_ref() } {
        Some(ctx) => ctx,
        None => return -1,
    };

    todo!()
}

#[tracing::instrument(skip(ctx), ret)]
#[no_mangle]
pub extern "C" fn chewing_set_easySymbolInput(ctx: *mut ChewingContext, mode: c_int) {
    let ctx = match unsafe { ctx.as_mut() } {
        Some(ctx) => ctx,
        None => return,
    };

    ctx.editor.set_editor_options(EditorOptions {
        easy_symbol_input: match mode {
            0 => false,
            _ => true,
        },
        ..ctx.editor.editor_options()
    });
}

#[tracing::instrument(skip(ctx), ret)]
#[no_mangle]
pub extern "C" fn chewing_get_easySymbolInput(ctx: *const ChewingContext) -> c_int {
    let ctx = match unsafe { ctx.as_ref() } {
        Some(ctx) => ctx,
        None => return -1,
    };

    ctx.editor.editor_options().easy_symbol_input as c_int
}

#[tracing::instrument(skip(ctx), ret)]
#[no_mangle]
pub extern "C" fn chewing_set_phraseChoiceRearward(ctx: *mut ChewingContext, mode: c_int) {
    let ctx = match unsafe { ctx.as_mut() } {
        Some(ctx) => ctx,
        None => return,
    };

    ctx.editor.set_editor_options(EditorOptions {
        phrase_choice_rearward: match mode {
            0 => false,
            _ => true,
        },
        ..Default::default()
    });
}

#[tracing::instrument(skip(ctx), ret)]
#[no_mangle]
pub extern "C" fn chewing_get_phraseChoiceRearward(ctx: *const ChewingContext) -> c_int {
    let ctx = match unsafe { ctx.as_ref() } {
        Some(ctx) => ctx,
        None => return -1,
    };

    todo!()
}

#[tracing::instrument(skip(ctx), ret)]
#[no_mangle]
pub extern "C" fn chewing_set_autoLearn(ctx: *mut ChewingContext, mode: c_int) {
    todo!()
}

#[tracing::instrument(skip(ctx), ret)]
#[no_mangle]
pub extern "C" fn chewing_get_autoLearn(ctx: *const ChewingContext) -> c_int {
    todo!()
}

#[tracing::instrument(skip(ctx), ret)]
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

#[tracing::instrument(skip(ctx), ret)]
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

#[tracing::instrument(skip(ctx), ret)]
#[no_mangle]
pub extern "C" fn chewing_set_logger(
    ctx: *mut ChewingContext,
    logger: extern "C" fn(data: *mut c_void, level: c_int, fmt: *const c_char, arg: ...),
    data: *mut c_void,
) {
    let _ = tracing_subscriber::fmt::try_init();
}

#[tracing::instrument(skip(ctx), ret)]
#[no_mangle]
pub extern "C" fn chewing_userphrase_enumerate(ctx: *mut ChewingContext) -> c_int {
    let ctx = match unsafe { ctx.as_ref() } {
        Some(ctx) => ctx,
        None => return -1,
    };

    todo!()
}

#[tracing::instrument(skip(ctx), ret)]
#[no_mangle]
pub extern "C" fn chewing_userphrase_has_next(
    ctx: *mut ChewingContext,
    phrase_len: *mut c_uint,
    bopomofo_len: *mut c_uint,
) -> c_int {
    let ctx = match unsafe { ctx.as_ref() } {
        Some(ctx) => ctx,
        None => return 0,
    };

    todo!()
}

#[tracing::instrument(skip(ctx), ret)]
#[no_mangle]
pub extern "C" fn chewing_userphrase_get(
    ctx: *mut ChewingContext,
    phrase_buf: *mut c_char,
    phrase_len: c_uint,
    bopomofo_buf: *mut c_char,
    bopomofo_len: c_uint,
) -> c_int {
    let ctx = match unsafe { ctx.as_ref() } {
        Some(ctx) => ctx,
        None => return -1,
    };

    todo!()
}

#[tracing::instrument(skip(ctx), ret)]
#[no_mangle]
pub extern "C" fn chewing_userphrase_add(
    ctx: *mut ChewingContext,
    phrase_buf: *const c_char,
    bopomofo_buf: *const c_char,
) -> c_int {
    let ctx = match unsafe { ctx.as_ref() } {
        Some(ctx) => ctx,
        None => return -1,
    };

    todo!()
}

#[tracing::instrument(skip(ctx), ret)]
#[no_mangle]
pub extern "C" fn chewing_userphrase_remove(
    ctx: *mut ChewingContext,
    phrase_buf: *const c_char,
    bopomofo_buf: *const c_char,
) -> c_int {
    let ctx = match unsafe { ctx.as_ref() } {
        Some(ctx) => ctx,
        None => return -1,
    };

    todo!()
}

#[tracing::instrument(skip(ctx), ret)]
#[no_mangle]
pub extern "C" fn chewing_userphrase_lookup(
    ctx: *mut ChewingContext,
    phrase_buf: *const c_char,
    bopomofo_buf: *const c_char,
) -> c_int {
    let ctx = match unsafe { ctx.as_ref() } {
        Some(ctx) => ctx,
        None => return 0,
    };

    todo!()
}

#[tracing::instrument(skip(ctx), ret)]
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

#[tracing::instrument(skip(ctx), ret)]
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

#[tracing::instrument(skip(ctx), ret)]
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

#[tracing::instrument(skip(ctx), ret)]
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

#[tracing::instrument(skip(ctx), ret)]
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

#[tracing::instrument(skip(ctx), ret)]
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

#[tracing::instrument(skip(ctx), ret)]
#[no_mangle]
pub extern "C" fn chewing_commit_preedit_buf(ctx: *mut ChewingContext) -> c_int {
    let ctx = match unsafe { ctx.as_mut() } {
        Some(ctx) => ctx,
        None => return -1,
    };

    // FIXME
    if !ctx.editor.is_entering() || ctx.editor.display().is_empty() {
        -1
    } else {
        chewing_handle_Enter(ctx)
    }
}

#[tracing::instrument(skip(ctx), ret)]
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

#[tracing::instrument(skip(ctx), ret)]
#[no_mangle]
pub extern "C" fn chewing_clean_bopomofo_buf(ctx: *mut ChewingContext) -> c_int {
    let ctx = match unsafe { ctx.as_mut() } {
        Some(ctx) => ctx,
        None => return -1,
    };

    ctx.editor.clear_syllable_editor();
    0
}

#[tracing::instrument(ret)]
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

#[tracing::instrument(skip(ctx), ret)]
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

#[tracing::instrument(skip(ctx), ret)]
#[no_mangle]
pub extern "C" fn chewing_handle_Esc(ctx: *mut ChewingContext) -> c_int {
    let ctx = match unsafe { ctx.as_mut() } {
        Some(ctx) => ctx,
        None => return -1,
    };

    ctx.editor.process_keyevent(ctx.keyboard.map(KeyCode::Esc));
    0
}

#[tracing::instrument(skip(ctx), ret)]
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

#[tracing::instrument(skip(ctx), ret)]
#[no_mangle]
pub extern "C" fn chewing_handle_Del(ctx: *mut ChewingContext) -> c_int {
    let ctx = match unsafe { ctx.as_mut() } {
        Some(ctx) => ctx,
        None => return -1,
    };

    ctx.editor.process_keyevent(ctx.keyboard.map(KeyCode::Del));
    0
}

#[tracing::instrument(skip(ctx), ret)]
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

#[tracing::instrument(skip(ctx), ret)]
#[no_mangle]
pub extern "C" fn chewing_handle_Tab(ctx: *mut ChewingContext) -> c_int {
    let ctx = match unsafe { ctx.as_mut() } {
        Some(ctx) => ctx,
        None => return -1,
    };

    ctx.editor.process_keyevent(ctx.keyboard.map(KeyCode::Tab));
    0
}

#[tracing::instrument(skip(ctx), ret)]
#[no_mangle]
pub extern "C" fn chewing_handle_ShiftLeft(ctx: *mut ChewingContext) -> c_int {
    let ctx = match unsafe { ctx.as_mut() } {
        Some(ctx) => ctx,
        None => return -1,
    };

    todo!()
}

#[tracing::instrument(skip(ctx), ret)]
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

#[tracing::instrument(skip(ctx), ret)]
#[no_mangle]
pub extern "C" fn chewing_handle_ShiftRight(ctx: *mut ChewingContext) -> c_int {
    let ctx = match unsafe { ctx.as_mut() } {
        Some(ctx) => ctx,
        None => return -1,
    };

    todo!()
}

#[tracing::instrument(skip(ctx), ret)]
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

#[tracing::instrument(skip(ctx), ret)]
#[no_mangle]
pub extern "C" fn chewing_handle_Up(ctx: *mut ChewingContext) -> c_int {
    let ctx = match unsafe { ctx.as_mut() } {
        Some(ctx) => ctx,
        None => return -1,
    };

    ctx.editor.process_keyevent(ctx.keyboard.map(KeyCode::Up));
    0
}

#[tracing::instrument(skip(ctx), ret)]
#[no_mangle]
pub extern "C" fn chewing_handle_Home(ctx: *mut ChewingContext) -> c_int {
    let ctx = match unsafe { ctx.as_mut() } {
        Some(ctx) => ctx,
        None => return -1,
    };

    ctx.editor.process_keyevent(ctx.keyboard.map(KeyCode::Home));
    0
}

#[tracing::instrument(skip(ctx), ret)]
#[no_mangle]
pub extern "C" fn chewing_handle_End(ctx: *mut ChewingContext) -> c_int {
    let ctx = match unsafe { ctx.as_mut() } {
        Some(ctx) => ctx,
        None => return -1,
    };

    ctx.editor.process_keyevent(ctx.keyboard.map(KeyCode::End));
    0
}

#[tracing::instrument(skip(ctx), ret)]
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

#[tracing::instrument(skip(ctx), ret)]
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

#[tracing::instrument(skip(ctx), ret)]
#[no_mangle]
pub extern "C" fn chewing_handle_Down(ctx: *mut ChewingContext) -> c_int {
    let ctx = match unsafe { ctx.as_mut() } {
        Some(ctx) => ctx,
        None => return -1,
    };

    ctx.editor.process_keyevent(ctx.keyboard.map(KeyCode::Down));
    0
}

#[tracing::instrument(skip(ctx), ret)]
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

#[tracing::instrument(skip(ctx), ret)]
#[no_mangle]
pub extern "C" fn chewing_handle_Default(ctx: *mut ChewingContext, key: c_int) -> c_int {
    let ctx = match unsafe { ctx.as_mut() } {
        Some(ctx) => ctx,
        None => return -1,
    };

    ctx.editor
        .process_keyevent(ctx.keyboard.map_ascii(key as u8));
    0
}

#[tracing::instrument(skip(ctx), ret)]
#[no_mangle]
pub extern "C" fn chewing_handle_CtrlNum(ctx: *mut ChewingContext, key: c_int) -> c_int {
    let ctx = match unsafe { ctx.as_mut() } {
        Some(ctx) => ctx,
        None => return -1,
    };

    todo!()
}

#[tracing::instrument(skip(ctx), ret)]
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

#[tracing::instrument(skip(ctx), ret)]
#[no_mangle]
pub extern "C" fn chewing_handle_DblTab(ctx: *mut ChewingContext) -> c_int {
    let ctx = match unsafe { ctx.as_mut() } {
        Some(ctx) => ctx,
        None => return -1,
    };

    todo!()
}

#[tracing::instrument(skip(ctx), ret)]
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

#[tracing::instrument(skip(ctx), ret)]
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

#[tracing::instrument(skip(ctx), ret)]
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

#[tracing::instrument(skip(ctx), ret)]
#[no_mangle]
pub extern "C" fn chewing_commit_String_static(ctx: *const ChewingContext) -> *const c_char {
    let ctx = match unsafe { ctx.as_ref() } {
        Some(ctx) => ctx,
        None => return unsafe { global_empty_cstr() },
    };

    let buffer = ctx.editor.display_commit();
    unsafe { global_cstr(&buffer) }
}

#[tracing::instrument(skip(ctx), ret)]
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

#[tracing::instrument(skip(ctx), ret)]
#[no_mangle]
pub extern "C" fn chewing_buffer_String_static(ctx: *const ChewingContext) -> *const c_char {
    let ctx = match unsafe { ctx.as_ref() } {
        Some(ctx) => ctx,
        None => return unsafe { global_empty_cstr() },
    };

    let buffer = ctx.editor.display();
    unsafe { global_cstr(&buffer) }
}

#[tracing::instrument(skip(ctx), ret)]
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

#[tracing::instrument(skip(ctx), ret)]
#[no_mangle]
pub extern "C" fn chewing_buffer_Len(ctx: *const ChewingContext) -> c_int {
    let ctx = match unsafe { ctx.as_ref() } {
        Some(ctx) => ctx,
        None => return -1,
    };

    ctx.editor.display().chars().count() as c_int
}

#[tracing::instrument(skip(ctx), ret)]
#[no_mangle]
pub extern "C" fn chewing_bopomofo_String_static(ctx: *const ChewingContext) -> *const c_char {
    let ctx = match unsafe { ctx.as_ref() } {
        Some(ctx) => ctx,
        None => return unsafe { global_empty_cstr() },
    };

    let syllable = ctx.editor.syllable_buffer().to_string();
    unsafe { global_cstr(&syllable) }
}

#[tracing::instrument(skip(ctx), ret)]
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

#[tracing::instrument(skip(ctx), ret)]
#[no_mangle]
pub extern "C" fn chewing_cursor_Current(ctx: *const ChewingContext) -> c_int {
    let ctx = match unsafe { ctx.as_ref() } {
        Some(ctx) => ctx,
        None => return -1,
    };

    ctx.editor.cursor() as c_int
}

#[tracing::instrument(skip(ctx), ret)]
#[no_mangle]
pub extern "C" fn chewing_cand_CheckDone(ctx: *const ChewingContext) -> c_int {
    let ctx = match unsafe { ctx.as_ref() } {
        Some(ctx) => ctx,
        None => return -1,
    };

    todo!()
}

#[tracing::instrument(skip(ctx), ret)]
#[no_mangle]
pub extern "C" fn chewing_cand_TotalPage(ctx: *const ChewingContext) -> c_int {
    let ctx = match unsafe { ctx.as_ref() } {
        Some(ctx) => ctx,
        None => return -1,
    };

    ctx.editor.total_page().unwrap_or_default() as c_int
}

#[tracing::instrument(skip(ctx), ret)]
#[no_mangle]
pub extern "C" fn chewing_cand_ChoicePerPage(ctx: *const ChewingContext) -> c_int {
    let ctx = match unsafe { ctx.as_ref() } {
        Some(ctx) => ctx,
        None => return -1,
    };

    ctx.editor.editor_options().candidates_per_page as c_int
}

#[tracing::instrument(skip(ctx), ret)]
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

#[tracing::instrument(skip(ctx), ret)]
#[no_mangle]
pub extern "C" fn chewing_cand_CurrentPage(ctx: *const ChewingContext) -> c_int {
    let ctx = match unsafe { ctx.as_ref() } {
        Some(ctx) => ctx,
        None => return -1,
    };

    ctx.editor.current_page_no().unwrap_or_default() as c_int
}

#[tracing::instrument(skip(ctx), ret)]
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

#[tracing::instrument(skip(ctx), ret)]
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

#[tracing::instrument(skip(ctx), ret)]
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

#[tracing::instrument(skip(ctx), ret)]
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

#[tracing::instrument(skip(ctx), ret)]
#[no_mangle]
pub extern "C" fn chewing_cand_string_by_index(
    ctx: *mut ChewingContext,
    index: c_int,
) -> *mut c_char {
    let ctx = match unsafe { ctx.as_mut() } {
        Some(ctx) => ctx,
        None => return owned_into_raw(Owned::CString, CString::default().into_raw()),
    };

    todo!()
}

#[tracing::instrument(skip(ctx), ret)]
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
            return unsafe { global_cstr(&String::from(phrase)) };
        }
    }
    unsafe { global_empty_cstr() }
}

#[tracing::instrument(skip(ctx), ret)]
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

#[tracing::instrument(skip(ctx), ret)]
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

#[tracing::instrument(skip(ctx), ret)]
#[no_mangle]
pub extern "C" fn chewing_cand_close(ctx: *mut ChewingContext) -> c_int {
    // FIXME exit selecting mode
    chewing_handle_Up(ctx)
}

#[tracing::instrument(skip(ctx), ret)]
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

#[tracing::instrument(skip(ctx), ret)]
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

#[tracing::instrument(skip(ctx), ret)]
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

#[tracing::instrument(skip(ctx), ret)]
#[no_mangle]
pub extern "C" fn chewing_aux_Check(ctx: *const ChewingContext) -> c_int {
    let ctx = match unsafe { ctx.as_ref() } {
        Some(ctx) => ctx,
        None => return -1,
    };

    todo!()
}

#[tracing::instrument(skip(ctx), ret)]
#[no_mangle]
pub extern "C" fn chewing_aux_Length(ctx: *const ChewingContext) -> c_int {
    let ctx = match unsafe { ctx.as_ref() } {
        Some(ctx) => ctx,
        None => return -1,
    };

    0
}

#[tracing::instrument(skip(ctx), ret)]
#[no_mangle]
pub extern "C" fn chewing_aux_String(ctx: *const ChewingContext) -> *mut c_char {
    let ctx = match unsafe { ctx.as_ref() } {
        Some(ctx) => ctx,
        None => return owned_into_raw(Owned::CString, CString::default().into_raw()),
    };

    todo!()
}

#[tracing::instrument(skip(ctx), ret)]
#[no_mangle]
pub extern "C" fn chewing_aux_String_static(ctx: *const ChewingContext) -> *const c_char {
    let ctx = match unsafe { ctx.as_ref() } {
        Some(ctx) => ctx,
        None => return unsafe { global_empty_cstr() },
    };

    todo!()
}

#[tracing::instrument(skip(ctx), ret)]
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

#[tracing::instrument(skip(ctx), ret)]
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

#[tracing::instrument(skip(ctx), ret)]
#[no_mangle]
pub extern "C" fn chewing_kbtype_Total(ctx: *const ChewingContext) -> c_int {
    (0..)
        .into_iter()
        .map_while(|id| KeyboardLayoutCompat::try_from(id).ok())
        .count() as c_int
}

#[tracing::instrument(skip(ctx), ret)]
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

#[tracing::instrument(skip(ctx), ret)]
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

#[tracing::instrument(skip(ctx), ret)]
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

#[tracing::instrument(skip(ctx), ret)]
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

#[tracing::instrument(skip(ctx), ret)]
#[no_mangle]
#[deprecated]
pub extern "C" fn chewing_zuin_Check(ctx: *const ChewingContext) -> c_int {
    chewing_bopomofo_Check(ctx) ^ 1
}

#[tracing::instrument(skip(ctx), ret)]
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

#[tracing::instrument(ret)]
#[no_mangle]
#[deprecated]
pub extern "C" fn chewing_Init(data_path: *const c_char, hash_path: *const c_char) -> c_int {
    todo!()
}

#[tracing::instrument(ret)]
#[no_mangle]
#[deprecated]
pub extern "C" fn chewing_Terminate() {
    todo!()
}

#[tracing::instrument(skip(ctx), ret)]
#[no_mangle]
#[deprecated]
pub extern "C" fn chewing_Configure(
    ctx: *mut ChewingContext,
    pcd: *mut ChewingConfigData,
) -> c_int {
    todo!()
}

#[tracing::instrument(skip(ctx), ret)]
#[no_mangle]
#[deprecated]
pub extern "C" fn chewing_set_hsuSelKeyType(ctx: *mut ChewingContext, mode: c_int) {
    todo!()
}

#[tracing::instrument(skip(ctx), ret)]
#[no_mangle]
#[deprecated]
pub extern "C" fn chewing_get_hsuSelKeyType(ctx: *mut ChewingContext) -> c_int {
    todo!()
}
