/**
 * test-bopomofo.c
 *
 * Copyright (c) 2012
 *      libchewing Core Team. See ChangeLog for details.
 *
 * See the file "COPYING" for information on usage and redistribution
 * of this file.
 */

#ifdef HAVE_CONFIG_H
#    include <config.h>
#endif

#include <assert.h>
#include <stdlib.h>
#include <stdio.h>
#include <string.h>

#include "bopomofo-private.h"
#include "chewing.h"
#include "plat_types.h"
#include "testhelper.h"

FILE *fd;

void test_select_candidate_no_rearward()
{
    /*
     * The following phrases are in dict
     * 一上來
     * 上來
     * 移上來
     * 移上
     */

    static const char *CAND_1[] = {
        "\xE4\xB8\x80\xE4\xB8\x8A\xE4\xBE\x86" /* 一上來 */ ,
        "\xE7\xA7\xBB\xE4\xB8\x8A\xE4\xBE\x86" /* 移上來 */ ,
    };

    static const char *CAND_2[] = {
        "\xE7\xA7\xBB\xE4\xB8\x8A" /* 移上 */ ,
    };

    ChewingContext *ctx;

    clean_userphrase();

    ctx = chewing_new();
    start_testcase(ctx, fd);

    chewing_set_maxChiSymbolLen(ctx, 16);

    type_keystroke_by_string(ctx, "u6g;4x96<L><L><L>"); /* ㄧˊㄕㄤˋㄌㄞˊ */

    type_keystroke_by_string(ctx, "<D>");       /* ㄧˊㄕㄤˋㄌㄞˊ */
    ok_candidate(ctx, CAND_1, ARRAY_SIZE(CAND_1));

    type_keystroke_by_string(ctx, "<D>");       /* ㄕㄤˋㄌㄞˊ */
    ok_candidate(ctx, CAND_2, ARRAY_SIZE(CAND_2));

    type_keystroke_by_string(ctx, "<D><D>2<E>");        /* select 移上來 */
    ok_commit_buffer(ctx, CAND_1[1]);

    chewing_delete(ctx);
}

void test_select_candidate_rearward()
{
    /*
     * The following phrases are in dict
     * 一上來
     * 上來
     * 移上來
     * 移上
     */

    static const char *CAND_1[] = {
        "\xE4\xB8\x80\xE4\xB8\x8A\xE4\xBE\x86" /* 一上來 */ ,
        "\xE7\xA7\xBB\xE4\xB8\x8A\xE4\xBE\x86" /* 移上來 */ ,
    };

    static const char *CAND_2[] = {
        "\xE4\xB8\x8A\xE4\xBE\x86" /* 上來 */ ,
    };
    ChewingContext *ctx;

    clean_userphrase();

    ctx = chewing_new();
    start_testcase(ctx, fd);

    chewing_set_maxChiSymbolLen(ctx, 16);
    chewing_set_phraseChoiceRearward(ctx, 1);

    type_keystroke_by_string(ctx, "u6g;4x96");  /* ㄧˊㄕㄤˋㄌㄞˊ */
    ok_preedit_buffer(ctx, CAND_1[0]);

    type_keystroke_by_string(ctx, "<D>");       /* ㄧˊㄕㄤˋㄌㄞˊ */
    ok_candidate(ctx, CAND_1, ARRAY_SIZE(CAND_1));

    type_keystroke_by_string(ctx, "<D>");       /* ㄕㄤˋㄌㄞˊ */
    ok_candidate(ctx, CAND_2, ARRAY_SIZE(CAND_2));

    type_keystroke_by_string(ctx, "<D><D>2<E>");        /* select 移上來 */
    ok_commit_buffer(ctx, CAND_1[1]);

    chewing_delete(ctx);
}

void test_select_candidate_no_rearward_with_symbol()
{
    ChewingContext *ctx;
    int ret;
    char *buf;
    int len;

    clean_userphrase();

    ctx = chewing_new();
    start_testcase(ctx, fd);

    type_keystroke_by_string(ctx, "hk4g4`31u6vu84" /* 測試，一下 */ );

    type_keystroke_by_string(ctx, "<EE><H><D>");
    ret = chewing_cand_TotalChoice(ctx);
    ok(ret > 0, "chewing_cand_TotalChoice() returns `%d' shall greater than 0 at pos `%d'", ret, 0);
    chewing_cand_Enumerate(ctx);
    buf = chewing_cand_String(ctx);
    len = ueStrLen(buf);
    ok(len == 2, "candidate `%s' length `%d' shall be `%d' at pos `%d'", buf, len, 2, 0);
    chewing_free(buf);

    type_keystroke_by_string(ctx, "<EE><H><R><D>");
    ret = chewing_cand_TotalChoice(ctx);
    ok(ret > 0, "chewing_cand_TotalChoice() returns `%d' shall greater than 0 at pos `%d'", ret, 1);
    chewing_cand_Enumerate(ctx);
    buf = chewing_cand_String(ctx);
    len = ueStrLen(buf);
    ok(len == 1, "candidate `%s' length `%d' shall be `%d' at pos `%d'", buf, len, 1, 1);
    chewing_free(buf);

    type_keystroke_by_string(ctx, "<EE><H><R><R><D>");
    ret = chewing_cand_TotalChoice(ctx);
    ok(ret > 0, "chewing_cand_TotalChoice() returns `%d' shall greater than 0 at pos `%d'", ret, 2);
    chewing_cand_Enumerate(ctx);
    buf = chewing_cand_String(ctx);
    len = ueStrLen(buf);
    ok(len == 1, "candidate `%s' length `%d' shall be `%d' at pos `%d'", buf, len, 1, 2);
    chewing_free(buf);

    type_keystroke_by_string(ctx, "<EE><H><R><R><R><D>");
    ret = chewing_cand_TotalChoice(ctx);
    ok(ret > 0, "chewing_cand_TotalChoice() returns `%d' shall greater than 0 at pos `%d'", ret, 3);
    chewing_cand_Enumerate(ctx);
    buf = chewing_cand_String(ctx);
    len = ueStrLen(buf);
    ok(len == 2, "candidate `%s' length `%d' shall be `%d' at pos `%d'", buf, len, 2, 3);
    chewing_free(buf);

    type_keystroke_by_string(ctx, "<EE><H><R><R><R><R><D>");
    ret = chewing_cand_TotalChoice(ctx);
    ok(ret > 0, "chewing_cand_TotalChoice() returns `%d' shall greater than 0 at pos `%d'", ret, 4);
    chewing_cand_Enumerate(ctx);
    buf = chewing_cand_String(ctx);
    len = ueStrLen(buf);
    ok(len == 1, "candidate `%s' length `%d' shall be `%d' at pos `%d'", buf, len, 1, 4);
    chewing_free(buf);

    chewing_delete(ctx);
}

void test_select_candidate_rearward_with_symbol()
{
    ChewingContext *ctx;
    int ret;
    char *buf;
    int len;

    clean_userphrase();

    ctx = chewing_new();
    start_testcase(ctx, fd);
    chewing_set_phraseChoiceRearward(ctx, 1);

    type_keystroke_by_string(ctx, "hk4g4`31u6vu84" /* 測試，一下 */ );

    type_keystroke_by_string(ctx, "<EE><H><D>");
    ret = chewing_cand_TotalChoice(ctx);
    ok(ret > 0, "chewing_cand_TotalChoice() returns `%d' shall greater than 0 at pos `%d'", ret, 0);
    chewing_cand_Enumerate(ctx);
    buf = chewing_cand_String(ctx);
    len = ueStrLen(buf);
    ok(len == 1, "candidate `%s' length `%d' shall be `%d' at pos `%d'", buf, len, 1, 0);
    chewing_free(buf);

    type_keystroke_by_string(ctx, "<EE><H><R><D>");
    ret = chewing_cand_TotalChoice(ctx);
    ok(ret > 0, "chewing_cand_TotalChoice() returns `%d' shall greater than 0 at pos `%d'", ret, 1);
    chewing_cand_Enumerate(ctx);
    buf = chewing_cand_String(ctx);
    len = ueStrLen(buf);
    ok(len == 2, "candidate `%s' length `%d' shall be `%d' at pos `%d'", buf, len, 2, 1);
    chewing_free(buf);

    type_keystroke_by_string(ctx, "<EE><H><R><R><D>");
    ret = chewing_cand_TotalChoice(ctx);
    ok(ret > 0, "chewing_cand_TotalChoice() returns `%d' shall greater than 0 at pos `%d'", ret, 2);
    chewing_cand_Enumerate(ctx);
    buf = chewing_cand_String(ctx);
    len = ueStrLen(buf);
    ok(len == 1, "candidate `%s' length `%d' shall be `%d' at pos `%d'", buf, len, 1, 2);
    chewing_free(buf);

    type_keystroke_by_string(ctx, "<EE><H><R><R><R><D>");
    ret = chewing_cand_TotalChoice(ctx);
    ok(ret > 0, "chewing_cand_TotalChoice() returns `%d' shall greater than 0 at pos `%d'", ret, 3);
    chewing_cand_Enumerate(ctx);
    buf = chewing_cand_String(ctx);
    len = ueStrLen(buf);
    ok(len == 1, "candidate `%s' length `%d' shall be `%d' at pos `%d'", buf, len, 1, 3);
    chewing_free(buf);

    type_keystroke_by_string(ctx, "<EE><H><R><R><R><R><D>");
    ret = chewing_cand_TotalChoice(ctx);
    ok(ret > 0, "chewing_cand_TotalChoice() returns `%d' shall greater than 0 at pos `%d'", ret, 4);
    chewing_cand_Enumerate(ctx);
    buf = chewing_cand_String(ctx);
    len = ueStrLen(buf);
    ok(len == 2, "candidate `%s' length `%d' shall be `%d' at pos `%d'", buf, len, 2, 4);
    chewing_free(buf);

    chewing_delete(ctx);
}

