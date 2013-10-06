/**
 * test-error-handling.c
 *
 * Copyright (c) 2013
 *	libchewing Core Team. See ChangeLog for details.
 *
 * See the file "COPYING" for information on usage and redistribution
 * of this file.
 */

#include <stdlib.h>

#include "testhelper.h"
#include "chewing.h"

void test_null()
{
	int ret;

	print_function_name();

	chewing_set_logger( NULL, NULL, NULL );

	ret = chewing_userphrase_enumerate( NULL );
	ok( ret == -1, "chewing_userphrase_enumerate() returns `%d' shall be `%d'", ret, -1 );

	ret = chewing_userphrase_has_next( NULL, NULL, NULL );
	ok( ret == 0, "chewing_userphrase_has_next() returns `%d' shall be `%d'", ret, 0 );

	ret = chewing_userphrase_get( NULL, NULL, 0, NULL, 0 );
	ok( ret == -1, "chewing_userphrase_get() returns `%d' shall be `%d'", ret, -1 );

	ret = chewing_userphrase_add( NULL, NULL, NULL );
	ok( ret == -1, "chewing_userphrase_add() returns `%d' shall be `%d'", ret, -1 );

	ret = chewing_userphrase_remove( NULL, NULL, NULL );
	ok( ret == -1, "chewing_userphrase_remove() returns `%d' shall be `%d'", ret, -1 );

	ret = chewing_userphrase_lookup( NULL, NULL, NULL );
	ok( ret == 0, "chewing_userphrase_lookup() returns `%d' shall be `%d'", ret, 0 );

	ret = chewing_cand_open( NULL );
	ok ( ret == -1, "chewing_cand_open() returns `%d' shall be `%d'", ret, -1 );
}

int main()
{
	putenv( "CHEWING_PATH=" CHEWING_DATA_PREFIX );
	putenv( "CHEWING_USER_PATH=" TEST_HASH_DIR );

	test_null();

	return exit_status();
}
