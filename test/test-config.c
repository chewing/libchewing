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

#include <stdlib.h>
#include <string.h>

#include "chewing.h"
#include "test.h"

static const int MIN_CAND_PER_PAGE = 1;
static const int MAX_CAND_PER_PAGE = 10;
static const int DEFAULT_CAND_PER_PAGE = 10;

static const int DEFAULT_SELECT_KEY[] = {
	'1', '2', '3', '4', '5', '6', '7', '8', '9', '0' };

static int ALTERNATE_SELECT_KEY[] = {
	'a', 's', 'd', 'f', 'g', 'h', 'j', 'k', 'l', ';' };

const TestData DATA = { .token = "`a", .expected = "…" };

void test_default_value()
{
	int *select_key;

	chewing_Init( 0, 0 );

	ChewingContext *ctx = chewing_new();
	ok( ctx, "chewing_new shall not return NULL" );

	select_key = chewing_get_selKey( ctx );
	ok( select_key, "chewing_get_selKey shall not return NULL" );
	ok( !memcmp( select_key, DEFAULT_SELECT_KEY,
		sizeof( DEFAULT_SELECT_KEY )),
		"select key shall be default value");
	chewing_free( select_key );

	ok( chewing_get_maxChiSymbolLen( ctx ) == 0,
		"maxChiSymbolLen shall be 0" );

	ok( chewing_get_addPhraseDirection( ctx ) == 0,
		"addPhraseDirection shall be 0" );

	ok( chewing_get_spaceAsSelection( ctx ) == 0,
		"spaceAsSelection shall be 0" );

	ok( chewing_get_escCleanAllBuf( ctx ) == 0,
		"escCleanAllBuf shall be 0" );

	ok( chewing_get_hsuSelKeyType( ctx ) == 0,
		"hsuSelKeyType shall be 0" );

	ok( chewing_get_autoShiftCur( ctx ) == 0,
		"autoShiftCur shall be 0" );

	ok( chewing_get_easySymbolInput( ctx ) == 0,
		"easySymbolInput shall be 0" );

	ok( chewing_get_phraseChoiceRearward( ctx ) == 0,
		"phraseChoiceRearward shall be 0" );

	ok( chewing_get_ChiEngMode( ctx ) == CHINESE_MODE,
		"ChiEngMode shall be CHINESE_MODE" );

	ok( chewing_get_ShapeMode( ctx ) == HALFSHAPE_MODE,
		"ShapeMode shall be HALFSHAPE_MODE" );

	chewing_delete( ctx );
	chewing_Terminate();
}

void test_set_select_key()
{
	ChewingContext *ctx;
	int *select_key;

	chewing_Init( 0, 0 );

	ctx = chewing_new();
	ok( ctx, "chewing_new shall not return NULL" );

	chewing_set_maxChiSymbolLen( ctx, 16 );

	// XXX: chewing_set_selKey shall accept const char *.
	chewing_set_selKey( ctx,
		ALTERNATE_SELECT_KEY, ARRAY_SIZE( ALTERNATE_SELECT_KEY ));
	select_key = chewing_get_selKey( ctx );
	ok( select_key, "chewing_get_selKey shall not return NULL" );
	ok( !memcmp( select_key, ALTERNATE_SELECT_KEY,
		sizeof( ALTERNATE_SELECT_KEY )),
		"select key shall be ALTERNATE_SELECT_KEY");

	type_keystoke_by_string( ctx, DATA.token );
	ok_preedit_buffer( ctx, DATA.expected );

	chewing_free( select_key );

	chewing_delete( ctx );
	chewing_Terminate();
}

void test_set_maxChiSymbolLen()
{
	ChewingContext *ctx;

	chewing_Init( 0, 0 );

	ctx = chewing_new();
	ok( ctx, "chewing_new shall not return NULL" );

	chewing_set_maxChiSymbolLen( ctx, 16 );
	ok( chewing_get_maxChiSymbolLen( ctx ) == 16,
		"maxChiSymbolLen shall be 16" );

	chewing_set_maxChiSymbolLen( ctx, -1 );
	ok( chewing_get_maxChiSymbolLen( ctx ) == 16,
		"maxChiSymbolLen shall not change" );

	chewing_set_maxChiSymbolLen( ctx, 51 /* MAX_PHONE_SEQ_LEN + 1 */ );
	ok( chewing_get_maxChiSymbolLen( ctx ) == 16,
		"maxChiSymbolLen shall not change" );

	// XXX: test if new maxChiSymbolLen works as expect

	chewing_delete( ctx );
	chewing_Terminate();
}

void test_hsu_select_key()
{
	ChewingContext *ctx;
	int type;

	chewing_Init( 0, 0 );

	ctx = chewing_new();
	ok( ctx, "chewing_new shall not return NULL" );

	chewing_set_hsuSelKeyType( ctx, HSU_SELKEY_TYPE1 );
	type = chewing_get_hsuSelKeyType( ctx );
	ok( type == HSU_SELKEY_TYPE1, "`%d' shall be `%d'", type, HSU_SELKEY_TYPE1 );

	chewing_delete( ctx );
	chewing_Terminate();
}

void test_cand_per_page()
{
	chewing_Init( 0, 0 );

	ChewingContext *ctx = chewing_new();
	ok( ctx, "chewing_new shall not return NULL" );

	ok( chewing_get_candPerPage( ctx ) == DEFAULT_CAND_PER_PAGE,
		"candPerPage shall be default value" );

	chewing_set_candPerPage( ctx, MIN_CAND_PER_PAGE - 1 );
	ok( chewing_get_candPerPage( ctx ) == DEFAULT_CAND_PER_PAGE,
		"candPerPage shall not change" );

	chewing_set_candPerPage( ctx, MAX_CAND_PER_PAGE + 1 );
	ok( chewing_get_candPerPage( ctx ) == DEFAULT_CAND_PER_PAGE,
		"candPerPage shall not change" );

	chewing_delete( ctx );
	chewing_Terminate();
}

int main()
{
	putenv( "CHEWING_PATH=" CHEWING_DATA_PREFIX );
	putenv( "CHEWING_USER_PATH=" TEST_HASH_DIR );

	test_default_value();

	test_set_select_key();
	test_set_maxChiSymbolLen();
	test_hsu_select_key();

	test_cand_per_page();

	return exit_status();
}
