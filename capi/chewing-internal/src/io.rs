use std::ffi::{c_char, c_int, c_uint, c_ushort, c_void};

use chewing_public::types::{ChewingConfigData, IntervalType};

use crate::types::ChewingContext;

#[no_mangle]
pub extern "C" fn rust_link_io() {}

#[no_mangle]
pub extern "C" fn chewing_new() -> *mut ChewingContext {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_new2(
    syspath: *const c_char,
    userpath: *const c_char,
    logger: extern "C" fn(data: *mut c_void, level: c_int, fmt: *const c_char, arg: ...),
    loggerdata: *mut c_void,
) -> *mut ChewingContext {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_delete(ctx: *mut ChewingContext) {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_free(ptr: *mut c_void) {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_Reset(ctx: *mut ChewingContext) -> c_int {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_set_KBType(ctx: *mut ChewingContext, kbtype: c_int) -> c_int {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_get_KBType(ctx: *const ChewingContext) -> c_int {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_get_KBString(ctx: *const ChewingContext) -> *mut c_char {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_KBStr2Num(str: *const c_char) -> c_int {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_set_ChiEngMode(ctx: *mut ChewingContext, mode: c_int) -> c_void {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_get_ChiEngMode(ctx: *const ChewingContext) -> c_int {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_set_ShapeMode(ctx: *mut ChewingContext, mode: c_int) -> c_void {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_get_ShapeMode(ctx: *const ChewingContext) -> c_int {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_set_candPerPage(ctx: *mut ChewingContext, n: c_int) -> c_void {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_get_candPerPage(ctx: *const ChewingContext) -> c_int {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_set_maxChiSymbolLen(ctx: *mut ChewingContext, n: c_int) {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_get_maxChiSymbolLen(ctx: *const ChewingContext) -> c_int {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_set_selKey(
    ctx: *mut ChewingContext,
    sel_keys: *const c_int,
    len: c_int,
) -> c_void {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_get_selKey(ctx: *const ChewingContext) -> *mut c_int {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_set_addPhraseDirection(
    ctx: *mut ChewingContext,
    direction: c_int,
) -> c_void {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_get_addPhraseDirection(ctx: *const ChewingContext) -> c_int {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_set_spaceAsSelection(ctx: *mut ChewingContext, mode: c_int) -> c_void {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_get_spaceAsSelection(ctx: *const ChewingContext) -> c_int {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_set_escCleanAllBuf(ctx: *mut ChewingContext, mode: c_int) -> c_void {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_get_escCleanAllBuf(ctx: *const ChewingContext) -> c_int {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_set_autoShiftCur(ctx: *mut ChewingContext, mode: c_int) -> c_void {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_get_autoShiftCur(ctx: *const ChewingContext) -> c_int {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_set_easySymbolInput(ctx: *mut ChewingContext, mode: c_int) -> c_void {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_get_easySymbolInput(ctx: *const ChewingContext) -> c_int {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_set_phraseChoiceRearward(
    ctx: *mut ChewingContext,
    mode: c_int,
) -> c_void {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_get_phraseChoiceRearward(ctx: *const ChewingContext) -> c_int {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_set_autoLearn(ctx: *mut ChewingContext, mode: c_int) -> c_void {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_get_autoLearn(ctx: *const ChewingContext) -> c_int {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_get_phoneSeq(ctx: *const ChewingContext) -> *mut c_ushort {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_get_phoneSeqLen(ctx: *const ChewingContext) -> c_int {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_set_logger(
    ctx: *mut ChewingContext,
    logger: extern "C" fn(data: *mut c_void, level: c_int, fmt: *const c_char, arg: ...),
    data: *mut c_void,
) {
}

#[no_mangle]
pub extern "C" fn chewing_userphrase_enumerate(ctx: *mut ChewingContext) -> c_int {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_userphrase_has_next(
    ctx: *mut ChewingContext,
    phrase_len: *mut c_uint,
    bopomofo_len: *mut c_uint,
) -> c_int {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_userphrase_get(
    ctx: *mut ChewingContext,
    phrase_buf: *mut c_char,
    phrase_len: c_uint,
    bopomofo_buf: *mut c_char,
    bopomofo_len: c_uint,
) -> c_int {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_userphrase_add(
    ctx: *mut ChewingContext,
    phrase_buf: *const c_char,
    bopomofo_buf: *const c_char,
) -> c_int {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_userphrase_remove(
    ctx: *mut ChewingContext,
    phrase_buf: *const c_char,
    bopomofo_buf: *const c_char,
) -> c_int {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_userphrase_lookup(
    ctx: *mut ChewingContext,
    phrase_buf: *const c_char,
    bopomofo_buf: *const c_char,
) -> c_int {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_cand_list_first(ctx: *mut ChewingContext) -> c_int {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_cand_list_last(ctx: *mut ChewingContext) -> c_int {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_cand_list_has_next(ctx: *mut ChewingContext) -> c_int {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_cand_list_has_prev(ctx: *mut ChewingContext) -> c_int {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_cand_list_next(ctx: *mut ChewingContext) -> c_int {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_cand_list_prev(ctx: *mut ChewingContext) -> c_int {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_commit_preedit_buf(ctx: *mut ChewingContext) -> c_int {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_clean_preedit_buf(ctx: *mut ChewingContext) -> c_int {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_clean_bopomofo_buf(ctx: *mut ChewingContext) -> c_int {
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
pub extern "C" fn chewing_handle_Space(ctx: *mut ChewingContext) -> c_int {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_handle_Esc(ctx: *mut ChewingContext) -> c_int {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_handle_Enter(ctx: *mut ChewingContext) -> c_int {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_handle_Del(ctx: *mut ChewingContext) -> c_int {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_handle_Backspace(ctx: *mut ChewingContext) -> c_int {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_handle_Tab(ctx: *mut ChewingContext) -> c_int {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_handle_ShiftLeft(ctx: *mut ChewingContext) -> c_int {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_handle_Left(ctx: *mut ChewingContext) -> c_int {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_handle_ShiftRight(ctx: *mut ChewingContext) -> c_int {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_handle_Right(ctx: *mut ChewingContext) -> c_int {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_handle_Up(ctx: *mut ChewingContext) -> c_int {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_handle_Home(ctx: *mut ChewingContext) -> c_int {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_handle_End(ctx: *mut ChewingContext) -> c_int {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_handle_PageUp(ctx: *mut ChewingContext) -> c_int {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_handle_PageDown(ctx: *mut ChewingContext) -> c_int {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_handle_Down(ctx: *mut ChewingContext) -> c_int {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_handle_Capslock(ctx: *mut ChewingContext) -> c_int {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_handle_Default(ctx: *mut ChewingContext, key: c_int) -> c_int {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_handle_CtrlNum(ctx: *mut ChewingContext, key: c_int) -> c_int {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_handle_ShiftSpace(ctx: *mut ChewingContext) -> c_int {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_handle_DblTab(ctx: *mut ChewingContext) -> c_int {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_handle_Numlock(ctx: *mut ChewingContext, key: c_int) -> c_int {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_commit_Check(ctx: *const ChewingContext) -> c_int {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_commit_String(ctx: *const ChewingContext) -> *mut c_char {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_commit_String_static(ctx: *const ChewingContext) -> *const c_char {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_buffer_String(ctx: *const ChewingContext) -> *mut c_char {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_buffer_String_static(ctx: *const ChewingContext) -> *const c_char {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_buffer_Check(ctx: *const ChewingContext) -> c_int {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_buffer_Len(ctx: *const ChewingContext) -> c_int {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_bopomofo_String_static(ctx: *const ChewingContext) -> *const c_char {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_bopomofo_Check(ctx: *const ChewingContext) -> c_int {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_cursor_Current(ctx: *const ChewingContext) -> c_int {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_cand_CheckDone(ctx: *const ChewingContext) -> c_int {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_cand_TotalPage(ctx: *const ChewingContext) -> c_int {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_cand_ChoicePerPage(ctx: *const ChewingContext) -> c_int {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_cand_TotalChoice(ctx: *const ChewingContext) -> c_int {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_cand_CurrentPage(ctx: *const ChewingContext) -> c_int {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_cand_Enumerate(ctx: *mut ChewingContext) {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_cand_hasNext(ctx: *mut ChewingContext) -> c_int {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_cand_String(ctx: *mut ChewingContext) -> *mut c_char {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_cand_String_static(ctx: *mut ChewingContext) -> *const c_char {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_cand_string_by_index(
    ctx: *mut ChewingContext,
    index: c_int,
) -> *mut c_char {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_cand_string_by_index_static(
    ctx: *mut ChewingContext,
    index: c_int,
) -> *const c_char {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_cand_choose_by_index(ctx: *mut ChewingContext, index: c_int) -> c_int {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_cand_open(ctx: *mut ChewingContext) -> c_int {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_cand_close(ctx: *mut ChewingContext) -> c_int {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_interval_Enumerate(ctx: *mut ChewingContext) {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_interval_hasNext(ctx: *mut ChewingContext) -> c_int {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_interval_Get(ctx: *mut ChewingContext, it: *mut IntervalType) {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_aux_Check(ctx: *const ChewingContext) -> c_int {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_aux_Length(ctx: *const ChewingContext) -> c_int {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_aux_String(ctx: *const ChewingContext) -> *mut c_char {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_aux_String_static(ctx: *const ChewingContext) -> *const c_char {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_keystroke_CheckIgnore(ctx: *const ChewingContext) -> c_int {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_keystroke_CheckAbsorb(ctx: *const ChewingContext) -> c_int {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_kbtype_Total(ctx: *const ChewingContext) -> c_int {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_kbtype_Enumerate(ctx: *mut ChewingContext) {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_kbtype_hasNext(ctx: *mut ChewingContext) -> c_int {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_kbtype_String(ctx: *mut ChewingContext) -> *mut c_char {
    todo!()
}

#[no_mangle]
pub extern "C" fn chewing_kbtype_String_static(ctx: *mut ChewingContext) -> *const c_char {
    todo!()
}

#[no_mangle]
#[deprecated]
pub extern "C" fn chewing_zuin_Check(ctx: *const ChewingContext) -> c_int {
    todo!()
}

#[no_mangle]
#[deprecated]
pub extern "C" fn chewing_zuin_String(
    ctx: *const ChewingContext,
    zuin_count: *mut c_int,
) -> *mut c_char {
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
    ctx: *mut ChewingContext,
    pcd: *mut ChewingConfigData,
) -> c_int {
    todo!()
}

#[no_mangle]
#[deprecated]
pub extern "C" fn chewing_set_hsuSelKeyType(ctx: *mut ChewingContext, mode: c_int) {
    todo!()
}

#[no_mangle]
#[deprecated]
pub extern "C" fn chewing_get_hsuSelKeyType(ctx: *mut ChewingContext) -> c_int {
    todo!()
}
