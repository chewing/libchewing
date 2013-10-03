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
#include <config.h>
#endif

#include <stdlib.h>
#include <stdio.h>
#include <string.h>

#include "chewing.h"
#include "plat_types.h"
#include "hash-private.h"
#include "testhelper.h"

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
		"\xE4\xB8\x80\xE4\xB8\x8A\xE4\xBE\x86" /* 一上來 */,
		"\xE7\xA7\xBB\xE4\xB8\x8A\xE4\xBE\x86" /* 移上來 */,
	};

	static const char *CAND_2[] = {
		"\xE7\xA7\xBB\xE4\xB8\x8A" /* 移上 */,
	};

	ChewingContext *ctx;

	print_function_name();

	clean_userphrase();

	ctx = chewing_new();

	chewing_set_maxChiSymbolLen( ctx, 16 );

	type_keystroke_by_string( ctx, "u6g;4x96<L><L><L>" ); /* ㄧˊㄕㄤˋㄌㄞˊ */

	type_keystroke_by_string( ctx, "<D>" ); /* ㄧˊㄕㄤˋㄌㄞˊ */
	ok_candidate( ctx, CAND_1, ARRAY_SIZE( CAND_1 ) );

	type_keystroke_by_string( ctx, "<D>" ); /* ㄕㄤˋㄌㄞˊ */
	ok_candidate( ctx, CAND_2, ARRAY_SIZE( CAND_2 ) );

	type_keystroke_by_string( ctx, "<D><D>2<E>" ); /* select 移上來 */
	ok_commit_buffer( ctx, CAND_1[1] );

	chewing_delete( ctx );
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
		"\xE4\xB8\x80\xE4\xB8\x8A\xE4\xBE\x86" /* 一上來 */,
		"\xE7\xA7\xBB\xE4\xB8\x8A\xE4\xBE\x86" /* 移上來 */,
	};

	static const char *CAND_2[] = {
		"\xE4\xB8\x8A\xE4\xBE\x86" /* 上來 */,
	};
	ChewingContext *ctx;

	print_function_name();

	clean_userphrase();

	ctx = chewing_new();

	chewing_set_maxChiSymbolLen( ctx, 16 );
	chewing_set_phraseChoiceRearward( ctx, 1 );

	type_keystroke_by_string( ctx, "u6g;4x96" ); /* ㄧˊㄕㄤˋㄌㄞˊ */
	ok_preedit_buffer( ctx, CAND_1[0] );

	type_keystroke_by_string( ctx, "<D>" ); /* ㄧˊㄕㄤˋㄌㄞˊ */
	ok_candidate( ctx, CAND_1, ARRAY_SIZE( CAND_1 ) );

	type_keystroke_by_string( ctx, "<D>" ); /* ㄕㄤˋㄌㄞˊ */
	ok_candidate( ctx, CAND_2, ARRAY_SIZE( CAND_2 ) );

	type_keystroke_by_string( ctx, "<D><D>2<E>" ); /* select 移上來 */
	ok_commit_buffer( ctx, CAND_1[1] );

	chewing_delete( ctx );
}

void test_select_candidate_no_rearward_with_symbol()
{
	ChewingContext *ctx;
	int ret;
	char *buf;
	int len;

	print_function_name();

	clean_userphrase();

	ctx = chewing_new();

	type_keystroke_by_string( ctx, "hk4g4`31u6vu84" /* 測試，一下 */);

	type_keystroke_by_string( ctx, "<EE><H><D>" );
	ret = chewing_cand_TotalChoice( ctx );
	ok( ret > 0, "chewing_cand_TotalChoice() returns `%d' shall greater than 0 at pos `%d'", ret, 0 );
	chewing_cand_Enumerate( ctx );
	buf = chewing_cand_String( ctx );
	len = ueStrLen( buf );
	ok( len == 2, "candidate `%s' length `%d' shall be `%d' at pos `%d'", buf, len, 2, 0 );
	chewing_free( buf );

	type_keystroke_by_string( ctx, "<EE><H><R><D>" );
	ret = chewing_cand_TotalChoice( ctx );
	ok( ret > 0, "chewing_cand_TotalChoice() returns `%d' shall greater than 0 at pos `%d'", ret, 1 );
	chewing_cand_Enumerate( ctx );
	buf = chewing_cand_String( ctx );
	len = ueStrLen( buf );
	ok( len == 1, "candidate `%s' length `%d' shall be `%d' at pos `%d'", buf, len, 1, 1 );
	chewing_free( buf );

	type_keystroke_by_string( ctx, "<EE><H><R><R><D>" );
	ret = chewing_cand_TotalChoice( ctx );
	ok( ret > 0, "chewing_cand_TotalChoice() returns `%d' shall greater than 0 at pos `%d'", ret, 2 );
	chewing_cand_Enumerate( ctx );
	buf = chewing_cand_String( ctx );
	len = ueStrLen( buf );
	ok( len == 1, "candidate `%s' length `%d' shall be `%d' at pos `%d'", buf, len, 1, 2 );
	chewing_free( buf );

	type_keystroke_by_string( ctx, "<EE><H><R><R><R><D>" );
	ret = chewing_cand_TotalChoice( ctx );
	ok( ret > 0, "chewing_cand_TotalChoice() returns `%d' shall greater than 0 at pos `%d'", ret, 3 );
	chewing_cand_Enumerate( ctx );
	buf = chewing_cand_String( ctx );
	len = ueStrLen( buf );
	ok( len == 2, "candidate `%s' length `%d' shall be `%d' at pos `%d'", buf, len, 2, 3 );
	chewing_free( buf );

	type_keystroke_by_string( ctx, "<EE><H><R><R><R><R><D>" );
	ret = chewing_cand_TotalChoice( ctx );
	ok( ret > 0, "chewing_cand_TotalChoice() returns `%d' shall greater than 0 at pos `%d'", ret, 4 );
	chewing_cand_Enumerate( ctx );
	buf = chewing_cand_String( ctx );
	len = ueStrLen( buf );
	ok( len == 1, "candidate `%s' length `%d' shall be `%d' at pos `%d'", buf, len, 1, 4 );
	chewing_free( buf );

	chewing_delete( ctx );
}

