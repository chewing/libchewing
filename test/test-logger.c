/**
 * test-logger.c
 *
 * Copyright (c) 2013
 *      libchewing Core Team. See ChangeLog for details.
 *
 * See the file "COPYING" for information on usage and redistribution
 * of this file.
 */

#ifdef HAVE_CONFIG_H
#    include <config.h>
#endif

#include <assert.h>
#include <stdarg.h>
#include <stdio.h>
#include <stdlib.h>

#include "chewing.h"
#include "testhelper.h"

FILE *fd;

void test_set_null_logger()
{
    ChewingContext *ctx;

    ctx = chewing_new();
    start_testcase(ctx, fd);

    chewing_set_logger(ctx, NULL, 0);
    type_keystroke_by_string(ctx, "hk4g4");

    chewing_delete(ctx);
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


    test_set_null_logger();

    fclose(fd);

    return exit_status();
}
