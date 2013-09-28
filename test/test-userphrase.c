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
#include <config.h>
#endif

#include <string.h>
#include <stdlib.h>
#include <stdio.h>

#include "chewing.h"
#include "plat_types.h"
#include "hash-private.h"
#include "testhelper.h"

void test_ShiftLeft_not_entering_chewing()
{
	ChewingContext *ctx;

	print_function_name();

	ctx = chewing_new();
	type_keystroke_by_string( ctx, "<SL>" );
	ok_keystroke_rtn( ctx, KEYSTROKE_IGNORE );

	chewing_delete( ctx );
}

void test_ShiftLeft_add_userphrase()
{
	static const char phrase[] = "\xE6\xB8\xAC\xE8\xA9\xA6" /* 測試 */;
	static const char bopomofo[] = "\xE3\x84\x98\xE3\x84\x9C\xCB\x8B \xE3\x84\x95\xCB\x8B" /* ㄘㄜˋ ㄕˋ */;
	int cursor;
	ChewingContext *ctx;

	print_function_name();

	clean_userphrase();

	ctx = chewing_new();
	chewing_set_maxChiSymbolLen( ctx, 16 );

	ok( has_userphrase( ctx, bopomofo, phrase ) == 0,
		"`%s' shall not be in userphrase", phrase );

	type_keystroke_by_string( ctx, "hk4g4<SL><SL><E>" );
	ok_preedit_buffer( ctx, phrase );
	cursor = chewing_cursor_Current( ctx );
	ok( cursor == 0, "cursor position `%d' shall be 0", cursor );
	ok( has_userphrase( ctx, bopomofo, phrase ) == 1,
		"`%s' shall be in userphrase", phrase );

	chewing_delete( ctx );
}

void test_ShiftLeft()
{
	test_ShiftLeft_not_entering_chewing();
	test_ShiftLeft_add_userphrase();
}

void test_ShiftRight_not_entering_chewing()
{
	ChewingContext *ctx;

	print_function_name();

	ctx = chewing_new();
	type_keystroke_by_string( ctx, "<SR>" );
	ok_keystroke_rtn( ctx, KEYSTROKE_IGNORE );

	chewing_delete( ctx );
}

void test_ShiftRight_add_userphrase()
{
	static const char phrase[] = "\xE6\xB8\xAC\xE8\xA9\xA6" /* 測試 */;
	static const char bopomofo[] = "\xE3\x84\x98\xE3\x84\x9C\xCB\x8B \xE3\x84\x95\xCB\x8B" /* ㄘㄜˋ ㄕˋ */;
	int cursor;
	ChewingContext *ctx;

	print_function_name();

	clean_userphrase();

	ctx = chewing_new();
	chewing_set_maxChiSymbolLen( ctx, 16 );

	ok( has_userphrase( ctx, bopomofo, phrase ) == 0,
		"`%s' shall not be in userphrase", phrase );

	type_keystroke_by_string( ctx, "hk4g4<L><L><SR><SR><E>" );
	ok_preedit_buffer( ctx, phrase );
	cursor = chewing_cursor_Current( ctx );
	ok( cursor == 2, "cursor position `%d' shall be 2", cursor );
	ok( has_userphrase( ctx, bopomofo, phrase ) == 1,
		"`%s' shall be in userphrase", phrase );

	chewing_delete( ctx );
}

void test_ShiftRight()
{
	test_ShiftRight_not_entering_chewing();
	test_ShiftRight_add_userphrase();
}

void test_CtrlNum_add_phrase_right()
{
	static const char phrase[] = "\xE6\xB8\xAC\xE8\xA9\xA6" /* 測試 */;
	static const char bopomofo[] = "\xE3\x84\x98\xE3\x84\x9C\xCB\x8B \xE3\x84\x95\xCB\x8B" /* ㄘㄜˋ ㄕˋ */;
	int cursor;
	ChewingContext *ctx;

	print_function_name();

	clean_userphrase();

	ctx = chewing_new();
	chewing_set_maxChiSymbolLen( ctx, 16 );
	chewing_set_addPhraseDirection( ctx, 0 );

	ok( has_userphrase( ctx, bopomofo, phrase ) == 0,
		"`%s' shall not be in userphrase", phrase );

	type_keystroke_by_string( ctx, "hk4g4<H><C2>" );
	ok_preedit_buffer( ctx, phrase );
	cursor = chewing_cursor_Current( ctx );
	ok( cursor == 0, "cursor position `%d' shall be 0", cursor );
	ok( has_userphrase( ctx, bopomofo, phrase ) == 1,
		"`%s' shall be in userphrase", phrase );

	chewing_delete( ctx );
}

