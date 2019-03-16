/**
 * test-userphrase.c
 *
 * Copyright (c) 2013
 *      libchewing Core Team. See ChangeLog for details.
 *
 * See the file "COPYING" for information on usage and redistribution
 * of this file.
 */

#ifdef HAVE_CONFIG_H
#    include <config.h>
#endif

#include <assert.h>
#include <string.h>
#include <stdlib.h>
#include <stdio.h>

#include "chewing.h"
#include "plat_types.h"
#include "testhelper.h"

FILE *fd;

void test_ShiftLeft_not_entering_chewing()
{
    ChewingContext *ctx;

    ctx = chewing_new();
    start_testcase(ctx, fd);
    type_keystroke_by_string(ctx, "<SL>");
    ok_keystroke_rtn(ctx, KEYSTROKE_IGNORE);

    chewing_delete(ctx);
}

void test_ShiftLeft_add_userphrase()
{
    static const char phrase[] = "\xE6\xB8\xAC\xE8\xA9\xA6" /* 測試 */ ;
    static const char bopomofo[] = "\xE3\x84\x98\xE3\x84\x9C\xCB\x8B \xE3\x84\x95\xCB\x8B" /* ㄘㄜˋ ㄕˋ */ ;
    static const char msg[] = "\xE5\x8A\xA0\xE5\x85\xA5\xEF\xBC\x9A\xE6\xB8\xAC\xE8\xA9\xA6" /* 加入：測試 */ ;

    int cursor;
    ChewingContext *ctx;

    clean_userphrase();

    ctx = chewing_new();
    start_testcase(ctx, fd);
    chewing_set_maxChiSymbolLen(ctx, 16);

    ok(has_userphrase(ctx, bopomofo, phrase) == 0, "`%s' shall not be in userphrase", phrase);

    type_keystroke_by_string(ctx, "hk4g4<SL><SL><E>");
    ok_preedit_buffer(ctx, phrase);
    cursor = chewing_cursor_Current(ctx);
    ok(cursor == 0, "cursor position `%d' shall be 0", cursor);
    ok(has_userphrase(ctx, bopomofo, phrase) == 1, "`%s' shall be in userphrase", phrase);
    ok_aux_buffer(ctx, msg);

    chewing_delete(ctx);
}

void test_ShiftLeft()
{
    test_ShiftLeft_not_entering_chewing();
    test_ShiftLeft_add_userphrase();
}

void test_ShiftRight_not_entering_chewing()
{
    ChewingContext *ctx;

    ctx = chewing_new();
    start_testcase(ctx, fd);
    type_keystroke_by_string(ctx, "<SR>");
    ok_keystroke_rtn(ctx, KEYSTROKE_IGNORE);

    chewing_delete(ctx);
}

void test_ShiftRight_add_userphrase()
{
    static const char phrase[] = "\xE6\xB8\xAC\xE8\xA9\xA6" /* 測試 */ ;
    static const char bopomofo[] = "\xE3\x84\x98\xE3\x84\x9C\xCB\x8B \xE3\x84\x95\xCB\x8B" /* ㄘㄜˋ ㄕˋ */ ;
    static const char msg[] = "\xE5\x8A\xA0\xE5\x85\xA5\xEF\xBC\x9A\xE6\xB8\xAC\xE8\xA9\xA6" /* 加入：測試 */ ;

    int cursor;
    ChewingContext *ctx;

    clean_userphrase();

    ctx = chewing_new();
    start_testcase(ctx, fd);
    chewing_set_maxChiSymbolLen(ctx, 16);

    ok(has_userphrase(ctx, bopomofo, phrase) == 0, "`%s' shall not be in userphrase", phrase);

    type_keystroke_by_string(ctx, "hk4g4<L><L><SR><SR><E>");
    ok_preedit_buffer(ctx, phrase);
    cursor = chewing_cursor_Current(ctx);
    ok(cursor == 2, "cursor position `%d' shall be 2", cursor);
    ok(has_userphrase(ctx, bopomofo, phrase) == 1, "`%s' shall be in userphrase", phrase);
    ok_aux_buffer(ctx, msg);

    chewing_delete(ctx);
}

void test_ShiftRight()
{
    test_ShiftRight_not_entering_chewing();
    test_ShiftRight_add_userphrase();
}

void test_CtrlNum_add_phrase_right()
{
    static const char phrase[] = "\xE6\xB8\xAC\xE8\xA9\xA6" /* 測試 */ ;
    static const char bopomofo[] = "\xE3\x84\x98\xE3\x84\x9C\xCB\x8B \xE3\x84\x95\xCB\x8B" /* ㄘㄜˋ ㄕˋ */ ;
    static const char msg[] = "\xE5\x8A\xA0\xE5\x85\xA5\xEF\xBC\x9A\xE6\xB8\xAC\xE8\xA9\xA6" /* 加入：測試 */ ;
    static const char msg_already_have[] =
        "\xE5\xB7\xB2\xE6\x9C\x89\xEF\xBC\x9A\xE6\xB8\xAC\xE8\xA9\xA6" /* 已有：測試 */ ;
    static const char msg_error[] =
        "\xE5\x8A\xA0\xE8\xA9\x9E\xE5\xA4\xB1\xE6\x95\x97\xEF\xBC\x9A\xE5\xAD\x97\xE6\x95\xB8"
        "\xE4\xB8\x8D\xE7\xAC\xA6\xE6\x88\x96\xE5\xA4\xBE\xE9\x9B\x9C\xE7\xAC\xA6\xE8\x99\x9F"
        /* 加詞失敗：字數不符或夾雜符號 */;

    int cursor;
    ChewingContext *ctx;

    clean_userphrase();

    ctx = chewing_new();
    start_testcase(ctx, fd);
    chewing_set_maxChiSymbolLen(ctx, 16);
    chewing_set_addPhraseDirection(ctx, 0);

    ok(has_userphrase(ctx, bopomofo, phrase) == 0, "`%s' shall not be in userphrase", phrase);

    type_keystroke_by_string(ctx, "hk4g4<H><C2>");
    ok_aux_buffer(ctx, msg);
    ok_preedit_buffer(ctx, phrase);
    cursor = chewing_cursor_Current(ctx);
    ok(cursor == 0, "cursor position `%d' shall be 0", cursor);
    ok(has_userphrase(ctx, bopomofo, phrase) == 1, "`%s' shall be in userphrase", phrase);

    type_keystroke_by_string(ctx, "<C2>");
    ok_aux_buffer(ctx, msg_already_have);

    type_keystroke_by_string(ctx, "<EN><C2>");
    ok_aux_buffer(ctx, msg_error);

    chewing_delete(ctx);
}

