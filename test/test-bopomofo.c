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

	type_keystoke_by_string( ctx, "u6g;4x96<L><L><L>" ); /* ㄧˊㄕㄤˋㄌㄞˊ */

	type_keystoke_by_string( ctx, "<D>" ); /* ㄧˊㄕㄤˋㄌㄞˊ */
	ok_candidate( ctx, CAND_1, ARRAY_SIZE( CAND_1 ) );

	type_keystoke_by_string( ctx, "<D>" ); /* ㄕㄤˋㄌㄞˊ */
	ok_candidate( ctx, CAND_2, ARRAY_SIZE( CAND_2 ) );

	type_keystoke_by_string( ctx, "<D><D>2<E>" ); /* select 移上來 */
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

	type_keystoke_by_string( ctx, "u6g;4x96" ); /* ㄧˊㄕㄤˋㄌㄞˊ */
	ok_preedit_buffer( ctx, CAND_1[0] );

	type_keystoke_by_string( ctx, "<D>" ); /* ㄧˊㄕㄤˋㄌㄞˊ */
	ok_candidate( ctx, CAND_1, ARRAY_SIZE( CAND_1 ) );

	type_keystoke_by_string( ctx, "<D>" ); /* ㄕㄤˋㄌㄞˊ */
	ok_candidate( ctx, CAND_2, ARRAY_SIZE( CAND_2 ) );

	type_keystoke_by_string( ctx, "<D><D>2<E>" ); /* select 移上來 */
	ok_commit_buffer( ctx, CAND_1[1] );

	chewing_delete( ctx );
	chewing_Terminate();
}

void test_select_candidate() {
	test_select_candidate_no_phrase_choice_rearward();
	test_select_candidate_phrase_choice_rearward();
}

void test_Esc_not_entering_chewing()
{
	ChewingContext *ctx;

	chewing_Init( NULL, NULL );

	ctx = chewing_new();
	type_keystoke_by_string( ctx, "<EE>" );
	ok_keystoke_rtn( ctx, KEYSTROKE_IGNORE );

	chewing_delete( ctx );
	chewing_Terminate();
}

void test_Esc_in_select()
{
	ChewingContext *ctx;

	chewing_Init( NULL, NULL );

	ctx = chewing_new();
	type_keystoke_by_string( ctx, "`<EE>" );
	ok_candidate( ctx, NULL, 0 );

	chewing_delete( ctx );
	chewing_Terminate();
}

void test_Esc_entering_zuin()
{
	ChewingContext *ctx;

	chewing_Init( NULL, NULL );

	ctx = chewing_new();
	type_keystoke_by_string( ctx, "hk<EE>" );
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
	type_keystoke_by_string( ctx, "<DC>" );
	ok_keystoke_rtn( ctx, KEYSTROKE_IGNORE );

	chewing_delete( ctx );
	chewing_Terminate();
}

void test_Del_in_select()
{
	ChewingContext *ctx;

	chewing_Init( NULL, NULL );

	ctx = chewing_new();
	type_keystoke_by_string( ctx, "`<DC>" );
	ok_keystoke_rtn( ctx, KEYSTROKE_ABSORB ); /* XXX: shall be ignore? */

	chewing_delete( ctx );
	chewing_Terminate();
}

void test_Del_word()
{
	ChewingContext *ctx;

	chewing_Init( NULL, NULL );

	ctx = chewing_new();
	chewing_set_maxChiSymbolLen( ctx, 16 );

	type_keystoke_by_string( ctx, "hk4u g4<L><L><DC><E>" );
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
	type_keystoke_by_string( ctx, "<B>" );
	ok_keystoke_rtn( ctx, KEYSTROKE_IGNORE );

	chewing_delete( ctx );
	chewing_Terminate();
}

void test_Backspace_in_select()
{
	ChewingContext *ctx;

	chewing_Init( NULL, NULL );

	ctx = chewing_new();
	type_keystoke_by_string( ctx, "`<B>" );
	ok_keystoke_rtn( ctx, KEYSTROKE_ABSORB ); /* XXX: shall be ignore? */

	chewing_delete( ctx );
	chewing_Terminate();
}

void test_Backspace_remove_bopomofo()
{
	ChewingContext *ctx;

	chewing_Init( NULL, NULL );

	ctx = chewing_new();
	type_keystoke_by_string( ctx, "hk<B>" );
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

	type_keystoke_by_string( ctx, "hk4u g4<L><B><E>" );
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
	type_keystoke_by_string( ctx, "<U>" );
	ok_keystoke_rtn( ctx, KEYSTROKE_IGNORE );

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
	type_keystoke_by_string( ctx, "<D>" );
	ok_keystoke_rtn( ctx, KEYSTROKE_IGNORE );

	chewing_delete( ctx );
	chewing_Terminate();
}

void test_Down()
{
	test_Down_not_entering_chewing();
}

void test_ShiftLeft_not_entering_chewing()
{
	ChewingContext *ctx;

	chewing_Init( NULL, NULL );

	ctx = chewing_new();
	type_keystoke_by_string( ctx, "<SL>" );
	ok_keystoke_rtn( ctx, KEYSTROKE_IGNORE );

	chewing_delete( ctx );
	chewing_Terminate();
}

