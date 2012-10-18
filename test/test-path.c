/**
 * test-path.c
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

#include "test_harness.h"
#include "global-private.h"
#include "plat_path.h"
#include "plat_types.h"

#define ENV_NAME "CHEWING_PATH_TESTING_ENV"

static const char *FILES[] = {
	CHAR_FILE,
	CHAR_INDEX_BEGIN_FILE,
	DICT_FILE,
	PH_INDEX_FILE,
	PHONE_TREE_FILE,
	SYMBOL_TABLE_FILE,
	SOFTKBD_TABLE_FILE,
	PINYIN_TAB_NAME,
	NULL,
};

void test_plat_path_found()
{
	int ret;
	char output[ PATH_MAX ];

	ret = find_path_by_files(
		CHEWING_DATA_PREFIX "_no_such_path:"
		CHEWING_DATA_PREFIX, FILES, output, sizeof( output ) );

	ok( ret == 0, "find_path_by_files shall return 0" );
	ok( strcmp( output, CHEWING_DATA_PREFIX ) == 0,
		"output shall be " CHEWING_DATA_PREFIX );
}

void test_plat_path_cannot_find()
{
	int ret;
	char output[ PATH_MAX ];

	ret = find_path_by_files(
			CHEWING_DATA_PREFIX "_no_such_path",
			FILES, output, sizeof( output ) );

	ok( ret != 0, "find_path_by_files shall not return 0" );
}

int main()
{
	test_plat_path_found();
	test_plat_path_cannot_find();
	return exit_status();
}
