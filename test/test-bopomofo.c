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
#include "test.h"

void test_select_candidate_no_phrase_choice_rearward()
{
	// The following phrases are in dict
	// 一上來
	// 上來
	// 移上來
	// 移上
	// 快上

	static const char *CAND_1[] = {
		"一上來",
		"移上來",
	};

	static const char *CAND_2[] = {
		"移上",
	};

	remove( TEST_HASH_DIR PLAT_SEPARATOR HASH_FILE );

	chewing_Init( NULL, NULL );

	ChewingContext *ctx = chewing_new();

	chewing_set_maxChiSymbolLen( ctx, 16 );

	type_keystoke_by_string( ctx, "u6g;4x96<L><L><L>" ); // ㄧˊㄕㄤˋㄌㄞˊ

	// ㄧˊㄕㄤˋㄌㄞˊ
	type_keystoke_by_string( ctx, "<D>" );
	ok_candidate( ctx, CAND_1, ARRAY_SIZE( CAND_1 ) );

	// ㄕㄤˋㄌㄞˊ
	type_keystoke_by_string( ctx, "<D>" );
	ok_candidate( ctx, CAND_2, ARRAY_SIZE( CAND_2 ) );

	// select 移上來
	type_keystoke_by_string( ctx, "<D><D>2<E>" );
	ok_commit_buffer( ctx, CAND_1[1] );

	chewing_delete( ctx );
	chewing_Terminate();
}

void test_select_candidate_phrase_choice_rearward()
{
	// The following phrases are in dict
	// 一上來
	// 上來
	// 移上來
	// 移上
	// 快上

	static const char *CAND_1[] = {
		"一上來",
		"移上來",
	};

	static const char *CAND_2[] = {
		"上來",
		"快上", // XXX: bug?
	};

	remove( TEST_HASH_DIR PLAT_SEPARATOR HASH_FILE );

	chewing_Init( NULL, NULL );

	ChewingContext *ctx = chewing_new();

	chewing_set_maxChiSymbolLen( ctx, 16 );
	chewing_set_phraseChoiceRearward( ctx, 1 );

	type_keystoke_by_string( ctx, "u6g;4x96" ); // ㄧˊㄕㄤˋㄌㄞˊ
	ok_preedit_buffer( ctx, CAND_1[0] );

	// ㄧˊㄕㄤˋㄌㄞˊ
	type_keystoke_by_string( ctx, "<D>" );
	ok_candidate( ctx, CAND_1, ARRAY_SIZE( CAND_1 ) );

	// ㄕㄤˋㄌㄞˊ
	type_keystoke_by_string( ctx, "<D>" );
	ok_candidate( ctx, CAND_2, ARRAY_SIZE( CAND_2 ) );

	// select 移上來
	type_keystoke_by_string( ctx, "<D><D>2<E>" );
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
	// XXX: Test escCleanAllBuf here
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
	ok_keystoke_rtn( ctx, KEYSTROKE_ABSORB ); // XXX: shall be ignore?

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
	ok_commit_buffer( ctx, "測試" );

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
	ok_keystoke_rtn( ctx, KEYSTROKE_ABSORB ); // XXX: shall be ignore?

	chewing_delete( ctx );
	chewing_Terminate();
}

void test_Backspace_remove_bopomofo()
{
	ChewingContext *ctx;

	chewing_Init( NULL, NULL );

	ctx = chewing_new();
	type_keystoke_by_string( ctx, "hk<B>" );
	ok_zuin_buffer( ctx, "ㄘ" );

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
	ok_commit_buffer( ctx, "測試" );

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
	// XXX: What is spec of Up?
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
	static const char phrase[] = "測試";
	static const char bopomofo[] = "ㄘㄜˋ ㄕˋ";
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
	ok( cursor == 0, "cursor position `%d' shall be 2", cursor );
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

	return exit_status();
}