void test_ShiftLeft_add_userphrase()
{
	static const char phrase[] = "\xE6\xB8\xAC\xE8\xA9\xA6" /* 測試 */;
	static const char bopomofo[] = "\xE3\x84\x98\xE3\x84\x9C\xCB\x8B \xE3\x84\x95\xCB\x8B" /* ㄘㄜˋ ㄕˋ */;
	int cursor;
	ChewingContext *ctx;

	remove( TEST_HASH_DIR PLAT_SEPARATOR HASH_FILE );

	chewing_Init( NULL, NULL );

	ctx = chewing_new();
	chewing_set_maxChiSymbolLen( ctx, 16 );

	ok( has_userphrase( ctx, bopomofo, phrase ) == 0,
		"`%s' shall not be in userphrase", phrase );

	type_keystoke_by_string( ctx, "hk4g4<SL><SL><E>" );
	ok_preedit_buffer( ctx, phrase );
	cursor = chewing_cursor_Current( ctx );
	ok( cursor == 0, "cursor position `%d' shall be 0", cursor );
	ok( has_userphrase( ctx, bopomofo, phrase ) == 1,
		"`%s' shall be in userphrase", phrase );

	chewing_delete( ctx );
	chewing_Terminate();
}

void test_ShiftLeft()
{
	test_ShiftLeft_not_entering_chewing();
	test_ShiftLeft_add_userphrase();
}

void test_ShiftRight_not_entering_chewing()
{
	ChewingContext *ctx;

	chewing_Init( NULL, NULL );

	ctx = chewing_new();
	type_keystoke_by_string( ctx, "<SR>" );
	ok_keystoke_rtn( ctx, KEYSTROKE_IGNORE );

	chewing_delete( ctx );
	chewing_Terminate();
}

void test_ShiftRight_add_userphrase()
{
	static const char phrase[] = "\xE6\xB8\xAC\xE8\xA9\xA6" /* 測試 */;
	static const char bopomofo[] = "\xE3\x84\x98\xE3\x84\x9C\xCB\x8B \xE3\x84\x95\xCB\x8B" /* ㄘㄜˋ ㄕˋ */;
	int cursor;
	ChewingContext *ctx;

	remove( TEST_HASH_DIR PLAT_SEPARATOR HASH_FILE );

	chewing_Init( NULL, NULL );

	ctx = chewing_new();
	chewing_set_maxChiSymbolLen( ctx, 16 );

	ok( has_userphrase( ctx, bopomofo, phrase ) == 0,
		"`%s' shall not be in userphrase", phrase );

	type_keystoke_by_string( ctx, "hk4g4<L><L><SR><SR><E>" );
	ok_preedit_buffer( ctx, phrase );
	cursor = chewing_cursor_Current( ctx );
	ok( cursor == 2, "cursor position `%d' shall be 2", cursor );
	ok( has_userphrase( ctx, bopomofo, phrase ) == 1,
		"`%s' shall be in userphrase", phrase );

	chewing_delete( ctx );
	chewing_Terminate();
}

void test_ShiftRight()
{
	test_ShiftRight_not_entering_chewing();
	test_ShiftRight_add_userphrase();
}

void test_Tab_insert_breakpoint_between_word()
{
	ChewingContext *ctx;
	IntervalType it;

	chewing_Init( NULL, NULL );

	ctx = chewing_new();
	chewing_set_maxChiSymbolLen( ctx, 16 );

	type_keystoke_by_string( ctx, "hk4g4<L>" );
	chewing_interval_Enumerate( ctx );

	ok( chewing_interval_hasNext( ctx ) == 1, "shall have next interval" );
	chewing_interval_Get( ctx, &it );
	ok( it.from == 0 && it.to == 2, "interval (%d, %d) shall be (0, 2)",
		it.from, it.to );

	ok( chewing_interval_hasNext( ctx ) == 0, "shall not have next interval" );

	/* inserts a breakpoint between 測 and 試 */
	type_keystoke_by_string( ctx, "<T>" );
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

	type_keystoke_by_string( ctx, "u -4<L>" );
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
	type_keystoke_by_string( ctx, "<T>" );
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

	type_keystoke_by_string( ctx, "hk4<T>g4" );
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

void test_Capslock()
{
	ChewingContext *ctx;

	chewing_Init( NULL, NULL );

	ctx = chewing_new();

	type_keystoke_by_string( ctx, "<CB>" );
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

	type_keystoke_by_string( ctx, "hk4g4" );
	cursor = chewing_cursor_Current( ctx );
	ok( cursor == 2, "cursor `%d' shall be 2", cursor );

	type_keystoke_by_string( ctx, "<H>" );
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

	type_keystoke_by_string( ctx, "hk4g4<L><L>" );
	cursor = chewing_cursor_Current( ctx );
	ok( cursor == 0, "cursor `%d' shall be 0", cursor );

	type_keystoke_by_string( ctx, "<EN>" );
	cursor = chewing_cursor_Current( ctx );
	ok( cursor == 2, "cursor `%d' shall be 2", cursor );

	chewing_delete( ctx );
	chewing_Terminate();
}

void test_get_phoneSeq()
{
	static const unsigned short PHONE[] = { 10268, 8708 };
	ChewingContext *ctx;
	unsigned short *phone;
	int len;
	int i;

	chewing_Init( NULL, NULL );

	ctx = chewing_new();
	chewing_set_maxChiSymbolLen( ctx, 16 );

	type_keystoke_by_string( ctx, "hk4g4" );

	len = chewing_get_phoneSeqLen( ctx );
	ok( len == ARRAY_SIZE( PHONE ), "phoneSeqLen `%d' shall be `%d'", len, ARRAY_SIZE( PHONE ) );

	phone = chewing_get_phoneSeq( ctx );
	for ( i = 0; i < len; ++i ) {
		ok( phone[i] == PHONE[i], "phone in position %d is `%d', shall be `%d'", i, phone[i], PHONE[i] );
	}
	chewing_free( phone );

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
	test_ShiftLeft();
	test_ShiftRight();
	test_Tab();
	test_Capslock();
	test_Home();
	test_End();

	test_get_phoneSeq();

	return exit_status();
}