void test_CtrlNum_add_phrase_left()
{
	static const char phrase[] = "\xE6\xB8\xAC\xE8\xA9\xA6" /* 測試 */;
	static const char bopomofo[] = "\xE3\x84\x98\xE3\x84\x9C\xCB\x8B \xE3\x84\x95\xCB\x8B" /* ㄘㄜˋ ㄕˋ */;
	int cursor;
	ChewingContext *ctx;

	print_function_name();

	clean_userphrase();

	ctx = chewing_new();
	chewing_set_maxChiSymbolLen( ctx, 16 );
	chewing_set_addPhraseDirection( ctx, 1 );

	ok( has_userphrase( ctx, bopomofo, phrase ) == 0,
		"`%s' shall not be in userphrase", phrase );

	type_keystroke_by_string( ctx, "hk4g4<C2>" );
	ok_preedit_buffer( ctx, phrase );
	cursor = chewing_cursor_Current( ctx );
	ok( cursor == 2, "cursor position `%d' shall be 2", cursor );
	ok( has_userphrase( ctx, bopomofo, phrase ) == 1,
		"`%s' shall be in userphrase", phrase );

	chewing_delete( ctx );
}

void test_CtrlNum_add_phrase_right_symbol_in_between()
{
	static const char bopomofo[] = "\xE3\x84\x98\xE3\x84\x9C\xCB\x8B \xE3\x84\x95\xCB\x8B" /* ㄘㄜˋ ㄕˋ */;
	int cursor;
	ChewingContext *ctx;

	print_function_name();

	clean_userphrase();

	ctx = chewing_new();
	chewing_set_maxChiSymbolLen( ctx, 16 );
	chewing_set_addPhraseDirection( ctx, 0 );

	ok( has_userphrase( ctx, bopomofo, NULL ) == 0,
		"`%s' shall not be in userphrase", bopomofo );

	type_keystroke_by_string( ctx, "hk4`1g4<H><C2>" );
	cursor = chewing_cursor_Current( ctx );
	ok( cursor == 0, "cursor position `%d' shall be 0", cursor );

	ok( has_userphrase( ctx, bopomofo, NULL ) == 0,
		"`%s' shall not be in userphrase", bopomofo );

	chewing_delete( ctx );
}

void test_CtrlNum_add_phrase_left_symbol_in_between()
{
	static const char bopomofo[] = "\xE3\x84\x98\xE3\x84\x9C\xCB\x8B \xE3\x84\x95\xCB\x8B" /* ㄘㄜˋ ㄕˋ */;
	int cursor;
	ChewingContext *ctx;

	print_function_name();

	clean_userphrase();

	ctx = chewing_new();
	chewing_set_maxChiSymbolLen( ctx, 16 );
	chewing_set_addPhraseDirection( ctx, 1 );

	ok( has_userphrase( ctx, bopomofo, NULL ) == 0,
		"`%s' shall not be in userphrase", bopomofo );

	type_keystroke_by_string( ctx, "hk4`1g4<C2>" );
	cursor = chewing_cursor_Current( ctx );
	ok( cursor == 3, "cursor position `%d' shall be 3", cursor );

	ok( has_userphrase( ctx, bopomofo, NULL ) == 0,
		"`%s' shall not be in userphrase", bopomofo );

	chewing_delete( ctx );
}

void test_CtrlNum()
{
	test_CtrlNum_add_phrase_right();
	test_CtrlNum_add_phrase_left();
	test_CtrlNum_add_phrase_right_symbol_in_between();
	test_CtrlNum_add_phrase_left_symbol_in_between();
}

void test_userphrase_auto_learn()
{
	static const char bopomofo[] = "\xE3\x84\x8E\xE3\x84\x9C \xE3\x84\x8E\xE3\x84\x9C" /* ㄎㄜ ㄎㄜ */;
	ChewingContext *ctx;

	print_function_name();

	clean_userphrase();

	ctx = chewing_new();
	chewing_set_maxChiSymbolLen( ctx, 16 );
	chewing_set_addPhraseDirection( ctx, 1 );

	ok( has_userphrase( ctx, bopomofo, NULL ) == 0,
		"`%s' shall not be in userphrase", bopomofo );

	type_keystroke_by_string( ctx, "dk dk <E>" );
	ok( has_userphrase( ctx, bopomofo, NULL ) == 1,
		"`%s' shall be in userphrase", bopomofo );

	chewing_delete( ctx );
}