void test_select_candidate_no_rearward_start_with_symbol()
{
    ChewingContext *ctx;
    int ret;
    char *buf;
    int len;

    clean_userphrase();

    ctx = chewing_new();
    start_testcase(ctx, fd);

    type_keystroke_by_string(ctx, "`31hk4g4" /* ，測試 */ );

    type_keystroke_by_string(ctx, "<EE><H><D>");
    ret = chewing_cand_TotalChoice(ctx);
    ok(ret > 0, "chewing_cand_TotalChoice() returns `%d' shall greater than 0 at pos `%d'", ret, 0);
    chewing_cand_Enumerate(ctx);
    buf = chewing_cand_String(ctx);
    len = ueStrLen(buf);
    ok(len == 1, "candidate `%s' length `%d' shall be `%d' at pos `%d'", buf, len, 1, 0);
    chewing_free(buf);

    type_keystroke_by_string(ctx, "<EE><H><R><D>");
    ret = chewing_cand_TotalChoice(ctx);
    ok(ret > 0, "chewing_cand_TotalChoice() returns `%d' shall greater than 0 at pos `%d'", ret, 1);
    chewing_cand_Enumerate(ctx);
    buf = chewing_cand_String(ctx);
    len = ueStrLen(buf);
    ok(len == 2, "candidate `%s' length `%d' shall be `%d' at pos `%d'", buf, len, 2, 1);
    chewing_free(buf);

    type_keystroke_by_string(ctx, "<EE><H><R><R><D>");
    ret = chewing_cand_TotalChoice(ctx);
    ok(ret > 0, "chewing_cand_TotalChoice() returns `%d' shall greater than 0 at pos `%d'", ret, 2);
    chewing_cand_Enumerate(ctx);
    buf = chewing_cand_String(ctx);
    len = ueStrLen(buf);
    ok(len == 1, "candidate `%s' length `%d' shall be `%d' at pos `%d'", buf, len, 1, 2);
    chewing_free(buf);

    chewing_delete(ctx);
}

void test_select_candidate_rearward_start_with_symbol()
{
    ChewingContext *ctx;
    int ret;
    char *buf;
    int len;

    clean_userphrase();

    ctx = chewing_new();
    start_testcase(ctx, fd);
    chewing_set_phraseChoiceRearward(ctx, 1);

    type_keystroke_by_string(ctx, "`31hk4g4" /* ，測試 */ );

    type_keystroke_by_string(ctx, "<EE><H><D>");
    ret = chewing_cand_TotalChoice(ctx);
    ok(ret > 0, "chewing_cand_TotalChoice() returns `%d' shall greater than 0 at pos `%d'", ret, 0);
    chewing_cand_Enumerate(ctx);
    buf = chewing_cand_String(ctx);
    len = ueStrLen(buf);
    ok(len == 1, "candidate `%s' length `%d' shall be `%d' at pos `%d'", buf, len, 1, 0);
    chewing_free(buf);

    type_keystroke_by_string(ctx, "<EE><H><R><D>");
    ret = chewing_cand_TotalChoice(ctx);
    ok(ret > 0, "chewing_cand_TotalChoice() returns `%d' shall greater than 0 at pos `%d'", ret, 1);
    chewing_cand_Enumerate(ctx);
    buf = chewing_cand_String(ctx);
    len = ueStrLen(buf);
    ok(len == 1, "candidate `%s' length `%d' shall be `%d' at pos `%d'", buf, len, 1, 1);
    chewing_free(buf);

    type_keystroke_by_string(ctx, "<EE><H><R><R><D>");
    ret = chewing_cand_TotalChoice(ctx);
    ok(ret > 0, "chewing_cand_TotalChoice() returns `%d' shall greater than 0 at pos `%d'", ret, 2);
    chewing_cand_Enumerate(ctx);
    buf = chewing_cand_String(ctx);
    len = ueStrLen(buf);
    ok(len == 2, "candidate `%s' length `%d' shall be `%d' at pos `%d'", buf, len, 2, 2);
    chewing_free(buf);

    chewing_delete(ctx);
}

void test_del_bopomofo_as_mode_switch()
{
    ChewingContext *ctx;

    clean_userphrase();

    ctx = chewing_new();
    start_testcase(ctx, fd);

    type_keystroke_by_string(ctx, "2k");        /* ㄉㄜ */
    ok_bopomofo_buffer(ctx, "\xe3\x84\x89\xe3\x84\x9c" /* ㄉㄜ */ );
    ok_preedit_buffer(ctx, "");
    chewing_set_ChiEngMode(ctx, SYMBOL_MODE);
    ok_bopomofo_buffer(ctx, "");
    ok_preedit_buffer(ctx, "");

    chewing_set_ChiEngMode(ctx, CHINESE_MODE);

    type_keystroke_by_string(ctx, "ji");        /* ㄨㄛ */
    ok_bopomofo_buffer(ctx, "\xe3\x84\xa8\xe3\x84\x9b" /* ㄨㄛ */ );
    ok_preedit_buffer(ctx, "");
    chewing_set_ChiEngMode(ctx, SYMBOL_MODE);
    ok_bopomofo_buffer(ctx, "");
    ok_preedit_buffer(ctx, "");

    chewing_delete(ctx);
}

void test_select_candidate_4_bytes_utf8()
{
    ChewingContext *ctx;

    clean_userphrase();

    ctx = chewing_new();
    start_testcase(ctx, fd);
    chewing_set_maxChiSymbolLen(ctx, 16);
    chewing_set_phraseChoiceRearward(ctx, 1);
    chewing_set_autoShiftCur(ctx, 1);

    type_keystroke_by_string(ctx, "2k62k6");    /* ㄉㄜˊ ㄉㄜˊ */
    ok_preedit_buffer(ctx, "\xE5\xBE\x97\xE5\xBE\x97" /* 得得 */ );

    type_keystroke_by_string(ctx, "<H>");

    type_keystroke_by_string(ctx, "<D>8");
    ok_preedit_buffer(ctx, "\xF0\xA2\x94\xA8\xE5\xBE\x97" /* 𢔨得 */ );

    type_keystroke_by_string(ctx, "<D>8");

    ok_preedit_buffer(ctx, "\xF0\xA2\x94\xA8\xF0\xA2\x94\xA8" /* 𢔨𢔨 */ );

    chewing_delete(ctx);
}

void test_select_candidate_in_middle_no_reaward()
{
    ChewingContext *ctx;
    int ret;
    const char *cand;

    clean_userphrase();

    ctx = chewing_new();
    start_testcase(ctx, fd);

    type_keystroke_by_string(ctx, "hk4g4u6<L><L>" /* 測試儀*/);

    ret = chewing_cand_open(ctx);
    ok(ret == 0, "chewing_cand_open return %d shall be %d", ret, 0);

    cand = chewing_cand_string_by_index_static(ctx, 0);
    ok(strcmp(cand, "\xE9\x81\xA9\xE5\xAE\x9C") == 0, "first candidate `%s' shall be `%s'", cand, "\xE9\x81\xA9\xE5\xAE\x9C" /* 適宜 */);

    ret = chewing_cand_list_next(ctx);
    ok(ret == 0, "chewing_cand_list_next return %d shall be %d", ret, 0);

    cand = chewing_cand_string_by_index_static(ctx, 0);
    ok(strcmp(cand, "\xE5\xB8\x82") == 0, "first candidate `%s' shall be `%s'", cand, "\xE5\xB8\x82" /* 市 */);

    chewing_delete(ctx);
}

void test_select_candidate_in_middle_reaward()
{
    ChewingContext *ctx;
    int ret;
    const char *cand;

    clean_userphrase();

    ctx = chewing_new();
    start_testcase(ctx, fd);
    chewing_set_phraseChoiceRearward(ctx, 1);

    type_keystroke_by_string(ctx, "hk4g4u6<L><L>" /* 測試儀*/);

    ret = chewing_cand_open(ctx);
    ok(ret == 0, "chewing_cand_open return %d shall be %d", ret, 0);

    cand = chewing_cand_string_by_index_static(ctx, 0);
    ok(strcmp(cand, "\xE6\xB8\xAC\xE8\xA9\xA6") == 0, "first candidate `%s' shall be `%s'", cand, "\xE6\xB8\xAC\xE8\xA9\xA6" /* 測試 */);

    ret = chewing_cand_list_next(ctx);
    ok(ret == 0, "chewing_cand_list_next return %d shall be %d", ret, 0);

    cand = chewing_cand_string_by_index_static(ctx, 0);
    ok(strcmp(cand, "\xE5\xB8\x82") == 0, "first candidate `%s' shall be `%s'", cand, "\xE5\xB8\x82" /* 市 */);

    chewing_delete(ctx);
}

