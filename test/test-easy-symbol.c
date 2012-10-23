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

#include "test_harness.h"

typedef struct {
	char * token;
	char * expected;
} TestData;

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

static const TestData CHINESE[] = {
	{ .token = "hk4g4", .expected = "測試" },
};

void verify_test_data( ChewingContext *ctx, const TestData *data )
{
	for ( int i = 0; i < strlen(data->token); ++i ) {
		chewing_handle_Default( ctx, data->token[i] );
	}
	chewing_handle_Enter( ctx );

	char *buf = chewing_commit_String( ctx );
	ok( !strcmp( buf, data->expected ), "output shall be expected value" );
	chewing_free( buf );
}

void test_type_easy_symbol()
{
	putenv( "CHEWING_PATH=" CHEWING_DATA_PREFIX );
	putenv( "CHEWING_USER_PATH=" TEST_HASH_DIR );

	chewing_Init( NULL, NULL );

	ChewingContext *ctx = chewing_new();
	ok( ctx, "chewing_new shall not return NULL" );

	chewing_set_maxChiSymbolLen( ctx, 16 );
	chewing_set_easySymbolInput( ctx, 1 );

	for ( int i = 0; i < sizeof( EASY_SYMBOL ) / sizeof( EASY_SYMBOL[0] );
		++i ) {
		verify_test_data( ctx, &EASY_SYMBOL[i] );
	}

	chewing_delete( ctx );
	chewing_Terminate();
}

void test_mode_change()
{
	putenv( "CHEWING_PATH=" CHEWING_DATA_PREFIX );
	putenv( "CHEWING_USER_PATH=" TEST_HASH_DIR );

	chewing_Init( NULL, NULL );

	ChewingContext *ctx = chewing_new();
	ok( ctx, "chewing_new shall not return NULL" );

	chewing_set_maxChiSymbolLen( ctx, 16 );

	verify_test_data( ctx, &CHINESE[0] );

	chewing_set_easySymbolInput( ctx, 1 );
	verify_test_data( ctx, &EASY_SYMBOL[0] );

	chewing_set_easySymbolInput( ctx, 0 );
	verify_test_data( ctx, &CHINESE[0] );

	chewing_delete( ctx );
	chewing_Terminate();
}

int main()
{
	test_type_easy_symbol();
	test_mode_change();
	return exit_status();
}
