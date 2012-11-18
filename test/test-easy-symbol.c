/**
 * test-easy-symbol.c
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
#include <string.h>

#include "chewing.h"
#include "test.h"

static const TestData EASY_SYMBOL[] = {
	{ .token = "Q", .expected = "〔" },
	{ .token = "W", .expected = "〕" },
	{ .token = "A", .expected = "【" },
	{ .token = "S", .expected = "】" },
	{ .token = "Z", .expected = "《" },
	{ .token = "X", .expected = "》" },
	{ .token = "E", .expected = "｛" },
	{ .token = "R", .expected = "｝" },
	{ .token = "D", .expected = "「" },
	{ .token = "F", .expected = "」" },
	{ .token = "C", .expected = "『" },
	{ .token = "V", .expected = "』" },
	{ .token = "T", .expected = "‘" },
	{ .token = "Y", .expected = "’" },
	{ .token = "G", .expected = "“" },
	{ .token = "H", .expected = "”" },
	{ .token = "B", .expected = "〝" },
	{ .token = "N", .expected = "〞" },
	{ .token = "U", .expected = "＋" },
	{ .token = "I", .expected = "－" },
	{ .token = "O", .expected = "×" },
	{ .token = "P", .expected = "÷" },
	{ .token = "J", .expected = "≠" },
	{ .token = "K", .expected = "≒" },
	{ .token = "L", .expected = "Orz" },
	{ .token = "M", .expected = "…" },
};

static const TestData CHINESE = { .token = "hk4g4<E>", .expected = "測試" };

void test_type_easy_symbol()
{
	chewing_Init( NULL, NULL );

	ChewingContext *ctx = chewing_new();
	ok( ctx, "chewing_new shall not return NULL" );

	chewing_set_maxChiSymbolLen( ctx, 16 );
	chewing_set_easySymbolInput( ctx, 1 );

	for ( int i = 0; i < ARRAY_SIZE( EASY_SYMBOL ); ++i ) {
		type_keystoke_by_string( ctx, EASY_SYMBOL[i].token );
		ok_preedit_buffer( ctx, EASY_SYMBOL[i].expected );
		type_keystoke_by_string( ctx, "<E>" );
		ok_commit_buffer( ctx, EASY_SYMBOL[i].expected );
	}

	chewing_delete( ctx );
	chewing_Terminate();
}

void test_mode_change()
{
	chewing_Init( NULL, NULL );

	ChewingContext *ctx = chewing_new();
	ok( ctx, "chewing_new shall not return NULL" );

	chewing_set_maxChiSymbolLen( ctx, 16 );

	type_keystoke_by_string( ctx, CHINESE.token );
	ok_commit_buffer( ctx, CHINESE.expected );

	chewing_set_easySymbolInput( ctx, 1 );
	type_keystoke_by_string( ctx, EASY_SYMBOL[0].token );
	type_keystoke_by_string( ctx, "<E>" );
	ok_commit_buffer( ctx, EASY_SYMBOL[0].expected );

	chewing_set_easySymbolInput( ctx, 0 );
	type_keystoke_by_string( ctx, CHINESE.token );
	ok_commit_buffer( ctx, CHINESE.expected );

	chewing_delete( ctx );
	chewing_Terminate();
}

int main()
{
	putenv( "CHEWING_PATH=" CHEWING_DATA_PREFIX );
	putenv( "CHEWING_USER_PATH=" TEST_HASH_DIR );

	test_type_easy_symbol();
	test_mode_change();
	return exit_status();
}
