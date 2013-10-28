/**
 * test-error-handling.c
 *
 * Copyright (c) 2013
 *	libchewing Core Team. See ChangeLog for details.
 *
 * See the file "COPYING" for information on usage and redistribution
 * of this file.
 */
#include <assert.h>
#include <stdlib.h>

#include "testhelper.h"
#include "chewing.h"

FILE *fd;

void test_null()
{
	int ret;

	start_testcase( NULL, fd );

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

	ret = chewing_cand_close( NULL );
	ok ( ret == -1, "chewing_cand_open() returns `%d' shall be `%d'", ret, -1 );

	ret = chewing_cand_choose_by_index( NULL, 0 );
	ok ( ret == -1, "chewing_cand_choose_by_index() returns `%d' shall be `%d'", ret, -1 );

	ret = chewing_cand_list_first( NULL );
	ok ( ret == -1, "chewing_cand_list_first() returns `%d' shall be `%d'", ret, -1 );

	ret = chewing_cand_list_last( NULL );
	ok ( ret == -1, "chewing_cand_list_last() returns `%d' shall be `%d'", ret, -1 );

	ret = chewing_cand_list_has_next( NULL );
	ok ( ret == 0, "chewing_cand_list_has_next() returns `%d' shall be `%d'", ret, 0 );

	ret = chewing_cand_list_has_prev( NULL );
	ok ( ret == 0, "chewing_cand_list_has_prev() returns `%d' shall be `%d'", ret, 0 );

	ret = chewing_cand_list_next( NULL );
	ok ( ret == -1, "chewing_cand_list_next() returns `%d' shall be `%d'", ret, -1 );

	ret = chewing_cand_list_prev( NULL );
	ok ( ret == -1, "chewing_cand_list_prev() returns `%d' shall be `%d'", ret, -1 );

	ret = chewing_commit_preedit_buf( NULL );
	ok ( ret == -1, "chewing_commit_preedit_buf() returns `%d' shall be `%d'", ret, -1 );

	ret = chewing_clean_preedit_buf( NULL );
	ok ( ret == -1, "chewing_clean_preedit_buf() returns `%d' shall be `%d'", ret, -1 );
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


	test_null();

	fclose( fd );

	return exit_status();
}