void test_select_candidate_rearward_with_symbol()
{
	ChewingContext *ctx;
	int ret;
	char *buf;
	int len;

	print_function_name();

	clean_userphrase();

	ctx = chewing_new();
	chewing_set_phraseChoiceRearward( ctx, 1 );

	type_keystroke_by_string( ctx, "hk4g4`31u6vu84" /* 測試，一下 */);

	type_keystroke_by_string( ctx, "<EE><H><D>" );
	ret = chewing_cand_TotalChoice( ctx );
	ok( ret > 0, "chewing_cand_TotalChoice() returns `%d' shall greater than 0 at pos `%d'", ret, 0 );
	chewing_cand_Enumerate( ctx );
	buf = chewing_cand_String( ctx );
	len = ueStrLen( buf );
	ok( len == 1, "candidate `%s' length `%d' shall be `%d' at pos `%d'", buf, len, 1, 0 );
	chewing_free( buf );

	type_keystroke_by_string( ctx, "<EE><H><R><D>" );
	ret = chewing_cand_TotalChoice( ctx );
	ok( ret > 0, "chewing_cand_TotalChoice() returns `%d' shall greater than 0 at pos `%d'", ret, 1 );
	chewing_cand_Enumerate( ctx );
	buf = chewing_cand_String( ctx );
	len = ueStrLen( buf );
	ok( len == 2, "candidate `%s' length `%d' shall be `%d' at pos `%d'", buf, len, 2, 1 );
	chewing_free( buf );

	type_keystroke_by_string( ctx, "<EE><H><R><R><D>" );
	ret = chewing_cand_TotalChoice( ctx );
	ok( ret > 0, "chewing_cand_TotalChoice() returns `%d' shall greater than 0 at pos `%d'", ret, 2 );
	chewing_cand_Enumerate( ctx );
	buf = chewing_cand_String( ctx );
	len = ueStrLen( buf );
	ok( len == 1, "candidate `%s' length `%d' shall be `%d' at pos `%d'", buf, len, 1, 2 );
	chewing_free( buf );

	type_keystroke_by_string( ctx, "<EE><H><R><R><R><D>" );
	ret = chewing_cand_TotalChoice( ctx );
	ok( ret > 0, "chewing_cand_TotalChoice() returns `%d' shall greater than 0 at pos `%d'", ret, 3 );
	chewing_cand_Enumerate( ctx );
	buf = chewing_cand_String( ctx );
	len = ueStrLen( buf );
	ok( len == 1, "candidate `%s' length `%d' shall be `%d' at pos `%d'", buf, len, 1, 3 );
	chewing_free( buf );

	type_keystroke_by_string( ctx, "<EE><H><R><R><R><R><D>" );
	ret = chewing_cand_TotalChoice( ctx );
	ok( ret > 0, "chewing_cand_TotalChoice() returns `%d' shall greater than 0 at pos `%d'", ret, 4 );
	chewing_cand_Enumerate( ctx );
	buf = chewing_cand_String( ctx );
	len = ueStrLen( buf );
	ok( len == 2, "candidate `%s' length `%d' shall be `%d' at pos `%d'", buf, len, 2, 4 );
	chewing_free( buf );

	chewing_delete( ctx );
}

