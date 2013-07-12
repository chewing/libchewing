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

void test_select_candidate_no_phrase_choice_rearward()
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

	remove( TEST_HASH_DIR PLAT_SEPARATOR HASH_FILE );

	chewing_Init( NULL, NULL );

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
	chewing_Terminate();
}

void test_select_candidate_phrase_choice_rearward()
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

	remove( TEST_HASH_DIR PLAT_SEPARATOR HASH_FILE );

	chewing_Init( NULL, NULL );

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
	chewing_Terminate();
}

void test_select_candidate_4_bytes_utf8()
{
	ChewingContext *ctx;

	remove( TEST_HASH_DIR PLAT_SEPARATOR HASH_FILE );

	chewing_Init( NULL, NULL );

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
	chewing_Terminate();
}

void test_select_candidate() {
	test_select_candidate_no_phrase_choice_rearward();
	test_select_candidate_phrase_choice_rearward();
	test_select_candidate_4_bytes_utf8();
}

void test_Esc_not_entering_chewing()
{
	ChewingContext *ctx;

	chewing_Init( NULL, NULL );

	ctx = chewing_new();
	type_keystroke_by_string( ctx, "<EE>" );
	ok_keystroke_rtn( ctx, KEYSTROKE_IGNORE );

	chewing_delete( ctx );
	chewing_Terminate();
}

void test_Esc_in_select()
{
	ChewingContext *ctx;

	chewing_Init( NULL, NULL );

	ctx = chewing_new();
	type_keystroke_by_string( ctx, "`<EE>" );
	ok_candidate( ctx, NULL, 0 );

	chewing_delete( ctx );
	chewing_Terminate();
}

void test_Esc_entering_zuin()
{
	ChewingContext *ctx;

	chewing_Init( NULL, NULL );

	ctx = chewing_new();
	type_keystroke_by_string( ctx, "hk<EE>" );
	ok_zuin_buffer( ctx, "" );

	chewing_delete( ctx );
	chewing_Terminate();
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

	chewing_Init( NULL, NULL );

	ctx = chewing_new();
	type_keystroke_by_string( ctx, "<DC>" );
	ok_keystroke_rtn( ctx, KEYSTROKE_IGNORE );

	chewing_delete( ctx );
	chewing_Terminate();
}

void test_Del_in_select()
{
	ChewingContext *ctx;

	chewing_Init( NULL, NULL );

	ctx = chewing_new();
	type_keystroke_by_string( ctx, "`<DC>" );
	ok_keystroke_rtn( ctx, KEYSTROKE_ABSORB ); /* XXX: shall be ignore? */

	chewing_delete( ctx );
	chewing_Terminate();
}

void test_Del_word()
{
	ChewingContext *ctx;

	chewing_Init( NULL, NULL );

	ctx = chewing_new();
	chewing_set_maxChiSymbolLen( ctx, 16 );

	type_keystroke_by_string( ctx, "hk4u g4<L><L><DC><E>" );
	ok_commit_buffer( ctx, "\xE6\xB8\xAC\xE8\xA9\xA6" /* 測試 */ );

	chewing_delete( ctx );
	chewing_Terminate();
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

	chewing_Init( NULL, NULL );

	ctx = chewing_new();
	type_keystroke_by_string( ctx, "<B>" );
	ok_keystroke_rtn( ctx, KEYSTROKE_IGNORE );

	chewing_delete( ctx );
	chewing_Terminate();
}

void test_Backspace_in_select()
{
	ChewingContext *ctx;

	chewing_Init( NULL, NULL );

	ctx = chewing_new();
	type_keystroke_by_string( ctx, "`<B>" );
	ok_keystroke_rtn( ctx, KEYSTROKE_ABSORB ); /* XXX: shall be ignore? */

	chewing_delete( ctx );
	chewing_Terminate();
}

void test_Backspace_remove_bopomofo()
{
	ChewingContext *ctx;

	chewing_Init( NULL, NULL );

	ctx = chewing_new();
	type_keystroke_by_string( ctx, "hk<B>" );
	ok_zuin_buffer( ctx, "\xE3\x84\x98" /* ㄘ */ );

	chewing_delete( ctx );
	chewing_Terminate();
}

void test_Backspace_word()
{
	ChewingContext *ctx;

	chewing_Init( NULL, NULL );

	ctx = chewing_new();
	chewing_set_maxChiSymbolLen( ctx, 16 );

	type_keystroke_by_string( ctx, "hk4u g4<L><B><E>" );
	ok_commit_buffer( ctx, "\xE6\xB8\xAC\xE8\xA9\xA6" /* 測試 */ );

	chewing_delete( ctx );
	chewing_Terminate();
}

void test_Backspace()
{
	test_Backspace_not_entering_chewing();
	test_Backspace_in_select();
	test_Backspace_remove_bopomofo();
	test_Backspace_word();
}

void test_Up_not_entering_chewing()
{
	ChewingContext *ctx;

	chewing_Init( NULL, NULL );

	ctx = chewing_new();
	type_keystroke_by_string( ctx, "<U>" );
	ok_keystroke_rtn( ctx, KEYSTROKE_IGNORE );

	chewing_delete( ctx );
	chewing_Terminate();
}

void test_Up()
{
	test_Up_not_entering_chewing();
	/* XXX: What is spec of Up? */
}

void test_Down_not_entering_chewing()
{
	ChewingContext *ctx;

	chewing_Init( NULL, NULL );

	ctx = chewing_new();
	type_keystroke_by_string( ctx, "<D>" );
	ok_keystroke_rtn( ctx, KEYSTROKE_IGNORE );

	chewing_delete( ctx );
	chewing_Terminate();
}

void test_Down()
{
	test_Down_not_entering_chewing();
}