void test_select_candidate()
{
    test_select_candidate_no_rearward();
    test_select_candidate_rearward();
    test_select_candidate_no_rearward_with_symbol();
    test_select_candidate_rearward_with_symbol();
    test_select_candidate_no_rearward_start_with_symbol();
    test_select_candidate_rearward_start_with_symbol();
    test_select_candidate_4_bytes_utf8();
    test_del_bopomofo_as_mode_switch();
    test_select_candidate_in_middle_no_reaward();
    test_select_candidate_in_middle_reaward();
}

void test_Esc_not_entering_chewing()
{
    ChewingContext *ctx;

    ctx = chewing_new();
    start_testcase(ctx, fd);
    type_keystroke_by_string(ctx, "<EE>");
    ok_keystroke_rtn(ctx, KEYSTROKE_IGNORE);

    chewing_delete(ctx);
}

void test_Esc_in_select()
{
    ChewingContext *ctx;

    ctx = chewing_new();
    start_testcase(ctx, fd);
    type_keystroke_by_string(ctx, "`<EE>");
    ok_candidate(ctx, NULL, 0);

    chewing_delete(ctx);
}

void test_Esc_entering_bopomofo()
{
    ChewingContext *ctx;

    ctx = chewing_new();
    start_testcase(ctx, fd);
    type_keystroke_by_string(ctx, "hk<EE>");
    ok_bopomofo_buffer(ctx, "");

    chewing_delete(ctx);
}

void test_Esc_escCleanAllBuf()
{
    ChewingContext *ctx;

    ctx = chewing_new();
    start_testcase(ctx, fd);
    chewing_set_escCleanAllBuf(ctx, 1);

    type_keystroke_by_string(ctx, "hk4g4<EE>");
    ok_bopomofo_buffer(ctx, "");
    ok_preedit_buffer(ctx, "");
    ok_commit_buffer(ctx, "");

    chewing_delete(ctx);
}

void test_Esc()
{
    test_Esc_not_entering_chewing();
    test_Esc_in_select();
    test_Esc_entering_bopomofo();
    test_Esc_escCleanAllBuf();
}

void test_Del_not_entering_chewing()
{
    ChewingContext *ctx;

    ctx = chewing_new();
    start_testcase(ctx, fd);
    type_keystroke_by_string(ctx, "<DC>");
    ok_keystroke_rtn(ctx, KEYSTROKE_IGNORE);

    chewing_delete(ctx);
}

void test_Del_in_select()
{
    ChewingContext *ctx;

    ctx = chewing_new();
    start_testcase(ctx, fd);
    type_keystroke_by_string(ctx, "`<DC>");
    ok_keystroke_rtn(ctx, KEYSTROKE_ABSORB);    /* XXX: shall be ignore? */

    chewing_delete(ctx);
}

void test_Del_word()
{
    ChewingContext *ctx;

    ctx = chewing_new();
    start_testcase(ctx, fd);
    chewing_set_maxChiSymbolLen(ctx, 16);

    type_keystroke_by_string(ctx, "hk4u g4<L><L><DC><E>");
    ok_commit_buffer(ctx, "\xE6\xB8\xAC\xE8\xA9\xA6" /* 測試 */ );

    chewing_delete(ctx);
}

void test_Del()
{
    test_Del_not_entering_chewing();
    test_Del_in_select();
    test_Del_word();
}

void test_Backspace_not_entering_chewing()
{
    ChewingContext *ctx;

    ctx = chewing_new();
    start_testcase(ctx, fd);
    type_keystroke_by_string(ctx, "<B>");
    ok_keystroke_rtn(ctx, KEYSTROKE_IGNORE);

    chewing_delete(ctx);
}

void test_Backspace_in_select()
{
    ChewingContext *ctx;
    int ret;

    ctx = chewing_new();
    start_testcase(ctx, fd);

    type_keystroke_by_string(ctx, "`<B>");
    ok_candidate(ctx, NULL, 0);

    type_keystroke_by_string(ctx, "hk4");
    ret = chewing_cand_TotalChoice(ctx);
    ok(ret == 0, "chewing_cand_TotalChoice() returns `%d' shall be `%d'", ret, 0);

    type_keystroke_by_string(ctx, "<D>");
    ret = chewing_cand_TotalChoice(ctx);
    ok(ret > 0, "chewing_cand_TotalChoice() returns `%d' shall be greater than `%d'", ret, 0);

    type_keystroke_by_string(ctx, "<B>");
    ret = chewing_cand_TotalChoice(ctx);
    ok(ret == 0, "chewing_cand_TotalChoice() returns `%d' shall be `%d'", ret, 0);

    chewing_delete(ctx);
}

void test_Backspace_remove_bopomofo()
{
    ChewingContext *ctx;

    ctx = chewing_new();
    start_testcase(ctx, fd);
    type_keystroke_by_string(ctx, "hk<B>");
    ok_bopomofo_buffer(ctx, "\xE3\x84\x98" /* ㄘ */ );

    chewing_delete(ctx);
}

void test_Backspace_word()
{
    ChewingContext *ctx;

    ctx = chewing_new();
    start_testcase(ctx, fd);
    chewing_set_maxChiSymbolLen(ctx, 16);

    type_keystroke_by_string(ctx, "hk4u g4<L><B><E>");
    ok_commit_buffer(ctx, "\xE6\xB8\xAC\xE8\xA9\xA6" /* 測試 */ );

    chewing_delete(ctx);
}

void test_Backspace()
{
    test_Backspace_not_entering_chewing();
    test_Backspace_in_select();
    test_Backspace_remove_bopomofo();
    test_Backspace_word();
}

void test_Up_close_candidate_window_word()
{
    ChewingContext *ctx;
    int ret;

    ctx = chewing_new();
    start_testcase(ctx, fd);

    type_keystroke_by_string(ctx, "hk4");
    ret = chewing_cand_TotalChoice(ctx);
    ok(ret == 0, "chewing_cand_TotalChoice() returns `%d' shall be `%d'", ret, 0);

    type_keystroke_by_string(ctx, "<D>");
    ret = chewing_cand_TotalChoice(ctx);
    ok(ret > 0, "chewing_cand_TotalChoice() returns `%d' shall be greater than `%d'", ret, 0);

    type_keystroke_by_string(ctx, "<U>");
    ret = chewing_cand_TotalChoice(ctx);
    ok(ret == 0, "chewing_cand_TotalChoice() returns `%d' shall be `%d'", ret, 0);

    chewing_delete(ctx);
}

void test_Up_close_candidate_window_symbol()
{
    ChewingContext *ctx;
    int ret;

    ctx = chewing_new();
    start_testcase(ctx, fd);

    type_keystroke_by_string(ctx, "_");
    ret = chewing_cand_TotalChoice(ctx);
    ok(ret == 0, "chewing_cand_TotalChoice() returns `%d' shall be `%d'", ret, 0);

    type_keystroke_by_string(ctx, "<D>");
    ret = chewing_cand_TotalChoice(ctx);
    ok(ret > 0, "chewing_cand_TotalChoice() returns `%d' shall be greater than `%d'", ret, 0);

    type_keystroke_by_string(ctx, "<U>");
    ret = chewing_cand_TotalChoice(ctx);
    ok(ret == 0, "chewing_cand_TotalChoice() returns `%d' shall be `%d'", ret, 0);

    chewing_delete(ctx);
}

void test_Up_not_entering_chewing()
{
    ChewingContext *ctx;

    ctx = chewing_new();
    start_testcase(ctx, fd);
    type_keystroke_by_string(ctx, "<U>");
    ok_keystroke_rtn(ctx, KEYSTROKE_IGNORE);

    chewing_delete(ctx);
}

void test_Up()
{
    test_Up_close_candidate_window_word();
    test_Up_close_candidate_window_symbol();
    test_Up_not_entering_chewing();
}

void test_Down_open_candidate_window()
{
    ChewingContext *ctx;
    int ret;

    ctx = chewing_new();
    start_testcase(ctx, fd);

    type_keystroke_by_string(ctx, "hk4");
    ret = chewing_cand_TotalChoice(ctx);
    ok(ret == 0, "chewing_cand_TotalChoice() returns `%d' shall be `%d'", ret, 0);

    type_keystroke_by_string(ctx, "<D>");
    ret = chewing_cand_TotalChoice(ctx);
    ok(ret > 0, "chewing_cand_TotalChoice() returns `%d' shall be greater than `%d'", ret, 0);

    type_keystroke_by_string(ctx, "3");
    ret = chewing_cand_TotalChoice(ctx);
    ok(ret == 0, "chewing_cand_TotalChoice() returns `%d' shall be `%d'", ret, 0);
    ok_preedit_buffer(ctx, "\xE6\xB8\xAC" /* 測 */ );

    chewing_delete(ctx);
}

void test_Down_reopen_symbol_candidate()
{
    ChewingContext *ctx;

    ctx = chewing_new();
    start_testcase(ctx, fd);

    type_keystroke_by_string(ctx, "_<D><R>");
    ok(chewing_cand_CurrentPage(ctx) == 1, "current page shall be 1");

    type_keystroke_by_string(ctx, "<D>");
    ok(chewing_cand_CurrentPage(ctx) == 0, "current page shall be 0");

    chewing_delete(ctx);
}

void test_Down_not_entering_chewing()
{
    ChewingContext *ctx;

    ctx = chewing_new();
    start_testcase(ctx, fd);
    type_keystroke_by_string(ctx, "<D>");
    ok_keystroke_rtn(ctx, KEYSTROKE_IGNORE);

    chewing_delete(ctx);
}