void test_select_candidate_no_rearward_start_with_symbol()
{
	ChewingContext *ctx;
	int ret;
	char *buf;
	int len;

	print_function_name();

	clean_userphrase();

	ctx = chewing_new();

	type_keystroke_by_string( ctx, "`31hk4g4" /* ，測試 */);

	type_keystroke_by_string( ctx, "<EE><H><D>" );
	ret = chewing_cand_TotalChoice( ctx );
	ok( ret > 0, "chewing_cand_TotalChoice() returns `%d' shall greater than 0 at pos `%d'", ret, 0 );
	chewing_cand_Enumerate( ctx );
	buf = chewing_cand_String( ctx );
	len = ueStrLen( buf );
	ok( len == 1, "candidate `%s' length `%d' shall be `%d' at pos `%d'", buf, len, 1, 0 );
	chewing_free( buf );

	type_keystroke_by_string( ctx, "<EE><H><R><D>" );
	ret = chewing_cand_TotalChoice( ctx );
	ok( ret > 0, "chewing_cand_TotalChoice() returns `%d' shall greater than 0 at pos `%d'", ret, 1 );
	chewing_cand_Enumerate( ctx );
	buf = chewing_cand_String( ctx );
	len = ueStrLen( buf );
	ok( len == 2, "candidate `%s' length `%d' shall be `%d' at pos `%d'", buf, len, 2, 1 );
	chewing_free( buf );

	type_keystroke_by_string( ctx, "<EE><H><R><R><D>" );
	ret = chewing_cand_TotalChoice( ctx );
	ok( ret > 0, "chewing_cand_TotalChoice() returns `%d' shall greater than 0 at pos `%d'", ret, 2 );
	chewing_cand_Enumerate( ctx );
	buf = chewing_cand_String( ctx );
	len = ueStrLen( buf );
	ok( len == 1, "candidate `%s' length `%d' shall be `%d' at pos `%d'", buf, len, 1, 2 );
	chewing_free( buf );

	chewing_delete( ctx );
}

void test_select_candidate_rearward_start_with_symbol()
{
	ChewingContext *ctx;
	int ret;
	char *buf;
	int len;

	print_function_name();

	clean_userphrase();

	ctx = chewing_new();
	chewing_set_phraseChoiceRearward( ctx, 1 );

	type_keystroke_by_string( ctx, "`31hk4g4" /* ，測試 */);

	type_keystroke_by_string( ctx, "<EE><H><D>" );
	ret = chewing_cand_TotalChoice( ctx );
	ok( ret > 0, "chewing_cand_TotalChoice() returns `%d' shall greater than 0 at pos `%d'", ret, 0 );
	chewing_cand_Enumerate( ctx );
	buf = chewing_cand_String( ctx );
	len = ueStrLen( buf );
	ok( len == 1, "candidate `%s' length `%d' shall be `%d' at pos `%d'", buf, len, 1, 0 );
	chewing_free( buf );

	type_keystroke_by_string( ctx, "<EE><H><R><D>" );
	ret = chewing_cand_TotalChoice( ctx );
	ok( ret > 0, "chewing_cand_TotalChoice() returns `%d' shall greater than 0 at pos `%d'", ret, 1 );
	chewing_cand_Enumerate( ctx );
	buf = chewing_cand_String( ctx );
	len = ueStrLen( buf );
	ok( len == 1, "candidate `%s' length `%d' shall be `%d' at pos `%d'", buf, len, 1, 1 );
	chewing_free( buf );

	type_keystroke_by_string( ctx, "<EE><H><R><R><D>" );
	ret = chewing_cand_TotalChoice( ctx );
	ok( ret > 0, "chewing_cand_TotalChoice() returns `%d' shall greater than 0 at pos `%d'", ret, 2 );
	chewing_cand_Enumerate( ctx );
	buf = chewing_cand_String( ctx );
	len = ueStrLen( buf );
	ok( len == 2, "candidate `%s' length `%d' shall be `%d' at pos `%d'", buf, len, 2, 2 );
	chewing_free( buf );

	chewing_delete( ctx );
}

void test_del_bopomofo_as_mode_switch()
{
	ChewingContext *ctx;

	print_function_name();

	clean_userphrase();

	ctx = chewing_new();

	type_keystroke_by_string( ctx, "2k" ); /* ㄉㄜ */
	ok_zuin_buffer( ctx, "\xe3\x84\x89\xe3\x84\x9c" /* ㄉㄜ */ );

	chewing_set_ChiEngMode( ctx, SYMBOL_MODE );
	ok_zuin_buffer( ctx, "" );

	chewing_delete( ctx );
}

