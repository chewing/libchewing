/**
 * test-error-handling.c
 *
 * Copyright (c) 2013
 *      libchewing Core Team.
 *
 * See the file "COPYING" for information on usage and redistribution
 * of this file.
 */
#include <assert.h>
#include <stdlib.h>
#include <string.h>

#include "testhelper.h"
#include "chewing.h"

FILE *fd;

void test_null()
{
    int ret;
    char *buf;
    const char *const_buf;
    int *key;
    unsigned short *phone;

    start_testcase(NULL, fd);

    chewing_Reset(NULL);        // shall not crash

    ret = chewing_set_KBType(NULL, 0);
    ok(ret == -1, "chewing_set_KBType() returns `%d' shall be `%d'", ret, -1);

    ret = chewing_get_KBType(NULL);
    ok(ret == -1, "chewing_get_KBType() returns `%d' shall be `%d'", ret, -1);

    buf = chewing_get_KBString(NULL);
    ok(strcmp(buf, "") == 0, "chewing_get_KBString() returns `%s' shall be `%s'", buf, "");
    chewing_free(buf);

    chewing_delete(NULL);       // shall not crash

    chewing_free(NULL);         // shall not crash

    chewing_set_candPerPage(NULL, 0);   // shall not crash

    ret = chewing_get_candPerPage(NULL);
    ok(ret == -1, "chewing_get_candPerPage() returns `%d' shall be `%d'", ret, -1);

    chewing_set_maxChiSymbolLen(NULL, 0);       // shall not crash

    ret = chewing_get_maxChiSymbolLen(NULL);
    ok(ret == -1, "chewing_get_maxChiSymbolLen() returns `%d' shall be `%d'", ret, -1);

    chewing_set_selKey(NULL, NULL, 0);  // shall not crash

    key = chewing_get_selKey(NULL);
    ok(key == NULL, "chewing_get_selKey() returns NULL");
    chewing_free(key);

    chewing_set_addPhraseDirection(NULL, 0);    // shall not crash

    ret = chewing_get_addPhraseDirection(NULL);
    ok(ret == -1, "chewing_get_addPhraseDirection() returns `%d' shall be `%d'", ret, -1);

    chewing_set_spaceAsSelection(NULL, 0);      // shall not crash

    ret = chewing_get_spaceAsSelection(NULL);
    ok(ret == -1, "chewing_get_spaceAsSelection() returns `%d' shall be `%d'", ret, -1);

    chewing_set_escCleanAllBuf(NULL, 0);        // shall not crash

    ret = chewing_get_escCleanAllBuf(NULL);
    ok(ret == -1, "chewing_get_escCleanAllBuf() returns `%d' shall be `%d'", ret, -1);

    chewing_set_autoShiftCur(NULL, 0);  // shall not crash

    ret = chewing_get_autoShiftCur(NULL);
    ok(ret == -1, "chewing_get_autoShiftCur() returns `%d' shall be `%d'", ret, -1);

    chewing_set_easySymbolInput(NULL, 0);       // shall not crash

    ret = chewing_get_easySymbolInput(NULL);
    ok(ret == -1, "chewing_get_easySymbolInput() returns `%d' shall be `%d'", ret, -1);

    chewing_set_phraseChoiceRearward(NULL, 0);

    ret = chewing_get_phraseChoiceRearward(NULL);
    ok(ret == -1, "chewing_get_phraseChoiceRearward() returns `%d' shall be `%d'", ret, -1);

    chewing_set_ChiEngMode(NULL, 0);    // shall not crash

    ret = chewing_get_ChiEngMode(NULL);
    ok(ret == -1, "chewing_get_ChiEngMode() returns `%d' shall be `%d'", ret, -1);

    chewing_set_ShapeMode(NULL, 0);     // shall not crash

    ret = chewing_handle_Space(NULL);
    ok(ret == -1, "chewing_handle_Space() returns `%d' shall be `%d'", ret, -1);

    ret = chewing_handle_Esc(NULL);
    ok(ret == -1, "chewing_handle_Esc() returns `%d' shall be `%d'", ret, -1);

    ret = chewing_handle_Enter(NULL);
    ok(ret == -1, "chewing_handle_Enter() returns `%d' shall be `%d'", ret, -1);

    ret = chewing_handle_Del(NULL);
    ok(ret == -1, "chewing_handle_Del() returns `%d' shall be `%d'", ret, -1);

    ret = chewing_handle_Backspace(NULL);
    ok(ret == -1, "chewing_handle_Backspace() returns `%d' shall be `%d'", ret, -1);

    ret = chewing_handle_Up(NULL);
    ok(ret == -1, "chewing_handle_Up() returns `%d' shall be `%d'", ret, -1);

    ret = chewing_handle_Down(NULL);
    ok(ret == -1, "chewing_handle_Down() returns `%d' shall be `%d'", ret, -1);

    ret = chewing_handle_ShiftLeft(NULL);
    ok(ret == -1, "chewing_handle_ShiftLeft() returns `%d' shall be `%d'", ret, -1);

    ret = chewing_handle_Left(NULL);
    ok(ret == -1, "chewing_handle_Left() returns `%d' shall be `%d'", ret, -1);

    ret = chewing_handle_ShiftRight(NULL);
    ok(ret == -1, "chewing_handle_ShiftRight() returns `%d' shall be `%d'", ret, -1);

    ret = chewing_handle_Right(NULL);
    ok(ret == -1, "chewing_handle_Right() returns `%d' shall be `%d'", ret, -1);

    ret = chewing_handle_Tab(NULL);
    ok(ret == -1, "chewing_handle_Tab() returns `%d' shall be `%d'", ret, -1);

    ret = chewing_handle_DblTab(NULL);
    ok(ret == -1, "chewing_handle_DblTab() returns `%d' shall be `%d'", ret, -1);

    ret = chewing_handle_Capslock(NULL);
    ok(ret == -1, "chewing_handle_Capslock() returns `%d' shall be `%d'", ret, -1);

    ret = chewing_handle_Home(NULL);
    ok(ret == -1, "chewing_handle_Home() returns `%d' shall be `%d'", ret, -1);

    ret = chewing_handle_PageUp(NULL);
    ok(ret == -1, "chewing_handle_PageUp() returns `%d' shall be `%d'", ret, -1);

    ret = chewing_handle_PageDown(NULL);
    ok(ret == -1, "chewing_handle_PageDown() returns `%d' shall be `%d'", ret, -1);

    ret = chewing_handle_Default(NULL, 0);
    ok(ret == -1, "chewing_handle_Default() returns `%d' shall be `%d'", ret, -1);

    ret = chewing_handle_CtrlNum(NULL, 0);
    ok(ret == -1, "chewing_handle_CtrlNum() returns `%d' shall be `%d'", ret, -1);

    ret = chewing_handle_ShiftSpace(NULL);
    ok(ret == -1, "chewing_handle_ShiftSpace() returns `%d' shall be `%d'", ret, -1);

    ret = chewing_handle_Numlock(NULL, 0);
    ok(ret == -1, "chewing_handle_Numlock() returns `%d' shall be `%d'", ret, -1);

    phone = chewing_get_phoneSeq(NULL);
    ok(phone == NULL, "chewing_get_phoneSeq() returns NULL");
    chewing_free(phone);

    ret = chewing_get_phoneSeqLen(NULL);
    ok(ret == -1, "chewing_get_phoneSeqLen() returns `%d' shall be `%d'", ret, -1);

    chewing_set_logger(NULL, NULL, NULL);

    ret = chewing_userphrase_enumerate(NULL);
    ok(ret == -1, "chewing_userphrase_enumerate() returns `%d' shall be `%d'", ret, -1);

    ret = chewing_userphrase_has_next(NULL, NULL, NULL);
    ok(ret == 0, "chewing_userphrase_has_next() returns `%d' shall be `%d'", ret, 0);

    ret = chewing_userphrase_get(NULL, NULL, 0, NULL, 0);
    ok(ret == -1, "chewing_userphrase_get() returns `%d' shall be `%d'", ret, -1);

    ret = chewing_userphrase_add(NULL, NULL, NULL);
    ok(ret == -1, "chewing_userphrase_add() returns `%d' shall be `%d'", ret, -1);

    ret = chewing_userphrase_remove(NULL, NULL, NULL);
    ok(ret == -1, "chewing_userphrase_remove() returns `%d' shall be `%d'", ret, -1);

    ret = chewing_userphrase_lookup(NULL, NULL, NULL);
    ok(ret == 0, "chewing_userphrase_lookup() returns `%d' shall be `%d'", ret, 0);

    ret = chewing_cand_open(NULL);
    ok(ret == -1, "chewing_cand_open() returns `%d' shall be `%d'", ret, -1);

    ret = chewing_cand_close(NULL);
    ok(ret == -1, "chewing_cand_open() returns `%d' shall be `%d'", ret, -1);

    ret = chewing_cand_choose_by_index(NULL, 0);
    ok(ret == -1, "chewing_cand_choose_by_index() returns `%d' shall be `%d'", ret, -1);

    ret = chewing_cand_list_first(NULL);
    ok(ret == -1, "chewing_cand_list_first() returns `%d' shall be `%d'", ret, -1);

    ret = chewing_cand_list_last(NULL);
    ok(ret == -1, "chewing_cand_list_last() returns `%d' shall be `%d'", ret, -1);

    ret = chewing_cand_list_has_next(NULL);
    ok(ret == 0, "chewing_cand_list_has_next() returns `%d' shall be `%d'", ret, 0);

    ret = chewing_cand_list_has_prev(NULL);
    ok(ret == 0, "chewing_cand_list_has_prev() returns `%d' shall be `%d'", ret, 0);

    ret = chewing_cand_list_next(NULL);
    ok(ret == -1, "chewing_cand_list_next() returns `%d' shall be `%d'", ret, -1);

    ret = chewing_cand_list_prev(NULL);
    ok(ret == -1, "chewing_cand_list_prev() returns `%d' shall be `%d'", ret, -1);

    ret = chewing_commit_preedit_buf(NULL);
    ok(ret == -1, "chewing_commit_preedit_buf() returns `%d' shall be `%d'", ret, -1);

    ret = chewing_clean_preedit_buf(NULL);
    ok(ret == -1, "chewing_clean_preedit_buf() returns `%d' shall be `%d'", ret, -1);

    ret = chewing_clean_bopomofo_buf(NULL);
    ok(ret == -1, "chewing_clean_bopomofo_buf() returns `%d' shall be `%d'", ret, -1);

    ret = chewing_commit_Check(NULL);
    ok(ret == -1, "chewing_commit_Check() returns `%d' shall be `%d'", ret, -1);

    buf = chewing_commit_String(NULL);
    ok(strcmp(buf, "") == 0, "chewing_commit_String() returns `%s' shall be `%s'", buf, "");
    chewing_free(buf);

    const_buf = chewing_commit_String_static(NULL);
    ok(strcmp(const_buf, "") == 0, "chewing_commit_String() returns `%s' shall be `%s'", const_buf, "");

    ret = chewing_buffer_Check(NULL);
    ok(ret == -1, "chewing_buffer_Check() returns `%d' shall be `%d'", ret, -1);

    ret = chewing_buffer_Len(NULL);
    ok(ret == -1, "chewing_buffer_Len() returns `%d' shall be `%d'", ret, -1);

    buf = chewing_buffer_String(NULL);
    ok(strcmp(buf, "") == 0, "chewing_buffer_String() returns `%s' shall be `%s'", buf, "");
    chewing_free(buf);

    const_buf = chewing_buffer_String_static(NULL);
    ok(strcmp(const_buf, "") == 0, "chewing_buffer_String_static() returns `%s' shall be `%s'", const_buf, "");

    buf = chewing_bopomofo_String(NULL);
    ok(strcmp(buf, "") == 0, "chewing_bopomofo_String() returns `%s' shall be `%s'", buf, "");
    chewing_free(buf);

    const_buf = chewing_bopomofo_String_static(NULL);
    ok(strcmp(const_buf, "") == 0, "chewing_bopomofo_String_static() returns `%s' shall be `%s'", const_buf, "");

BEGIN_IGNORE_DEPRECATIONS
    buf = chewing_zuin_String(NULL, NULL);
END_IGNORE_DEPRECATIONS
    ok(strcmp(buf, "") == 0, "chewing_zuin_String() returns `%s' shall be `%s'", buf, "");
    chewing_free(buf);

    ret = chewing_bopomofo_Check(NULL);
    ok(ret == -1, "chewing_bopomofo_Check() returns `%d' shall be `%d'", ret, -1);

BEGIN_IGNORE_DEPRECATIONS
    chewing_zuin_Check(NULL); // shall not crash
END_IGNORE_DEPRECATIONS

    ret = chewing_cursor_Current(NULL);
    ok(ret == -1, "chewing_cursor_Current() returns `%d' shall be `%d'", ret, -1);

    ret = chewing_cand_CheckDone(NULL);
    ok(ret == -1, "chewing_cand_CheckDone() returns `%d' shall be `%d'", ret, -1);

    ret = chewing_cand_TotalPage(NULL);
    ok(ret == -1, "chewing_cand_TotalPage() returns `%d' shall be `%d'", ret, -1);

    ret = chewing_cand_ChoicePerPage(NULL);
    ok(ret == -1, "chewing_cand_ChoicePerPage() returns `%d' shall be `%d'", ret, -1);

    ret = chewing_cand_TotalChoice(NULL);
    ok(ret == -1, "chewing_cand_TotalChoice() returns `%d' shall be `%d'", ret, -1);

    ret = chewing_cand_CurrentPage(NULL);
    ok(ret == -1, "chewing_cand_CurrentPage() returns `%d' shall be `%d'", ret, -1);

    chewing_cand_Enumerate(NULL);       // shall not crash

    ret = chewing_cand_hasNext(NULL);
    ok(ret == -1, "chewing_cand_hasNext() returns `%d' shall be `%d'", ret, -1);

    const_buf = chewing_cand_String_static(NULL);
    ok(strcmp(const_buf, "") == 0, "chewing_cand_String_static() returns `%s' shall be `%s'", const_buf, "");

    buf = chewing_cand_String(NULL);
    ok(strcmp(buf, "") == 0, "chewing_cand_String() returns `%s' shall be `%s'", buf, "");
    chewing_free(buf);

    chewing_interval_Enumerate(NULL);   // shall not crash

    ret = chewing_interval_hasNext(NULL);
    ok(ret == -1, "chewing_interval_hasNext() returns `%d' shall be `%d'", ret, -1);

    chewing_interval_Get(NULL, NULL);   // shall not crash

    ret = chewing_aux_Check(NULL);
    ok(ret == -1, "chewing_aux_Check() returns `%d' shall be `%d'", ret, -1);

    ret = chewing_aux_Length(NULL);
    ok(ret == -1, "chewing_aux_Length() returns `%d' shall be `%d'", ret, -1);

    const_buf = chewing_aux_String_static(NULL);
    ok(strcmp(const_buf, "") == 0, "chewing_aux_String_static() returns `%s' shall be `%s'", const_buf, "");

    buf = chewing_aux_String(NULL);
    ok(strcmp(buf, "") == 0, "chewing_aux_String() returns `%s' shall be `%s'", buf, "");
    chewing_free(buf);

    ret = chewing_keystroke_CheckIgnore(NULL);
    ok(ret == -1, "chewing_keystroke_CheckIgnore() returns `%d' shall be `%d'", ret, -1);

    ret = chewing_keystroke_CheckAbsorb(NULL);
    ok(ret == -1, "chewing_keystroke_CheckAbsorb() returns `%d' shall be `%d'", ret, -1);

    chewing_kbtype_Enumerate(NULL);     // shall not crash

    ret = chewing_kbtype_hasNext(NULL);
    ok(ret == -1, "chewing_kbtype_hasNext() returns `%d' shall be `%d'", ret, -1);

    const_buf = chewing_kbtype_String_static(NULL);
    ok(strcmp(const_buf, "") == 0, "chewing_kbtype_String_static() returns `%s' shall be `%s'", const_buf, "");

    buf = chewing_kbtype_String(NULL);
    ok(strcmp(buf, "") == 0, "chewing_kbtype_String() returns `%s' shall be `%s'", buf, "");
    chewing_free(buf);
}