void test_CtrlNum_add_phrase_left()
{
    static const char phrase[] = "\xE6\xB8\xAC\xE8\xA9\xA6" /* 測試 */ ;
    static const char bopomofo[] = "\xE3\x84\x98\xE3\x84\x9C\xCB\x8B \xE3\x84\x95\xCB\x8B" /* ㄘㄜˋ ㄕˋ */ ;
    static const char msg_add[] = "\xE5\x8A\xA0\xE5\x85\xA5\xEF\xBC\x9A\xE6\xB8\xAC\xE8\xA9\xA6" /* 加入：測試 */ ;
    static const char msg_already_have[] =
        "\xE5\xB7\xB2\xE6\x9C\x89\xEF\xBC\x9A\xE6\xB8\xAC\xE8\xA9\xA6" /* 已有：測試 */ ;
    static const char msg_error[] =
        "\xE5\x8A\xA0\xE8\xA9\x9E\xE5\xA4\xB1\xE6\x95\x97\xEF\xBC\x9A\xE5\xAD\x97\xE6\x95\xB8"
        "\xE4\xB8\x8D\xE7\xAC\xA6\xE6\x88\x96\xE5\xA4\xBE\xE9\x9B\x9C\xE7\xAC\xA6\xE8\x99\x9F"
        /* 加詞失敗：字數不符或夾雜符號 */;

    int cursor;
    ChewingContext *ctx;

    clean_userphrase();

    ctx = chewing_new();
    start_testcase(ctx, fd);
    chewing_set_maxChiSymbolLen(ctx, 16);
    chewing_set_addPhraseDirection(ctx, 1);

    ok(has_userphrase(ctx, bopomofo, phrase) == 0, "`%s' shall not be in userphrase", phrase);

    type_keystroke_by_string(ctx, "hk4g4<C2>");
    ok_aux_buffer(ctx, msg_add);
    ok_preedit_buffer(ctx, phrase);
    cursor = chewing_cursor_Current(ctx);
    ok(cursor == 2, "cursor position `%d' shall be 2", cursor);
    ok(has_userphrase(ctx, bopomofo, phrase) == 1, "`%s' shall be in userphrase", phrase);

    type_keystroke_by_string(ctx, "<C2>");
    ok_aux_buffer(ctx, msg_already_have);

    type_keystroke_by_string(ctx, "<H><C2>");
    ok_aux_buffer(ctx, msg_error);

    chewing_delete(ctx);
}

void test_CtrlNum_add_phrase_right_symbol_in_between()
{
    static const char bopomofo[] = "\xE3\x84\x98\xE3\x84\x9C\xCB\x8B \xE3\x84\x95\xCB\x8B" /* ㄘㄜˋ ㄕˋ */ ;
    int cursor;
    ChewingContext *ctx;

    clean_userphrase();

    ctx = chewing_new();
    start_testcase(ctx, fd);
    chewing_set_maxChiSymbolLen(ctx, 16);
    chewing_set_addPhraseDirection(ctx, 0);

    ok(has_userphrase(ctx, bopomofo, NULL) == 0, "`%s' shall not be in userphrase", bopomofo);

    type_keystroke_by_string(ctx, "hk4`1g4<H><C2>");
    cursor = chewing_cursor_Current(ctx);
    ok(cursor == 0, "cursor position `%d' shall be 0", cursor);

    ok(has_userphrase(ctx, bopomofo, NULL) == 0, "`%s' shall not be in userphrase", bopomofo);

    chewing_delete(ctx);
}

void test_CtrlNum_add_phrase_left_symbol_in_between()
{
    static const char bopomofo[] = "\xE3\x84\x98\xE3\x84\x9C\xCB\x8B \xE3\x84\x95\xCB\x8B" /* ㄘㄜˋ ㄕˋ */ ;
    int cursor;
    ChewingContext *ctx;

    clean_userphrase();

    ctx = chewing_new();
    start_testcase(ctx, fd);
    chewing_set_maxChiSymbolLen(ctx, 16);
    chewing_set_addPhraseDirection(ctx, 1);

    ok(has_userphrase(ctx, bopomofo, NULL) == 0, "`%s' shall not be in userphrase", bopomofo);

    type_keystroke_by_string(ctx, "hk4`1g4<C2>");
    cursor = chewing_cursor_Current(ctx);
    ok(cursor == 3, "cursor position `%d' shall be 3", cursor);

    ok(has_userphrase(ctx, bopomofo, NULL) == 0, "`%s' shall not be in userphrase", bopomofo);

    chewing_delete(ctx);
}

void test_CtrlNum_add_phrase_right_start_with_symbol()
{
    static const char bopomofo[] =
        "\xE3\x84\x89\xE3\x84\xA4\xCB\x87 \xE3\x84\x8A\xE3\x84\xA8\xCB\x87 \xE3\x84\x91\xE3\x84\xA7\xE3\x84\xA4\xCB\x8A" /* ㄉㄤˇ ㄊㄨˇ ㄑㄧㄤˊ */ ;
    static const char phrase[] = "\xE6\x93\x8B\xE5\x9C\x9F\xE7\x89\x86"; /* 擋土牆 */

    const char *const_buf;
    ChewingContext *ctx;

    clean_userphrase();
    ctx = chewing_new();
    start_testcase(ctx, fd);
    chewing_set_maxChiSymbolLen(ctx, 16);
    chewing_set_addPhraseDirection(ctx, 0);

    ok(has_userphrase(ctx, bopomofo, NULL) == 0, "`%s' shall not be in userphrase", bopomofo);

    type_keystroke_by_string(ctx, "`1hk4g42;3wj3fu;6<L><L><L><D>3<C3>");
    ok(has_userphrase(ctx, bopomofo, NULL) == 1, "`%s' shall be in userphrase", bopomofo);

    chewing_cand_open(ctx);
    chewing_cand_Enumerate(ctx);
    const_buf = chewing_cand_string_by_index_static(ctx, 0);
    ok(strcmp(const_buf, phrase) == 0, "first candidate `%s' shall be `%s'", const_buf, phrase);

    chewing_delete(ctx);
} 