void test_select_candidate_4_bytes_utf8()
{
	ChewingContext *ctx;

	print_function_name();

	clean_userphrase();

	ctx = chewing_new();

	chewing_set_maxChiSymbolLen( ctx, 16 );
	chewing_set_phraseChoiceRearward( ctx, 1 );
	chewing_set_autoShiftCur( ctx, 1 );

	type_keystroke_by_string( ctx, "2k62k6" ); /* ㄉㄜˊ ㄉㄜˊ */
	ok_preedit_buffer( ctx, "\xE5\xBE\x97\xE5\xBE\x97" /* 得得 */ );

	type_keystroke_by_string( ctx, "<H>" );

	type_keystroke_by_string( ctx, "<D>8" );
	ok_preedit_buffer( ctx, "\xF0\xA2\x94\xA8\xE5\xBE\x97" /* 𢔨得 */ );

	type_keystroke_by_string( ctx, "<D>8" );

	ok_preedit_buffer( ctx, "\xF0\xA2\x94\xA8\xF0\xA2\x94\xA8" /* 𢔨𢔨 */ );

	chewing_delete( ctx );
}

void test_select_candidate() {
	test_select_candidate_no_rearward();
	test_select_candidate_rearward();
	test_select_candidate_no_rearward_with_symbol();
	test_select_candidate_rearward_with_symbol();
	test_select_candidate_no_rearward_start_with_symbol();
	test_select_candidate_rearward_start_with_symbol();
	test_select_candidate_4_bytes_utf8();
	test_del_bopomofo_as_mode_switch();
}

void test_Esc_not_entering_chewing()
{
	ChewingContext *ctx;

	print_function_name();

	ctx = chewing_new();
	type_keystroke_by_string( ctx, "<EE>" );
	ok_keystroke_rtn( ctx, KEYSTROKE_IGNORE );

	chewing_delete( ctx );
}

void test_Esc_in_select()
{
	ChewingContext *ctx;

	print_function_name();

	ctx = chewing_new();
	type_keystroke_by_string( ctx, "`<EE>" );
	ok_candidate( ctx, NULL, 0 );

	chewing_delete( ctx );
}

void test_Esc_entering_zuin()
{
	ChewingContext *ctx;

	print_function_name();

	ctx = chewing_new();
	type_keystroke_by_string( ctx, "hk<EE>" );
	ok_zuin_buffer( ctx, "" );

	chewing_delete( ctx );
}

void test_Esc()
{
	test_Esc_not_entering_chewing();
	test_Esc_in_select();
	test_Esc_entering_zuin();
	/* XXX: Test escCleanAllBuf here */
}

void test_Del_not_entering_chewing()
{
	ChewingContext *ctx;

	print_function_name();

	ctx = chewing_new();
	type_keystroke_by_string( ctx, "<DC>" );
	ok_keystroke_rtn( ctx, KEYSTROKE_IGNORE );

	chewing_delete( ctx );
}

void test_Del_in_select()
{
	ChewingContext *ctx;

	print_function_name();

	ctx = chewing_new();
	type_keystroke_by_string( ctx, "`<DC>" );
	ok_keystroke_rtn( ctx, KEYSTROKE_ABSORB ); /* XXX: shall be ignore? */

	chewing_delete( ctx );
}

void test_Del_word()
{
	ChewingContext *ctx;

	print_function_name();

	ctx = chewing_new();
	chewing_set_maxChiSymbolLen( ctx, 16 );

	type_keystroke_by_string( ctx, "hk4u g4<L><L><DC><E>" );
	ok_commit_buffer( ctx, "\xE6\xB8\xAC\xE8\xA9\xA6" /* 測試 */ );

	chewing_delete( ctx );
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

	print_function_name();

	ctx = chewing_new();
	type_keystroke_by_string( ctx, "<B>" );
	ok_keystroke_rtn( ctx, KEYSTROKE_IGNORE );

	chewing_delete( ctx );
}

void test_Backspace_in_select()
{
	ChewingContext *ctx;

	print_function_name();

	ctx = chewing_new();
	type_keystroke_by_string( ctx, "`<B>" );
	ok_keystroke_rtn( ctx, KEYSTROKE_ABSORB ); /* XXX: shall be ignore? */

	chewing_delete( ctx );
}

void test_Backspace_remove_bopomofo()
{
	ChewingContext *ctx;

	print_function_name();

	ctx = chewing_new();
	type_keystroke_by_string( ctx, "hk<B>" );
	ok_zuin_buffer( ctx, "\xE3\x84\x98" /* ㄘ */ );

	chewing_delete( ctx );
}

