/**
 * test-config.c
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

#include <assert.h>
#include <stdlib.h>
#include <string.h>

#include "chewing.h"
#include "testhelper.h"

static const int MIN_CAND_PER_PAGE = 1;
static const int MAX_CAND_PER_PAGE = 10;
static const int DEFAULT_CAND_PER_PAGE = 10;
static const int MIN_CHI_SYMBOL_LEN = 0;
static const int MAX_CHI_SYMBOL_LEN = 39;

static const int DEFAULT_SELECT_KEY[] = {
	'1', '2', '3', '4', '5', '6', '7', '8', '9', '0' };

static int ALTERNATE_SELECT_KEY[] = {
	'a', 's', 'd', 'f', 'g', 'h', 'j', 'k', 'l', ';' };

const TestData DATA = { "`a", "\xE2\x80\xA6" /* … */ };

FILE *fd;

void test_default_value()
{
	int *select_key;
	ChewingContext *ctx;

	ctx = chewing_new();
	start_testcase( ctx, fd );

	select_key = chewing_get_selKey( ctx );
	ok( select_key, "chewing_get_selKey shall not return NULL" );
	ok( !memcmp( select_key, DEFAULT_SELECT_KEY,
		sizeof( DEFAULT_SELECT_KEY )),
		"default select key shall be default value" );
	chewing_free( select_key );

	ok( chewing_get_candPerPage( ctx ) == DEFAULT_CAND_PER_PAGE,
		"default candPerPage shall be %d", DEFAULT_CAND_PER_PAGE );

	ok( chewing_get_maxChiSymbolLen( ctx ) == MAX_CHI_SYMBOL_LEN,
		"default maxChiSymbolLen shall be %d", MAX_CHI_SYMBOL_LEN );

	ok( chewing_get_addPhraseDirection( ctx ) == 0,
		"default addPhraseDirection shall be 0" );

	ok( chewing_get_spaceAsSelection( ctx ) == 0,
		"default spaceAsSelection shall be 0" );

	ok( chewing_get_escCleanAllBuf( ctx ) == 0,
		"default escCleanAllBuf shall be 0" );

	ok( chewing_get_hsuSelKeyType( ctx ) == 0,
		"default hsuSelKeyType shall be 0" );

	ok( chewing_get_autoShiftCur( ctx ) == 0,
		"default autoShiftCur shall be 0" );

	ok( chewing_get_easySymbolInput( ctx ) == 0,
		"default easySymbolInput shall be 0" );

	ok( chewing_get_phraseChoiceRearward( ctx ) == 0,
		"default phraseChoiceRearward shall be 0" );

	ok( chewing_get_ChiEngMode( ctx ) == CHINESE_MODE,
		"default ChiEngMode shall be CHINESE_MODE" );

	ok( chewing_get_ShapeMode( ctx ) == HALFSHAPE_MODE,
		"default ShapeMode shall be HALFSHAPE_MODE" );

	chewing_delete( ctx );
}

void test_set_candPerPage()
{
	const int VALUE[] = {
		MIN_CAND_PER_PAGE,
		MAX_CAND_PER_PAGE,
	};

	const int INVALID_VALUE[] = {
		MIN_CAND_PER_PAGE - 1,
		MAX_CAND_PER_PAGE + 1,
	};

	ChewingContext *ctx;
	size_t i;
	size_t j;

	ctx = chewing_new();
	start_testcase( ctx, fd );

	for ( i = 0; i < ARRAY_SIZE( VALUE ); ++i ) {
		chewing_set_candPerPage( ctx, VALUE[i] );
		ok( chewing_get_candPerPage( ctx ) == VALUE[i],
			"candPerPage shall be `%d'", VALUE[i] );

		for ( j = 0; j < ARRAY_SIZE( INVALID_VALUE ); ++j ) {
			// mode shall not change when set mode has invalid value.
			chewing_set_candPerPage( ctx, INVALID_VALUE[j] );
			ok( chewing_get_candPerPage( ctx ) == VALUE[i],
				"candPerPage shall be `%d'", VALUE[i] );
		}
	}

	chewing_delete( ctx );
}