void test_CtrlNum_add_phrase_left_start_with_symbol()
{
    static const char bopomofo[] =
        "\xE3\x84\x89\xE3\x84\xA4\xCB\x87 \xE3\x84\x8A\xE3\x84\xA8\xCB\x87 \xE3\x84\x91\xE3\x84\xA7\xE3\x84\xA4\xCB\x8A" /* ㄉㄤˇ ㄊㄨˇ ㄑㄧㄤˊ */ ;
    static const char phrase[] = "\xE6\x93\x8B\xE5\x9C\x9F\xE7\x89\x86"; /* 擋土牆 */

    const char *const_buf;
    ChewingContext *ctx;

    clean_userphrase();
    ctx = chewing_new();
    start_testcase(ctx, fd);
    chewing_set_maxChiSymbolLen(ctx, 16);
    chewing_set_addPhraseDirection(ctx, 1);

    ok(has_userphrase(ctx, bopomofo, NULL) == 0, "`%s' shall not be in userphrase", bopomofo);

    type_keystroke_by_string(ctx, "`1hk4g42;3wj3fu;6<L><L><L><D>3<EN><C3>");
    ok(has_userphrase(ctx, bopomofo, NULL) == 1, "`%s' shall be in userphrase", bopomofo);

    type_keystroke_by_string(ctx, "<L><L><L>");
    chewing_cand_open(ctx);
    chewing_cand_Enumerate(ctx);
    const_buf = chewing_cand_string_by_index_static(ctx, 0);
    ok(strcmp(const_buf, phrase) == 0, "first candidate `%s' shall be `%s'", const_buf, phrase);

    chewing_delete(ctx);
}

void test_CtrlNum()
{
    test_CtrlNum_add_phrase_right();
    test_CtrlNum_add_phrase_left();
    test_CtrlNum_add_phrase_right_symbol_in_between();
    test_CtrlNum_add_phrase_left_symbol_in_between();
    test_CtrlNum_add_phrase_right_start_with_symbol();
    test_CtrlNum_add_phrase_left_start_with_symbol();
}

void test_userphrase_auto_learn()
{
    static const char bopomofo_1[] =
        "\xE3\x84\x8E\xE3\x84\x9C \xE3\x84\x8E\xE3\x84\x9C \xE3\x84\x8E\xE3\x84\x9C" /* ㄎㄜ ㄎㄜ ㄎㄜ */ ;
    static const char bopomofo_2[] = "\xE3\x84\x98\xE3\x84\x9C\xCB\x8B \xE3\x84\x95\xCB\x8B";   /* ㄘㄜˋ ㄕˋ */
    ChewingContext *ctx;

    clean_userphrase();

    ctx = chewing_new();
    start_testcase(ctx, fd);

    ok(has_userphrase(ctx, bopomofo_1, NULL) == 0, "`%s' shall not be in userphrase", bopomofo_1);
    ok(has_userphrase(ctx, bopomofo_2, NULL) == 0, "`%s' shall not be in userphrase", bopomofo_2);

    chewing_set_autoLearn(ctx, AUTOLEARN_DISABLED);
    ok(chewing_get_autoLearn(ctx) == AUTOLEARN_DISABLED, "AutoLearn shall be `%d'", AUTOLEARN_DISABLED);
    type_keystroke_by_string(ctx, "dk dk dk hk4g4<E>");
    ok(has_userphrase(ctx, bopomofo_1, NULL) == 0, "`%s' shall not be in userphrase", bopomofo_1);
    ok(has_userphrase(ctx, bopomofo_2, NULL) == 0, "`%s' shall not be in userphrase", bopomofo_2);

    chewing_set_autoLearn(ctx, AUTOLEARN_ENABLED);
    ok(chewing_get_autoLearn(ctx) == AUTOLEARN_ENABLED, "AutoLearn shall be `%d'", AUTOLEARN_ENABLED);
    type_keystroke_by_string(ctx, "dk dk dk hk4g4<E>");
    ok(has_userphrase(ctx, bopomofo_1, NULL) == 1, "`%s' shall be in userphrase", bopomofo_1);
    ok(has_userphrase(ctx, bopomofo_2, NULL) == 1, "`%s' shall be in userphrase", bopomofo_2);

    chewing_delete(ctx);
}

void test_userphrase_auto_learn_with_symbol()
{
    static const char bopomofo_1[] = "\xE3\x84\x8E\xE3\x84\x9C" /* ㄎㄜ */ ;
    static const char bopomofo_2[] = "\xE3\x84\x8E\xE3\x84\x9C \xE3\x84\x8E\xE3\x84\x9C" /* ㄎㄜ ㄎㄜ */ ;
    static const char bopomofo_3[] =
        "\xE3\x84\x8E\xE3\x84\x9C \xE3\x84\x8E\xE3\x84\x9C \xE3\x84\x8E\xE3\x84\x9C" /* ㄎㄜ ㄎㄜ ㄎㄜ */ ;
    ChewingContext *ctx;

    clean_userphrase();

    ctx = chewing_new();
    start_testcase(ctx, fd);

    ok(has_userphrase(ctx, bopomofo_1, NULL) == 0, "`%s' shall not be in userphrase", bopomofo_1);

    ok(has_userphrase(ctx, bopomofo_2, NULL) == 0, "`%s' shall not be in userphrase", bopomofo_2);

    ok(has_userphrase(ctx, bopomofo_3, NULL) == 0, "`%s' shall not be in userphrase", bopomofo_3);

    type_keystroke_by_string(ctx, "`31dk `31dk dk `31<E>");

    ok(has_userphrase(ctx, bopomofo_1, NULL) == 1, "`%s' shall be in userphrase", bopomofo_1);

    ok(has_userphrase(ctx, bopomofo_2, NULL) == 1, "`%s' shall be in userphrase", bopomofo_2);

    ok(has_userphrase(ctx, bopomofo_3, NULL) == 0, "`%s' shall not be in userphrase", bopomofo_3);

    chewing_delete(ctx);
}

