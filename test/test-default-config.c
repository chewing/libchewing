/**
 * test-default-config.c
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

#include <string.h>

#include "chewing.h"
#include "test.h"

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

int main()
{
	putenv( "CHEWING_PATH=" CHEWING_DATA_PREFIX );
	putenv( "CHEWING_USER_PATH=" TEST_HASH_DIR );

	test_default_select_key();

	return exit_status();
}