void test_set_maxChiSymbolLen()
{
	ChewingContext *ctx;
	int i;

	ctx = chewing_new();
	start_testcase( ctx, fd );

	chewing_set_maxChiSymbolLen( ctx, 16 );
	ok( chewing_get_maxChiSymbolLen( ctx ) == 16,
		"maxChiSymbolLen shall be 16" );

	chewing_set_maxChiSymbolLen( ctx, MIN_CHI_SYMBOL_LEN - 1 );
	ok( chewing_get_maxChiSymbolLen( ctx ) == 16,
		"maxChiSymbolLen shall not change when set to %d",
		MIN_CHI_SYMBOL_LEN - 1 );

	chewing_set_maxChiSymbolLen( ctx, MAX_CHI_SYMBOL_LEN + 1 );
	ok( chewing_get_maxChiSymbolLen( ctx ) == 16,
		"maxChiSymbolLen shall not change when set to %d",
		MAX_CHI_SYMBOL_LEN + 1 );


	// Test auto commit
	chewing_set_maxChiSymbolLen( ctx, MAX_CHI_SYMBOL_LEN );

	// In boundary
	for ( i = 0; i < MAX_CHI_SYMBOL_LEN; ++i )
		type_keystroke_by_string( ctx, "hk4" );
	ok( chewing_commit_Check( ctx ) == 0,
		"auto commit shall not be triggered when entering %d symbols",
		MAX_CHI_SYMBOL_LEN );

	// Out of boundary
	type_keystroke_by_string( ctx, "hk4" );
	ok( chewing_commit_Check( ctx ) == 1,
		"auto commit shall be triggered when entering %d symbols",
		MAX_CHI_SYMBOL_LEN + 1);

	chewing_delete( ctx );
}

void test_maxChiSymbolLen()
{
	ChewingContext *ctx;
	int i;

	ctx = chewing_new();
	start_testcase( ctx, fd );

	chewing_set_maxChiSymbolLen( ctx, MAX_CHI_SYMBOL_LEN );

	for ( i = 0; i < MAX_CHI_SYMBOL_LEN; ++i ) {
		type_keystroke_by_string( ctx, "hk4" );
	}

	// Use easy symbol 'Orz' as last input for worst case scenario.
	chewing_set_easySymbolInput( ctx, 1 );
	type_keystroke_by_string( ctx, "L" );

	chewing_delete( ctx );
}

void test_set_selKey_normal()
{
	ChewingContext *ctx;
	int *select_key;

	ctx = chewing_new();
	start_testcase( ctx, fd );

	// XXX: chewing_set_selKey shall accept const char *.
	chewing_set_selKey( ctx,
		ALTERNATE_SELECT_KEY, ARRAY_SIZE( ALTERNATE_SELECT_KEY ));
	select_key = chewing_get_selKey( ctx );
	ok( select_key, "chewing_get_selKey shall not return NULL" );
	ok( !memcmp( select_key, ALTERNATE_SELECT_KEY,
		sizeof( ALTERNATE_SELECT_KEY )),
		"select key shall be ALTERNATE_SELECT_KEY");

	type_keystroke_by_string( ctx, DATA.token );
	ok_preedit_buffer( ctx, DATA.expected );

	chewing_free( select_key );

	chewing_delete( ctx );
}

void test_set_selKey_error_handling()
{
	ChewingContext *ctx;
	int *select_key;

	ctx = chewing_new();
	start_testcase( ctx, fd );

	chewing_set_selKey( NULL, ALTERNATE_SELECT_KEY, ARRAY_SIZE( ALTERNATE_SELECT_KEY ) );
	select_key = chewing_get_selKey( ctx );
	ok( select_key, "chewing_get_selKey shall not return NULL" );
	ok( !memcmp( select_key, DEFAULT_SELECT_KEY,
		sizeof( DEFAULT_SELECT_KEY )),
		"select key shall be DEFAULT_SELECT_KEY");
	chewing_free( select_key );

	chewing_set_selKey( ctx, NULL, ARRAY_SIZE( ALTERNATE_SELECT_KEY ) );
	select_key = chewing_get_selKey( ctx );
	ok( select_key, "chewing_get_selKey shall not return NULL" );
	ok( !memcmp( select_key, DEFAULT_SELECT_KEY,
		sizeof( DEFAULT_SELECT_KEY )),
		"select key shall be DEFAULT_SELECT_KEY");
	chewing_free( select_key );

	chewing_set_selKey( ctx, ALTERNATE_SELECT_KEY, 0 );
	select_key = chewing_get_selKey( ctx );
	ok( select_key, "chewing_get_selKey shall not return NULL" );
	ok( !memcmp( select_key, DEFAULT_SELECT_KEY,
		sizeof( DEFAULT_SELECT_KEY )),
		"select key shall be DEFAULT_SELECT_KEY");
	chewing_free( select_key );

	chewing_set_selKey( ctx, ALTERNATE_SELECT_KEY, 11 );
	select_key = chewing_get_selKey( ctx );
	ok( select_key, "chewing_get_selKey shall not return NULL" );
	ok( !memcmp( select_key, DEFAULT_SELECT_KEY,
		sizeof( DEFAULT_SELECT_KEY )),
		"select key shall be DEFAULT_SELECT_KEY");
	chewing_free( select_key );

	chewing_delete( ctx );
}