void test_Down()
{
    test_Down_open_candidate_window();
    test_Down_not_entering_chewing();
}

void test_Tab_insert_breakpoint_between_word()
{
    ChewingContext *ctx;
    IntervalType it;

    ctx = chewing_new();
    start_testcase(ctx, fd);
    chewing_set_maxChiSymbolLen(ctx, 16);

    type_keystroke_by_string(ctx, "hk4g4<L>");
    chewing_interval_Enumerate(ctx);

    ok(chewing_interval_hasNext(ctx) == 1, "shall have next interval");
    chewing_interval_Get(ctx, &it);
    ok(it.from == 0 && it.to == 2, "interval (%d, %d) shall be (0, 2)", it.from, it.to);

    ok(chewing_interval_hasNext(ctx) == 0, "shall not have next interval");

    /* inserts a breakpoint between 測 and 試 */
    type_keystroke_by_string(ctx, "<T>");
    chewing_interval_Enumerate(ctx);

    ok(chewing_interval_hasNext(ctx) == 1, "shall have next interval");
    chewing_interval_Get(ctx, &it);
    ok(it.from == 0 && it.to == 1, "interval (%d, %d) shall be (0, 1)", it.from, it.to);

    ok(chewing_interval_hasNext(ctx) == 1, "shall have next interval");
    chewing_interval_Get(ctx, &it);
    ok(it.from == 1 && it.to == 2, "interval (%d, %d) shall be (1, 2)", it.from, it.to);

    ok(chewing_interval_hasNext(ctx) == 0, "shall not have next interval");

    chewing_delete(ctx);
}

void test_Tab_connect_word()
{
    ChewingContext *ctx;
    IntervalType it;

    ctx = chewing_new();
    start_testcase(ctx, fd);
    chewing_set_maxChiSymbolLen(ctx, 16);

    type_keystroke_by_string(ctx, "u -4<L>");
    chewing_interval_Enumerate(ctx);

    ok(chewing_interval_hasNext(ctx) == 1, "shall have next interval");
    chewing_interval_Get(ctx, &it);
    ok(it.from == 0 && it.to == 1, "interval (%d, %d) shall be (0, 1)", it.from, it.to);

    ok(chewing_interval_hasNext(ctx) == 1, "shall have next interval");
    chewing_interval_Get(ctx, &it);
    ok(it.from == 1 && it.to == 2, "interval (%d, %d) shall be (1, 2)", it.from, it.to);

    ok(chewing_interval_hasNext(ctx) == 0, "shall not have next interval");

    /* connect 一 and 二 */
    type_keystroke_by_string(ctx, "<T>");
    chewing_interval_Enumerate(ctx);

    ok(chewing_interval_hasNext(ctx) == 1, "shall have next interval");
    chewing_interval_Get(ctx, &it);
    ok(it.from == 0 && it.to == 2, "interval (%d, %d) shall be (0, 2)", it.from, it.to);

    ok(chewing_interval_hasNext(ctx) == 0, "shall not have next interval");

    chewing_delete(ctx);
}

void test_Tab_at_the_end()
{
    ChewingContext *ctx;

    ctx = chewing_new();
    start_testcase(ctx, fd);

    type_keystroke_by_string(ctx, "hk4g4u6vu84");
    ok_preedit_buffer(ctx, "\xE6\xB8\xAC\xE8\xA9\xA6\xE4\xB8\x80\xE4\xB8\x8B" /* 測試一下 */ );

    type_keystroke_by_string(ctx, "<T>");
    ok_preedit_buffer(ctx, "\xE6\xB8\xAC\xE8\xA9\xA6\xE5\x84\x80\xE4\xB8\x8B" /* 測試儀下 */ );

    type_keystroke_by_string(ctx, "<T>");
    ok_preedit_buffer(ctx, "\xE6\xB8\xAC\xE8\xA9\xA6\xE4\xB8\x80\xE4\xB8\x8B" /* 測試一下 */ );

    chewing_delete(ctx);
}

void test_Tab()
{
    test_Tab_insert_breakpoint_between_word();
    test_Tab_connect_word();
    test_Tab_at_the_end();
}

void test_DblTab()
{
    /* FIXME: Implement this. */
}

void test_Capslock()
{
    ChewingContext *ctx;
    int mode;

    ctx = chewing_new();
    start_testcase(ctx, fd);

    mode = chewing_get_ChiEngMode(ctx);
    ok(mode == CHINESE_MODE, "mode shall be CHINESE_MODE");

    type_keystroke_by_string(ctx, "ji");        /* ㄨㄛ */
    ok_bopomofo_buffer(ctx, "\xe3\x84\xa8\xe3\x84\x9b" /* ㄨㄛ */ );
    ok_preedit_buffer(ctx, "");
    ok_commit_buffer(ctx, "");

    type_keystroke_by_string(ctx, "<CB>");

    mode = chewing_get_ChiEngMode(ctx);
    ok(mode == SYMBOL_MODE, "mode shall change to SYMBOL_MODE");

    ok_bopomofo_buffer(ctx, "");
    ok_preedit_buffer(ctx, "");
    ok_commit_buffer(ctx, "");

    chewing_delete(ctx);
}

void test_Home()
{
    ChewingContext *ctx;
    int cursor;

    ctx = chewing_new();
    start_testcase(ctx, fd);
    chewing_set_maxChiSymbolLen(ctx, 16);

    type_keystroke_by_string(ctx, "hk4g4");
    cursor = chewing_cursor_Current(ctx);
    ok(cursor == 2, "cursor `%d' shall be 2", cursor);

    type_keystroke_by_string(ctx, "<H>");
    cursor = chewing_cursor_Current(ctx);
    ok(cursor == 0, "cursor `%d' shall be 0", cursor);

    chewing_delete(ctx);
}

void test_End()
{
    ChewingContext *ctx;
    int cursor;

    ctx = chewing_new();
    start_testcase(ctx, fd);
    chewing_set_maxChiSymbolLen(ctx, 16);

    type_keystroke_by_string(ctx, "hk4g4<L><L>");
    cursor = chewing_cursor_Current(ctx);
    ok(cursor == 0, "cursor `%d' shall be 0", cursor);

    type_keystroke_by_string(ctx, "<EN>");
    cursor = chewing_cursor_Current(ctx);
    ok(cursor == 2, "cursor `%d' shall be 2", cursor);

    chewing_delete(ctx);
}

void test_PageUp_not_entering_chewing()
{
    ChewingContext *ctx;
    int cursor;

    ctx = chewing_new();
    start_testcase(ctx, fd);
    chewing_set_maxChiSymbolLen(ctx, 16);

    type_keystroke_by_string(ctx, "hk4g4<L><L>");
    cursor = chewing_cursor_Current(ctx);
    ok(cursor == 0, "cursor `%d' shall be 0", cursor);

    type_keystroke_by_string(ctx, "<PU>");
    cursor = chewing_cursor_Current(ctx);
    ok(cursor == 2, "cursor `%d' shall be 2", cursor);

    chewing_delete(ctx);
}

void test_PageUp_in_select()
{
    ChewingContext *ctx;

    ctx = chewing_new();
    start_testcase(ctx, fd);

    chewing_set_candPerPage(ctx, 10);

    type_keystroke_by_string(ctx, "hk4<D>");
    ok(chewing_cand_TotalPage(ctx) == 3, "total page shall be 3");
    ok(chewing_cand_CurrentPage(ctx) == 0, "current page shall be 0");

    type_keystroke_by_string(ctx, "<PU>");      /* rollover */
    ok(chewing_cand_CurrentPage(ctx) == 2, "current page shall be 2");

    type_keystroke_by_string(ctx, "<PU>");      /* to previous page */
    ok(chewing_cand_CurrentPage(ctx) == 1, "current page shall be 1");

    chewing_delete(ctx);
}

void test_PageUp()
{
    test_PageUp_not_entering_chewing();
    test_PageUp_in_select();
}

void test_PageDown_not_entering_chewing()
{
    ChewingContext *ctx;
    int cursor;

    ctx = chewing_new();
    start_testcase(ctx, fd);
    chewing_set_maxChiSymbolLen(ctx, 16);

    type_keystroke_by_string(ctx, "hk4g4<L><L>");
    cursor = chewing_cursor_Current(ctx);
    ok(cursor == 0, "cursor `%d' shall be 0", cursor);

    type_keystroke_by_string(ctx, "<PD>");
    cursor = chewing_cursor_Current(ctx);
    ok(cursor == 2, "cursor `%d' shall be 2", cursor);

    chewing_delete(ctx);
}

void test_PageDown_in_select()
{
    ChewingContext *ctx;

    ctx = chewing_new();
    start_testcase(ctx, fd);

    chewing_set_candPerPage(ctx, 10);

    type_keystroke_by_string(ctx, "hk4<D>");
    ok(chewing_cand_TotalPage(ctx) == 3, "total page shall be 3");
    ok(chewing_cand_CurrentPage(ctx) == 0, "current page shall be 0");

    type_keystroke_by_string(ctx, "<PD>");
    ok(chewing_cand_CurrentPage(ctx) == 1, "current page shall be 1");

    type_keystroke_by_string(ctx, "<PD><PD>");  /* rollover */
    ok(chewing_cand_CurrentPage(ctx) == 0, "current page shall be 0");

    chewing_delete(ctx);
}

void test_PageDown()
{
    test_PageDown_not_entering_chewing();
    test_PageDown_in_select();
}