void test_userphrase_auto_learn_hardcode_break()
{
    /* 的 is a hardcode break point, see ChewingIsBreakPoint */
    static const char phrase[] = "\xE7\x9A\x84\xE7\x9A\x84" /* 的的 */ ;
    static const char bopomofo[] =
        "\xE3\x84\x89\xE3\x84\x9C\xCB\x99 \xE3\x84\x89\xE3\x84\x9C\xCB\x99" /* ㄉㄜ˙ ㄉㄜ˙ */ ;
    ChewingContext *ctx;

    clean_userphrase();

    ctx = chewing_new();
    start_testcase(ctx, fd);
    chewing_set_maxChiSymbolLen(ctx, 16);
    chewing_set_addPhraseDirection(ctx, 1);

    ok(has_userphrase(ctx, bopomofo, phrase) == 0, "`%s' shall not be in userphrase", phrase);

    type_keystroke_by_string(ctx, "2k72k7<E>");
    ok(has_userphrase(ctx, bopomofo, phrase) == 0, "`%s' shall not be in userphrase", phrase);

    chewing_delete(ctx);
}

void test_userphrase_auto_learn_only_after_commit()
{
    /* GitHub #206: It should add the word after user actually finish the character selection. */

    const char bopomofo_1[] = "\xE3\x84\x94\xE3\x84\xA4\xCB\x8A \xE3\x84\x86\xE3\x84\xA2\xCB\x8A"; /* ㄔㄤˊ ㄆㄢˊ */
    const char bopomofo_2[] = "\xE3\x84\x94\xE3\x84\xA4\xCB\x8A"; /* ㄔㄤˊ */

    ChewingContext *ctx;

    clean_userphrase();

    ctx = chewing_new();
    start_testcase(ctx, fd);

    /* user just inputs some characters: don't auto learn. */
    type_keystroke_by_string(ctx, "t;6q06");
    ok(has_userphrase(ctx, bopomofo_1, NULL) == 0, "`%s' shall not be in userphrase", bopomofo_1);
    ok(has_userphrase(ctx, bopomofo_2, NULL) == 0, "`%s' shall not be in userphrase", bopomofo_2);

    /* user selectes a candidate on the list, but doesn't commit: don't auto learn. */
    type_keystroke_by_string(ctx, "<L><L><D>7");
    ok(has_userphrase(ctx, bopomofo_1, NULL) == 0, "`%s' shall not be in userphrase", bopomofo_1);
    ok(has_userphrase(ctx, bopomofo_2, NULL) == 0, "`%s' shall not be in userphrase", bopomofo_2);

    /* user selectes another cadidate and commit: auto learn phrase(s), but not the selected candidate. */
    type_keystroke_by_string(ctx, "<L><D>2<E>");
    ok(has_userphrase(ctx, bopomofo_1, NULL) == 1, "`%s' shall be in userphrase", bopomofo_1);
    ok(has_userphrase(ctx, bopomofo_2, NULL) == 0, "`%s' shall not be in userphrase", bopomofo_2);

    chewing_delete(ctx);
}

void test_userphrase()
{
    test_userphrase_auto_learn();
    test_userphrase_auto_learn_with_symbol();
    test_userphrase_auto_learn_hardcode_break();
    test_userphrase_auto_learn_only_after_commit();
}

void test_userphrase_enumerate_normal()
{
    ChewingContext *ctx;
    int ret;
    unsigned int expect_len;

    const char phrase[] = "\xE6\xB8\xAC\xE8\xA9\xA6" /* 測試 */ ;
    char phrase_buf[50];
    unsigned int phrase_len;

    const char bopomofo[] = "\xE3\x84\x98\xE3\x84\x9C\xCB\x8B \xE3\x84\x95\xCB\x8B";    /* ㄘㄜˋ ㄕˋ */
    char bopomofo_buf[50];
    unsigned int bopomofo_len;

    clean_userphrase();

    ctx = chewing_new();
    start_testcase(ctx, fd);

    ret = chewing_userphrase_add(ctx, phrase, bopomofo);
    ok(ret == 1, "chewing_userphrase_add() return value `%d' shall be `%d'", ret, 1);
    ret = chewing_userphrase_lookup(ctx, phrase, bopomofo);
    ok(ret == 1, "chewing_userphrase_lookup() return value `%d' shall be `%d'", ret, 1);

    ret = chewing_userphrase_enumerate(ctx);
    ok(ret == 0, "chewing_userphrase_enumerate() return value `%d' shall be `%d'", ret, 0);

    ret = chewing_userphrase_has_next(ctx, &phrase_len, &bopomofo_len);
    ok(ret == 1, "chewing_userphrase_has_next() return value `%d' shall be `%d'", ret, 1);
    expect_len = strlen(phrase) + 1;
    ok(phrase_len >= expect_len, "chewing_userphrase_has_next() shall set phrase_len `%d' >= `%d'", phrase_len,
       expect_len);
    expect_len = strlen(bopomofo) + 1;
    ok(bopomofo_len >= expect_len, "chewing_userphrase_has_next() shall set bopomofo_len `%d' >= `%d'", bopomofo_len,
       expect_len);
    ret = chewing_userphrase_get(ctx, phrase_buf, sizeof(phrase_buf), bopomofo_buf, sizeof(bopomofo_buf));
    ok(ret == 0, "chewing_userphrase_get() return value `%d' shall be `%d'", ret, 0);
    ok(strcmp(phrase_buf, phrase) == 0, "chewing_userphrase_get() shall set phrase_buf `%s' to `%s'", phrase_buf,
       phrase);
    ok(strcmp(bopomofo_buf, bopomofo) == 0, "chewing_userphrase_get() shall set bopomofo_buf `%s' to `%s'",
       bopomofo_buf, bopomofo);

    ret = chewing_userphrase_has_next(ctx, &phrase_len, &bopomofo_len);
    ok(ret == 0, "chewing_userphrase_has_next() return value `%d' shall be `%d'", ret, 0);

    chewing_delete(ctx);
}