void test_set_selKey()
{
	test_set_selKey_normal();
	test_set_selKey_error_handling();
}

void test_set_addPhraseDirection()
{
	ChewingContext *ctx;
	int value;
	int mode;

	ctx = chewing_new();
	start_testcase( ctx, fd );

	for ( value = 0; value < 2; ++value ) {
		chewing_set_addPhraseDirection( ctx, value );
		mode = chewing_get_addPhraseDirection( ctx );
		ok( mode == value,
			"addPhraseDirection `%d' shall be `%d'", mode, value );

		chewing_set_addPhraseDirection( ctx, -1 );
		mode = chewing_get_addPhraseDirection( ctx );
		ok( mode == value,
			"addPhraseDirection `%d' shall be `%d'", mode, value );

		chewing_set_addPhraseDirection( ctx, 2 );
		mode = chewing_get_addPhraseDirection( ctx );
		ok( mode == value,
			"addPhraseDirection `%d' shall be `%d'", mode, value );
	}

	chewing_delete( ctx );
}

void test_set_spaceAsSelection()
{
	ChewingContext *ctx;
	int value;
	int mode;

	ctx = chewing_new();
	start_testcase( ctx, fd );

	for ( value = 0; value < 2; ++value ) {
		chewing_set_spaceAsSelection( ctx, value );
		mode = chewing_get_spaceAsSelection( ctx );
		ok( mode == value,
			"spaceAsSelection `%d' shall be `%d'", mode, value );

		chewing_set_spaceAsSelection( ctx, -1 );
		mode = chewing_get_spaceAsSelection( ctx );
		ok( mode == value,
			"spaceAsSelection `%d' shall be `%d'", mode, value );

		chewing_set_spaceAsSelection( ctx, 2 );
		mode = chewing_get_spaceAsSelection( ctx );
		ok( mode == value,
			"spaceAsSelection `%d' shall be `%d'", mode, value );
	}

	chewing_delete( ctx );
}

void test_set_escCleanAllBuf()
{
	ChewingContext *ctx;
	int value;
	int mode;

	ctx = chewing_new();
	start_testcase( ctx, fd );

	for ( value = 0; value < 2; ++value ) {
		chewing_set_escCleanAllBuf( ctx, value );
		mode = chewing_get_escCleanAllBuf( ctx );
		ok( mode == value,
			"escCleanAllBuf shall be `%d'", value );

		chewing_set_escCleanAllBuf( ctx, -1 );
		mode = chewing_get_escCleanAllBuf( ctx );
		ok( mode == value,
			"escCleanAllBuf shall be `%d'", value );

		chewing_set_escCleanAllBuf( ctx, 2 );
		mode = chewing_get_escCleanAllBuf( ctx );
		ok( mode == value,
			"escCleanAllBuf shall be `%d'", value );
	}

	chewing_delete( ctx );
}

void test_set_autoShiftCur()
{
	ChewingContext *ctx;
	int value;
	int mode;

	ctx = chewing_new();
	start_testcase( ctx, fd );

	for ( value = 0; value < 2; ++value ) {
		chewing_set_autoShiftCur( ctx, value );
		mode = chewing_get_autoShiftCur( ctx );
		ok( mode = chewing_get_autoShiftCur( ctx ) == value,
			"autoShiftCur shall be `%d'", value );

		chewing_set_autoShiftCur( ctx, -1 );
		mode = chewing_get_autoShiftCur( ctx );
		ok( mode = chewing_get_autoShiftCur( ctx ) == value,
			"autoShiftCur shall be `%d'", value );

		chewing_set_autoShiftCur( ctx, 2 );
		mode = chewing_get_autoShiftCur( ctx );
		ok( mode = chewing_get_autoShiftCur( ctx ) == value,
			"autoShiftCur shall be `%d'", value );
	}

	chewing_delete( ctx );
}

void test_set_easySymbolInput()
{
	ChewingContext *ctx;
	int value;
	int mode;

	ctx = chewing_new();
	start_testcase( ctx, fd );

	for ( value = 0; value < 2; ++value ) {
		chewing_set_easySymbolInput( ctx, value );
		mode = chewing_get_easySymbolInput( ctx );
		ok( mode == value,
			"easySymbolInput `%d', shall be `%d'", mode, value );

		chewing_set_easySymbolInput( ctx, -1 );
		mode = chewing_get_easySymbolInput( ctx );
		ok( mode == value,
			"easySymbolInput `%d', shall be `%d'", mode, value );

		chewing_set_easySymbolInput( ctx, 2 );
		mode = chewing_get_easySymbolInput( ctx );
		ok( mode == value,
			"easySymbolInput `%d', shall be `%d'", mode, value );
	}

	chewing_delete( ctx );
}