void test_FallbackDictionary()
{
    const TestData SIMPLE_INPUT[] = {
        {"so4fu0 y42u03ai6g4<E>", "內千字點模市" },
        {"so4fu0 <D>5y42u03<D>4ai6g4<D>9<E>", "內嵌字典模式" },
    };
    size_t i;
    ChewingContext *ctx;

    putenv("CHEWING_PATH=" CHEWING_DATA_PREFIX "ERROR");

    ctx = chewing_new();
    start_testcase(ctx, fd);
    chewing_set_maxChiSymbolLen(ctx, 16);

    for (i = 0; i < ARRAY_SIZE(SIMPLE_INPUT); ++i) {
        type_keystroke_by_string(ctx, SIMPLE_INPUT[i].token);
        ok_commit_buffer(ctx, SIMPLE_INPUT[i].expected);
    }

    chewing_delete(ctx);

    putenv("CHEWING_PATH=" CHEWING_DATA_PREFIX);
}

int main(int argc, char *argv[])
{
    char *logname;
    int ret;

    putenv("CHEWING_PATH=" CHEWING_DATA_PREFIX);
    putenv("CHEWING_USER_PATH=" TEST_HASH_DIR);

    ret = asprintf(&logname, "%s.log", argv[0]);
    if (ret == -1)
        return -1;
    fd = fopen(logname, "w");
    assert(fd);
    free(logname);


    test_null();
    test_FallbackDictionary();

    fclose(fd);

    return exit_status();
}
