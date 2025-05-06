/**
 * test-regression.c
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
#include <stdio.h>
#include <stdlib.h>

#include "chewing.h"
#include "testhelper.h"

void test_libchewing_googlecode_issue_472()
{
    static const char *const INPUT[] = {
        "<T>|&Wt<H>mrJY)G<C2>OqJ<H><H>Yl<R>p0<EE>QE[^<C1>k",
        "+F<C9>hQ$UIICMr!X8/9<C3>(N<T>yU2!-LUI<D>`CS<D>jShm9SF}<EN>[`QYu<C8>k",
        "hk4`2<D>jk",
        "hk4`j 0",
        "hk4<C0>j 0",
    };
    size_t i;
    ChewingContext *ctx;

    ctx = chewing_new();
    start_testcase(ctx);
    chewing_set_maxChiSymbolLen(ctx, 16);
    chewing_set_autoShiftCur(ctx, 1);

    for (i = 0; i < ARRAY_SIZE(INPUT); ++i) {
        chewing_Reset(ctx);
        type_keystroke_by_string(ctx, INPUT[i]);
    }

    chewing_delete(ctx);
}

void test_libchewing_googlecode_issue_473()
{
    static const char *const INPUT[] = {
        "t<N->_ M1<N+>H[Ls3<L><N1>PL+Z]4<C1>&(^H*H<TT>Sc<N->P]!|<CB>-<C6>S<H><N1><C0>U<B>d}P!f<EN><N.><C7>V!U!w|4-=S<C1>b<N2>Q",
        "wv<C0><C5><N9>$FIF<D><N4>B *<C2>E4*<C2>q)Kf)<SS><TT>4=<N5>%<R>mN4<EN>H<N9><N.>8s{XTD<N6>jZV(y3G`9<C6>JTy<B>J<C1>SNc<E>hC<SL><N/><R><C6>@an<C3><N7>wzF<C3>P*<N*><B>l<C3><N6>W<N*> $<SR><N.><N1><E><E><N0><N6>Y",
    };
    size_t i;
    ChewingContext *ctx;

    ctx = chewing_new();
    start_testcase(ctx);
    chewing_set_maxChiSymbolLen(ctx, 16);
    chewing_set_autoShiftCur(ctx, 1);
    chewing_set_candPerPage(ctx, 9);
    chewing_set_addPhraseDirection(ctx, 1);
    chewing_set_spaceAsSelection(ctx, 1);

    for (i = 0; i < ARRAY_SIZE(INPUT); ++i) {
        chewing_Reset(ctx);
        type_keystroke_by_string(ctx, INPUT[i]);
    }

    chewing_delete(ctx);
}

void test_libchewing_issue_30()
{
    ChewingContext *ctx;
    int cursor;

    clean_userphrase();

    ctx = chewing_new();
    start_testcase(ctx);
    chewing_set_maxChiSymbolLen(ctx, 16);
    chewing_set_autoShiftCur(ctx, 1);
    chewing_set_spaceAsSelection(ctx, 1);
    chewing_set_phraseChoiceRearward(ctx, 1);

    type_keystroke_by_string(ctx, "hk4g4<H> 3 1");
    cursor = chewing_cursor_Current(ctx);
    ok(cursor == 2, "cursor position `%d' shall be `2'", cursor);

    chewing_delete(ctx);
}

void test_libchewing_issue_108()
{
    ChewingContext *ctx;

    clean_userphrase();

    ctx = chewing_new();
    start_testcase(ctx);

    type_keystroke_by_string(ctx, "yjo4cl3183<E>");

    chewing_delete(ctx);
}

void test_libchewing_issue_194()
{
    ChewingContext *ctx;

    clean_userphrase();

    ctx = chewing_new();
    start_testcase(ctx);

    chewing_set_ChiEngMode(ctx, SYMBOL_MODE);
    type_keystroke_by_string(ctx, "test");
    chewing_set_ChiEngMode(ctx, CHINESE_MODE);

    ok_commit_buffer(ctx, "t");

    chewing_delete(ctx);
}

void test_libchewing_data_issue_1()
{
    const TestData DATA = { "e03y.3", "\xE8\xB6\x95\xE8\xB5\xB0" /* 趕走 */  };
    ChewingContext *ctx;

    clean_userphrase();

    ctx = chewing_new();
    start_testcase(ctx);
    chewing_set_maxChiSymbolLen(ctx, 16);
    type_keystroke_by_string(ctx, DATA.token);
    ok_preedit_buffer(ctx, DATA.expected);

    chewing_delete(ctx);
}

void test_forgot_selection()
{
    ChewingContext *ctx;

    clean_userphrase();

    ctx = chewing_new();
    start_testcase(ctx);

    chewing_set_escCleanAllBuf(ctx, 1);

    type_keystroke_by_string(ctx, "<EE>hk4g4<L><L><D>3<R><R>g4");
    ok_preedit_buffer(ctx, "策士市");

    type_keystroke_by_string(ctx, "<EE>hk4g4<L><L><D>2<R><R>g4");
    ok_preedit_buffer(ctx, "策試市");

    chewing_delete(ctx);
}

