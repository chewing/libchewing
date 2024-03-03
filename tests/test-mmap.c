/**
 * test-mmap.c
 *
 * Copyright (c) 2005
 *      libchewing Core Team. See ChangeLog for details.
 *
 * See the file "COPYING" for information on usage and redistribution
 * of this file.
 */

#ifdef HAVE_CONFIG_H
#    include <config.h>
#endif

#ifndef _GNU_SOURCE
#    define _GNU_SOURCE
#endif

#include <assert.h>
#include <stdio.h>
#include <string.h>
#include <stdlib.h>

#include "testhelper.h"
#include "plat_mmap.h"

FILE *fd;

void test_UnitFromPlatMmap()
{
    unsigned int idx;
    plat_mmap m_mmap;
    size_t offset = 0;
    size_t csize;
    char *data_buf;
    char hard_copy[] = "ji3cp3vu3cj0 vup dj4up <E>";
    int i;

    start_testcase(NULL, fd);

    idx = plat_mmap_create(&m_mmap, TEST_DATA_DIR PLAT_SEPARATOR "default-test.txt", FLAG_ATTRIBUTE_READ);
    ok(idx == 28, "plat_mmap_create");
    if (idx > 0) {
        csize = idx;
        data_buf = (char *) plat_mmap_set_view(&m_mmap, &offset, &csize);
        for (i = 0; i < 26; i++) {
            if (data_buf[i] != hard_copy[i])
                break;
        }
        ok(i == 26, "plat_mmap_set_view");
    }
    plat_mmap_close(&m_mmap);
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


    test_UnitFromPlatMmap();

    fclose(fd);

    return exit_status();
}