void test_userphrase_enumerate_empty()
{
    ChewingContext *ctx;
    int ret;
    const char phrase[] = "\xE6\xB8\xAC\xE8\xA9\xA6" /* 測試 */ ;
    unsigned int phrase_len;
    const char bopomofo[] = "\xE3\x84\x98\xE3\x84\x9C\xCB\x8B \xE3\x84\x95\xCB\x8B";    /* ㄘㄜˋ ㄕˋ */
    unsigned int bopomofo_len;

    clean_userphrase();

    ctx = chewing_new();
    start_testcase(ctx, fd);

    ret = chewing_userphrase_lookup(ctx, phrase, bopomofo);
    ok(ret == 0, "chewing_userphrase_lookup() return value `%d' shall be `%d'", ret, 0);

    ret = chewing_userphrase_enumerate(ctx);
    ok(ret == 0, "chewing_userphrase_enumerate() return value `%d' shall be `%d'", ret, 0);

    ret = chewing_userphrase_has_next(ctx, &phrase_len, &bopomofo_len);
    ok(ret == 0, "chewing_userphrase_has_next() return value `%d' shall be `%d'", ret, 0);

    chewing_delete(ctx);
}

void test_userphrase_enumerate_rewind()
{
    ChewingContext *ctx;
    int ret;
    unsigned int expect_len;

    const char phrase[] = "\xE6\xB8\xAC\xE8\xA9\xA6" /* 測試 */ ;
    char phrase_buf[50];
    unsigned int phrase_len;

    const char bopomofo[] = "\xE3\x84\x98\xE3\x84\x9C\xCB\x8B \xE3\x84\x95\xCB\x8B";    /* ㄘㄜˋ ㄕˋ */
    char bopomofo_buf[50];
    unsigned int bopomofo_len;

    clean_userphrase();

    ctx = chewing_new();
    start_testcase(ctx, fd);

    ret = chewing_userphrase_add(ctx, phrase, bopomofo);
    ok(ret == 1, "chewing_userphrase_add() return value `%d' shall be `%d'", ret, 1);
    ret = chewing_userphrase_lookup(ctx, phrase, bopomofo);
    ok(ret == 1, "chewing_userphrase_lookup() return value `%d' shall be `%d'", ret, 1);

    ret = chewing_userphrase_enumerate(ctx);
    ok(ret == 0, "chewing_userphrase_enumerate() return value `%d' shall be `%d'", ret, 0);

    ret = chewing_userphrase_has_next(ctx, &phrase_len, &bopomofo_len);
    ok(ret == 1, "chewing_userphrase_has_next() return value `%d' shall be `%d'", ret, 1);
    expect_len = strlen(phrase) + 1;
    ok(phrase_len >= expect_len, "chewing_userphrase_has_next() shall set phrase_len `%d' >= `%d'", phrase_len,
       expect_len);
    expect_len = strlen(bopomofo) + 1;
    ok(bopomofo_len >= expect_len, "chewing_userphrase_has_next() shall set bopomofo_len `%d' >= `%d'", bopomofo_len,
       expect_len);
    ret = chewing_userphrase_get(ctx, phrase_buf, sizeof(phrase_buf), bopomofo_buf, sizeof(bopomofo_buf));
    ok(ret == 0, "chewing_userphrase_get() return value `%d' shall be `%d'", ret, 0);
    ok(strcmp(phrase_buf, phrase) == 0, "chewing_userphrase_get() shall set phrase_buf `%s' to `%s'", phrase_buf,
       phrase);
    ok(strcmp(bopomofo_buf, bopomofo) == 0, "chewing_userphrase_get() shall set bopomofo_buf `%s' to `%s'",
       bopomofo_buf, bopomofo);

    ret = chewing_userphrase_enumerate(ctx);
    ok(ret == 0, "chewing_userphrase_enumerate() return value `%d' shall be `%d'", ret, 0);

    ret = chewing_userphrase_has_next(ctx, &phrase_len, &bopomofo_len);
    ok(ret == 1, "chewing_userphrase_has_next() return value `%d' shall be `%d'", ret, 1);
    expect_len = strlen(phrase) + 1;
    ok(phrase_len >= expect_len, "chewing_userphrase_has_next() shall set phrase_len `%d' >= `%d'", phrase_len,
       expect_len);
    expect_len = strlen(bopomofo) + 1;
    ok(bopomofo_len >= expect_len, "chewing_userphrase_has_next() shall set bopomofo_len `%d' >= `%d'", bopomofo_len,
       expect_len);
    ret = chewing_userphrase_get(ctx, phrase_buf, sizeof(phrase_buf), bopomofo_buf, sizeof(bopomofo_buf));
    ok(ret == 0, "chewing_userphrase_get() return value `%d' shall be `%d'", ret, 0);
    ok(strcmp(phrase_buf, phrase) == 0, "chewing_userphrase_get() shall set phrase_buf `%s' to `%s'", phrase_buf,
       phrase);
    ok(strcmp(bopomofo_buf, bopomofo) == 0, "chewing_userphrase_get() shall set bopomofo_buf `%s' to `%s'",
       bopomofo_buf, bopomofo);

    chewing_delete(ctx);
}

void test_userphrase_enumerate()
{
    test_userphrase_enumerate_normal();
    test_userphrase_enumerate_empty();
    test_userphrase_enumerate_rewind();
}

void test_userphrase_manipulate_normal()
{
    ChewingContext *ctx;
    const char phrase[] = "\xE6\xB8\xAC\xE8\xA9\xA6" /* 測試 */ ;
    const char bopomofo[] = "\xE3\x84\x98\xE3\x84\x9C\xCB\x8B \xE3\x84\x95\xCB\x8B";    /* ㄘㄜˋ ㄕˋ */
    int ret;

    clean_userphrase();

    ctx = chewing_new();
    start_testcase(ctx, fd);
    ret = chewing_userphrase_lookup(ctx, phrase, bopomofo);
    ok(ret == 0, "chewing_userphrase_lookup() return value `%d' shall be `%d'", ret, 0);

    ret = chewing_userphrase_add(ctx, phrase, bopomofo);
    ok(ret == 1, "chewing_userphrase_add() return value `%d' shall be `%d'", ret, 1);
    ret = chewing_userphrase_lookup(ctx, phrase, bopomofo);
    ok(ret == 1, "chewing_userphrase_lookup() return value `%d' shall be `%d'", ret, 1);

    ret = chewing_userphrase_remove(ctx, phrase, bopomofo);
    ok(ret == 1, "chewing_userphrase_remove() return value `%d' shall be `%d'", ret, 1);
    ret = chewing_userphrase_lookup(ctx, phrase, bopomofo);
    ok(ret == 0, "chewing_userphrase_lookup() return value `%d' shall be `%d'", ret, 0);

    chewing_delete(ctx);

    /* New chewing instance shall not have remove userphrase. */
    ctx = chewing_new();

    ret = chewing_userphrase_lookup(ctx, phrase, bopomofo);
    ok(ret == 0, "chewing_userphrase_lookup() return value `%d' shall be `%d'", ret, 0);

    chewing_delete(ctx);
}