void test_Backspace_word()
{
	ChewingContext *ctx;

	print_function_name();

	ctx = chewing_new();
	chewing_set_maxChiSymbolLen( ctx, 16 );

	type_keystroke_by_string( ctx, "hk4u g4<L><B><E>" );
	ok_commit_buffer( ctx, "\xE6\xB8\xAC\xE8\xA9\xA6" /* 測試 */ );

	chewing_delete( ctx );
}

void test_Backspace()
{
	test_Backspace_not_entering_chewing();
	test_Backspace_in_select();
	test_Backspace_remove_bopomofo();
	test_Backspace_word();
}

void test_Up_close_candidate_window()
{
	ChewingContext *ctx;
	int ret;

	print_function_name();

	ctx = chewing_new();

	type_keystroke_by_string( ctx, "hk4" );
	ret = chewing_cand_TotalChoice( ctx );
	ok( ret == 0, "chewing_cand_TotalChoice() returns `%d' shall be `%d'", ret, 0 );

	type_keystroke_by_string( ctx, "<D>" );
	ret = chewing_cand_TotalChoice( ctx );
	ok( ret > 0, "chewing_cand_TotalChoice() returns `%d' shall be greater than `%d'", ret, 0 );

	type_keystroke_by_string( ctx, "<U>" );
	ret = chewing_cand_TotalChoice( ctx );
	ok( ret == 0, "chewing_cand_TotalChoice() returns `%d' shall be `%d'", ret, 0 );

	chewing_delete( ctx );
}

void test_Up_not_entering_chewing()
{
	ChewingContext *ctx;

	print_function_name();

	ctx = chewing_new();
	type_keystroke_by_string( ctx, "<U>" );
	ok_keystroke_rtn( ctx, KEYSTROKE_IGNORE );

	chewing_delete( ctx );
}

void test_Up()
{
	test_Up_close_candidate_window();
	test_Up_not_entering_chewing();
}

void test_Down_open_candidate_window()
{
	ChewingContext *ctx;
	int ret;

	print_function_name();

	ctx = chewing_new();

	type_keystroke_by_string( ctx, "hk4" );
	ret = chewing_cand_TotalChoice( ctx );
	ok( ret == 0, "chewing_cand_TotalChoice() returns `%d' shall be `%d'", ret, 0 );

	type_keystroke_by_string( ctx, "<D>" );
	ret = chewing_cand_TotalChoice( ctx );
	ok( ret > 0, "chewing_cand_TotalChoice() returns `%d' shall be greater than `%d'", ret, 0 );

	type_keystroke_by_string( ctx, "3" );
	ret = chewing_cand_TotalChoice( ctx );
	ok( ret == 0, "chewing_cand_TotalChoice() returns `%d' shall be `%d'", ret, 0 );
	ok_preedit_buffer( ctx, "\xE6\xB8\xAC" /* 測 */ );

	chewing_delete( ctx );
}

void test_Down_not_entering_chewing()
{
	ChewingContext *ctx;

	print_function_name();

	ctx = chewing_new();
	type_keystroke_by_string( ctx, "<D>" );
	ok_keystroke_rtn( ctx, KEYSTROKE_IGNORE );

	chewing_delete( ctx );
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

	print_function_name();

	ctx = chewing_new();
	chewing_set_maxChiSymbolLen( ctx, 16 );

	type_keystroke_by_string( ctx, "hk4g4<L>" );
	chewing_interval_Enumerate( ctx );

	ok( chewing_interval_hasNext( ctx ) == 1, "shall have next interval" );
	chewing_interval_Get( ctx, &it );
	ok( it.from == 0 && it.to == 2, "interval (%d, %d) shall be (0, 2)",
		it.from, it.to );

	ok( chewing_interval_hasNext( ctx ) == 0, "shall not have next interval" );

	/* inserts a breakpoint between 測 and 試 */
	type_keystroke_by_string( ctx, "<T>" );
	chewing_interval_Enumerate( ctx );

	ok( chewing_interval_hasNext( ctx ) == 1, "shall have next interval" );
	chewing_interval_Get( ctx, &it );
	ok( it.from == 0 && it.to == 1, "interval (%d, %d) shall be (0, 1)",
		it.from, it.to );

	ok( chewing_interval_hasNext( ctx ) == 1, "shall have next interval" );
	chewing_interval_Get( ctx, &it );
	ok( it.from == 1 && it.to == 2, "interval (%d, %d) shall be (1, 2)",
		it.from, it.to );

	ok( chewing_interval_hasNext( ctx ) == 0, "shall not have next interval" );

	chewing_delete( ctx );
}

