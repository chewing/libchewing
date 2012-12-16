/**
 * test-fullshape.c
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

#include "chewing.h"
#include "testhelper.h"

static const TestData FULLSHAPE_DATA[] = {
	{ .token = "0", .expected = "０" },
	{ .token = "1", .expected = "１" },
	{ .token = "2", .expected = "２" },
	{ .token = "3", .expected = "３" },
	{ .token = "4", .expected = "４" },
	{ .token = "5", .expected = "５" },
	{ .token = "6", .expected = "６" },
	{ .token = "7", .expected = "７" },
	{ .token = "8", .expected = "８" },
	{ .token = "9", .expected = "９" },
	{ .token = "a", .expected = "ａ" },
	{ .token = "b", .expected = "ｂ" },
	{ .token = "c", .expected = "ｃ" },
	{ .token = "d", .expected = "ｄ" },
	{ .token = "e", .expected = "ｅ" },
	{ .token = "f", .expected = "ｆ" },
	{ .token = "g", .expected = "ｇ" },
	{ .token = "h", .expected = "ｈ" },
	{ .token = "i", .expected = "ｉ" },
	{ .token = "j", .expected = "ｊ" },
	{ .token = "k", .expected = "ｋ" },
	{ .token = "l", .expected = "ｌ" },
	{ .token = "m", .expected = "ｍ" },
	{ .token = "n", .expected = "ｎ" },
	{ .token = "o", .expected = "ｏ" },
	{ .token = "p", .expected = "ｐ" },
	{ .token = "q", .expected = "ｑ" },
	{ .token = "r", .expected = "ｒ" },
	{ .token = "s", .expected = "ｓ" },
	{ .token = "t", .expected = "ｔ" },
	{ .token = "u", .expected = "ｕ" },
	{ .token = "v", .expected = "ｖ" },
	{ .token = "w", .expected = "ｗ" },
	{ .token = "x", .expected = "ｘ" },
	{ .token = "y", .expected = "ｙ" },
	{ .token = "z", .expected = "ｚ" },
	{ .token = "A", .expected = "Ａ" },
	{ .token = "B", .expected = "Ｂ" },
	{ .token = "C", .expected = "Ｃ" },
	{ .token = "D", .expected = "Ｄ" },
	{ .token = "E", .expected = "Ｅ" },
	{ .token = "F", .expected = "Ｆ" },
	{ .token = "G", .expected = "Ｇ" },
	{ .token = "H", .expected = "Ｈ" },
	{ .token = "I", .expected = "Ｉ" },
	{ .token = "J", .expected = "Ｊ" },
	{ .token = "K", .expected = "Ｋ" },
	{ .token = "L", .expected = "Ｌ" },
	{ .token = "M", .expected = "Ｍ" },
	{ .token = "N", .expected = "Ｎ" },
	{ .token = "O", .expected = "Ｏ" },
	{ .token = "P", .expected = "Ｐ" },
	{ .token = "Q", .expected = "Ｑ" },
	{ .token = "R", .expected = "Ｒ" },
	{ .token = "S", .expected = "Ｓ" },
	{ .token = "T", .expected = "Ｔ" },
	{ .token = "U", .expected = "Ｕ" },
	{ .token = "V", .expected = "Ｖ" },
	{ .token = "W", .expected = "Ｗ" },
	{ .token = "X", .expected = "Ｘ" },
	{ .token = "Y", .expected = "Ｙ" },
	{ .token = "Z", .expected = "Ｚ" },
	{ .token = " ", .expected = "　" },
	{ .token = "\"", .expected = "”" },
	{ .token = "'", .expected = "’" },
	{ .token = "/", .expected = "／" },
	{ .token = "<<>", .expected = "＜" },
	{ .token = ">", .expected = "＞" },
	{ .token = "`", .expected = "‵" },
	{ .token = "[", .expected = "〔" },
	{ .token = "]", .expected = "〕" },
	{ .token = "{", .expected = "｛" },
	{ .token = "}", .expected = "｝" },
	{ .token = "+", .expected = "＋" },
	{ .token = "-", .expected = "－" },
};

void test_fullshape_input()
{
	chewing_Init( NULL, NULL );

	ChewingContext *ctx = chewing_new();

	chewing_set_ChiEngMode( ctx, SYMBOL_MODE );
	chewing_set_ShapeMode( ctx, FULLSHAPE_MODE );

	for ( int i = 0; i < ARRAY_SIZE( FULLSHAPE_DATA ); ++i ) {
		type_keystoke_by_string( ctx, FULLSHAPE_DATA[i].token );
		// fullshape symbol does not present in preedit buffer.
		ok_preedit_buffer( ctx, "" );
		ok_commit_buffer( ctx, FULLSHAPE_DATA[i].expected );
	}

	chewing_delete( ctx );
	chewing_Terminate();
}

void test_set_fullshape()
{
	chewing_Init( NULL, NULL );

	ChewingContext *ctx = chewing_new();

	ok( chewing_get_ShapeMode( ctx ) == HALFSHAPE_MODE,
		"default is HALFSHAPE_MODE" );

	chewing_set_ShapeMode( ctx, FULLSHAPE_MODE );
	ok( chewing_get_ShapeMode( ctx ) == FULLSHAPE_MODE,
		"mode shall change to FULLSHAPE_MODE" );

	// XXX: What is the correct behavior when input parameter is wrong?
//	chewing_set_ShapeMode( ctx, -1 );
//	ok( chewing_get_ShapeMode( ctx ) == FULLSHAPE_MODE,
//		"mode shall not change when parameter is invalid" );

	chewing_set_ShapeMode( ctx, HALFSHAPE_MODE );
	ok( chewing_get_ShapeMode( ctx ) == HALFSHAPE_MODE,
		"mode shall change to HALFSHAPE_MODE" );

	chewing_set_ShapeMode( ctx, -1 );
	ok( chewing_get_ShapeMode( ctx ) == HALFSHAPE_MODE,
		"mode shall not change when parameter is invalid" );


	chewing_delete( ctx );
	chewing_Terminate();
}

int main()
{
	putenv( "CHEWING_PATH=" CHEWING_DATA_PREFIX );
	putenv( "CHEWING_USER_PATH=" TEST_HASH_DIR );

	test_set_fullshape();
	test_fullshape_input();

	return exit_status();
}
