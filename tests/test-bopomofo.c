/**
 * test-bopomofo.c
 *
 * Copyright (c) 2012
 *      libchewing Core Team.
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

    type_keystroke_by_string(ctx, "<D>");       /* 移上 */
    ok_candidate(ctx, CAND_2, ARRAY_SIZE(CAND_2));

    type_keystroke_by_string(ctx, "<D><L><D>2<E>");        /* select 移上來 */
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

    type_keystroke_by_string(ctx, "<D><L><D>2<E>");        /* select 移上來 */
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

void test_select_candidate_second_page()
{
    ChewingContext *ctx;

    clean_userphrase();

    ctx = chewing_new();
    start_testcase(ctx, fd);

    chewing_set_candPerPage(ctx, 9);
    type_keystroke_by_string(ctx, "u4<D><R>4"); /* ㄧˋ */
    ok_preedit_buffer(ctx, "役");

    chewing_delete(ctx);
}

void test_select_candidate_second_page_rewind()
{
    ChewingContext *ctx;

    static const char *CAND[] = {
        "紛紛",
        "分分"
    };

    clean_userphrase();

    ctx = chewing_new();
    start_testcase(ctx, fd);

    chewing_set_candPerPage(ctx, 9);
    chewing_set_spaceAsSelection(ctx, 1);
    chewing_set_phraseChoiceRearward(ctx, 1);
    type_keystroke_by_string(ctx, "zp zp <D><D><R><D><D>"); /* ㄈㄣ ㄈㄣ */
    ok_candidate(ctx, CAND, ARRAY_SIZE(CAND));

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
    test_select_candidate_second_page();
    test_select_candidate_second_page_rewind();
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
    ok_commit_buffer(ctx, "測試" );

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
    ok_commit_buffer(ctx, "測試" );

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

void test_Down_open_candidate_window_after_deleting_symbol()
{
    ChewingContext *ctx;
    int ret;

    ctx = chewing_new();
    start_testcase(ctx, fd);

    type_keystroke_by_string(ctx, "<<>hk4g4<<>" /* ，測試， */);
    ret = chewing_cand_TotalChoice(ctx);
    ok(ret == 0, "chewing_cand_TotalChoice() returns `%d' shall be `%d'", ret, 0);

    type_keystroke_by_string(ctx, "<H><DC><EN><D>" /* Home Delete End Down */);
    ret = chewing_cand_TotalChoice(ctx);
    ok(ret > 0, "chewing_cand_TotalChoice() returns `%d' shall be greater than `%d'", ret, 0);

    type_keystroke_by_string(ctx, "2");
    ret = chewing_cand_TotalChoice(ctx);
    ok(ret == 0, "chewing_cand_TotalChoice() returns `%d' shall be `%d'", ret, 0);
    ok_preedit_buffer(ctx, "\xE6\xB8\xAC\xE8\xA9\xA6\xE2\x86\x90" /* 測試← */ );

    chewing_delete(ctx);
}

void test_Down()
{
    test_Down_open_candidate_window();
    test_Down_not_entering_chewing();
    test_Down_open_candidate_window_after_deleting_symbol();
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

    type_keystroke_by_string(ctx, "<CB>");

    mode = chewing_get_ChiEngMode(ctx);
    ok(mode == CHINESE_MODE, "mode shall change to CHINESE_MODE");

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

    chewing_set_ChiEngMode(ctx, CHINESE_MODE);
    type_keystroke_by_string(ctx, "<SS>");
    mode = chewing_get_ShapeMode(ctx);
    ok(mode == HALFSHAPE_MODE, "mode shall be HALFSHAPE_MODE");

    type_keystroke_by_string(ctx, " ");
    ok_commit_buffer(ctx, " ");

    type_keystroke_by_string(ctx, "hk4 <E>");
    ok_commit_buffer(ctx, "冊 ");

    chewing_set_ChiEngMode(ctx, SYMBOL_MODE);
    type_keystroke_by_string(ctx, "a ");
    ok_commit_buffer(ctx, " ");

    chewing_delete(ctx);
}

void test_ShiftSpaceDisabled()
{
    ChewingContext *ctx;
    int mode;

    ctx = chewing_new();
    start_testcase(ctx, fd);

    chewing_config_set_int(ctx, "chewing.enable_fullwidth_toggle_key", 0);

    mode = chewing_get_ShapeMode(ctx);
    ok(mode == HALFSHAPE_MODE, "mode shall be HALFSHAPE_MODE");

    type_keystroke_by_string(ctx, "<SS>");
    mode = chewing_get_ShapeMode(ctx);
    ok(mode == HALFSHAPE_MODE, "mode shall be HALFSHAPE_MODE");

    type_keystroke_by_string(ctx, " ");
    ok_commit_buffer(ctx, " "); /* Space */

    chewing_set_ChiEngMode(ctx, SYMBOL_MODE);
    type_keystroke_by_string(ctx, "a");
    ok_commit_buffer(ctx, "a"); /* a */

    chewing_set_ChiEngMode(ctx, CHINESE_MODE);
    type_keystroke_by_string(ctx, "<SS>");
    mode = chewing_get_ShapeMode(ctx);
    ok(mode == HALFSHAPE_MODE, "mode shall be HALFSHAPE_MODE");

    type_keystroke_by_string(ctx, " ");
    ok_commit_buffer(ctx, " ");

    type_keystroke_by_string(ctx, "hk4 <E>");
    ok_commit_buffer(ctx, "冊 ");

    chewing_set_ChiEngMode(ctx, SYMBOL_MODE);
    type_keystroke_by_string(ctx, "a ");
    ok_commit_buffer(ctx, " ");

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

void test_Space_empty_buffer()
{
    ChewingContext *ctx;

    clean_userphrase();

    ctx = chewing_new();
    start_testcase(ctx, fd);
    chewing_set_spaceAsSelection(ctx, 1);

    type_keystroke_by_string(ctx, " ");
    ok_preedit_buffer(ctx, "");
    ok_commit_buffer(ctx, " ");

    chewing_delete(ctx);
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

void test_Space_selection_insert_eng_mode()
{
    ChewingContext *ctx;

    clean_userphrase();

    ctx = chewing_new();
    start_testcase(ctx, fd);
    chewing_set_spaceAsSelection(ctx, 1);

    type_keystroke_by_string(ctx, "hk4");
    chewing_set_ChiEngMode(ctx, SYMBOL_MODE);
    type_keystroke_by_string(ctx, " j");
    ok_preedit_buffer(ctx, "冊 j");

    chewing_delete(ctx);
}

void test_Space()
{
    test_Space_empty_buffer();
    test_Space_selection_word();
    test_Space_selection_symbol();
    test_Space_selection_insert_eng_mode();
}

void test_FuzzySearchMode()
{
    const TestData FUZZY_INPUT[] = {
        {"eji6aup6284cjo42941ul3<E>", "國民大會代表" },
        {"eji aup 28 cjo 29 1ul <E>", "國民大會代表" },
        {"ej au 2 cj 2 1 <E>", "國民大會代表" },
        {"e a 2 c 2 1 <E>", "國民大會代表" },
        {"ea2c21 <E>", "國民大會代表" },
    };
    size_t i;
    ChewingContext *ctx;

    ctx = chewing_new();
    start_testcase(ctx, fd);
    chewing_set_maxChiSymbolLen(ctx, 16);
    chewing_config_set_int(ctx, "chewing.conversion_engine", FUZZY_CHEWING_CONVERSION_ENGINE);

    for (i = 0; i < ARRAY_SIZE(FUZZY_INPUT); ++i) {
        type_keystroke_by_string(ctx, FUZZY_INPUT[i].token);
        ok_commit_buffer(ctx, FUZZY_INPUT[i].expected);
    }

    chewing_delete(ctx);
}

void test_FuzzySearchMode_Hanyu()
{
    const TestData FUZZY_INPUT[] = {
        {"guo2min2da4hui4dai4biao3<E>", "國民大會代表" },
        {"guo min da hui dai biao <E>", "國民大會代表" },
        {"g m d h d b <E>", "國民大會代表" },
    };
    size_t i;
    ChewingContext *ctx;

    ctx = chewing_new();
    start_testcase(ctx, fd);
    chewing_set_maxChiSymbolLen(ctx, 16);
    chewing_set_KBType(ctx, KB_HANYU_PINYIN);
    chewing_config_set_int(ctx, "chewing.conversion_engine", FUZZY_CHEWING_CONVERSION_ENGINE);

    for (i = 0; i < ARRAY_SIZE(FUZZY_INPUT); ++i) {
        type_keystroke_by_string(ctx, FUZZY_INPUT[i].token);
        ok_commit_buffer(ctx, FUZZY_INPUT[i].expected);
    }

    chewing_delete(ctx);
}

void test_SimpleEngine()
{
    const TestData SIMPLE_INPUT[] = {
        {"ru03120 15j41up 1ai61g41!<E>", "簡單住因模市！" },
        {"ru03<EE>20 <EE>5j4<EE>up <EE>ai6<EE>g4<EE>!<E>", "簡單住因模市！" },
        {"ru03120 15j44up 2ai61g4<D>2!<E>", "簡單注音模式！" },
        {"ru03120 15j44up 2ai61g4<D>2!<H>20 1tjp61<E>", "單純簡單注音模式！" },
    };
    size_t i;
    ChewingContext *ctx;

    ctx = chewing_new();
    start_testcase(ctx, fd);
    chewing_set_maxChiSymbolLen(ctx, 16);
    chewing_config_set_int(ctx, "chewing.conversion_engine", SIMPLE_CONVERSION_ENGINE);

    for (i = 0; i < ARRAY_SIZE(SIMPLE_INPUT); ++i) {
        type_keystroke_by_string(ctx, SIMPLE_INPUT[i].token);
        ok_commit_buffer(ctx, SIMPLE_INPUT[i].expected);
    }

    chewing_delete(ctx);
}

void test_Acknowledge()
{
    ChewingContext *ctx;

    ctx = chewing_new();
    start_testcase(ctx, fd);

    type_keystroke_by_string(ctx, "hk4g4<E>");
    ok_commit_buffer(ctx, "測試");

    chewing_ack(ctx);
    ok_commit_buffer(ctx, "");

    chewing_delete(ctx);
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

    type_keystroke_by_string(ctx, "hk4g4<L><T><L><D>1<EN>`31hk4" /* 測試，測 */ );
    ok_preedit_buffer(ctx, "，冊");
    ok_commit_buffer(ctx, "測試");
    type_keystroke_by_string(ctx, "g4" /* 試 */ );
    ok_preedit_buffer(ctx, "，測試");
    // check commit buffer when KeyBehavior is not COMMIT is undefined
    // ok_commit_buffer(ctx, "測試");

    chewing_delete(ctx);
}

void test_auto_commit_symbol()
{
    ChewingContext *ctx;

    ctx = chewing_new();
    start_testcase(ctx, fd);
    chewing_set_maxChiSymbolLen(ctx, 2);

    type_keystroke_by_string(ctx, "`31hk4g4" /* ，測試 */ );
    ok_preedit_buffer(ctx, "\xE6\xB8\xAC\xE8\xA9\xA6" /* 測試 */ );
    ok_commit_buffer(ctx, "\xEF\xBC\x8C" /* ， */ );

    chewing_delete(ctx);
}

void test_auto_commit()
{
    test_auto_commit_phrase();
    test_auto_commit_symbol();
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
        type_keystroke_by_string(ctx, "k");
    }
    chewing_delete(ctx);
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

    type_keystroke_by_string(ctx, "l");
    ok_bopomofo_buffer(ctx, "\xE3\x84\x8C" /* ㄌ */);
    type_keystroke_by_string(ctx, "f"); /* convert "ㄌ" to "ㄦ" */
    ok_bopomofo_buffer(ctx, "");
    ok_preedit_buffer(ctx, "\xE7\x88\xBE" /* 爾 */);
    chewing_clean_preedit_buf(ctx);

    type_keystroke_by_string(ctx, "g");
    ok_bopomofo_buffer(ctx, "ㄍ");
    type_keystroke_by_string(ctx, "e");
    ok_bopomofo_buffer(ctx, "ㄍㄧ");
    type_keystroke_by_string(ctx, " ");
    ok_preedit_buffer(ctx, "機");  /* convert "ㄍㄧ" to "ㄐㄧ" */
    chewing_clean_preedit_buf(ctx);

    type_keystroke_by_string(ctx, "g");
    ok_bopomofo_buffer(ctx, "ㄍ");
    type_keystroke_by_string(ctx, "e");
    ok_bopomofo_buffer(ctx, "ㄍㄧ");
    type_keystroke_by_string(ctx, "n");
    ok_bopomofo_buffer(ctx, "ㄐㄧㄣ");
    type_keystroke_by_string(ctx, " ");
    ok_preedit_buffer(ctx, "今");  /* convert "ㄍㄧㄣ" to "ㄐㄧㄣ" */
    chewing_clean_preedit_buf(ctx);

    type_keystroke_by_string(ctx, "g");
    ok_bopomofo_buffer(ctx, "ㄍ");
    type_keystroke_by_string(ctx, "e");
    ok_bopomofo_buffer(ctx, "ㄍㄧ");
    type_keystroke_by_string(ctx, "e");
    ok_bopomofo_buffer(ctx, "ㄐㄧㄝ");
    type_keystroke_by_string(ctx, "j");
    ok_preedit_buffer(ctx, "界");  /* convert "ㄍㄧㄝ" to "ㄐㄧㄝ" */
    chewing_clean_preedit_buf(ctx);

    type_keystroke_by_string(ctx, "g");
    ok_bopomofo_buffer(ctx, "ㄍ");
    type_keystroke_by_string(ctx, "u");
    ok_bopomofo_buffer(ctx, "ㄍㄩ");
    type_keystroke_by_string(ctx, " ");
    ok_preedit_buffer(ctx, "居");  /* convert "ㄍㄩ" to "ㄐㄩ" */
    chewing_clean_preedit_buf(ctx);

    type_keystroke_by_string(ctx, "g");
    ok_bopomofo_buffer(ctx, "ㄍ");
    type_keystroke_by_string(ctx, "u");
    ok_bopomofo_buffer(ctx, "ㄍㄩ");
    type_keystroke_by_string(ctx, "e");
    ok_bopomofo_buffer(ctx, "ㄐㄩㄝ");
    type_keystroke_by_string(ctx, "d");
    ok_preedit_buffer(ctx, "決");  /* convert "ㄍㄩㄝ" to "ㄐㄩㄝ" */
    chewing_clean_preedit_buf(ctx);

    chewing_delete(ctx);
}

// Example from https://web.archive.org/web/20240525033152/http://bcc16.ncu.edu.tw/2/nature/DOC/hsu-key/gokey.html
void test_KB_HSU_example()
{
    ChewingContext *ctx;

    ctx = chewing_new();
    start_testcase(ctx, fd);

    chewing_set_KBType(ctx, KB_HSU);
    chewing_set_phraseChoiceRearward(ctx, 1);

    type_keystroke_by_string(ctx, "bnfjxl cen deljudmeldrjki jk ");
    ok_preedit_buffer(ctx, "本中心訂於明日開張");
    chewing_clean_preedit_buf(ctx);

    type_keystroke_by_string(ctx, "xhfjxl cen <D>2vedxkjnefnldhwfhwfdejuljgxl dxdcx ");
    ok_preedit_buffer(ctx, "我衷心期望你能好好地用功讀書");
    chewing_clean_preedit_buf(ctx);

    type_keystroke_by_string(ctx, "xajlgsbewfmeldty dgsjxl cen <D>3");
    ok_preedit_buffer(ctx, "為了表明他的忠心");
    chewing_clean_preedit_buf(ctx);

    type_keystroke_by_string(ctx, "jenj<D>3zjjefdgslejlekj");
    ok_preedit_buffer(ctx, "盡自己的力量");
    chewing_clean_preedit_buf(ctx);

    type_keystroke_by_string(ctx, "jenjzjjefdgsfkdjem ");
    ok_preedit_buffer(ctx, "進自己的房間");
    chewing_clean_preedit_buf(ctx);

    type_keystroke_by_string(ctx, "cekj<D>2tidbafjelffk zjcof");
    ok_preedit_buffer(ctx, "向台北警方自首");
    chewing_clean_preedit_buf(ctx);

    type_keystroke_by_string(ctx, "cekj<D>4tidbafjelffk txldekjdgseofnldlej");
    ok_preedit_buffer(ctx, "像台北警方同樣的有能力");
    chewing_clean_preedit_buf(ctx);

    type_keystroke_by_string(ctx, "jeofuejcdrjceyjxfljcd<D><D>4xfcdxffn ");
    ok_preedit_buffer(ctx, "九月十日下午二時五十五分");
    chewing_clean_preedit_buf(ctx);

    type_keystroke_by_string(ctx, "e j j <D><D>3kgfijdgscewfhxy mw ");
    ok_preedit_buffer(ctx, "一隻隻可愛的小花貓");
    chewing_clean_preedit_buf(ctx);

    type_keystroke_by_string(ctx, "sm sxajdwj<D><D>1xfsxajdgscewfhidxfdwj<D><D>1cd<D>1rnd");
    ok_preedit_buffer(ctx, "三歲到五歲的小孩五到十人");
    chewing_clean_preedit_buf(ctx);

    type_keystroke_by_string(ctx, "jxl cewjdxl lxjxfdxmjleojcde cekj<D><D>1xfnxljcdve hwjjeoflod");
    ok_preedit_buffer(ctx, "忠孝東路五段六十一巷五弄十七號九樓");
    chewing_clean_preedit_buf(ctx);

    type_keystroke_by_string(ctx, "tidxm dgsrndgxl hnfgxaj");
    ok_preedit_buffer(ctx, "台灣的人工很貴");
    chewing_clean_preedit_buf(ctx);

    type_keystroke_by_string(ctx, "ty <D>2pijlekfrndgxl <D><D>4xhfcfulj");
    ok_preedit_buffer(ctx, "他派兩人供我使用");
    chewing_clean_preedit_buf(ctx);

    type_keystroke_by_string(ctx, "ceflgse ggshnfcx fxddgszwf<D>3");
    ok_preedit_buffer(ctx, "洗了一個很舒服的澡");
    chewing_clean_preedit_buf(ctx);

    type_keystroke_by_string(ctx, "tidbafcjcj<D><D>2e ggsmaflejdgsvldcj");
    ok_preedit_buffer(ctx, "台北市是一個美麗的城市");
    chewing_clean_preedit_buf(ctx);

    chewing_delete(ctx);
}

void test_KB_HSU_choice_append()
{
    const TestData CHOICE_INFO_APPEND[] = {
        {"e " /* ㄧ */, "\xE3\x84\x9D" /* ㄝ */ },
        {"g " /* ㄜ */, "\xE3\x84\x8D" /* ㄍ */ },
        {"h " /* ㄛ */, "\xE3\x84\x8F" /* ㄏ */ },
        {"k " /* ㄤ */, "\xE3\x84\x8E" /* ㄎ */ },
        {"c " /* ㄕ */, "\xE3\x84\x92" /* ㄒ */ },
        {"n " /* ㄣ */, "\xE3\x84\x8B" /* ㄋ */ },
        {"m " /* ㄢ */, "\xE3\x84\x87" /* ㄇ */ },
        {"s " /* ㄙ */, "\xCB\x99" /* ˙ */ },
        {"d " /* ㄉ */, "\xCB\x8A" /* ˊ */ },
        {"f " /* ㄈ */, "\xCB\x87" /* ˇ */ },
        {"j " /* ㄓ */, "\xCB\x8B" /* ˋ */ },
        {"l " /* ㄦ */, "\xE3\x84\xA5" /* ㄥ */ },
        {"a " /* ㄘ */, "\xE3\x84\x9F" /* ㄟ */ },
        {"j " /* ㄓ */, "\xE3\x84\x90" /* ㄐ */ },
        {"l " /* ㄦ */, "\xE3\x84\x8C" /* ㄌ */ },
    };
    size_t i;
    ChewingContext *ctx;
    int totalChoice;
    const char *cand;

    ctx = chewing_new();
    start_testcase(ctx, fd);
    chewing_set_KBType(ctx, KB_HSU);

    for (i = 0; i < ARRAY_SIZE(CHOICE_INFO_APPEND); ++i) {

        type_keystroke_by_string(ctx, CHOICE_INFO_APPEND[i].token);

        chewing_cand_open(ctx);
        totalChoice = chewing_cand_TotalChoice(ctx);

        if (i == 14) {
            cand = chewing_cand_string_by_index_static(ctx, totalChoice - 3);
        } else if (i == 13 || i == 12) {
            cand = chewing_cand_string_by_index_static(ctx, totalChoice - 2);
        } else {
            cand = chewing_cand_string_by_index_static(ctx, totalChoice - 1);
        }

        ok(strcmp(cand, CHOICE_INFO_APPEND[i].expected) == 0, "returned candidate is `%s' shall be `%s'", cand, CHOICE_INFO_APPEND[i].expected);

        chewing_cand_close(ctx);
        chewing_clean_preedit_buf(ctx);
    }
    chewing_delete(ctx);
}

void test_KB_HSU_choice_append_select()
{
    ChewingContext *ctx;
    clean_userphrase();

    ctx = chewing_new();
    start_testcase(ctx, fd);
    chewing_set_KBType(ctx, KB_HSU);

    type_keystroke_by_string(ctx, "k <D>4");
    ok_preedit_buffer(ctx, "ㄎ");

    type_keystroke_by_string(ctx, "<T><D>4");
    ok_preedit_buffer(ctx, "ㄎ");

    type_keystroke_by_string(ctx, "<E>");
    ok_commit_buffer(ctx, "ㄎ");

    chewing_delete(ctx);
}

void test_KB_HSU_JVC()
{
    static const struct {
        char *keystroke;
        char *bopomofo;
        char *cand;
    } DATA[] = {
        { "j", "\xE3\x84\x93", /* ㄓ */ "\xE4\xB9\x8B", /* 之 */ },
        { "v", "\xE3\x84\x94", /* ㄔ */ "\xE5\x90\x83", /* 吃 */ },
        { "c", "\xE3\x84\x95", /* ㄕ */ "\xE5\xA4\xB1", /* 失 */ },
    };

    ChewingContext *ctx;
    clean_userphrase();

    ctx = chewing_new();
    start_testcase(ctx, fd);
    chewing_set_KBType(ctx, KB_HSU);

    for (int i = 0; i < ARRAY_SIZE(DATA); ++i) {
        type_keystroke_by_string(ctx, DATA[i].keystroke);
        ok_bopomofo_buffer(ctx, DATA[i].bopomofo);
        type_keystroke_by_string(ctx, " ");
        ok_bopomofo_buffer(ctx, "");
        ok_preedit_buffer(ctx, DATA[i].cand);

        chewing_cand_close(ctx);
        chewing_clean_preedit_buf(ctx);
    }

    type_keystroke_by_string(ctx, "cek");
    ok_bopomofo_buffer(ctx, "\xE3\x84\x92\xE3\x84\xA7\xE3\x84\xA4" /* ㄒㄧㄤ */ );
    type_keystroke_by_string(ctx, "<EE>");

    type_keystroke_by_string(ctx, "cke");
    ok_bopomofo_buffer(ctx, "\xE3\x84\x92\xE3\x84\xA7\xE3\x84\xA4" /* ㄒㄧㄤ */ );
    type_keystroke_by_string(ctx, "<B><B>k");
    ok_bopomofo_buffer(ctx, "\xE3\x84\x95\xE3\x84\xA4" /* ㄕㄤ */ );
    chewing_clean_preedit_buf(ctx);

    chewing_delete(ctx);
}

void test_KB_ET26()
{
    ChewingContext *ctx;
    clean_userphrase();

    ctx = chewing_new();
    start_testcase(ctx, fd);

    chewing_set_KBType(ctx, KB_ET26);

    type_keystroke_by_string(ctx, "cen kxken gn drdhnjbtk");
    ok_preedit_buffer(ctx, "\xE6\x96\xB0\xE9\x85\xB7\xE9\x9F\xB3\xE7\x9C\x9F\xE7\x9A\x84\xE5\xBE\x88\xE6\xA3\x92"
                      /* 新酷音真的很棒 */ );
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

void test_KB_ET26_choice_append()
{
    const TestData CHOICE_INFO_APPEND[] = {
        { "p " /* ㄡ */, "\xE3\x84\x86" /* ㄆ */ },
        { "t " /* ㄤ */, "\xE3\x84\x8A" /* ㄊ */ },
        { "w " /* ㄘ */, "\xE3\x84\x9D" /* ㄝ */ },
        { "g " /* ㄓ */, "\xE3\x84\x90" /* ㄐ */ },
        { "h " /* ㄦ */, "\xE3\x84\x8F" /* ㄏ */ },
        { "l " /* ㄥ */, "\xE3\x84\x8C" /* ㄌ */ },
        { "c " /* ㄕ */, "\xE3\x84\x92" /* ㄒ */ },
        { "n " /* ㄣ */, "\xE3\x84\x8B" /* ㄋ */ },
        { "m " /* ㄢ */, "\xE3\x84\x87" /* ㄇ */ },
        { "d " /* ㄉ */, "\xCB\x99" /* ˙ */ },
        { "f " /* ㄈ */, "\xCB\x8A" /* ˊ */ },
        { "j " /* ㄖ */, "\xCB\x87" /* ˇ */ },
        { "k " /* ㄎ */, "\xCB\x8B" /* ˋ */ },
        { "q " /* ㄗ */, "\xE3\x84\x9F" /* ㄟ */ },
        { "v " /* ㄍ */, "\xE3\x84\x91" /* ㄑ */ },
    };

    size_t i;
    ChewingContext *ctx;
    int totalChoice;
    const char *cand;
    clean_userphrase();

    ctx = chewing_new();
    start_testcase(ctx, fd);
    chewing_set_KBType(ctx, KB_ET26);

    for (i = 0; i < ARRAY_SIZE(CHOICE_INFO_APPEND); ++i) {

        type_keystroke_by_string(ctx, CHOICE_INFO_APPEND[i].token);

        chewing_cand_open(ctx);
        totalChoice = chewing_cand_TotalChoice(ctx);

        if (i == 13 || i == 14) {
            cand = chewing_cand_string_by_index_static(ctx, totalChoice - 2);
        } else {
            cand = chewing_cand_string_by_index_static(ctx, totalChoice - 1);
        }

        ok(strcmp(cand, CHOICE_INFO_APPEND[i].expected) == 0, "returned candidate is `%s' shall be `%s'", cand, CHOICE_INFO_APPEND[i].expected);

        chewing_cand_close(ctx);
        chewing_clean_preedit_buf(ctx);
    }
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

    type_keystroke_by_string(ctx, "xin");
    ok_bopomofo_buffer(ctx, "xin");

    type_keystroke_by_string(ctx, "<EE>");
    ok_bopomofo_buffer(ctx, "");

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

void test_KB_HANYU_direct_symbol_output()
{
    ChewingContext *ctx;

    ctx = chewing_new();
    start_testcase(ctx, fd);

    chewing_set_KBType(ctx, KB_HANYU_PINYIN);

    type_keystroke_by_string(ctx, "pin yin  123 mo2shi4");
    ok_preedit_buffer(ctx, "拼音 123 模式");
    chewing_clean_preedit_buf(ctx);

    chewing_set_KBType(ctx, KB_HANYU_PINYIN);
    chewing_set_ShapeMode(ctx, FULLSHAPE_MODE);

    type_keystroke_by_string(ctx, "pin yin  123 mo2shi4");
    ok_preedit_buffer(ctx, "拼音　１２３　模式");
    chewing_clean_preedit_buf(ctx);

    chewing_delete(ctx);
}

void test_KB_THL()
{
    ChewingContext *ctx;

    ctx = chewing_new();
    start_testcase(ctx, fd);

    chewing_set_KBType(ctx, KB_THL_PINYIN);

    type_keystroke_by_string(ctx, "sin");
    ok_bopomofo_buffer(ctx, "sin");

    type_keystroke_by_string(ctx, "<EE>");
    ok_bopomofo_buffer(ctx, "");

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

    type_keystroke_by_string(ctx, "shin");
    ok_bopomofo_buffer(ctx, "shin");

    type_keystroke_by_string(ctx, "<EE>");
    ok_bopomofo_buffer(ctx, "");

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

void test_KB_DVORAK()
{
    ChewingContext *ctx;

    ctx = chewing_new();
    start_testcase(ctx, fd);

    chewing_set_KBType(ctx, KB_DVORAK);
    type_keystroke_by_string(ctx, "kgl eh4gl 5l 2t7jl31s4");
    chewing_set_ChiEngMode(ctx, SYMBOL_MODE);
    type_keystroke_by_string(ctx, "testTEST");
    ok_preedit_buffer(ctx, "\xE6\x96\xB0\xE9\x85\xB7\xE9\x9F\xB3\xE7\x9C\x9F\xE7\x9A\x84\xE5\xBE\x88\xE6\xA3\x92testTEST"
                      /* 新酷音真的很棒 */ );
    chewing_clean_preedit_buf(ctx);

    chewing_delete(ctx);
}

void test_KB_DVORAK_HSU()
{
    ChewingContext *ctx;

    ctx = chewing_new();
    start_testcase(ctx, fd);

    chewing_set_KBType(ctx, KB_DVORAK_HSU);
    type_keystroke_by_string(ctx, "idl vbcdl cl hu;jlynvc");
    chewing_set_ChiEngMode(ctx, SYMBOL_MODE);
    type_keystroke_by_string(ctx, "kd;kKD:K");
    ok_preedit_buffer(ctx, "\xE6\x96\xB0\xE9\x85\xB7\xE9\x9F\xB3\xE7\x9C\x9F\xE7\x9A\x84\xE5\xBE\x88\xE6\xA3\x92testTEST"
                      /* 新酷音真的很棒 */ );
    chewing_clean_preedit_buf(ctx);

    chewing_delete(ctx);
}

void test_KB_COLEMAK()
{
    ChewingContext *ctx;

    ctx = chewing_new();
    start_testcase(ctx, fd);

    chewing_set_KBType(ctx, KB_COLEMAK);
    type_keystroke_by_string(ctx, "vl; sn4l; 5; 2e7c;31o4");
    ok_preedit_buffer(ctx, "\xE6\x96\xB0\xE9\x85\xB7\xE9\x9F\xB3\xE7\x9C\x9F\xE7\x9A\x84\xE5\xBE\x88\xE6\xA3\x92"
                      /* 新酷音真的很棒 */ );
    chewing_clean_preedit_buf(ctx);

    chewing_delete(ctx);
}

void test_KB_COLEMAK_DH_ANSI()
{
    ChewingContext *ctx;

    ctx = chewing_new();
    start_testcase(ctx, fd);

    chewing_set_KBType(ctx, KB_COLEMAK_DH_ANSI);
    type_keystroke_by_string(ctx, "vl; sn4l; 5; 2e7d;31o4");
    ok_preedit_buffer(ctx, "\xE6\x96\xB0\xE9\x85\xB7\xE9\x9F\xB3\xE7\x9C\x9F\xE7\x9A\x84\xE5\xBE\x88\xE6\xA3\x92"
                      /* 新酷音真的很棒 */ );
    chewing_clean_preedit_buf(ctx);

    chewing_delete(ctx);
}


void test_KB_COLEMAK_DH_ORTH()
{
    ChewingContext *ctx;

    ctx = chewing_new();
    start_testcase(ctx, fd);

    chewing_set_KBType(ctx, KB_COLEMAK_DH_ORTH);
    type_keystroke_by_string(ctx, "dl; sn4l; 5; 2e7c;31o4");
    ok_preedit_buffer(ctx, "\xE6\x96\xB0\xE9\x85\xB7\xE9\x9F\xB3\xE7\x9C\x9F\xE7\x9A\x84\xE5\xBE\x88\xE6\xA3\x92"
                      /* 新酷音真的很棒 */ );
    chewing_clean_preedit_buf(ctx);

    chewing_delete(ctx);
}


void test_KB()
{
    test_KB_HSU();
    test_KB_HSU_example();
    test_KB_HSU_choice_append();
    test_KB_HSU_choice_append_select();
    test_KB_HSU_JVC();
    test_KB_ET26();
    test_KB_ET26_choice_append();
    test_KB_DACHEN_CP26();
    test_KB_DVORAK();
    test_KB_DVORAK_HSU();
    test_KB_COLEMAK();
    test_KB_COLEMAK_DH_ANSI();
    test_KB_COLEMAK_DH_ORTH();

    test_KB_HANYU();
    test_KB_HANYU_direct_symbol_output();
    test_KB_THL();
    test_KB_MPS2();
}

void test_chewing_phone_to_bopomofo()
{
    char *u8phone;
    char *rt;
    int  expected_len;
    int  len;
    uint16_t phone;
    uint16_t expect;
    /*
     *  the libchewing divides a completed bopomofo into 4 parts,
     *      1st part: ㄅㄆㄇㄈㄉㄊㄋㄌㄍㄎㄏㄐㄑㄒㄓㄔㄕㄖㄗㄘㄙ
     *      2nd part: ㄧㄨㄩ
     *      3rd part: ㄚㄛㄜㄝㄞㄟㄠㄡㄢㄣㄤㄥㄦ
     *      4th part:  ˙ˊˇˋ
     *
     *  calculates each part's offset and stores into a 16-bit unsigned by following rule:
     *  16-bit unsinged = ( 1st part offset )<<9 + ( 2nd part offset )<<7 + ( 3rd part offset )<<3 + (4th part offset),
     *
     *  e.g., ㄆㄣ, 1st part offset = 2, 2nd part offset = 0, 3rd part offset = 10, 4th part offset = 0,
     *  so the number for ㄆㄣ is (2<<9)+(0<<7)+(10<<3)+(0) = 1104
     */

    start_testcase(NULL, fd);

    u8phone = "\xE3\x84\x86\xE3\x84\xA3" /* ㄆㄣ */ ;
    phone = UintFromPhone(u8phone);
    expect = (2 << 9) + (0 << 7) + (10 << 3) + (0);
    ok(phone == expect, "UintFromPhone `%s' shall be `%d', got `%d'", u8phone, expect, phone);

    expected_len = strlen(u8phone) + 1;
    len = chewing_phone_to_bopomofo(expect, NULL, 0);
    ok(len == expected_len, "chewing_phone_to_bopomofo returns `%d' shall be `%d'", len, expected_len);
    rt = calloc(sizeof(char), len);
    chewing_phone_to_bopomofo(expect, rt, len);
    ok(strcmp(rt, u8phone) == 0, "PhoneFromUint d%d' shall be `%s', got `%s'", expect, u8phone, rt);
    free(rt);

    u8phone = "\xE3\x84\x8A\xE3\x84\xA7\xE3\x84\xA2" /* ㄊㄧㄢ */ ;
    phone = UintFromPhone(u8phone);
    expect = (6 << 9) + (1 << 7) + (9 << 3) + (0);
    ok(phone == expect, "UintFromPhone `%s' shall be `%d', got `%d'", u8phone, expect, phone);

    expected_len = strlen(u8phone) + 1;
    len = chewing_phone_to_bopomofo(expect, NULL, 0);
    ok(len == expected_len, "chewing_phone_to_bopomofo returns `%d' shall be `%d'", len, expected_len);
    rt = calloc(sizeof(char), len);
    chewing_phone_to_bopomofo(expect, rt, len);
    ok(strcmp(rt, u8phone) == 0, "PhoneFromUint d%d' shall be `%s', got `%s'", expect, u8phone, rt);
    free(rt);

    u8phone = "\xE3\x84\x92\xE3\x84\xA7\xE3\x84\x9A\xCB\x8B" /* ㄒㄧㄚˋ */ ;
    phone = UintFromPhone(u8phone);
    expect = (14 << 9) + (1 << 7) + (1 << 3)+ (4);
    ok(phone == expect, "UintFromPhone `%s' shall be `%d', got `%d'", u8phone, expect, phone);

    expected_len = strlen(u8phone) + 1;
    len = chewing_phone_to_bopomofo(expect, NULL, 0);
    ok(len == expected_len, "chewing_phone_to_bopomofo returns `%d' shall be `%d'", len, expected_len);
    rt = calloc(sizeof(char), len);
    chewing_phone_to_bopomofo(expect, rt, len);
    ok(strcmp(rt, u8phone) == 0, "PhoneFromUint `%d' shall be `%s', got `%s'", expect, u8phone, rt);
    free(rt);

    len = chewing_phone_to_bopomofo(0, NULL, 0);
    ok(len == -1, "chewing_phone_to_bopomofo returns `%d' shall be `%d'", len, -1);
}

void test_static_buffer_reuse()
{
    ChewingContext *ctx;
    const char *buf[6];

    clean_userphrase();

    ctx = chewing_new();
    start_testcase(ctx, fd);

    type_keystroke_by_string(ctx, "hk4g4ggg");
    ok_preedit_buffer(ctx, "測試");
    chewing_cand_Enumerate(ctx);
    chewing_kbtype_Enumerate(ctx);

    buf[0] = chewing_commit_String_static(ctx);
    buf[1] = chewing_buffer_String_static(ctx);
    buf[2] = chewing_bopomofo_String_static(ctx);
    buf[3] = chewing_cand_String_static(ctx);
    buf[4] = chewing_aux_String_static(ctx);
    buf[5] = chewing_kbtype_String_static(ctx);

    for (int i = 0; i < 6; ++i) {
        for (int j = 0; j < 6; ++j) {
            if (i == j) continue;
            ok(buf[i] != buf[j], "static buf[%d] != buf[%d]", i, j);
        }
    }

    chewing_delete(ctx);
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
    test_ShiftSpaceDisabled();
    test_Numlock();
    test_Space();
    test_FuzzySearchMode();
    test_FuzzySearchMode_Hanyu();
    test_SimpleEngine();
    test_Acknowledge();

    test_get_phoneSeq();
    test_bopomofo_buffer();

    test_longest_phrase();
    test_auto_commit();

    test_interval();

    test_jk_selection();

    test_KB();

    test_chewing_phone_to_bopomofo();

    test_static_buffer_reuse();

    fclose(fd);

    return exit_status();
}