void test_ShiftSpace()
{
    ChewingContext *ctx;
    int mode;

    ctx = chewing_new();
    start_testcase(ctx, fd);

    mode = chewing_get_ShapeMode(ctx);
    ok(mode == HALFSHAPE_MODE, "mode shall be HALFSHAPE_MODE");

    type_keystroke_by_string(ctx, "<SS>");
    mode = chewing_get_ShapeMode(ctx);
    ok(mode == FULLSHAPE_MODE, "mode shall be FULLSHAPE_MODE");

    type_keystroke_by_string(ctx, " ");
    ok_commit_buffer(ctx, "\xE3\x80\x80"); /* Fullshape Space (U+3000) */

    chewing_set_ChiEngMode(ctx, SYMBOL_MODE);
    type_keystroke_by_string(ctx, "a");
    ok_commit_buffer(ctx, "\xEF\xBD\x81"); /* Fullshape a */

    chewing_delete(ctx);
}

void test_Numlock_numeric_input()
{
    const TestData NUMLOCK_INPUT[] = {
        {"<N0>", "0"},
        {"<N1>", "1"},
        {"<N2>", "2"},
        {"<N3>", "3"},
        {"<N4>", "4"},
        {"<N5>", "5"},
        {"<N6>", "6"},
        {"<N7>", "7"},
        {"<N8>", "8"},
        {"<N9>", "9"},
        {"<N+>", "+"},
        {"<N->", "-"},
        {"<N*>", "*"},
        {"<N/>", "/"},
        {"<N.>", "."},
    };
    size_t i;
    ChewingContext *ctx;

    ctx = chewing_new();
    start_testcase(ctx, fd);
    chewing_set_maxChiSymbolLen(ctx, 16);

    for (i = 0; i < ARRAY_SIZE(NUMLOCK_INPUT); ++i) {
        type_keystroke_by_string(ctx, NUMLOCK_INPUT[i].token);
        ok_commit_buffer(ctx, NUMLOCK_INPUT[i].expected);
    }

    chewing_delete(ctx);
}

void test_Numlock_select_candidate()
{
    const TestData NUMLOCK_SELECT[] = {
        {"hk4<D><N1><E>", "\xE5\x86\x8A" /* 冊 */ },
        {"hk4<D><N2><E>", "\xE7\xAD\x96" /* 策 */ },
        {"hk4<D><N3><E>", "\xE6\xB8\xAC" /* 測 */ },
        {"hk4<D><N4><E>", "\xE5\x81\xB4" /* 側 */ },
        {"hk4<D><N5><E>", "\xE5\xBB\x81" /* 廁 */ },
        {"hk4<D><N6><E>", "\xE6\x83\xBB" /* 惻 */ },
        {"hk4<D><N7><E>", "\xE7\xAD\xB4" /* 筴 */ },
        {"hk4<D><N8><E>", "\xE7\x95\x9F" /* 畟 */ },
        {"hk4<D><N9><E>", "\xE8\x8C\xA6" /* 茦 */ },
        {"hk4<D><N0><E>", "\xE7\xB2\xA3" /* 粣 */ },
    };
    size_t i;
    ChewingContext *ctx;

    ctx = chewing_new();
    start_testcase(ctx, fd);
    chewing_set_maxChiSymbolLen(ctx, 16);

    for (i = 0; i < ARRAY_SIZE(NUMLOCK_SELECT); ++i) {
        type_keystroke_by_string(ctx, NUMLOCK_SELECT[i].token);
        ok_commit_buffer(ctx, NUMLOCK_SELECT[i].expected);
    }

    chewing_delete(ctx);
}

void test_Numlock()
{
    test_Numlock_numeric_input();
    test_Numlock_select_candidate();
}

void test_Space_selection_word()
{
    ChewingContext *ctx;
    char *buf;
    int len;

    clean_userphrase();

    ctx = chewing_new();
    start_testcase(ctx, fd);
    chewing_set_spaceAsSelection(ctx, 1);

    type_keystroke_by_string(ctx, "hk4g4<H>" /* 測試 */ );

    type_keystroke_by_string(ctx, " "); /* open candidate window */

    chewing_cand_Enumerate(ctx);
    buf = chewing_cand_String(ctx);
    len = ueStrLen(buf);
    ok(len == 2, "candidate `%s' length `%d' shall be `%d'", buf, len, 2);
    chewing_free(buf);

    type_keystroke_by_string(ctx, " "); /* next candidate list */

    chewing_cand_Enumerate(ctx);
    buf = chewing_cand_String(ctx);
    len = ueStrLen(buf);
    ok(len == 1, "candidate `%s' length `%d' shall be `%d'", buf, len, 1);
    chewing_free(buf);

    type_keystroke_by_string(ctx, " "); /* next page */
    ok(chewing_cand_CurrentPage(ctx) == 1, "current page shall be 1");

    chewing_delete(ctx);
}

void test_Space_selection_symbol()
{
    const char CAND_1[] = "\xE2\x80\xA6" /* … */ ;
    const char CAND_2[] = "\xE9\x9B\x99\xE7\xB7\x9A\xE6\xA1\x86" /* 雙線框 */ ;

    ChewingContext *ctx;
    const char *const_buf;

    clean_userphrase();

    ctx = chewing_new();
    start_testcase(ctx, fd);
    chewing_set_spaceAsSelection(ctx, 1);

    type_keystroke_by_string(ctx, "`");
    chewing_cand_Enumerate(ctx);
    const_buf = chewing_cand_String_static(ctx);
    ok(strcmp(const_buf, CAND_1) == 0, "first candidate list head `%s' shall be `%s'", const_buf, CAND_1);

    type_keystroke_by_string(ctx, " ");
    chewing_cand_Enumerate(ctx);
    const_buf = chewing_cand_String_static(ctx);
    ok(strcmp(const_buf, CAND_2) == 0, "second candidate list head `%s' shall be `%s'", const_buf, CAND_2);

    /* rollover */
    type_keystroke_by_string(ctx, " ");
    chewing_cand_Enumerate(ctx);
    const_buf = chewing_cand_String_static(ctx);
    ok(strcmp(const_buf, CAND_1) == 0, "first candidate list head `%s' shall be `%s'", const_buf, CAND_1);

    chewing_delete(ctx);
}

void test_Space()
{
    test_Space_selection_word();
    test_Space_selection_symbol();
}

void test_get_phoneSeq()
{
    static const struct {
        char *token;
        unsigned short phone[5];
    } DATA[] = {
        {
            "hk4g4", {10268, 8708, 0}
        },
        {
            "hk4g4`31hk4g4", {10268, 8708, 10268, 8708, 0}
        },
        {
            "`31`31", {0}
        },
    };
    ChewingContext *ctx;
    size_t i;
    int expected_len;
    int len;
    unsigned short *phone;

    ctx = chewing_new();
    start_testcase(ctx, fd);
    chewing_set_maxChiSymbolLen(ctx, 16);

    for (i = 0; i < ARRAY_SIZE(DATA); ++i) {
        chewing_Reset(ctx);
        type_keystroke_by_string(ctx, DATA[i].token);

        expected_len = 0;
        while (DATA[i].phone[expected_len] != 0)
            ++expected_len;
        len = chewing_get_phoneSeqLen(ctx);
        ok(len == expected_len, "phoneSeqLen `%d' shall be `%d'", len, expected_len);

        phone = chewing_get_phoneSeq(ctx);
        ok(memcmp(phone, DATA[i].phone, sizeof(phone[0]) * expected_len) == 0, "phoneSeq shall be expected value");
        chewing_free(phone);
    }

    chewing_delete(ctx);
}

void test_bopomofo_buffer()
{
    ChewingContext *ctx;

    ctx = chewing_new();
    start_testcase(ctx, fd);

    type_keystroke_by_string(ctx, "1ul");
    ok_bopomofo_buffer(ctx, "\xE3\x84\x85\xE3\x84\xA7\xE3\x84\xA0" /* ㄅㄧㄠ */ );

    type_keystroke_by_string(ctx, " ");
    ok_bopomofo_buffer(ctx, "");

    type_keystroke_by_string(ctx, "ul");
    ok_bopomofo_buffer(ctx, "\xE3\x84\xA7\xE3\x84\xA0" /* ㄧㄠ */ );

    type_keystroke_by_string(ctx, " ");
    ok_bopomofo_buffer(ctx, "");

    type_keystroke_by_string(ctx, "3");
    ok_bopomofo_buffer(ctx, "\xCB\x87" /* ˇ */ );

    type_keystroke_by_string(ctx, " ");
    ok_bopomofo_buffer(ctx, "");

    chewing_delete(ctx);
}

void test_longest_phrase()
{
    ChewingContext *ctx;
    IntervalType it;

    ctx = chewing_new();
    start_testcase(ctx, fd);

    type_keystroke_by_string(ctx, "rup ji up6ji 1j4bj6y4ru32k7e.3ji "
                             /* ㄐㄧㄣ ㄨㄛ ㄧㄣˊ ㄨㄛ ㄅㄨˋ ㄖㄨˊ ㄗˋ ㄐㄧˇ ㄉㄜ˙ ㄍㄡˇ ㄨㄛ */
        );
    ok_preedit_buffer(ctx,
                      "\xE9\x87\x91\xE7\xAA\xA9\xE9\x8A\x80\xE7\xAA\xA9\xE4\xB8\x8D\xE5\xA6\x82\xE8\x87\xAA\xE5\xB7\xB1\xE7\x9A\x84\xE7\x8B\x97\xE7\xAA\xA9"
                      /* 金窩銀窩不如自己的狗窩 */ );

    chewing_interval_Enumerate(ctx);

    ok(chewing_interval_hasNext(ctx) == 1, "shall have next interval");
    chewing_interval_Get(ctx, &it);
    ok(it.from == 0 && it.to == 11, "interval (%d, %d) shall be (0, 11)", it.from, it.to);

    chewing_delete(ctx);
}