void test_Tab_connect_word()
{
	ChewingContext *ctx;
	IntervalType it;

	print_function_name();

	ctx = chewing_new();
	chewing_set_maxChiSymbolLen( ctx, 16 );

	type_keystroke_by_string( ctx, "u -4<L>" );
	chewing_interval_Enumerate( ctx );

	ok( chewing_interval_hasNext( ctx ) == 1, "shall have next interval" );
	chewing_interval_Get( ctx, &it );
	ok( it.from == 0 && it.to == 1, "interval (%d, %d) shall be (0, 1)",
		it.from, it.to );

	ok( chewing_interval_hasNext( ctx ) == 1, "shall have next interval" );
	chewing_interval_Get( ctx, &it );
	ok( it.from == 1 && it.to == 2, "interval (%d, %d) shall be (1, 2)",
		it.from, it.to );

	ok( chewing_interval_hasNext( ctx ) == 0, "shall not have next interval" );

	/* connect 一 and 二 */
	type_keystroke_by_string( ctx, "<T>" );
	chewing_interval_Enumerate( ctx );

	ok( chewing_interval_hasNext( ctx ) == 1, "shall have next interval" );
	chewing_interval_Get( ctx, &it );
	ok( it.from == 0 && it.to == 2, "interval (%d, %d) shall be (0, 2)",
		it.from, it.to );

	ok( chewing_interval_hasNext( ctx ) == 0, "shall not have next interval" );

	chewing_delete( ctx );
}

void test_Tab_at_the_end()
{
	ChewingContext *ctx;

	print_function_name();

	ctx = chewing_new();

	type_keystroke_by_string( ctx, "hk4g4u6vu84" );
	ok_preedit_buffer( ctx, "\xE6\xB8\xAC\xE8\xA9\xA6\xE4\xB8\x80\xE4\xB8\x8B" /* 測試一下 */ );

	type_keystroke_by_string( ctx, "<T>" );
	ok_preedit_buffer( ctx, "\xE6\xB8\xAC\xE8\xA9\xA6\xE5\x84\x80\xE4\xB8\x8B" /* 測試儀下 */ );

	type_keystroke_by_string( ctx, "<T>" );
	ok_preedit_buffer( ctx, "\xE6\xB8\xAC\xE8\xA9\xA6\xE4\xB8\x80\xE4\xB8\x8B" /* 測試一下 */ );

	chewing_delete( ctx );
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

	print_function_name();

	ctx = chewing_new();

	type_keystroke_by_string( ctx, "<CB>" );
	ok( chewing_get_ChiEngMode( ctx ) == SYMBOL_MODE,
		"mode shall change to SYMBOL_MODE" );

	chewing_delete( ctx );
}

void test_Home()
{
	ChewingContext *ctx;
	int cursor;

	print_function_name();

	ctx = chewing_new();
	chewing_set_maxChiSymbolLen( ctx, 16 );

	type_keystroke_by_string( ctx, "hk4g4" );
	cursor = chewing_cursor_Current( ctx );
	ok( cursor == 2, "cursor `%d' shall be 2", cursor );

	type_keystroke_by_string( ctx, "<H>" );
	cursor = chewing_cursor_Current( ctx );
	ok( cursor == 0, "cursor `%d' shall be 0", cursor );

	chewing_delete( ctx );
}

void test_End()
{
	ChewingContext *ctx;
	int cursor;

	print_function_name();

	ctx = chewing_new();
	chewing_set_maxChiSymbolLen( ctx, 16 );

	type_keystroke_by_string( ctx, "hk4g4<L><L>" );
	cursor = chewing_cursor_Current( ctx );
	ok( cursor == 0, "cursor `%d' shall be 0", cursor );

	type_keystroke_by_string( ctx, "<EN>" );
	cursor = chewing_cursor_Current( ctx );
	ok( cursor == 2, "cursor `%d' shall be 2", cursor );

	chewing_delete( ctx );
}

void test_PageUp()
{
	ChewingContext *ctx;
	int cursor;

	print_function_name();

	ctx = chewing_new();
	chewing_set_maxChiSymbolLen( ctx, 16 );

	type_keystroke_by_string( ctx, "hk4g4<L><L>" );
	cursor = chewing_cursor_Current( ctx );
	ok( cursor == 0, "cursor `%d' shall be 0", cursor );

	type_keystroke_by_string( ctx, "<PU>" );
	cursor = chewing_cursor_Current( ctx );
	ok( cursor == 2, "cursor `%d' shall be 2", cursor );

	chewing_delete( ctx );
}

void test_PageDown()
{
	ChewingContext *ctx;
	int cursor;

	print_function_name();

	ctx = chewing_new();
	chewing_set_maxChiSymbolLen( ctx, 16 );

	type_keystroke_by_string( ctx, "hk4g4<L><L>" );
	cursor = chewing_cursor_Current( ctx );
	ok( cursor == 0, "cursor `%d' shall be 0", cursor );

	type_keystroke_by_string( ctx, "<PD>" );
	cursor = chewing_cursor_Current( ctx );
	ok( cursor == 2, "cursor `%d' shall be 2", cursor );

	chewing_delete( ctx );
}