void test_userphrase_auto_learn_hardcode_break()
{
	/* 的 is a hardcode break point, see ChewingIsBreakPoint */
	static const char phrase[] = "\xE7\x9A\x84\xE7\x9A\x84" /* 的的 */;
	static const char bopomofo[] = "\xE3\x84\x89\xE3\x84\x9C\xCB\x99 \xE3\x84\x89\xE3\x84\x9C\xCB\x99" /* ㄉㄜ˙ ㄉㄜ˙ */;
	ChewingContext *ctx;

	print_function_name();

	clean_userphrase();

	ctx = chewing_new();
	chewing_set_maxChiSymbolLen( ctx, 16 );
	chewing_set_addPhraseDirection( ctx, 1 );

	ok( has_userphrase( ctx, bopomofo, phrase ) == 0,
		"`%s' shall not be in userphrase", phrase );

	type_keystroke_by_string( ctx, "2k72k7<E>" );
	ok( has_userphrase( ctx, bopomofo, phrase ) == 0,
		"`%s' shall not be in userphrase", phrase );

	chewing_delete( ctx );
}

void test_userphrase()
{
	test_userphrase_auto_learn();
	test_userphrase_auto_learn_hardcode_break();
}

void test_userphrase_enumerate_normal()
{
	ChewingContext *ctx;
	int ret;
	unsigned int expect_len;

	const char phrase[] = "\xE6\xB8\xAC\xE8\xA9\xA6" /* 測試 */;
	char phrase_buf[50];
	unsigned int phrase_len;

	const char bopomofo[] = "\xE3\x84\x98\xE3\x84\x9C\xCB\x8B \xE3\x84\x95\xCB\x8B"; /* ㄘㄜˋ ㄕˋ */
	char bopomofo_buf[50];
	unsigned int bopomofo_len;

	print_function_name();

	clean_userphrase();

	ctx = chewing_new();

	type_keystroke_by_string( ctx, "hk4g4<E>");

	ret = chewing_userphrase_enumerate( ctx );
	ok( ret == 0, "chewing_userphrase_enumerate() return value `%d' shall be `%d'", ret, 0 );

	ret = chewing_userphrase_has_next( ctx, &phrase_len, &bopomofo_len );
	ok( ret == 1, "chewing_userphrase_has_next() return value `%d' shall be `%d'", ret, 1 );
	expect_len = strlen(phrase) + 1;
	ok( phrase_len >= expect_len, "chewing_userphrase_has_next() shall set phrase_len `%d' >= `%d'", phrase_len, expect_len );
	expect_len = strlen(bopomofo) + 1;
	ok( bopomofo_len >= expect_len, "chewing_userphrase_has_next() shall set bopomofo_len `%d' >= `%d'", bopomofo_len, expect_len );
	ret = chewing_userphrase_get( ctx, phrase_buf, sizeof( phrase_buf ), bopomofo_buf, sizeof( bopomofo_buf ) );
	ok( ret == 0, "chewing_userphrase_get() return value `%d' shall be `%d'", ret, 0 );
	ok( strcmp( phrase_buf, phrase ) == 0, "chewing_userphrase_get() shall set phrase_buf `%s' to `%s'", phrase_buf, phrase );
	ok( strcmp( bopomofo_buf, bopomofo ) == 0, "chewing_userphrase_get() shall set bopomofo_buf `%s' to `%s'", bopomofo_buf, bopomofo );

	ret = chewing_userphrase_has_next( ctx, &phrase_len, &bopomofo_len );
	ok( ret == 0, "chewing_userphrase_has_next() return value `%d' shall be `%d'", ret, 0 );

	chewing_delete( ctx );
}

void test_userphrase_enumerate_empty()
{
	ChewingContext *ctx;
	int ret;
	unsigned int phrase_len;
	unsigned int bopomofo_len;

	print_function_name();

	clean_userphrase();

	ctx = chewing_new();

	ret = chewing_userphrase_enumerate( ctx );
	ok( ret == 0, "chewing_userphrase_enumerate() return value `%d' shall be `%d'", ret, 0 );

	ret = chewing_userphrase_has_next( ctx, &phrase_len, &bopomofo_len );
	ok( ret == 0, "chewing_userphrase_has_next() return value `%d' shall be `%d'", ret, 0 );

	chewing_delete( ctx );
}

