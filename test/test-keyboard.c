/**
 * test-keyboard.c
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
#include "testhelper.h"

static char *KEYBOARD_STRING[] = {
	"KB_DEFAULT",
	"KB_HSU",
	"KB_IBM",
	"KB_GIN_YIEH",
	"KB_ET",
	"KB_ET26",
	"KB_DVORAK",
	"KB_DVORAK_HSU",
	"KB_DACHEN_CP26",
	"KB_HANYU_PINYIN",
	"KB_THL_PINYIN",
	"KB_MPS2_PINYIN",
};

static const int KEYBOARD_DEFAULT_TYPE = 0;

void test_set_keyboard_type()
{
	ChewingContext *ctx;
	size_t i;
	char *keyboard_string;
	int keyboard_type;

	chewing_Init( 0, 0 );

	ctx = chewing_new();

	keyboard_string = chewing_get_KBString( ctx );
	ok( strcmp( keyboard_string, KEYBOARD_STRING[KEYBOARD_DEFAULT_TYPE] ) == 0,
		"`%s' shall be `%s'", keyboard_string, KEYBOARD_STRING[KEYBOARD_DEFAULT_TYPE] );
	chewing_free( keyboard_string );
	keyboard_type = chewing_get_KBType( ctx );
	ok( keyboard_type == KEYBOARD_DEFAULT_TYPE ,
		"`%d' shall be `%d'", keyboard_type, KEYBOARD_DEFAULT_TYPE );

	for ( i = 0; i < ARRAY_SIZE( KEYBOARD_STRING ); ++i ) {
		ok ( chewing_set_KBType( ctx, i ) == 0, "return shall be 0" );

		keyboard_string = chewing_get_KBString( ctx );
		ok( strcmp( keyboard_string, KEYBOARD_STRING[i] ) == 0,
			"`%s' shall be `%s'", keyboard_string, KEYBOARD_STRING[i] );
		chewing_free( keyboard_string );
		keyboard_type = chewing_get_KBType( ctx );
		ok( keyboard_type == (int)i ,
			"`%d' shall be `%d'", keyboard_type, (int)i );
	}

	// The invalid KBType will reset KBType to default value.
	ok( chewing_set_KBType( ctx, -1 ) == -1, "return shall be -1" );
	keyboard_type = chewing_get_KBType( ctx );
	ok( keyboard_type == KEYBOARD_DEFAULT_TYPE ,
		"`%d' shall be `%d'", keyboard_type, KEYBOARD_DEFAULT_TYPE );

	ok( chewing_set_KBType( ctx, ARRAY_SIZE( KEYBOARD_STRING ) + 1 ),
		"return shall be -1" );
	keyboard_type = chewing_get_KBType( ctx );
	ok( keyboard_type == KEYBOARD_DEFAULT_TYPE ,
		"`%d' shall be `%d'", keyboard_type, KEYBOARD_DEFAULT_TYPE );

	chewing_delete( ctx );
	chewing_Terminate();
}

void test_KBStr2Num()
{
	int i;
	int ret;

	for ( i = 0; i < (int)ARRAY_SIZE( KEYBOARD_STRING ); ++i ) {
		// XXX: chewing_KBStr2Num shall accept const char *.
		ret = chewing_KBStr2Num( KEYBOARD_STRING[i] );
		ok( ret == i, "%d shall be %d", ret, i );
	}
}

void test_enumerate_keyboard_type()
{
	ChewingContext *ctx;
	size_t i;
	char *keyboard_string;

	chewing_Init( 0, 0 );

	ctx = chewing_new();

	ok( chewing_kbtype_Total( ctx ) == ARRAY_SIZE( KEYBOARD_STRING ),
		"total keyboard_string type shall be %d", ARRAY_SIZE( KEYBOARD_STRING ) );

	chewing_kbtype_Enumerate( ctx );
	for ( i = 0; i < ARRAY_SIZE( KEYBOARD_STRING ); ++i ) {
		ok( chewing_kbtype_hasNext( ctx ) == 1 ,
			"shall have next keyboard_string type" );
		keyboard_string = chewing_kbtype_String( ctx );
		ok( strcmp( keyboard_string, KEYBOARD_STRING[i] ) == 0,
			"`%s' shall be `%s'", keyboard_string, KEYBOARD_STRING[i] );
		chewing_free( keyboard_string );
	}
	ok( chewing_kbtype_hasNext( ctx ) == 0 ,
		"shall not have next keyboard_string type" );
	keyboard_string = chewing_kbtype_String( ctx );
	ok( strcmp( keyboard_string, "" ) == 0,
		"`%s' shall be `%s'", keyboard_string, "" );
	chewing_free( keyboard_string );

	chewing_delete( ctx );
	chewing_Terminate();
}

int main()
{
	putenv( "CHEWING_PATH=" CHEWING_DATA_PREFIX );
	putenv( "CHEWING_USER_PATH=" TEST_HASH_DIR );

	test_set_keyboard_type();
	test_KBStr2Num();
	test_enumerate_keyboard_type();

	return exit_status();
}
