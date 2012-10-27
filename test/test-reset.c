/**
 * test-reset.c
 *
 * Copyright (c) 2012
 *	libchewing Core Team. See ChangeLog for details.
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

void test_reset_shall_not_clean_static_data()
{
	char TOKEN[] = "hk4g4<E>";
	char EXPECTED[] = "測試";

	putenv( "CHEWING_PATH=" CHEWING_DATA_PREFIX );
	putenv( "CHEWING_USER_PATH=" TEST_HASH_DIR );

	chewing_Init( NULL, NULL );

	ChewingContext *ctx = chewing_new();
	ok( ctx, "chewing_new shall not return NULL" );

	chewing_set_KBType( ctx, chewing_KBStr2Num( "KB_DEFAULT" ) );

	chewing_set_maxChiSymbolLen( ctx, 16 );

	chewing_Reset( ctx );

	verify_keystoke( ctx, TOKEN, EXPECTED );

	chewing_delete( ctx );
	chewing_Terminate();
}

int main ()
{
	test_reset_shall_not_clean_static_data();
	return exit_status();
}