void test_auto_commit_phrase()
{
    ChewingContext *ctx;

    ctx = chewing_new();
    start_testcase(ctx, fd);
    chewing_set_maxChiSymbolLen(ctx, 3);

    type_keystroke_by_string(ctx, "hk4g4hk4g4" /* 測試測試 */ );
    ok_preedit_buffer(ctx, "\xE6\xB8\xAC\xE8\xA9\xA6" /* 測試 */ );
    ok_commit_buffer(ctx, "\xE6\xB8\xAC\xE8\xA9\xA6" /* 測試 */ );

    chewing_delete(ctx);
}

void test_auto_commit_symbol()
{
    ChewingContext *ctx;

    ctx = chewing_new();
    start_testcase(ctx, fd);
    chewing_set_maxChiSymbolLen(ctx, 2);

    type_keystroke_by_string(ctx, "`31hk4g4hk4g4" /* ，測試 */ );
    ok_preedit_buffer(ctx, "\xE6\xB8\xAC\xE8\xA9\xA6" /* 測試 */ );
    ok_commit_buffer(ctx, "\xEF\xBC\x8C" /* ， */ );

    chewing_delete(ctx);
}

void test_auto_commit()
{
    test_auto_commit_phrase();
    // FIXME: Auto commit for symbol seem to be incorrect.
    //test_auto_commit_symbol();
}

void test_interval()
{
    ChewingContext *ctx;
    IntervalType it;

    ctx = chewing_new();
    start_testcase(ctx, fd);

    type_keystroke_by_string(ctx, "`31hk4g4`31hk4g4`31" /* ，測試，測試， */ );

    ok_preedit_buffer(ctx, "\xEF\xBC\x8C\xE6\xB8\xAC\xE8\xA9\xA6\xEF\xBC\x8C\xE6\xB8\xAC\xE8\xA9\xA6\xEF\xBC\x8C"
                      /* ，測試，測試， */ );

    chewing_interval_Enumerate(ctx);

    ok(chewing_interval_hasNext(ctx) == 1, "shall have next interval");
    chewing_interval_Get(ctx, &it);
    ok(it.from == 1 && it.to == 3, "interval (%d, %d) shall be (1, 3)", it.from, it.to);

    ok(chewing_interval_hasNext(ctx) == 1, "shall have next interval");
    chewing_interval_Get(ctx, &it);
    ok(it.from == 4 && it.to == 6, "interval (%d, %d) shall be (4, 6)", it.from, it.to);

    ok(chewing_interval_hasNext(ctx) == 0, "shall not have next interval");

    chewing_delete(ctx);
}

void test_jk_selection()
{
#if 0
FIXME: libchewing is broken in this case
    ChewingContext *ctx;
    int ret;
    int i;
    const int EXPECT_CAND_LEN[] = { 1, 2, 1, 1, 2, 1, 1 };

    ctx = chewing_new();
    start_testcase(ctx, fd);

    type_keystroke_by_string(ctx, "`31hk4g4`31hk4g4`31" /* ，測試，測試， */ );

    ret = chewing_cand_open(ctx);
    ok(ret == 0, "chewing_cand_open() returns `%d' shall be `%d'", ret, 0);

    for (i = ARRAY_SIZE(EXPECT_CAND_LEN) - 1; i >= 0; --i) {
        ret = chewing_cand_TotalChoice(ctx);
        ok(ret > 0, "chewing_cand_TotalChoice() returns `%d' shall be greater than `%d'", ret, 0);
        ok_candidate_len(ctx, EXPECT_CAND_LEN[i]);
        type_keystroke_by_string(ctx, "j");
    }

    for (i = 0; i < ARRAY_SIZE(EXPECT_CAND_LEN); ++i) {
        ret = chewing_cand_TotalChoice(ctx);
        ok(ret > 0, "chewing_cand_TotalChoice() returns `%d' shall be greater than `%d'", ret, 0);
        ok_candidate_len(ctx, EXPECT_CAND_LEN[i]);
        type_keystroke_by_string(ctx, "i");
    }
    chewing_delete(ctx);
#endif
}


void test_KB_HSU()
{
    ChewingContext *ctx;

    ctx = chewing_new();
    start_testcase(ctx, fd);

    chewing_set_KBType(ctx, KB_HSU);

    type_keystroke_by_string(ctx, "cen kxjen jn dgshnfbkj");
    ok_preedit_buffer(ctx, "\xE6\x96\xB0\xE9\x85\xB7\xE9\x9F\xB3\xE7\x9C\x9F\xE7\x9A\x84\xE5\xBE\x88\xE6\xA3\x92" 
                      /* 新酷音真的很棒 */ );
    chewing_clean_preedit_buf(ctx);

    type_keystroke_by_string(ctx, "eq"); /* q is not a phone */
    ok_bopomofo_buffer(ctx, "\xE3\x84\xA7" /* ㄧ */ );
    ok_preedit_buffer(ctx, "");
    type_keystroke_by_string(ctx, "a "); /* no word is pronounced ㄧㄟ */
    ok_bopomofo_buffer(ctx, "");
    ok_preedit_buffer(ctx, "");
    chewing_clean_bopomofo_buf(ctx);
    chewing_clean_preedit_buf(ctx);

    type_keystroke_by_string(ctx, "m");
    ok_bopomofo_buffer(ctx, "\xE3\x84\x87" /* ㄇ */ );
    type_keystroke_by_string(ctx, " "); /* convert "ㄇ" to "ㄢ" */
    ok_bopomofo_buffer(ctx, "");
    ok_preedit_buffer(ctx, "\xE5\xAE\x89" /* 安 */);
    chewing_clean_preedit_buf(ctx);

    type_keystroke_by_string(ctx, "h");
    ok_bopomofo_buffer(ctx, "\xE3\x84\x8F" /* ㄏ */ );
    type_keystroke_by_string(ctx, "d");  /* convert "ㄏ" to "ㄛ" */
    ok_bopomofo_buffer(ctx, "");
    ok_preedit_buffer(ctx, "\xE5\x93\xA6" /* 哦 */);
    chewing_clean_preedit_buf(ctx);

    type_keystroke_by_string(ctx, "g");
    ok_bopomofo_buffer(ctx, "\xE3\x84\x8D" /* ㄍ */ );
    type_keystroke_by_string(ctx, " "); /* convert "ㄍ" to "ㄜ" */
    ok_bopomofo_buffer(ctx, "");
    ok_preedit_buffer(ctx, "\xE9\x98\xBF" /* 阿 */);
    chewing_clean_preedit_buf(ctx);

    type_keystroke_by_string(ctx, "n");
    ok_bopomofo_buffer(ctx, "\xE3\x84\x8B" /* ㄋ */ );
    type_keystroke_by_string(ctx, "f"); /* convert "ㄋ" to "ㄣ" */
    ok_bopomofo_buffer(ctx, "");
    ok_preedit_buffer(ctx, "\xE5\xB3\x8E" /* 峎 */);
    chewing_clean_preedit_buf(ctx);

    type_keystroke_by_string(ctx, "k");
    ok_bopomofo_buffer(ctx, "\xE3\x84\x8E" /* ㄎ */ );
    type_keystroke_by_string(ctx, " "); /* convert "ㄎ" to "ㄤ" */
    ok_bopomofo_buffer(ctx, "");
    ok_preedit_buffer(ctx, "\xE9\xAA\xAF" /* 骯 */);
    chewing_clean_preedit_buf(ctx);

    type_keystroke_by_string(ctx, "j");
    ok_bopomofo_buffer(ctx, "\xE3\x84\x90" /* ㄐ */);
    type_keystroke_by_string(ctx, " "); /* convert "ㄐ,ㄑ,ㄒ" to "ㄓ,ㄔ,ㄕ" */
    ok_bopomofo_buffer(ctx, "");
    ok_preedit_buffer(ctx, "\xE4\xB9\x8B" /* 之 */);
    chewing_clean_preedit_buf(ctx);

    type_keystroke_by_string(ctx, "l");
    ok_bopomofo_buffer(ctx, "\xE3\x84\x8C" /* ㄌ */);
    type_keystroke_by_string(ctx, "f"); /* convert "ㄌ" to "ㄦ" */
    ok_bopomofo_buffer(ctx, "");
    ok_preedit_buffer(ctx, "\xE7\x88\xBE" /* 爾 */);
    chewing_clean_preedit_buf(ctx);

    chewing_delete(ctx);
}


