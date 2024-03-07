/**
 * test-utf8.c
 *
 * Copyright (c) 2005
 *      libchewing Core Team. See ChangeLog for details.
 *
 * See the file "COPYING" for information on usage and redistribution
 * of this file.
 */
#include <assert.h>
#include <stdio.h>
#include <string.h>
#include <stdlib.h>

#include "testhelper.h"
#include "chewing-utf8-util.h"

FILE *fd;

void test_utf8()
{
    char *u8string;
    int u8len;
    int len;
    char u8string2[16];

    start_testcase(NULL, fd);

    u8string = "HelloWorld";
    u8len = ueStrLen(u8string);
    len = strlen(u8string);
    ok(u8len == len, "ueStrLen");

    u8string = "\xE6\xB8\xAC\xE8\xA9\xA6\xE8\xA8\x88\xE7\xAE\x97\xE9\x95\xB7\xE5\xBA\xA6";      /* 測試計算長度 */
    u8len = ueStrLen(u8string);
    ok(u8len == 6, "ueStrLen");

    u8string = "\xE6\xB8\xAC\xE8\xA9\xA6\xE8\xA8\x88\xE7\xAE\x97\xE9\x95\xB7\xE5\xBA\xA6";      /* 測試計算長度 */
    ueStrNCpy(u8string2, u8string, 3, STRNCPY_CLOSE);
    ok(!strcmp(u8string2, "\xE6\xB8\xAC\xE8\xA9\xA6\xE8\xA8\x88" /* 測試計 */ ), "ueStrNCpy");

    u8string = "\xE6\xB8\xAC\xE8\xA9\xA6\xE8\xA8\x88\xE7\xAE\x97\xE9\x95\xB7\xE5\xBA\xA6";      /* 測試計算長度 */
    u8string = ueStrSeek(u8string, 3);
    ok(!strcmp(u8string, "\xE7\xAE\x97\xE9\x95\xB7\xE5\xBA\xA6" /* 算長度 */ ), "ueStrSeek");

    u8string = "\xE6\xB8\xAC\xE8\xA9\xA6\xE8\xA8\x88\xE7\xAE\x97\xE9\x95\xB7\xE5\xBA\xA6";      /* 測試計算長度 */
    u8string = ueStrSeek(u8string, 0);
    ok(!strcmp
       (u8string, "\xE6\xB8\xAC\xE8\xA9\xA6\xE8\xA8\x88\xE7\xAE\x97\xE9\x95\xB7\xE5\xBA\xA6" /* 測試計算長度 */ ),
       "ueStrSeek");

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

    test_utf8();

    fclose(fd);

    return exit_status();
}
