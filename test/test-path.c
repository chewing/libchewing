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
#    include <config.h>
#endif

#include <assert.h>
#include <stdlib.h>
#include <string.h>

#include "testhelper.h"
#include "global-private.h"
#include "plat_path.h"
#include "plat_types.h"

#define ENV_NAME "CHEWING_PATH_TESTING_ENV"

FILE *fd;

int find_path_by_files(const char *search_path, const char *const *files, char *output, size_t output_len);

static const char *FILES[] = {
    DICT_FILE,
    PHONE_TREE_FILE,
    SYMBOL_TABLE_FILE,
    SOFTKBD_TABLE_FILE,
    PINYIN_TAB_NAME,
    NULL,
};

void test_plat_get_search_path()
{
    int ret;
    char output[PATH_MAX];

    start_testcase(NULL, fd);

    putenv("CHEWING_PATH=" CHEWING_DATA_PREFIX);
    ret = get_search_path(output, sizeof(output));
    ok(ret == 0, "get_search_path return 0");
    ok(!strcmp(output, CHEWING_DATA_PREFIX), "get_search_path succes");
    // TODO plat specific test
}

void test_plat_path_found()
{
    int ret;
    char output[PATH_MAX];

    start_testcase(NULL, fd);

    ret = find_path_by_files(CHEWING_DATA_PREFIX "_no_such_path" SEARCH_PATH_SEP
                             CHEWING_DATA_PREFIX, FILES, output, sizeof(output));

    ok(ret == 0, "find_path_by_files shall return 0");
    ok(strcmp(output, CHEWING_DATA_PREFIX) == 0, "output shall be " CHEWING_DATA_PREFIX);
}

void test_plat_path_cannot_find()
{
    int ret;
    char output[PATH_MAX];

    start_testcase(NULL, fd);

    ret = find_path_by_files(CHEWING_DATA_PREFIX "_no_such_path_1" SEARCH_PATH_SEP
                             CHEWING_DATA_PREFIX "_no_such_path_2", FILES, output, sizeof(output));

    ok(ret != 0, "find_path_by_files shall not return 0");
}

int main(int argc, char *argv[])
{
    char *logname;
    int ret;

    putenv("CHEWING_PATH=" CHEWING_DATA_PREFIX);
    putenv("CHEWING_USER_PATH=" TEST_HASH_DIR);

    ret = asprintf(&logname, "%s.log", argv[0]);
    if (ret == -1)
        return -1;
    fd = fopen(logname, "w");
    assert(fd);
    free(logname);


    test_plat_get_search_path();
    test_plat_path_found();
    test_plat_path_cannot_find();

    fclose(fd);

    return exit_status();
}