void test_KB_HSU_fuzzy()
{
    ChewingContext *ctx;

    ctx = chewing_new();
    start_testcase(ctx, fd);

    chewing_set_KBType(ctx, KB_HSU);

    type_keystroke_by_string(ctx, "ge"); /* fuzzy ㄍㄧ to ㄐㄧ */
    ok_bopomofo_buffer(ctx, "\xE3\x84\x90\xE3\x84\xA7" /* ㄐㄧ */);
    type_keystroke_by_string(ctx, "y");
    ok_bopomofo_buffer(ctx, "\xE3\x84\x90\xE3\x84\xA7\xE3\x84\x9A" /* ㄐㄧㄚ */);
    chewing_clean_bopomofo_buf(ctx);

    type_keystroke_by_string(ctx, "gm");
    ok_bopomofo_buffer(ctx, "\xE3\x84\x8D\xE3\x84\xA2" /* ㄍㄢ */);
    type_keystroke_by_string(ctx, "e");
    ok_bopomofo_buffer(ctx, "\xE3\x84\x90\xE3\x84\xA7\xE3\x84\xA2" /* ㄐㄧㄢ */);
    chewing_clean_bopomofo_buf(ctx);

    type_keystroke_by_string(ctx, "gu"); /* fuzzy ㄍㄩ to ㄐㄩ */
    ok_bopomofo_buffer(ctx, "\xE3\x84\x90\xE3\x84\xA9" /* ㄐㄩ */);
    chewing_clean_bopomofo_buf(ctx);

    type_keystroke_by_string(ctx, "gx"); /* ㄍㄨ shall stay unchanged */
    ok_bopomofo_buffer(ctx, "\xE3\x84\x8D\xE3\x84\xA8" /* ㄍㄨ */);
    chewing_clean_bopomofo_buf(ctx);

    chewing_delete(ctx);
}


void test_KB_HSU_JVC()
{
    ChewingContext *ctx;

    ctx = chewing_new();
    start_testcase(ctx, fd);

    chewing_set_KBType(ctx, KB_HSU);

    type_keystroke_by_string(ctx, "ce");
    ok_bopomofo_buffer(ctx, "\xE3\x84\x92\xE3\x84\xA7" /* ㄒㄧ */);
    chewing_clean_bopomofo_buf(ctx);

    type_keystroke_by_string(ctx, "vu");
    ok_bopomofo_buffer(ctx, "\xE3\x84\x91\xE3\x84\xA9" /* ㄑㄩ */);
    chewing_clean_bopomofo_buf(ctx);

    type_keystroke_by_string(ctx, "jx"); /* convert ㄐ to ㄓ */
    ok_bopomofo_buffer(ctx, "\xE3\x84\x93\xE3\x84\xA8" /* ㄓㄨ */);
    chewing_clean_bopomofo_buf(ctx);

    type_keystroke_by_string(ctx, "jy"); /* convert ㄐ to ㄓ */
    ok_bopomofo_buffer(ctx, "\xE3\x84\x93\xE3\x84\x9A" /* ㄓㄚ */);
    type_keystroke_by_string(ctx, "e"); /* convert back to ㄐ */
    ok_bopomofo_buffer(ctx, "\xE3\x84\x90\xE3\x84\xA7\xE3\x84\x9A" /* ㄐㄧㄚ */);
    chewing_clean_bopomofo_buf(ctx);

    chewing_delete(ctx);
}


void test_KB_ET26()
{
    ChewingContext *ctx;

    ctx = chewing_new();
    start_testcase(ctx, fd);

    chewing_set_KBType(ctx, KB_ET26);

    type_keystroke_by_string(ctx, "cen kxken gn drdhnjbtk");
    ok_preedit_buffer(ctx, "\xE6\x96\xB0\xE9\x85\xB7\xE9\x9F\xB3\xE7\x9C\x9F\xE7\x9A\x84\xE5\xBE\x88\xE6\xA3\x92" 
                      /* 新酷音真的很棒 */ );
    chewing_clean_preedit_buf(ctx);

    type_keystroke_by_string(ctx, "eq "); /* no word is pronounced ㄧㄟ */
    ok_bopomofo_buffer(ctx, "");
    ok_preedit_buffer(ctx, "");
    chewing_clean_bopomofo_buf(ctx);
    chewing_clean_preedit_buf(ctx);

    type_keystroke_by_string(ctx, "p");
    ok_bopomofo_buffer(ctx, "\xE3\x84\x86" /* ㄆ */ );
    type_keystroke_by_string(ctx, "f"); /* convert "ㄆ" to "ㄡ" */
    ok_bopomofo_buffer(ctx, "");
    ok_preedit_buffer(ctx, "\xE5\x90\xBD" /* 吽 */);
    chewing_clean_preedit_buf(ctx);

    type_keystroke_by_string(ctx, "m");
    ok_bopomofo_buffer(ctx, "\xE3\x84\x87" /* ㄇ */ );
    type_keystroke_by_string(ctx, " "); /* convert "ㄇ" to "ㄢ" */
    ok_bopomofo_buffer(ctx, "");
    ok_preedit_buffer(ctx, "\xE5\xAE\x89" /* 安 */);
    chewing_clean_preedit_buf(ctx);

    type_keystroke_by_string(ctx, "n");
    ok_bopomofo_buffer(ctx, "\xE3\x84\x8B" /* ㄋ */ );
    type_keystroke_by_string(ctx, "j"); /* convert "ㄋ" to "ㄣ" */
    ok_bopomofo_buffer(ctx, "");
    ok_preedit_buffer(ctx, "\xE5\xB3\x8E" /* 峎 */);
    chewing_clean_preedit_buf(ctx);

    type_keystroke_by_string(ctx, "t");
    ok_bopomofo_buffer(ctx, "\xE3\x84\x8A" /* ㄊ */ );
    type_keystroke_by_string(ctx, " "); /* convert "ㄊ" to "ㄤ" */
    ok_bopomofo_buffer(ctx, "");
    ok_preedit_buffer(ctx, "\xE9\xAA\xAF" /* 骯 */);
    chewing_clean_preedit_buf(ctx);

    type_keystroke_by_string(ctx, "l");
    ok_bopomofo_buffer(ctx, "\xE3\x84\x8C" /* ㄌ */ );
    type_keystroke_by_string(ctx, " "); /* convert "ㄌ" to "ㄥ" */
    ok_bopomofo_buffer(ctx, "");
    ok_preedit_buffer(ctx, "\xE9\x9E\xA5" /* 鞥 */);
    chewing_clean_preedit_buf(ctx);

    type_keystroke_by_string(ctx, "h");
    ok_bopomofo_buffer(ctx, "\xE3\x84\x8F" /* ㄏ */);
    type_keystroke_by_string(ctx, "j"); /* convert "ㄏ" to "ㄦ" */
    ok_bopomofo_buffer(ctx, "");
    ok_preedit_buffer(ctx, "\xE7\x88\xBE" /* 爾 */);
    chewing_clean_preedit_buf(ctx);

    type_keystroke_by_string(ctx, "g");
    ok_bopomofo_buffer(ctx, "\xE3\x84\x90" /* ㄐ */);
    type_keystroke_by_string(ctx, " "); /* convert "ㄐ,ㄒ" to "ㄓ,ㄕ" */
    ok_bopomofo_buffer(ctx, "");
    ok_preedit_buffer(ctx, "\xE4\xB9\x8B" /* 之 */);
    chewing_clean_preedit_buf(ctx);

    chewing_delete(ctx);
}


void test_KB_ET26_GVC()
{
    ChewingContext *ctx;

    ctx = chewing_new();
    start_testcase(ctx, fd);

    chewing_set_KBType(ctx, KB_ET26);

    type_keystroke_by_string(ctx, "ce");
    ok_bopomofo_buffer(ctx, "\xE3\x84\x92\xE3\x84\xA7" /* ㄒㄧ */);
    chewing_clean_bopomofo_buf(ctx);

    type_keystroke_by_string(ctx, "gu");
    ok_bopomofo_buffer(ctx, "\xE3\x84\x90\xE3\x84\xA9" /* ㄐㄩ */);
    chewing_clean_bopomofo_buf(ctx);

    type_keystroke_by_string(ctx, "gx"); /* convert ㄐ to ㄓ */
    ok_bopomofo_buffer(ctx, "\xE3\x84\x93\xE3\x84\xA8" /* ㄓㄨ */);
    chewing_clean_bopomofo_buf(ctx);

    type_keystroke_by_string(ctx, "ga"); /* convert ㄐ to ㄓ */
    ok_bopomofo_buffer(ctx, "\xE3\x84\x93\xE3\x84\x9A" /* ㄓㄚ */);
    type_keystroke_by_string(ctx, "e"); /* convert back to ㄐ */
    ok_bopomofo_buffer(ctx, "\xE3\x84\x90\xE3\x84\xA7\xE3\x84\x9A" /* ㄐㄧㄚ */);
    chewing_clean_bopomofo_buf(ctx);

    type_keystroke_by_string(ctx, "va");
    ok_bopomofo_buffer(ctx, "\xE3\x84\x8D\xE3\x84\x9A" /* ㄍㄚ */ );
    type_keystroke_by_string(ctx, "e"); /* convert ㄍ to ㄑ */
    ok_bopomofo_buffer(ctx, "\xE3\x84\x91\xE3\x84\xA7\xE3\x84\x9A" /* ㄑㄧㄚ */ );
    chewing_clean_bopomofo_buf(ctx);

    chewing_delete(ctx);
}