void test_userphrase_enumerate_rewind()
{
	ChewingContext *ctx;
	int ret;
	unsigned int expect_len;

	const char phrase[] = "\xE6\xB8\xAC\xE8\xA9\xA6" /* 測試 */;
	char phrase_buf[50];
	unsigned int phrase_len;

	const char bopomofo[] = "\xE3\x84\x98\xE3\x84\x9C\xCB\x8B \xE3\x84\x95\xCB\x8B"; /* ㄘㄜˋ ㄕˋ */
	char bopomofo_buf[50];
	unsigned int bopomofo_len;

	print_function_name();

	clean_userphrase();

	ctx = chewing_new();

	type_keystroke_by_string( ctx, "hk4g4<E>");

	ret = chewing_userphrase_enumerate( ctx );
	ok( ret == 0, "chewing_userphrase_enumerate() return value `%d' shall be `%d'", ret, 0 );

	ret = chewing_userphrase_has_next( ctx, &phrase_len, &bopomofo_len );
	ok( ret == 1, "chewing_userphrase_has_next() return value `%d' shall be `%d'", ret, 1 );
	expect_len = strlen(phrase) + 1;
	ok( phrase_len >= expect_len, "chewing_userphrase_has_next() shall set phrase_len `%d' >= `%d'", phrase_len, expect_len );
	expect_len = strlen(bopomofo) + 1;
	ok( bopomofo_len >= expect_len, "chewing_userphrase_has_next() shall set bopomofo_len `%d' >= `%d'", bopomofo_len, expect_len );
	ret = chewing_userphrase_get( ctx, phrase_buf, sizeof( phrase_buf ), bopomofo_buf, sizeof( bopomofo_buf ) );
	ok( ret == 0, "chewing_userphrase_get() return value `%d' shall be `%d'", ret, 0 );
	ok( strcmp( phrase_buf, phrase ) == 0, "chewing_userphrase_get() shall set phrase_buf `%s' to `%s'", phrase_buf, phrase );
	ok( strcmp( bopomofo_buf, bopomofo ) == 0, "chewing_userphrase_get() shall set bopomofo_buf `%s' to `%s'", bopomofo_buf, bopomofo );

	ret = chewing_userphrase_enumerate( ctx );
	ok( ret == 0, "chewing_userphrase_enumerate() return value `%d' shall be `%d'", ret, 0 );

	ret = chewing_userphrase_has_next( ctx, &phrase_len, &bopomofo_len );
	ok( ret == 1, "chewing_userphrase_has_next() return value `%d' shall be `%d'", ret, 1 );
	expect_len = strlen(phrase) + 1;
	ok( phrase_len >= expect_len, "chewing_userphrase_has_next() shall set phrase_len `%d' >= `%d'", phrase_len, expect_len );
	expect_len = strlen(bopomofo) + 1;
	ok( bopomofo_len >= expect_len, "chewing_userphrase_has_next() shall set bopomofo_len `%d' >= `%d'", bopomofo_len, expect_len );
	ret = chewing_userphrase_get( ctx, phrase_buf, sizeof( phrase_buf ), bopomofo_buf, sizeof( bopomofo_buf ) );
	ok( ret == 0, "chewing_userphrase_get() return value `%d' shall be `%d'", ret, 0 );
	ok( strcmp( phrase_buf, phrase ) == 0, "chewing_userphrase_get() shall set phrase_buf `%s' to `%s'", phrase_buf, phrase );
	ok( strcmp( bopomofo_buf, bopomofo ) == 0, "chewing_userphrase_get() shall set bopomofo_buf `%s' to `%s'", bopomofo_buf, bopomofo );

	chewing_delete( ctx );
}

void test_userphrase_enumerate()
{
	test_userphrase_enumerate_normal();
	test_userphrase_enumerate_empty();
	test_userphrase_enumerate_rewind();
}

int main()
{
	putenv( "CHEWING_PATH=" CHEWING_DATA_PREFIX );
	putenv( "CHEWING_USER_PATH=" TEST_HASH_DIR );

	test_ShiftLeft();
	test_ShiftRight();
	test_CtrlNum();
	test_userphrase();
	test_userphrase_enumerate();

	return exit_status();
}