void test_userphrase_manipulate_maximum()
{
    ChewingContext *ctx;
    const char phrase_in_limit[] =
        "\xE9\x87\x91\xE7\xAA\xA9\xE9\x8A\x80\xE7\xAA\xA9\xE4\xB8\x8D\xE5\xA6\x82\xE8\x87\xAA\xE5\xB7\xB1\xE7\x9A\x84\xE7\x8B\x97\xE7\xAA\xA9";
    /* 金窩銀窩不如自己的狗窩 */
    const char bopomofo_in_limit[] =
        "\xE3\x84\x90\xE3\x84\xA7\xE3\x84\xA3\x20\xE3\x84\xA8\xE3\x84\x9B\x20\xE3\x84\xA7\xE3\x84\xA3\xCB\x8A\x20\xE3\x84\xA8\xE3\x84\x9B\x20\xE3\x84\x85\xE3\x84\xA8\xCB\x8B\x20\xE3\x84\x96\xE3\x84\xA8\xCB\x8A\x20\xE3\x84\x97\xCB\x8B\x20\xE3\x84\x90\xE3\x84\xA7\xCB\x87\x20\xE3\x84\x89\xE3\x84\x9C\xCB\x99\x20\xE3\x84\x8D\xE3\x84\xA1\xCB\x87\x20\xE3\x84\xA8\xE3\x84\x9B";
    /* ㄐㄧㄣ ㄨㄛ ㄧㄣˊ ㄨㄛ ㄅㄨˋ ㄖㄨˊ ㄗˋ ㄐㄧˇ ㄉㄜ˙ ㄍㄡˇ ㄨㄛ */
    const char phrase_out_of_limit[] =
        "\xE9\x87\x91\xE7\xAA\xA9\xE9\x8A\x80\xE7\xAA\xA9\xE4\xB8\x8D\xE5\xA6\x82\xE8\x87\xAA\xE5\xB7\xB1\xE7\x9A\x84\xE7\x8B\x97\xE7\xAA\xA9\xE5\x97\x8E";
    /* 金窩銀窩不如自己的狗窩嗎 */
    const char bopomofo_out_of_limit[] =
        "\xE3\x84\x90\xE3\x84\xA7\xE3\x84\xA3\x20\xE3\x84\xA8\xE3\x84\x9B\x20\xE3\x84\xA7\xE3\x84\xA3\xCB\x8A\x20\xE3\x84\xA8\xE3\x84\x9B\x20\xE3\x84\x85\xE3\x84\xA8\xCB\x8B\x20\xE3\x84\x96\xE3\x84\xA8\xCB\x8A\x20\xE3\x84\x97\xCB\x8B\x20\xE3\x84\x90\xE3\x84\xA7\xCB\x87\x20\xE3\x84\x89\xE3\x84\x9C\xCB\x99\x20\xE3\x84\x8D\xE3\x84\xA1\xCB\x87\x20\xE3\x84\xA8\xE3\x84\x9B \xE3\x84\x87\xE3\x84\x9A\xCB\x99";
    /* ㄐㄧㄣ ㄨㄛ ㄧㄣˊ ㄨㄛ ㄅㄨˋ ㄖㄨˊ ㄗˋ ㄐㄧˇ ㄉㄜ˙ ㄍㄡˇ ㄨㄛ ㄇㄚ˙ */
    int ret;

    clean_userphrase();

    ctx = chewing_new();
    start_testcase(ctx, fd);

    ret = chewing_userphrase_add(ctx, phrase_in_limit, bopomofo_in_limit);
    ok(ret == 1, "chewing_userphrase_add() return value `%d' shall be `%d'", ret, 1);
    ret = chewing_userphrase_lookup(ctx, phrase_in_limit, bopomofo_in_limit);
    ok(ret == 1, "chewing_userphrase_lookup() return value `%d' shall be `%d'", ret, 1);

    ret = chewing_userphrase_add(ctx, phrase_out_of_limit, bopomofo_out_of_limit);
    ok(ret == 0, "chewing_userphrase_add() return value `%d' shall be `%d'", ret, 0);
    ret = chewing_userphrase_lookup(ctx, phrase_out_of_limit, bopomofo_out_of_limit);
    ok(ret == 0, "chewing_userphrase_lookup() return value `%d' shall be `%d'", ret, 0);

    chewing_delete(ctx);
}

void test_userphrase_manipulate_hash_collision()
{
    ChewingContext *ctx;

    /* 測試 */
    const char phrase_1[] = "\xE6\xB8\xAC\xE8\xA9\xA6";

    /* ㄘㄜˋ ㄕˋ */
    const char bopomofo_1[] = "\xE3\x84\x98\xE3\x84\x9C\xCB\x8B \xE3\x84\x95\xCB\x8B";

    /* 測試測試測試 */
    const char phrase_2[] = "\xE6\xB8\xAC\xE8\xA9\xA6" "\xE6\xB8\xAC\xE8\xA9\xA6" "\xE6\xB8\xAC\xE8\xA9\xA6";

    /* ㄘㄜˋ ㄕˋ ㄘㄜˋ ㄕˋ ㄘㄜˋ ㄕˋ */
    const char bopomofo_2[] =
        "\xE3\x84\x98\xE3\x84\x9C\xCB\x8B \xE3\x84\x95\xCB\x8B "
        "\xE3\x84\x98\xE3\x84\x9C\xCB\x8B \xE3\x84\x95\xCB\x8B "
        "\xE3\x84\x98\xE3\x84\x9C\xCB\x8B \xE3\x84\x95\xCB\x8B";

    int ret;

    clean_userphrase();

    ctx = chewing_new();
    start_testcase(ctx, fd);

    ret = chewing_userphrase_add(ctx, phrase_1, bopomofo_1);
    ok(ret == 1, "chewing_userphrase_add() return value `%d' shall be `%d'", ret, 1);
    ret = chewing_userphrase_add(ctx, phrase_2, bopomofo_2);
    ok(ret == 1, "chewing_userphrase_add() return value `%d' shall be `%d'", ret, 1);

    ret = chewing_userphrase_lookup(ctx, phrase_1, bopomofo_1);
    ok(ret == 1, "chewing_userphrase_lookup() return value `%d' shall be `%d'", ret, 1);
    ret = chewing_userphrase_lookup(ctx, phrase_2, bopomofo_2);
    ok(ret == 1, "chewing_userphrase_lookup() return value `%d' shall be `%d'", ret, 1);

    ret = chewing_userphrase_remove(ctx, phrase_1, bopomofo_1);
    ok(ret == 1, "chewing_userphrase_remove() return value `%d' shall be `%d'", ret, 1);
    ret = chewing_userphrase_remove(ctx, phrase_2, bopomofo_2);
    ok(ret == 1, "chewing_userphrase_remove() return value `%d' shall be `%d'", ret, 1);

    ret = chewing_userphrase_lookup(ctx, phrase_1, bopomofo_1);
    ok(ret == 0, "chewing_userphrase_lookup() return value `%d' shall be `%d'", ret, 0);
    ret = chewing_userphrase_lookup(ctx, phrase_2, bopomofo_2);
    ok(ret == 0, "chewing_userphrase_lookup() return value `%d' shall be `%d'", ret, 0);

    chewing_delete(ctx);
}

