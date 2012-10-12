#ifdef HAVE_CONFIG_H
        #include <config.h>
#endif

#include <stdlib.h>
#include <check.h>

#include "global-private.h"
#include "plat_types.h"
#include "plat_path.h"

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

START_TEST(test_plat_path_found)
{
	int ret;
	char output[1024];

	ret = find_path_by_files(
			CHEWING_DATA_PREFIX "_no_such_path:" CHEWING_DATA_PREFIX,
			FILES, output, sizeof( output ) );

	fail_unless( ret == 0, "find_path_by_files shall return 0" );
	fail_unless( strcmp( output, CHEWING_DATA_PREFIX ) == 0, "output shall be " CHEWING_DATA_PREFIX );
}
END_TEST

START_TEST(test_plat_path_cannot_find)
{
	int ret;
	char output[1024];

	ret = find_path_by_files(
			CHEWING_DATA_PREFIX "_no_such_path",
			FILES, output, sizeof( output ) );

	fail_unless( ret != 0, "find_path_by_files shall not return 0" );
}
END_TEST

START_TEST(test_find_path_env_expand)
{
	int ret;
	char output[1024];

	setenv( ENV_NAME, CHEWING_DATA_PREFIX, 1 );

#ifdef UNDER_POSIX
	ret = find_path_by_files( "$" ENV_NAME, FILES, output, sizeof( output ) );
#elif defined(_WIN32) || defined(_WIN64) || defined(_WIN32_WCE)
	ret = find_path_by_files( "%" ENV_NAME "%", FILES, output, sizeof( output ) );
#else
#error not implement
#endif

	fail_unless( ret == 0, "find_path_by_files shall return 0" );
	fail_unless( strcmp( output, CHEWING_DATA_PREFIX ) == 0, "output shall be " CHEWING_DATA_PREFIX );

	unsetenv( ENV_NAME );
}
END_TEST

Suite *path_suite (void)
{
	Suite *suite = suite_create( "plat_path" );
	TCase *tcase = tcase_create( "Core" );

	tcase_add_test( tcase, test_plat_path_found );
	tcase_add_test( tcase, test_plat_path_cannot_find );
	tcase_add_test( tcase, test_find_path_env_expand );

	suite_add_tcase( suite, tcase );
	return suite;
}
