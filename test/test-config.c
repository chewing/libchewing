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

void test_default_select_key()
{
	chewing_Init( 0, 0 );

	ChewingContext *ctx = chewing_new();
	ok( ctx, "chewing_new shall not return NULL" );

	int *select_key = chewing_get_selKey( ctx );
	ok( select_key, "chewing_get_selKey shall not return NULL" );
	ok( !memcmp( select_key, DEFAULT_SELECT_KEY,
		sizeof( DEFAULT_SELECT_KEY )),
		"select key shall be default value");
	chewing_free( select_key );

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

	test_default_select_key();
	test_cand_per_page();

	return exit_status();
}