void test_ShiftSpace()
{
	/* FIXME: Implement this. */
}

void test_Numlock_numeric_input()
{
	const TestData NUMLOCK_INPUT[] = {
		{ "<N0>", "0" },
		{ "<N1>", "1" },
		{ "<N2>", "2" },
		{ "<N3>", "3" },
		{ "<N4>", "4" },
		{ "<N5>", "5" },
		{ "<N6>", "6" },
		{ "<N7>", "7" },
		{ "<N8>", "8" },
		{ "<N9>", "9" },
		{ "<N+>", "+" },
		{ "<N->", "-" },
		{ "<N*>", "*" },
		{ "<N/>", "/" },
		{ "<N.>", "." },
	};
	size_t i;
	ChewingContext *ctx;

	print_function_name();

	ctx = chewing_new();
	chewing_set_maxChiSymbolLen( ctx, 16 );

	for ( i = 0; i < ARRAY_SIZE( NUMLOCK_INPUT ); ++i ) {
		type_keystroke_by_string( ctx, NUMLOCK_INPUT[i].token );
		ok_commit_buffer( ctx, NUMLOCK_INPUT[i].expected );
	}

	chewing_delete( ctx );
}

void test_Numlock_select_candidate()
{
	const TestData NUMLOCK_SELECT[] = {
		{ "hk4<D><N1><E>", "\xE5\x86\x8A" /* 冊 */ },
		{ "hk4<D><N2><E>", "\xE7\xAD\x96" /* 策 */ },
		{ "hk4<D><N3><E>", "\xE6\xB8\xAC" /* 測 */ },
		{ "hk4<D><N4><E>", "\xE5\x81\xB4" /* 側 */ },
		{ "hk4<D><N5><E>", "\xE5\xBB\x81" /* 廁 */ },
		{ "hk4<D><N6><E>", "\xE6\x83\xBB" /* 惻 */ },
		{ "hk4<D><N7><E>", "\xE7\xAD\xB4" /* 筴 */ },
		{ "hk4<D><N8><E>", "\xE7\x95\x9F" /* 畟 */ },
		{ "hk4<D><N9><E>", "\xE8\x8C\xA6" /* 茦 */ },
		{ "hk4<D><N0><E>", "\xE7\xB2\xA3" /* 粣 */ },
	};
	size_t i;
	ChewingContext *ctx;

	print_function_name();

	ctx = chewing_new();
	chewing_set_maxChiSymbolLen( ctx, 16 );

	for ( i = 0; i < ARRAY_SIZE( NUMLOCK_SELECT ); ++i ) {
		type_keystroke_by_string( ctx, NUMLOCK_SELECT[ i ].token );
		ok_commit_buffer( ctx, NUMLOCK_SELECT[i].expected );
	}

	chewing_delete( ctx );
}

void test_Numlock()
{
	test_Numlock_numeric_input();
	test_Numlock_select_candidate();
}

void test_Space_selection()
{
	ChewingContext *ctx;
	char *buf;
	int len;

	print_function_name();

	clean_userphrase();

	ctx = chewing_new();
	chewing_set_spaceAsSelection( ctx, 1 );

	type_keystroke_by_string( ctx, "hk4g4<H>" /* 測試 */ );

	type_keystroke_by_string( ctx, " " );

	chewing_cand_Enumerate( ctx );
	buf = chewing_cand_String( ctx );
	len = ueStrLen(buf);
	ok( len == 2, "candidate `%s' length `%d' shall be `%d'", buf, len, 2 );
	chewing_free( buf );

	type_keystroke_by_string( ctx, " " );

	chewing_cand_Enumerate( ctx );
	buf = chewing_cand_String( ctx );
	len = ueStrLen(buf);
	ok( len == 1, "candidate `%s' length `%d' shall be `%d'", buf, len, 1 );
	chewing_free( buf );

	chewing_delete( ctx );
}

void test_Space()
{
	test_Space_selection();
}