void test_Tab_insert_breakpoint_between_word()
{
	ChewingContext *ctx;
	IntervalType it;

	chewing_Init( NULL, NULL );

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
	chewing_Terminate();
}

void test_Tab_connect_word()
{
	ChewingContext *ctx;
	IntervalType it;

	chewing_Init( NULL, NULL );

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
	chewing_Terminate();
}

void test_Tab_at_the_end()
{
	ChewingContext *ctx;
	IntervalType it;

	chewing_Init( NULL, NULL );

	ctx = chewing_new();
	chewing_set_maxChiSymbolLen( ctx, 16 );

	type_keystroke_by_string( ctx, "hk4<T>g4" );
	chewing_interval_Enumerate( ctx );

	ok( chewing_interval_hasNext( ctx ) == 1, "shall have next interval" );
	chewing_interval_Get( ctx, &it );
	ok( it.from == 0 && it.to == 2, "interval (%d, %d) shall be (0, 2)",
		it.from, it.to );

	ok( chewing_interval_hasNext( ctx ) == 0, "shall not have next interval" );

	chewing_delete( ctx );
	chewing_Terminate();
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

	chewing_Init( NULL, NULL );

	ctx = chewing_new();

	type_keystroke_by_string( ctx, "<CB>" );
	ok( chewing_get_ChiEngMode( ctx ) == SYMBOL_MODE,
		"mode shall change to SYMBOL_MODE" );

	chewing_delete( ctx );
	chewing_Terminate();
}

void test_Home()
{
	ChewingContext *ctx;
	int cursor;

	chewing_Init( NULL, NULL );

	ctx = chewing_new();
	chewing_set_maxChiSymbolLen( ctx, 16 );

	type_keystroke_by_string( ctx, "hk4g4" );
	cursor = chewing_cursor_Current( ctx );
	ok( cursor == 2, "cursor `%d' shall be 2", cursor );

	type_keystroke_by_string( ctx, "<H>" );
	cursor = chewing_cursor_Current( ctx );
	ok( cursor == 0, "cursor `%d' shall be 0", cursor );

	chewing_delete( ctx );
	chewing_Terminate();
}

void test_End()
{
	ChewingContext *ctx;
	int cursor;

	chewing_Init( NULL, NULL );

	ctx = chewing_new();
	chewing_set_maxChiSymbolLen( ctx, 16 );

	type_keystroke_by_string( ctx, "hk4g4<L><L>" );
	cursor = chewing_cursor_Current( ctx );
	ok( cursor == 0, "cursor `%d' shall be 0", cursor );

	type_keystroke_by_string( ctx, "<EN>" );
	cursor = chewing_cursor_Current( ctx );
	ok( cursor == 2, "cursor `%d' shall be 2", cursor );

	chewing_delete( ctx );
	chewing_Terminate();
}

void test_PageUp()
{
	ChewingContext *ctx;
	int cursor;

	chewing_Init( NULL, NULL );

	ctx = chewing_new();
	chewing_set_maxChiSymbolLen( ctx, 16 );

	type_keystroke_by_string( ctx, "hk4g4<L><L>" );
	cursor = chewing_cursor_Current( ctx );
	ok( cursor == 0, "cursor `%d' shall be 0", cursor );

	type_keystroke_by_string( ctx, "<PU>" );
	cursor = chewing_cursor_Current( ctx );
	ok( cursor == 2, "cursor `%d' shall be 2", cursor );

	chewing_delete( ctx );
	chewing_Terminate();
}

void test_PageDown()
{
	ChewingContext *ctx;
	int cursor;

	chewing_Init( NULL, NULL );

	ctx = chewing_new();
	chewing_set_maxChiSymbolLen( ctx, 16 );

	type_keystroke_by_string( ctx, "hk4g4<L><L>" );
	cursor = chewing_cursor_Current( ctx );
	ok( cursor == 0, "cursor `%d' shall be 0", cursor );

	type_keystroke_by_string( ctx, "<PD>" );
	cursor = chewing_cursor_Current( ctx );
	ok( cursor == 2, "cursor `%d' shall be 2", cursor );

	chewing_delete( ctx );
	chewing_Terminate();
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

	chewing_Init( NULL, NULL );

	ctx = chewing_new();
	chewing_set_maxChiSymbolLen( ctx, 16 );

	for ( i = 0; i < ARRAY_SIZE( NUMLOCK_INPUT ); ++i ) {
		type_keystroke_by_string( ctx, NUMLOCK_INPUT[i].token );
		/* FIXME: Current buggy here */
		/* ok_commit_buffer( ctx, NUMLOCK_INPUT[i].expected ); */
	}

	chewing_delete( ctx );
	chewing_Terminate();
}

void test_Numlock_select_candidate()
{
	const TestData NUMLOCK_SELECT[] = {
		{ "hk4<D><N3><E>", "\xE6\xB8\xAC" /* 測 */ },
		{ "`<N1><E>", "\xE2\x80\xA6" /* … */ },
	};
	size_t i;
	ChewingContext *ctx;

	chewing_Init( NULL, NULL );

	ctx = chewing_new();
	chewing_set_maxChiSymbolLen( ctx, 16 );

	for ( i = 0; i < ARRAY_SIZE( NUMLOCK_SELECT ); ++i ) {
		type_keystroke_by_string( ctx, NUMLOCK_SELECT[ i ].token );
		ok_commit_buffer( ctx, NUMLOCK_SELECT[i].expected );
	}

	chewing_delete( ctx );
	chewing_Terminate();
}

void test_Numlock()
{
	test_Numlock_numeric_input();
	test_Numlock_select_candidate();
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

	chewing_Init( NULL, NULL );

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
	chewing_Terminate();
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

	test_get_phoneSeq();

	return exit_status();
}
