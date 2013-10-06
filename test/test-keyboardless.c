/**
 * test-keyboardless-api.c
 *
 * Copyright (c) 2013
 *	libchewing Core Team. See ChangeLog for details.
 *
 * See the file "COPYING" for information on usage and redistribution
 * of this file.
 */
#include <stdlib.h>

#include "testhelper.h"
#include "chewing.h"

void test_cand_open_word()
{
	ChewingContext *ctx;
	int ret;

	print_function_name();

	ctx = chewing_new();

	type_keystroke_by_string( ctx, "hk4" /* 測 */ );
	ret = chewing_cand_open( ctx );
	ok( ret == 0, "chewing_cand_open() returns `%d' shall be `%d'", ret, 0 );

	ret = chewing_cand_TotalChoice( ctx );
	ok ( ret > 0, "chewing_cand_TotalChoice() returns `%d' shall be greater than `%d'", ret, 0 );

	chewing_delete( ctx );
}

void test_cand_open_symbol()
{
	ChewingContext *ctx;
	int ret;

	print_function_name();

	ctx = chewing_new();

	type_keystroke_by_string( ctx, "`31" /* ， */ );
	ret = chewing_cand_open( ctx );
	ok( ret == 0, "chewing_cand_open() returns `%d' shall be `%d'", ret, 0 );

	ret = chewing_cand_TotalChoice( ctx );
	ok ( ret > 0, "chewing_cand_TotalChoice() returns `%d' shall be greater than `%d'", ret, 0 );

	chewing_delete( ctx );
}

void test_cand_open_already_open()
{
	ChewingContext *ctx;
	int ret;

	print_function_name();

	ctx = chewing_new();

	type_keystroke_by_string( ctx, "hk4" /* 測 */ );
	ret = chewing_cand_open( ctx );
	ok( ret == 0, "chewing_cand_open() returns `%d' shall be `%d'", ret, 0 );

	ret = chewing_cand_TotalChoice( ctx );
	ok ( ret > 0, "chewing_cand_TotalChoice() returns `%d' shall be greater than `%d'", ret, 0 );

	ret = chewing_cand_open( ctx );
	ok( ret == 0, "chewing_cand_open() returns `%d' shall be `%d'", ret, 0 );

	ret = chewing_cand_TotalChoice( ctx );
	ok ( ret > 0, "chewing_cand_TotalChoice() returns `%d' shall be greater than `%d'", ret, 0 );

	chewing_delete( ctx );
}

void test_cand_open_nothing_in_preedit()
{
	ChewingContext *ctx;
	int ret;

	print_function_name();

	ctx = chewing_new();

	ret = chewing_cand_open( ctx );
	ok( ret == -1, "chewing_cand_open() returns `%d' shall be `%d'", ret, -1 );

	ret = chewing_cand_TotalChoice( ctx );
	ok ( ret == 0, "chewing_cand_TotalChoice() returns `%d' shall be `%d'", ret, 0 );

	chewing_delete( ctx );
}

void test_cand_open_during_bopomofo()
{
	ChewingContext *ctx;
	int ret;

	print_function_name();

	/* FIXME: shall we clean bopomofo when chewing_cand_open is called? */

	ctx = chewing_new();

	type_keystroke_by_string( ctx, "hk" /* ㄘㄜ */ );
	ret = chewing_cand_open( ctx );
	ok( ret == -1, "chewing_cand_open() returns `%d' shall be `%d'", ret, -1 );

	ret = chewing_cand_TotalChoice( ctx );
	ok ( ret == 0, "chewing_cand_TotalChoice() returns `%d' shall be `%d'", ret, 0 );

	chewing_Reset( ctx );

	type_keystroke_by_string( ctx, "hk4g" /* 測ㄕ */ );
	ret = chewing_cand_open( ctx );
	ok( ret == 0, "chewing_cand_open() returns `%d' shall be `%d'", ret, 0 );

	ret = chewing_cand_TotalChoice( ctx );
	ok ( ret > 0, "chewing_cand_TotalChoice() returns `%d' shall be greater than `%d'", ret, 0 );

	chewing_delete( ctx );
}

void test_cand_open()
{
	test_cand_open_word();
	test_cand_open_symbol();
	test_cand_open_already_open();
	test_cand_open_nothing_in_preedit();
	test_cand_open_during_bopomofo();
}

int main()
{
	putenv( "CHEWING_PATH=" CHEWING_DATA_PREFIX );
	putenv( "CHEWING_USER_PATH=" TEST_HASH_DIR );

	test_cand_open();

	return exit_status();
}