void test_move_cursor_backwards()
{
    ChewingContext *ctx;

    clean_userphrase();

    ctx = chewing_new();
    start_testcase(ctx);

    type_keystroke_by_string(ctx, "hk4g4<L>hk4g4");
    ok_preedit_buffer(ctx, "冊測試市");

    chewing_delete(ctx);
}

void test_insert_symbol_between_selection()
{
    ChewingContext *ctx;

    clean_userphrase();

    ctx = chewing_new();
    start_testcase(ctx);

    type_keystroke_by_string(ctx, "hk4g4<L><L><D>3<R>?");
    ok_preedit_buffer(ctx, "冊？市");

    chewing_delete(ctx);
}

void test_empty_prefix_in_conversion_search()
{
    ChewingContext *ctx;

    clean_userphrase();

    ctx = chewing_new();
    start_testcase(ctx);

    type_keystroke_by_string(ctx, "hk4g4hk4g4<T><T><B><B><B><B><E>");
    ok_preedit_buffer(ctx, "");

    chewing_delete(ctx);
}

void test_empty_preedit_ignore_certain_keys()
{
    static const char *const KEYS[] = {
        "<EE>",
        "<E>",
        "<DC>",
        "<B>",
        "<T>",
        "<TT>",
        "<L>",
        "<R>",
        "<D>",
        "<U>",
        "<H>",
        "<EN>",
        "<PU>",
        "<PD>"
    };

    ChewingContext *ctx;
    int ret;

    clean_userphrase();

    ctx = chewing_new();
    start_testcase(ctx);

    for (int i = 0; i < ARRAY_SIZE(KEYS); ++i) {
        type_keystroke_by_string(ctx, KEYS[i]);
        ret = chewing_keystroke_CheckIgnore(ctx);
        ok(ret == 1, "%s key should be ignored", KEYS[i]);
        ret = chewing_keystroke_CheckAbsorb(ctx);
        ok(ret == 0, "%s key should not be absorbed", KEYS[i]);
        ret = chewing_commit_Check(ctx);
        ok(ret == 0, "%s key should not trigger commit", KEYS[i]);
    }

    chewing_delete(ctx);
}

void test_crash_found_by_fuzzing_20240505_0()
{
    ChewingContext *ctx;

    clean_userphrase();

    ctx = chewing_new();
    start_testcase(ctx);

    type_keystroke_by_string(ctx, "93<D>093<D>0<H><D>2");
    ok_preedit_buffer(ctx, "靄靄");

    chewing_delete(ctx);
}

void test_glue_two_symbols()
{
    ChewingContext *ctx;

    clean_userphrase();

    ctx = chewing_new();
    start_testcase(ctx);

    chewing_config_set_int(ctx, "chewing.conversion_engine", 2);

    type_keystroke_by_string(ctx, "!!<L><T><L>");
    ok_preedit_buffer(ctx, "！！");

    chewing_delete(ctx);
}

void test_end_of_buffer_select_phrase_backwards()
{
    ChewingContext *ctx;

    clean_userphrase();

    ctx = chewing_new();
    start_testcase(ctx);

    chewing_set_spaceAsSelection(ctx, 1);
    chewing_set_phraseChoiceRearward(ctx, 1);

    type_keystroke_by_string(ctx, "0  0         0");
    ok_preedit_buffer(ctx, "鵪");

    chewing_delete(ctx);
}

void test_zero_capacity_buffer_simple_conversion_engine()
{
    ChewingContext *ctx;

    clean_userphrase();

    ctx = chewing_new();
    start_testcase(ctx);

    chewing_set_KBType(ctx, 1);
    chewing_set_maxChiSymbolLen(ctx, 0);
    chewing_config_set_int(ctx, "chewing.conversion_engine", 0);

    type_keystroke_by_string(ctx, "x 0");
    ok_commit_buffer(ctx, "鄔");

    chewing_delete(ctx);
}

int main(int argc, char *argv[])
{
    putenv("CHEWING_PATH=" CHEWING_DATA_PREFIX);
    putenv("CHEWING_USER_PATH=" TEST_HASH_DIR);

    test_libchewing_data_issue_1();
    test_libchewing_issue_30();
    test_libchewing_issue_108();
    test_libchewing_issue_194();
    test_libchewing_googlecode_issue_472();
    test_libchewing_googlecode_issue_473();
    test_forgot_selection();
    test_move_cursor_backwards();
    test_insert_symbol_between_selection();
    test_empty_prefix_in_conversion_search();
    test_empty_preedit_ignore_certain_keys();
    test_crash_found_by_fuzzing_20240505_0();
    test_glue_two_symbols();
    test_end_of_buffer_select_phrase_backwards();
    test_zero_capacity_buffer_simple_conversion_engine();

    return exit_status();
}