void test_get_phoneSeq()
{
	static const struct {
		char *token;
		unsigned short phone[5];
	} DATA[] = {
		{ "hk4g4", { 10268, 8708, 0 } },
		{ "hk4g4`31hk4g4", { 10268, 8708, 10268, 8708, 0 } },
		{ "`31`31", { 0 } },
	};
	ChewingContext *ctx;
	size_t i;
	int expected_len;
	int len;
	unsigned short *phone;

	print_function_name();

	ctx = chewing_new();
	chewing_set_maxChiSymbolLen( ctx, 16 );

	for ( i = 0; i < ARRAY_SIZE( DATA ); ++i ) {
		chewing_Reset( ctx );
		type_keystroke_by_string( ctx, DATA[i].token );

		expected_len = 0;
		while ( DATA[i].phone[expected_len] != 0 )
			++expected_len;
		len = chewing_get_phoneSeqLen( ctx );
		ok( len == expected_len, "phoneSeqLen `%d' shall be `%d'", len, expected_len );

		phone = chewing_get_phoneSeq( ctx );
		ok ( memcmp( phone, DATA[i].phone, sizeof( phone[0] ) * expected_len ) == 0, "phoneSeq shall be expected value" );
		chewing_free( phone );
	}

	chewing_delete( ctx );
}

void test_zuin_buffer()
{
	ChewingContext *ctx;

	print_function_name();

	ctx = chewing_new();

	type_keystroke_by_string( ctx, "1ul" );
	ok_zuin_buffer( ctx, "\xE3\x84\x85\xE3\x84\xA7\xE3\x84\xA0" /* ㄅㄧㄠ */ );

	type_keystroke_by_string( ctx, " " );
	ok_zuin_buffer( ctx, "" );

	type_keystroke_by_string( ctx, "ul" );
	ok_zuin_buffer( ctx, "\xE3\x84\xA7\xE3\x84\xA0" /* ㄧㄠ */ );

	type_keystroke_by_string( ctx, " " );
	ok_zuin_buffer( ctx, "" );

	type_keystroke_by_string( ctx, "3");
	ok_zuin_buffer( ctx, "\xCB\x87" /* ˇ */);

	type_keystroke_by_string( ctx, " " );
	ok_zuin_buffer( ctx, "" );

	chewing_delete( ctx );
}

void test_longest_phrase()
{
	ChewingContext *ctx;
	IntervalType it;

	print_function_name();

	ctx = chewing_new();

	type_keystroke_by_string( ctx, "rup ji up6ji 1j4bj6y4ru32k7e.3ji "
		/* ㄐㄧㄣ ㄨㄛ ㄧㄣˊ ㄨㄛ ㄅㄨˋ ㄖㄨˊ ㄗˋ ㄐㄧˇ ㄉㄜ˙ ㄍㄡˇ ㄨㄛ */ );
	ok_preedit_buffer( ctx, "\xE9\x87\x91\xE7\xAA\xA9\xE9\x8A\x80\xE7\xAA\xA9\xE4\xB8\x8D\xE5\xA6\x82\xE8\x87\xAA\xE5\xB7\xB1\xE7\x9A\x84\xE7\x8B\x97\xE7\xAA\xA9"
		/* 金窩銀窩不如自己的狗窩 */ );

	chewing_interval_Enumerate( ctx );

	ok( chewing_interval_hasNext( ctx ) == 1, "shall have next interval" );
	chewing_interval_Get( ctx, &it );
	ok( it.from == 0 && it.to == 11, "interval (%d, %d) shall be (0, 11)",
		it.from, it.to );

	chewing_delete( ctx );
}

void test_auto_commit_phrase()
{
	ChewingContext *ctx;

	print_function_name();

	ctx = chewing_new();
	chewing_set_maxChiSymbolLen( ctx, 3 );

	type_keystroke_by_string( ctx, "hk4g4hk4g4" /* 測試測試 */ );
	ok_preedit_buffer( ctx, "\xE6\xB8\xAC\xE8\xA9\xA6" /* 測試 */ );
	ok_commit_buffer( ctx, "\xE6\xB8\xAC\xE8\xA9\xA6" /* 測試 */ );

	chewing_delete( ctx );
}

void test_auto_commit_symbol()
{
	ChewingContext *ctx;

	print_function_name();

	ctx = chewing_new();
	chewing_set_maxChiSymbolLen( ctx, 2 );

	type_keystroke_by_string( ctx, "`31hk4g4hk4g4" /* ，測試 */ );
	ok_preedit_buffer( ctx, "\xE6\xB8\xAC\xE8\xA9\xA6" /* 測試 */ );
	ok_commit_buffer( ctx, "\xEF\xBC\x8C" /* ， */ );

	chewing_delete( ctx );
}

void test_auto_commit()
{
	test_auto_commit_phrase();
	// FIXME: Auto commit for symbol seem to be incorrect.
	//test_auto_commit_symbol();
}

int main()
{
	putenv( "CHEWING_PATH=" CHEWING_DATA_PREFIX );
	putenv( "CHEWING_USER_PATH=" TEST_HASH_DIR );

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
	test_zuin_buffer();

	test_longest_phrase();
	test_auto_commit();

	return exit_status();
}
