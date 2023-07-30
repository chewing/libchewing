use std::{
    any::TypeId,
    collections::{hash_map::DefaultHasher, BTreeMap},
    ffi::{c_char, c_int, c_uint, c_ushort, c_void, CStr, CString},
    hash::{Hash, Hasher},
    ptr::{null, null_mut},
    rc::Rc,
    sync::OnceLock,
    u8,
};

use chewing::{
    conversion::ChewingConversionEngine,
    dictionary::{LayeredDictionary, SystemDictionaryLoader, UserDictionaryLoader},
    editor::{
        keyboard::{AnyKeyboardLayout, KeyCode, KeyboardLayout, Modifiers, Qwerty},
        BasicEditor, CharacterForm, Editor, LanguageMode,
    },
};
use chewing_public::types::{
    ChewingConfigData, IntervalType, CHINESE_MODE, FULLSHAPE_MODE, HALFSHAPE_MODE, SYMBOL_MODE,
};
use tracing::warn;

use crate::types::ChewingContext;

#[no_mangle]
pub extern "C" fn rust_link_io() {}

enum Owned {
    CString,
}

static mut OWNED: OnceLock<BTreeMap<*mut c_void, Owned>> = OnceLock::new();

fn owned_into_raw<T>(owned: Owned, ptr: *mut T) -> *mut T {
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
                }
            }
        }
        None => (),
    }
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
    let keyboard = AnyKeyboardLayout::Qwerty(Qwerty);
    let editor = Editor::new(conversion_engine, dict);
    let context = Box::new(ChewingContext { keyboard, editor });
    Box::into_raw(context)
}

#[no_mangle]
pub extern "C" fn chewing_delete(ctx: *mut ChewingContext) {
    if !ctx.is_null() {
        drop(unsafe { Box::from_raw(ctx) })
    }
}

#[no_mangle]
pub extern "C" fn chewing_free(ptr: *mut c_void) {
    drop_owned(ptr);
}

