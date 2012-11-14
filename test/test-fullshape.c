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
#include "test.h"

static const TestData FULLSHAPE_DATA[] = {
	{ .token = "0<E>", .expected = "０" },
	{ .token = "1<E>", .expected = "１" },
	{ .token = "2<E>", .expected = "２" },
	{ .token = "3<E>", .expected = "３" },
	{ .token = "4<E>", .expected = "４" },
	{ .token = "5<E>", .expected = "５" },
	{ .token = "6<E>", .expected = "６" },
	{ .token = "7<E>", .expected = "７" },
	{ .token = "8<E>", .expected = "８" },
	{ .token = "9<E>", .expected = "９" },
	{ .token = "a<E>", .expected = "ａ" },
	{ .token = "b<E>", .expected = "ｂ" },
	{ .token = "c<E>", .expected = "ｃ" },
	{ .token = "d<E>", .expected = "ｄ" },
	{ .token = "e<E>", .expected = "ｅ" },
	{ .token = "f<E>", .expected = "ｆ" },
	{ .token = "g<E>", .expected = "ｇ" },
	{ .token = "h<E>", .expected = "ｈ" },
	{ .token = "i<E>", .expected = "ｉ" },
	{ .token = "j<E>", .expected = "ｊ" },
	{ .token = "k<E>", .expected = "ｋ" },
	{ .token = "l<E>", .expected = "ｌ" },
	{ .token = "m<E>", .expected = "ｍ" },
	{ .token = "n<E>", .expected = "ｎ" },
	{ .token = "o<E>", .expected = "ｏ" },
	{ .token = "p<E>", .expected = "ｐ" },
	{ .token = "q<E>", .expected = "ｑ" },
	{ .token = "r<E>", .expected = "ｒ" },
	{ .token = "s<E>", .expected = "ｓ" },
	{ .token = "t<E>", .expected = "ｔ" },
	{ .token = "u<E>", .expected = "ｕ" },
	{ .token = "v<E>", .expected = "ｖ" },
	{ .token = "w<E>", .expected = "ｗ" },
	{ .token = "x<E>", .expected = "ｘ" },
	{ .token = "y<E>", .expected = "ｙ" },
	{ .token = "z<E>", .expected = "ｚ" },
	{ .token = "A<E>", .expected = "Ａ" },
	{ .token = "B<E>", .expected = "Ｂ" },
	{ .token = "C<E>", .expected = "Ｃ" },
	{ .token = "D<E>", .expected = "Ｄ" },
	{ .token = "E<E>", .expected = "Ｅ" },
	{ .token = "F<E>", .expected = "Ｆ" },
	{ .token = "G<E>", .expected = "Ｇ" },
	{ .token = "H<E>", .expected = "Ｈ" },
	{ .token = "I<E>", .expected = "Ｉ" },
	{ .token = "J<E>", .expected = "Ｊ" },
	{ .token = "K<E>", .expected = "Ｋ" },
	{ .token = "L<E>", .expected = "Ｌ" },
	{ .token = "M<E>", .expected = "Ｍ" },
	{ .token = "N<E>", .expected = "Ｎ" },
	{ .token = "O<E>", .expected = "Ｏ" },
	{ .token = "P<E>", .expected = "Ｐ" },
	{ .token = "Q<E>", .expected = "Ｑ" },
	{ .token = "R<E>", .expected = "Ｒ" },
	{ .token = "S<E>", .expected = "Ｓ" },
	{ .token = "T<E>", .expected = "Ｔ" },
	{ .token = "U<E>", .expected = "Ｕ" },
	{ .token = "V<E>", .expected = "Ｖ" },
	{ .token = "W<E>", .expected = "Ｗ" },
	{ .token = "X<E>", .expected = "Ｘ" },
	{ .token = "Y<E>", .expected = "Ｙ" },
	{ .token = "Z<E>", .expected = "Ｚ" },
	{ .token = " <E>", .expected = "　" },
	{ .token = "\"<E>", .expected = "”" },
	{ .token = "'<E>", .expected = "’" },
	{ .token = "/<E>", .expected = "／" },
	{ .token = "<<><E>", .expected = "＜" },
	{ .token = "><E>", .expected = "＞" },
	{ .token = "`<E>", .expected = "‵" },
	{ .token = "[<E>", .expected = "〔" },
	{ .token = "]<E>", .expected = "〕" },
	{ .token = "{<E>", .expected = "｛" },
	{ .token = "}<E>", .expected = "｝" },
	{ .token = "+<E>", .expected = "＋" },
	{ .token = "-<E>", .expected = "－" },
};

void test_fullshape_input()
{
	chewing_Init( NULL, NULL );

	ChewingContext *ctx = chewing_new();
	ok( ctx, "chewing_new shall not return NULL" );

	chewing_set_ChiEngMode( ctx, SYMBOL_MODE );
	chewing_set_ShapeMode( ctx, FULLSHAPE_MODE );

	for ( int i = 0; i < ARRAY_SIZE( FULLSHAPE_DATA ); ++i ) {
		type_keystoke_by_string( ctx,
			FULLSHAPE_DATA[i].token  );
		ok_commit_buffer( ctx, FULLSHAPE_DATA[i].expected );
	}

	chewing_set_spaceAsSelection( ctx, 1 );
	type_keystoke_by_string( ctx, " <E>" );
	ok_commit_buffer( ctx,  "　" );

	chewing_delete( ctx );
	chewing_Terminate();
}

void test_set_fullshape()
{
	chewing_Init( NULL, NULL );

	ChewingContext *ctx = chewing_new();
	ok( ctx, "chewing_new shall not return NULL" );

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