void test_set_phraseChoiceRearward()
{
	ChewingContext *ctx;
	int value;
	int mode;

	ctx = chewing_new();
	start_testcase( ctx, fd );

	for ( value = 0; value < 2; ++value ) {
		chewing_set_phraseChoiceRearward( ctx, value );
		mode = chewing_get_phraseChoiceRearward( ctx );
		ok(  mode == value,
			"phraseChoiceRearward `%d' shall be `%d'", mode, value );

		chewing_set_phraseChoiceRearward( ctx, -1 );
		mode = chewing_get_phraseChoiceRearward( ctx );
		ok(  mode == value,
			"phraseChoiceRearward `%d' shall be `%d'", mode, value );

		chewing_set_phraseChoiceRearward( ctx, 2 );
		mode = chewing_get_phraseChoiceRearward( ctx );
		ok(  mode == value,
			"phraseChoiceRearward `%d' shall be `%d'", mode, value );
	}

	chewing_delete( ctx );
}

void test_set_ChiEngMode()
{
	const int VALUE[] = {
		CHINESE_MODE,
		SYMBOL_MODE,
	};

	const int INVALID_VALUE[] = {
		-1,
		2,
	};

	ChewingContext *ctx;
	size_t i;
	size_t j;

	ctx = chewing_new();
	start_testcase( ctx, fd );

	for ( i = 0; i < ARRAY_SIZE( VALUE ); ++i ) {
		chewing_set_ChiEngMode( ctx, VALUE[i] );
		ok( chewing_get_ChiEngMode( ctx ) == VALUE[i],
			"ChiEngMode shall be `%d'", VALUE[i] );

		for ( j = 0; j < ARRAY_SIZE( INVALID_VALUE ); ++j ) {
			// mode shall not change when set mode has invalid value.
			chewing_set_ChiEngMode( ctx, INVALID_VALUE[j] );
			ok( chewing_get_ChiEngMode( ctx ) == VALUE[i],
				"ChiEngMode shall be `%d'", VALUE[i] );
		}
	}

	chewing_delete( ctx );
}

void test_set_ShapeMode()
{
	const int VALUE[] = {
		HALFSHAPE_MODE,
		FULLSHAPE_MODE,
	};

	const int INVALID_VALUE[] = {
		-1,
		2,
	};

	ChewingContext *ctx;
	size_t i;
	size_t j;

	ctx = chewing_new();
	start_testcase( ctx, fd );

	for ( i = 0; i < ARRAY_SIZE( VALUE ); ++i ) {
		chewing_set_ShapeMode( ctx, VALUE[i] );
		ok( chewing_get_ShapeMode( ctx ) == VALUE[i],
			"ShapeMode shall be `%d'", VALUE[i] );

		for ( j = 0; j < ARRAY_SIZE( INVALID_VALUE ); ++j ) {
			// mode shall not change when set mode has invalid value.
			chewing_set_ShapeMode( ctx, INVALID_VALUE[j] );
			ok( chewing_get_ShapeMode( ctx ) == VALUE[i],
				"ShapeMode shall be `%d'", VALUE[i] );
		}
	}

	chewing_delete( ctx );
}

void test_deprecated()
{
	ChewingContext *ctx;
	int type;
	ChewingConfigData configure;
	memset( &configure, 0, sizeof( ChewingConfigData ) );

	ctx = chewing_new();
	start_testcase( ctx, fd );

	chewing_set_hsuSelKeyType( ctx, HSU_SELKEY_TYPE1 );
	type = chewing_get_hsuSelKeyType( ctx );
	ok( type == HSU_SELKEY_TYPE1, "`%d' shall be `%d'", type, HSU_SELKEY_TYPE1 );

	chewing_Configure( ctx, &configure );

	chewing_delete( ctx );
}

int main(int argc, char *argv[])
{
	char *logname;
	int ret;

	putenv( "CHEWING_PATH=" CHEWING_DATA_PREFIX );
	putenv( "CHEWING_USER_PATH=" TEST_HASH_DIR );

	ret = asprintf( &logname, "%s.log", argv[0] );
	if ( ret == -1 ) return -1;
	fd = fopen( logname, "w" );
	assert( fd );
	free( logname );


	test_default_value();

	test_set_candPerPage();
	test_set_maxChiSymbolLen();
	test_maxChiSymbolLen();
	test_set_selKey();
	test_set_addPhraseDirection();
	test_set_spaceAsSelection();
	test_set_escCleanAllBuf();
	test_set_autoShiftCur();
	test_set_easySymbolInput();
	test_set_phraseChoiceRearward();
	test_set_ChiEngMode();
	test_set_ShapeMode();

	test_deprecated();

	fclose( fd );

	return exit_status();
}
