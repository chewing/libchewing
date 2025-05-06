/**
 * test-logger.c
 *
 * Copyright (c) 2013
 *      libchewing Core Team.
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

void test_set_null_logger()
{
    ChewingContext *ctx;

    ctx = chewing_new();
    start_testcase(ctx);

    chewing_set_logger(ctx, NULL, 0);
    type_keystroke_by_string(ctx, "hk4g4");

    chewing_delete(ctx);
}

void stdout_logger(void *data, int level, const char *fmt, ...)
{
    va_list ap;
    va_start(ap, fmt);
    vprintf(fmt, ap);
    va_end(ap);
}

void test_set_logger()
{
    ChewingContext *ctx;

    ctx = chewing_new();
    start_testcase(ctx);

    chewing_set_logger(ctx, stdout_logger, 0);
    type_keystroke_by_string(ctx, "hk4g4");

    chewing_delete(ctx);
}

int main(int argc, char *argv[])
{
    putenv("CHEWING_PATH=" CHEWING_DATA_PREFIX);
    putenv("CHEWING_USER_PATH=" TEST_HASH_DIR);

    test_set_logger();
    test_set_null_logger();

    return exit_status();
}