void test_KB_DACHEN_CP26()
{
    ChewingContext *ctx;

    ctx = chewing_new();
    start_testcase(ctx, fd);

    chewing_set_KBType(ctx, KB_DACHEN_CP26);

    type_keystroke_by_string(ctx, "vup djdup tp wkycprqlld");
    ok_preedit_buffer(ctx, "\xE6\x96\xB0\xE9\x85\xB7\xE9\x9F\xB3\xE7\x9C\x9F\xE7\x9A\x84\xE5\xBE\x88\xE6\xA3\x92" 
                      /* 新酷音真的很棒 */ );
    chewing_clean_preedit_buf(ctx);

    type_keystroke_by_string(ctx, "uo "); /* no word is pronounced ㄧㄟ */
    ok_bopomofo_buffer(ctx, "");
    ok_preedit_buffer(ctx, "");
    chewing_clean_bopomofo_buf(ctx);
    chewing_clean_preedit_buf(ctx);

    type_keystroke_by_string(ctx, "qq"); /* switch between "ㄅ" and "ㄆ" */
    ok_bopomofo_buffer(ctx, "\xE3\x84\x86" /* ㄆ */);
    chewing_clean_bopomofo_buf(ctx);

    type_keystroke_by_string(ctx, "ww"); /* switch between "ㄉ" and "ㄊ" */
    ok_bopomofo_buffer(ctx, "\xE3\x84\x8A" /* ㄊ */);
    chewing_clean_bopomofo_buf(ctx);

    type_keystroke_by_string(ctx, "tt"); /* switch between "ㄓ" and "ㄔ" */
    ok_bopomofo_buffer(ctx, "\xE3\x84\x94" /* ㄔ */);
    chewing_clean_bopomofo_buf(ctx);

    type_keystroke_by_string(ctx, "xmm"); /* switch between "ㄩ" and "ㄡ" */
    ok_bopomofo_buffer(ctx, "\xE3\x84\x8C\xE3\x84\xA1" /* ㄌㄡ */);
    chewing_clean_bopomofo_buf(ctx);

    type_keystroke_by_string(ctx, "xum"); /* convert "ㄧㄩ" to "ㄧㄡ" */
    ok_bopomofo_buffer(ctx, "\xE3\x84\x8C\xE3\x84\xA7\xE3\x84\xA1" /* ㄌㄧㄡ */);
    type_keystroke_by_string(ctx, "m"); /* convert "ㄧㄡ" to "ㄩ" */
    ok_bopomofo_buffer(ctx, "\xE3\x84\x8C\xE3\x84\xA9" /* ㄌㄩ */);
    chewing_clean_bopomofo_buf(ctx);

    type_keystroke_by_string(ctx, "ii"); /* switch between "ㄛ" and "ㄞ" */
    ok_bopomofo_buffer(ctx, "\xE3\x84\x9E" /* ㄞ */);
    chewing_clean_bopomofo_buf(ctx);

    type_keystroke_by_string(ctx, "oo"); /* switch between "ㄟ" and "ㄢ" */
    ok_bopomofo_buffer(ctx, "\xE3\x84\xA2" /* ㄢ */);
    chewing_clean_bopomofo_buf(ctx);

    type_keystroke_by_string(ctx, "ll"); /* switch between "ㄠ" and "ㄤ" */
    ok_bopomofo_buffer(ctx, "\xE3\x84\xA4" /* ㄤ */);
    chewing_clean_bopomofo_buf(ctx);

    type_keystroke_by_string(ctx, "pp"); /* switch between "ㄣ" and "ㄦ" */
    ok_bopomofo_buffer(ctx, "\xE3\x84\xA6" /* ㄦ */);
    chewing_clean_bopomofo_buf(ctx);

    type_keystroke_by_string(ctx, "wu"); /* switch among "ㄧ", "ㄚ" and "ㄧㄚ" */
    ok_bopomofo_buffer(ctx, "\xE3\x84\x89\xE3\x84\xA7" /* ㄉㄧ */);
    type_keystroke_by_string(ctx, "u");
    ok_bopomofo_buffer(ctx, "\xE3\x84\x89\xE3\x84\x9A" /* ㄉㄚ */);
    type_keystroke_by_string(ctx, "u");
    ok_bopomofo_buffer(ctx, "\xE3\x84\x89\xE3\x84\xA7\xE3\x84\x9A" /* ㄉㄧㄚ */);
    type_keystroke_by_string(ctx, "u");
    ok_bopomofo_buffer(ctx, "\xE3\x84\x89" /* ㄉ */);
    type_keystroke_by_string(ctx, "ju");
    ok_bopomofo_buffer(ctx, "\xE3\x84\x89\xE3\x84\xA8\xE3\x84\x9A" /* ㄉㄨㄚ */);
    chewing_clean_bopomofo_buf(ctx);

    type_keystroke_by_string(ctx, "bb"); /* convert "ㄖ" to "ㄝ" */
    ok_bopomofo_buffer(ctx, "\xE3\x84\x96\xE3\x84\x9D" /* ㄖㄝ */);
    chewing_clean_bopomofo_buf(ctx);

    type_keystroke_by_string(ctx, "njn"); /* convert "ㄙ" to "ㄥ" */
    ok_bopomofo_buffer(ctx, "\xE3\x84\x99\xE3\x84\xA8\xE3\x84\xA5" /* ㄙㄨㄥ */);
    chewing_clean_bopomofo_buf(ctx);

    chewing_delete(ctx);
}


void test_KB_HANYU()
{
    ChewingContext *ctx;

    ctx = chewing_new();
    start_testcase(ctx, fd);

    chewing_set_KBType(ctx, KB_HANYU_PINYIN);

    type_keystroke_by_string(ctx, "xin ku4yin zhen de5hen3bang4");
    ok_preedit_buffer(ctx, "\xE6\x96\xB0\xE9\x85\xB7\xE9\x9F\xB3\xE7\x9C\x9F\xE7\x9A\x84\xE5\xBE\x88\xE6\xA3\x92" 
                      /* 新酷音真的很棒 */ );
    chewing_clean_preedit_buf(ctx);

    type_keystroke_by_string(ctx, "huan ying2shi3yong4pin yin mo2shi4");
    ok_preedit_buffer(ctx, "\xE6\xAD\xA1\xE8\xBF\x8E\xE4\xBD\xBF\xE7\x94\xA8\xE6\x8B\xBC\xE9\x9F\xB3\xE6\xA8\xA1\xE5\xBC\x8F" 
                      /* 歡迎使用拼音模式 */ );
    chewing_clean_preedit_buf(ctx);

    chewing_delete(ctx);
}


void test_KB_THL()
{
    ChewingContext *ctx;

    ctx = chewing_new();
    start_testcase(ctx, fd);

    chewing_set_KBType(ctx, KB_THL_PINYIN);

    type_keystroke_by_string(ctx, "sin ku4yin jhen de5hen3bang4");
    ok_preedit_buffer(ctx, "\xE6\x96\xB0\xE9\x85\xB7\xE9\x9F\xB3\xE7\x9C\x9F\xE7\x9A\x84\xE5\xBE\x88\xE6\xA3\x92" 
                      /* 新酷音真的很棒 */ );
    chewing_clean_preedit_buf(ctx);

    type_keystroke_by_string(ctx, "huan ying2shih3yong4pin yin mo2shih4");
    ok_preedit_buffer(ctx, "\xE6\xAD\xA1\xE8\xBF\x8E\xE4\xBD\xBF\xE7\x94\xA8\xE6\x8B\xBC\xE9\x9F\xB3\xE6\xA8\xA1\xE5\xBC\x8F" 
                      /* 歡迎使用拼音模式 */ );
    chewing_clean_preedit_buf(ctx);

    chewing_delete(ctx);
}


void test_KB_MPS2()
{
    ChewingContext *ctx;

    ctx = chewing_new();
    start_testcase(ctx, fd);

    chewing_set_KBType(ctx, KB_MPS2_PINYIN);

    type_keystroke_by_string(ctx, "shin ku4in jen de5hen3bang4");
    ok_preedit_buffer(ctx, "\xE6\x96\xB0\xE9\x85\xB7\xE9\x9F\xB3\xE7\x9C\x9F\xE7\x9A\x84\xE5\xBE\x88\xE6\xA3\x92" 
                      /* 新酷音真的很棒 */ );
    chewing_clean_preedit_buf(ctx);

    type_keystroke_by_string(ctx, "huan ing2shr3iung4pin in muo2shz4");
    ok_preedit_buffer(ctx, "\xE6\xAD\xA1\xE8\xBF\x8E\xE4\xBD\xBF\xE7\x94\xA8\xE6\x8B\xBC\xE9\x9F\xB3\xE6\xA8\xA1\xE5\xBC\x8F" 
                      /* 歡迎使用拼音模式 */ );
    chewing_clean_preedit_buf(ctx);

    chewing_delete(ctx);
}


void test_KB()
{
    test_KB_HSU();
    test_KB_HSU_fuzzy();
    test_KB_HSU_JVC();

    test_KB_ET26();
    test_KB_ET26_GVC();

    test_KB_DACHEN_CP26();

    test_KB_HANYU();
    test_KB_THL();
    test_KB_MPS2();
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


    test_select_candidate();
    test_Esc();
    test_Del();
    test_Backspace();
    test_Up();
    test_Down();
    test_Tab();
    test_DblTab();
    test_Capslock();
    test_Home();
    test_End();
    test_PageUp();
    test_PageDown();
    test_ShiftSpace();
    test_Numlock();
    test_Space();

    test_get_phoneSeq();
    test_bopomofo_buffer();

    test_longest_phrase();
    test_auto_commit();

    test_interval();

    test_jk_selection();

    test_KB();

    fclose(fd);

    return exit_status();
}
