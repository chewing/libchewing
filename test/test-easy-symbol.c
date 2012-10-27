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
	{ .token = "Q<E>", .expected = "〔" },
	{ .token = "W<E>", .expected = "〕" },
	{ .token = "A<E>", .expected = "【" },
	{ .token = "S<E>", .expected = "】" },
	{ .token = "Z<E>", .expected = "《" },
	{ .token = "X<E>", .expected = "》" },
	{ .token = "E<E>", .expected = "｛" },
	{ .token = "R<E>", .expected = "｝" },
	{ .token = "D<E>", .expected = "「" },
	{ .token = "F<E>", .expected = "」" },
	{ .token = "C<E>", .expected = "『" },
	{ .token = "V<E>", .expected = "』" },
	{ .token = "T<E>", .expected = "‘" },
	{ .token = "Y<E>", .expected = "’" },
	{ .token = "G<E>", .expected = "“" },
	{ .token = "H<E>", .expected = "”" },
	{ .token = "B<E>", .expected = "〝" },
	{ .token = "N<E>", .expected = "〞" },
	{ .token = "U<E>", .expected = "＋" },
	{ .token = "I<E>", .expected = "－" },
	{ .token = "O<E>", .expected = "×" },
	{ .token = "P<E>", .expected = "÷" },
	{ .token = "J<E>", .expected = "≠" },
	{ .token = "K<E>", .expected = "≒" },
	{ .token = "L<E>", .expected = "Orz" },
	{ .token = "M<E>", .expected = "…" },
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
		verify_keystoke( ctx,
			EASY_SYMBOL[i].token, EASY_SYMBOL[i].expected );
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

	verify_keystoke( ctx, CHINESE.token, CHINESE.expected );

	chewing_set_easySymbolInput( ctx, 1 );
	verify_keystoke( ctx, EASY_SYMBOL[0].token, EASY_SYMBOL[0].expected );

	chewing_set_easySymbolInput( ctx, 0 );
	verify_keystoke( ctx, CHINESE.token, CHINESE.expected );

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