void test_userphrase_manipulate_error_handling()
{
    ChewingContext *ctx;
    int ret;

    clean_userphrase();

    ctx = chewing_new();
    start_testcase(ctx, fd);

    ret = chewing_userphrase_add(ctx, "\xE6\xB8\xAC\xE8\xA9\xA6" /* 測試 */ ,
                                 "\xE3\x84\x98\xE3\x84\x9C\xCB\x8B" /* ㄘㄜˋ */ );
    ok(ret == 0, "chewing_userphrase_add() return value `%d' shall be `%d'", ret, 0);

    ret = chewing_userphrase_add(ctx, "\xE6\xB8\xAC\xE8\xA9\xA6" /* 測試 */ ,
                                 "\xE3\x84\x98\xE3\x84\x9C\xCB\x8B \xCB\x8B\xE3\x84\x95" /* ㄘㄜˋ ˋㄕ */ );
    ok(ret == 0, "chewing_userphrase_add() return value `%d' shall be `%d'", ret, 0);

    ret = chewing_userphrase_remove(ctx, "\xE6\xB8\xAC\xE8\xA9\xA6" /* 測試 */ ,
                                    "\xE3\x84\x98\xE3\x84\x9C\xCB\x8B \xCB\x8B\xE3\x84\x95" /* ㄘㄜˋ ˋㄕ */ );
    ok(ret == 0, "chewing_userphrase_remove() return value `%d' shall be `%d'", ret, 0);

    chewing_delete(ctx);
}

void test_userphrase_manipulate_remove_same_phone()
{
    ChewingContext *ctx;
    int ret;

    const char phrase_1[] = "\xE6\xB8\xAC\xE8\xA9\xA6" /* 測試 */ ;
    const char phrase_2[] = "\xE5\x81\xB4\xE5\xAE\xA4" /* 側室 */ ;
    const char bopomofo[] = "\xE3\x84\x98\xE3\x84\x9C\xCB\x8B \xE3\x84\x95\xCB\x8B" /* ㄘㄜˋ ㄕˋ */ ;

    clean_userphrase();

    ctx = chewing_new();
    start_testcase(ctx, fd);

    ret = chewing_userphrase_add(ctx, phrase_1, bopomofo);
    ok(ret == 1, "chewing_userphrase_add() return value `%d' shall be `%d'", ret, 1);
    ret = chewing_userphrase_add(ctx, phrase_2, bopomofo);
    ok(ret == 1, "chewing_userphrase_add() return value `%d' shall be `%d'", ret, 1);

    ret = chewing_userphrase_lookup(ctx, phrase_1, bopomofo);
    ok(ret == 1, "chewing_lookup() return value `%d' shall be `%d'", ret, 1);
    ret = chewing_userphrase_lookup(ctx, phrase_2, bopomofo);
    ok(ret == 1, "chewing_lookup() return value `%d' shall be `%d'", ret, 1);

    ret = chewing_userphrase_remove(ctx, phrase_1, bopomofo);
    ok(ret == 1, "chewing_userphrase_remove() return value `%d' shall be `%d'", ret, 1);

    ret = chewing_userphrase_lookup(ctx, phrase_1, bopomofo);
    ok(ret == 0, "chewing_lookup() return value `%d' shall be `%d'", ret, 0);
    ret = chewing_userphrase_lookup(ctx, phrase_2, bopomofo);
    ok(ret == 1, "chewing_lookup() return value `%d' shall be `%d'", ret, 1);

    chewing_delete(ctx);
}

void test_userphrase_manipulate_remove_same_phrase()
{
    ChewingContext *ctx;
    int ret;

    const char phrase[] = "\xE4\xBB\x80\xE9\xBA\xBC" /* 什麼 */ ;
    const char bopomofo_1[] =
        "\xE3\x84\x95\xE3\x84\xA3\xCB\x8A \xE3\x84\x87\xE3\x84\x9C\xCB\x99" /* ㄕㄣˊ ㄇㄜ˙ */ ;
    const char bopomofo_2[] =
        "\xE3\x84\x95\xE3\x84\x9C\xCB\x8A \xE3\x84\x87\xE3\x84\x9C\xCB\x99" /* ㄕㄜˊ ㄇㄜ˙ */ ;

    clean_userphrase();

    ctx = chewing_new();
    start_testcase(ctx, fd);

    ret = chewing_userphrase_add(ctx, phrase, bopomofo_1);
    ok(ret == 1, "chewing_userphrase_add() return value `%d' shall be `%d'", ret, 1);
    ret = chewing_userphrase_add(ctx, phrase, bopomofo_2);
    ok(ret == 1, "chewing_userphrase_add() return value `%d' shall be `%d'", ret, 1);

    ret = chewing_userphrase_lookup(ctx, phrase, bopomofo_1);
    ok(ret == 1, "chewing_lookup() return value `%d' shall be `%d'", ret, 1);
    ret = chewing_userphrase_lookup(ctx, phrase, bopomofo_2);
    ok(ret == 1, "chewing_lookup() return value `%d' shall be `%d'", ret, 1);

    ret = chewing_userphrase_remove(ctx, phrase, bopomofo_1);
    ok(ret == 1, "chewing_userphrase_remove() return value `%d' shall be `%d'", ret, 1);

    ret = chewing_userphrase_lookup(ctx, phrase, bopomofo_1);
    ok(ret == 0, "chewing_lookup() return value `%d' shall be `%d'", ret, 0);
    ret = chewing_userphrase_lookup(ctx, phrase, bopomofo_2);
    ok(ret == 1, "chewing_lookup() return value `%d' shall be `%d'", ret, 1);

    chewing_delete(ctx);
}