#[no_mangle]
pub extern "C" fn chewing_Reset(ctx: &mut ChewingContext) -> c_int {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_set_KBType(ctx: &mut ChewingContext, kbtype: c_int) -> c_int {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_get_KBType(ctx: &ChewingContext) -> c_int {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_get_KBString(ctx: &ChewingContext) -> *mut c_char {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_KBStr2Num(str: *const c_char) -> c_int {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_set_ChiEngMode(ctx: &mut ChewingContext, mode: c_int) {
    match mode {
        CHINESE_MODE => ctx.editor.language_mode = LanguageMode::Chinese,
        SYMBOL_MODE => ctx.editor.language_mode = LanguageMode::English,
        _ => warn!("invalid language mode {}", mode),
    }
}

#[no_mangle]
pub extern "C" fn chewing_get_ChiEngMode(ctx: &ChewingContext) -> c_int {
    match ctx.editor.language_mode {
        LanguageMode::Chinese => CHINESE_MODE,
        LanguageMode::English => SYMBOL_MODE,
    }
}

#[no_mangle]
pub extern "C" fn chewing_set_ShapeMode(ctx: &mut ChewingContext, mode: c_int) {
    match mode {
        HALFSHAPE_MODE => ctx.editor.character_form = CharacterForm::Halfwidth,
        FULLSHAPE_MODE => ctx.editor.character_form = CharacterForm::Fullwidth,
        _ => warn!("invalid shape mode {}", mode),
    }
}

#[no_mangle]
pub extern "C" fn chewing_get_ShapeMode(ctx: &ChewingContext) -> c_int {
    match ctx.editor.character_form {
        CharacterForm::Halfwidth => HALFSHAPE_MODE,
        CharacterForm::Fullwidth => FULLSHAPE_MODE,
    }
}

#[no_mangle]
pub extern "C" fn chewing_set_candPerPage(ctx: &mut ChewingContext, n: c_int) {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_get_candPerPage(ctx: &ChewingContext) -> c_int {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_set_maxChiSymbolLen(ctx: &mut ChewingContext, n: c_int) {
    ctx.editor.options.auto_commit_threshold = n as usize
}

#[no_mangle]
pub extern "C" fn chewing_get_maxChiSymbolLen(ctx: &ChewingContext) -> c_int {
    ctx.editor.options.auto_commit_threshold as c_int
}

#[no_mangle]
pub extern "C" fn chewing_set_selKey(ctx: &mut ChewingContext, sel_keys: *const c_int, len: c_int) {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_get_selKey(ctx: &ChewingContext) -> *mut c_int {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_set_addPhraseDirection(ctx: &mut ChewingContext, direction: c_int) {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_get_addPhraseDirection(ctx: &ChewingContext) -> c_int {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_set_spaceAsSelection(ctx: &mut ChewingContext, mode: c_int) {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_get_spaceAsSelection(ctx: &ChewingContext) -> c_int {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_set_escCleanAllBuf(ctx: &mut ChewingContext, mode: c_int) {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_get_escCleanAllBuf(ctx: &ChewingContext) -> c_int {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_set_autoShiftCur(ctx: &mut ChewingContext, mode: c_int) {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_get_autoShiftCur(ctx: &ChewingContext) -> c_int {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_set_easySymbolInput(ctx: &mut ChewingContext, mode: c_int) {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_get_easySymbolInput(ctx: &ChewingContext) -> c_int {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_set_phraseChoiceRearward(ctx: &mut ChewingContext, mode: c_int) {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_get_phraseChoiceRearward(ctx: &ChewingContext) -> c_int {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_set_autoLearn(ctx: &mut ChewingContext, mode: c_int) {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_get_autoLearn(ctx: &ChewingContext) -> c_int {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_get_phoneSeq(ctx: &ChewingContext) -> *mut c_ushort {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_get_phoneSeqLen(ctx: &ChewingContext) -> c_int {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_set_logger(
    ctx: &mut ChewingContext,
    logger: extern "C" fn(data: *mut c_void, level: c_int, fmt: *const c_char, arg: ...),
    data: *mut c_void,
) {
}

#[no_mangle]
pub extern "C" fn chewing_userphrase_enumerate(ctx: &mut ChewingContext) -> c_int {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_userphrase_has_next(
    ctx: &mut ChewingContext,
    phrase_len: *mut c_uint,
    bopomofo_len: *mut c_uint,
) -> c_int {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_userphrase_get(
    ctx: &mut ChewingContext,
    phrase_buf: *mut c_char,
    phrase_len: c_uint,
    bopomofo_buf: *mut c_char,
    bopomofo_len: c_uint,
) -> c_int {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_userphrase_add(
    ctx: &mut ChewingContext,
    phrase_buf: *const c_char,
    bopomofo_buf: *const c_char,
) -> c_int {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_userphrase_remove(
    ctx: &mut ChewingContext,
    phrase_buf: *const c_char,
    bopomofo_buf: *const c_char,
) -> c_int {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_userphrase_lookup(
    ctx: &mut ChewingContext,
    phrase_buf: *const c_char,
    bopomofo_buf: *const c_char,
) -> c_int {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_cand_list_first(ctx: &mut ChewingContext) -> c_int {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_cand_list_last(ctx: &mut ChewingContext) -> c_int {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_cand_list_has_next(ctx: &mut ChewingContext) -> c_int {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_cand_list_has_prev(ctx: &mut ChewingContext) -> c_int {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_cand_list_next(ctx: &mut ChewingContext) -> c_int {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_cand_list_prev(ctx: &mut ChewingContext) -> c_int {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_commit_preedit_buf(ctx: &mut ChewingContext) -> c_int {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_clean_preedit_buf(ctx: &mut ChewingContext) -> c_int {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_clean_bopomofo_buf(ctx: &mut ChewingContext) -> c_int {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_phone_to_bopomofo(
    phone: c_ushort,
    buf: *mut c_char,
    len: c_ushort,
) -> c_int {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_handle_Space(ctx: &mut ChewingContext) -> c_int {
    let key_event = ctx.keyboard.map(KeyCode::Space);
    ctx.editor.process_keyevent(key_event);
    0
}

#[no_mangle]
pub extern "C" fn chewing_handle_Esc(ctx: &mut ChewingContext) -> c_int {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_handle_Enter(ctx: &mut ChewingContext) -> c_int {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_handle_Del(ctx: &mut ChewingContext) -> c_int {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_handle_Backspace(ctx: &mut ChewingContext) -> c_int {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_handle_Tab(ctx: &mut ChewingContext) -> c_int {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_handle_ShiftLeft(ctx: &mut ChewingContext) -> c_int {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_handle_Left(ctx: &mut ChewingContext) -> c_int {
    let key_event = ctx
        .keyboard
        .map_with_mod(KeyCode::Left, Modifiers::default());
    ctx.editor.process_keyevent(key_event);
    0
}

#[no_mangle]
pub extern "C" fn chewing_handle_ShiftRight(ctx: &mut ChewingContext) -> c_int {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_handle_Right(ctx: &mut ChewingContext) -> c_int {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_handle_Up(ctx: &mut ChewingContext) -> c_int {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_handle_Home(ctx: &mut ChewingContext) -> c_int {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_handle_End(ctx: &mut ChewingContext) -> c_int {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_handle_PageUp(ctx: &mut ChewingContext) -> c_int {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_handle_PageDown(ctx: &mut ChewingContext) -> c_int {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_handle_Down(ctx: &mut ChewingContext) -> c_int {
    let key_event = ctx
        .keyboard
        .map_with_mod(KeyCode::Down, Modifiers::default());
    ctx.editor.process_keyevent(key_event);
    0
}

#[no_mangle]
pub extern "C" fn chewing_handle_Capslock(ctx: &mut ChewingContext) -> c_int {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_handle_Default(ctx: &mut ChewingContext, key: c_int) -> c_int {
    let key_event = ctx.keyboard.map_ascii(key as u8);
    ctx.editor.process_keyevent(key_event);
    0
}

#[no_mangle]
pub extern "C" fn chewing_handle_CtrlNum(ctx: &mut ChewingContext, key: c_int) -> c_int {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_handle_ShiftSpace(ctx: &mut ChewingContext) -> c_int {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_handle_DblTab(ctx: &mut ChewingContext) -> c_int {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_handle_Numlock(ctx: &mut ChewingContext, key: c_int) -> c_int {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_commit_Check(ctx: &ChewingContext) -> c_int {
    if ctx.editor.display_commit().is_empty() {
        0
    } else {
        1
    }
}

#[no_mangle]
pub extern "C" fn chewing_commit_String(ctx: &ChewingContext) -> *mut c_char {
    let buffer = ctx.editor.display_commit();
    let cstr = match CString::new(buffer) {
        Ok(cstr) => cstr,
        Err(_) => return null_mut(),
    };
    owned_into_raw(Owned::CString, cstr.into_raw())
}

#[no_mangle]
pub extern "C" fn chewing_commit_String_static(ctx: &ChewingContext) -> *const c_char {
    // TODO: fix memory leak
    chewing_commit_String(ctx)
}

#[no_mangle]
pub extern "C" fn chewing_buffer_String(ctx: &ChewingContext) -> *mut c_char {
    let buffer = ctx.editor.display();
    let cstr = match CString::new(buffer) {
        Ok(cstr) => cstr,
        Err(_) => return null_mut(),
    };
    owned_into_raw(Owned::CString, cstr.into_raw())
}

#[no_mangle]
pub extern "C" fn chewing_buffer_String_static(ctx: &ChewingContext) -> *const c_char {
    // TODO: fix memory leak
    chewing_buffer_String(ctx)
}

#[no_mangle]
pub extern "C" fn chewing_buffer_Check(ctx: &ChewingContext) -> c_int {
    if ctx.editor.display().len() > 0 {
        1
    } else {
        0
    }
}

#[no_mangle]
pub extern "C" fn chewing_buffer_Len(ctx: &ChewingContext) -> c_int {
    ctx.editor.display().len() as c_int
}

#[no_mangle]
pub extern "C" fn chewing_bopomofo_String_static(ctx: &ChewingContext) -> *const c_char {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_bopomofo_Check(ctx: &ChewingContext) -> c_int {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_cursor_Current(ctx: &ChewingContext) -> c_int {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_cand_CheckDone(ctx: &ChewingContext) -> c_int {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_cand_TotalPage(ctx: &ChewingContext) -> c_int {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_cand_ChoicePerPage(ctx: &ChewingContext) -> c_int {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_cand_TotalChoice(ctx: &ChewingContext) -> c_int {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_cand_CurrentPage(ctx: &ChewingContext) -> c_int {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_cand_Enumerate(ctx: &mut ChewingContext) {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_cand_hasNext(ctx: &mut ChewingContext) -> c_int {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_cand_String(ctx: &mut ChewingContext) -> *mut c_char {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_cand_String_static(ctx: &mut ChewingContext) -> *const c_char {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_cand_string_by_index(
    ctx: &mut ChewingContext,
    index: c_int,
) -> *mut c_char {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_cand_string_by_index_static(
    ctx: &mut ChewingContext,
    index: c_int,
) -> *const c_char {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_cand_choose_by_index(ctx: &mut ChewingContext, index: c_int) -> c_int {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_cand_open(ctx: &mut ChewingContext) -> c_int {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_cand_close(ctx: &mut ChewingContext) -> c_int {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_interval_Enumerate(ctx: &mut ChewingContext) {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_interval_hasNext(ctx: &mut ChewingContext) -> c_int {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_interval_Get(ctx: &mut ChewingContext, it: *mut IntervalType) {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_aux_Check(ctx: &ChewingContext) -> c_int {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_aux_Length(ctx: &ChewingContext) -> c_int {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_aux_String(ctx: &ChewingContext) -> *mut c_char {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_aux_String_static(ctx: &ChewingContext) -> *const c_char {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_keystroke_CheckIgnore(ctx: &ChewingContext) -> c_int {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_keystroke_CheckAbsorb(ctx: &ChewingContext) -> c_int {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_kbtype_Total(ctx: &ChewingContext) -> c_int {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_kbtype_Enumerate(ctx: &mut ChewingContext) {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_kbtype_hasNext(ctx: &mut ChewingContext) -> c_int {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_kbtype_String(ctx: &mut ChewingContext) -> *mut c_char {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_kbtype_String_static(ctx: &mut ChewingContext) -> *const c_char {
    todo!()
}

#[no_mangle]
#[deprecated]
pub extern "C" fn chewing_zuin_Check(ctx: &ChewingContext) -> c_int {
    todo!()
}

#[no_mangle]
#[deprecated]
pub extern "C" fn chewing_zuin_String(ctx: &ChewingContext, zuin_count: *mut c_int) -> *mut c_char {
    todo!()
}

#[no_mangle]
#[deprecated]
pub extern "C" fn chewing_Init(data_path: *const c_char, hash_path: *const c_char) -> c_int {
    todo!()
}

#[no_mangle]
#[deprecated]
pub extern "C" fn chewing_Terminate() {
    todo!()
}

#[no_mangle]
#[deprecated]
pub extern "C" fn chewing_Configure(
    ctx: &mut ChewingContext,
    pcd: *mut ChewingConfigData,
) -> c_int {
    todo!()
}

#[no_mangle]
#[deprecated]
pub extern "C" fn chewing_set_hsuSelKeyType(ctx: &mut ChewingContext, mode: c_int) {
    todo!()
}

#[no_mangle]
#[deprecated]
pub extern "C" fn chewing_get_hsuSelKeyType(ctx: &mut ChewingContext) -> c_int {
    todo!()
}