void test_userphrase_manipulate_remove_non_userphrase()
{
    ChewingContext *ctx;
    int ret;

    const char phrase[] = "\xE6\xB8\xAC\xE8\xA9\xA6" /* 測試 */ ;
    const char bopomofo[] = "\xE3\x84\x98\xE3\x84\x9C\xCB\x8B \xE3\x84\x95\xCB\x8B" /* ㄘㄜˋ ㄕˋ */ ;

    clean_userphrase();

    ctx = chewing_new();
    start_testcase(ctx, fd);

    ret = chewing_userphrase_lookup(ctx, phrase, bopomofo);
    ok(ret == 0, "chewing_lookup() return value `%d' shall be `%d'", ret, 0);

    ret = chewing_userphrase_remove(ctx, phrase, bopomofo);
    ok(ret == 0, "chewing_userphrase_remove() return value `%d' shall be `%d'", ret, 0);

    chewing_delete(ctx);
}

void test_userphrase_manipulate()
{
    test_userphrase_manipulate_normal();
    test_userphrase_manipulate_maximum();
    test_userphrase_manipulate_hash_collision();
    test_userphrase_manipulate_error_handling();
    test_userphrase_manipulate_remove_same_phone();
    test_userphrase_manipulate_remove_same_phrase();
    test_userphrase_manipulate_remove_non_userphrase();
}

void test_userphrase_lookup()
{
    ChewingContext *ctx;
    int ret;

    clean_userphrase();

    ctx = chewing_new();
    start_testcase(ctx, fd);

    ret = chewing_userphrase_lookup(ctx, "\xE6\xB8\xAC\xE8\xA9\xA6" /* 測試 */ ,
                                    "\xE3\x84\x98\xE3\x84\x9C\xCB\x8B" /* ㄘㄜˋ */ );
    ok(ret == 0, "chewing_userphrase_lookup() return value `%d' shall be `%d'", ret, 0);

    ret = chewing_userphrase_lookup(ctx, "\xE6\xB8\xAC\xE8\xA9\xA6" /* 測試 */ ,
                                    "\xE3\x84\x98\xE3\x84\x9C\xCB\x8B \xCB\x8B\xE3\x84\x95" /* ㄘㄜˋ ˋㄕ */ );
    ok(ret == 0, "chewing_userphrase_lookup() return value `%d' shall be `%d'", ret, 0);

    chewing_delete(ctx);
}

void test_userphrase_double_free()
{
    ChewingContext *ctx = NULL;
    char p1[] = "\xE6\xB8\xAC";
    char p2[] = "\xE7\xAD\x96";
    char b1[] = "\xE3\x84\x98\xE3\x84\x9C\xCB\x8B";
    int ret = 0;

    clean_userphrase();

    start_testcase(ctx, fd);

    ctx = chewing_new();
    ret = chewing_userphrase_add(ctx, p1, b1);
    ok(ret == 1, "chewing_userphrase_add() return value `%d' shall be `%d'", ret, 1);
    ret = chewing_userphrase_add(ctx, p2, b1);
    ok(ret == 1, "chewing_userphrase_add() return value `%d' shall be `%d'", ret, 1);
    ret = chewing_userphrase_remove(ctx, p1, b1);
    ok(ret == 1, "chewing_userphrase_remove() return value `%d' shall be `%d'", ret, 1);
    chewing_delete(ctx);
    ctx = NULL;

    ctx = chewing_new();
    ret = chewing_userphrase_add(ctx, p1, b1);
    ok(ret == 1, "chewing_userphrase_add() return value `%d' shall be `%d'", ret, 1);
    ret = chewing_userphrase_add(ctx, p2, b1);
    ok(ret == 1, "chewing_userphrase_add() return value `%d' shall be `%d'", ret, 1);
    chewing_userphrase_remove(ctx, p1, b1);
    ok(ret == 1, "chewing_userphrase_remove() return value `%d' shall be `%d'", ret, 1);
    chewing_delete(ctx);
    ctx = NULL;
}

void test_userphrase_remove()
{
    ChewingContext *ctx = NULL;
    char p1[] = "\xE6\xB8\xAC";
    char p2[] = "\xE7\xAD\x96";
    char b1[] = "\xE3\x84\x98\xE3\x84\x9C\xCB\x8B";
    int ret = 0;

    clean_userphrase();

    start_testcase(ctx, fd);

    ctx = chewing_new();
    ret = chewing_userphrase_add(ctx, p1, b1);
    ok(ret == 1, "chewing_userphrase_add() return value `%d' shall be `%d'", ret, 1);
    ret = chewing_userphrase_add(ctx, p2, b1);
    ok(ret == 1, "chewing_userphrase_add() return value `%d' shall be `%d'", ret, 1);
    ret = chewing_userphrase_remove(ctx, p1, b1);
    ok(ret == 1, "chewing_userphrase_remove() return value `%d' shall be `%d'", ret, 1);
    chewing_delete(ctx);
    ctx = NULL;

    ctx = chewing_new();
    ret = chewing_userphrase_remove(ctx, p2, b1);
    ok(ret == 1, "chewing_userphrase_remove() return value `%d' shall be `%d'", ret, 1);
    chewing_delete(ctx);
    ctx = NULL;

    ctx = chewing_new();
    ret = chewing_userphrase_lookup(ctx, p2, b1);
    ok(ret == 0, "chewing_userphrase_lookup() return value `%d' shall be `%d'", ret, 0);
    chewing_delete(ctx);
    ctx = NULL;
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

    test_ShiftLeft();
    test_ShiftRight();
    test_CtrlNum();
    test_userphrase();
    test_userphrase_enumerate();
    test_userphrase_manipulate();
    test_userphrase_lookup();
    test_userphrase_double_free();
    test_userphrase_remove();

    fclose(fd);

    return exit_status();
}
