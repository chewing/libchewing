/**
 * test-key2pho.c
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
#include "global.h"
#include "chewing-utf8-util.h"
#include "key2pho-private.h"
#include "chewing-private.h"
#include "bopomofo-private.h"

FILE *fd;

void test_uint_and_phone()
{
    char *u8phone;
    char rt[MAX_UTF8_SIZE * BOPOMOFO_SIZE + 1];
    uint16_t phone;
    uint16_t expect;

    start_testcase(NULL, fd);

    u8phone = "\xE3\x84\x86\xE3\x84\xA3" /* ㄆㄣ */ ;
    phone = UintFromPhone(u8phone);
    expect = 1104;
    ok(phone == expect, "UintFromPhone `%s' shall be `%d', got `%d'", u8phone, expect, phone);

    PhoneFromUint(rt, sizeof(rt), expect);
    ok(strcmp(rt, u8phone) == 0, "PhoneFromUint d%d' shall be `%s', got `%s'", expect, u8phone, rt);


    u8phone = "\xE3\x84\x8A\xE3\x84\xA7\xE3\x84\xA2" /* ㄊㄧㄢ */ ;
    phone = UintFromPhone(u8phone);
    expect = 3272;
    ok(phone == expect, "UintFromPhone `%s' shall be `%d', got `%d'", u8phone, expect, phone);

    PhoneFromUint(rt, sizeof(rt), expect);
    ok(strcmp(rt, u8phone) == 0, "PhoneFromUint d%d' shall be `%s', got `%s'", expect, u8phone, rt);


    u8phone = "\xE3\x84\x92\xE3\x84\xA7\xE3\x84\x9A\xCB\x8B" /* ㄒㄧㄚˋ */ ;
    phone = UintFromPhone(u8phone);
    expect = 7308;
    ok(phone == expect, "UintFromPhone `%s' shall be `%d', got `%d'", u8phone, expect, phone);

    PhoneFromUint(rt, sizeof(rt), expect);
    ok(strcmp(rt, u8phone) == 0, "PhoneFromUint `%d' shall be `%s', got `%s'", expect, u8phone, rt);
}

void test_uint_and_phone_error()
{
    char *u8phone;
    char rt[MAX_UTF8_SIZE * BOPOMOFO_SIZE + 1];
    uint16_t phone;
    uint16_t expect;

    start_testcase(NULL, fd);

    u8phone = "\xE3\x84\x8A\xE3\x84\xA7\xE6\xB8\xAC" /* ㄊㄧ測 */ ;
    phone = UintFromPhone(u8phone);
    expect = 0;
    ok(phone == expect, "UintFromPhone `%s' shall be `%d', got `%d'", u8phone, expect, phone);

    u8phone = "\xE3\x84\x8E\xE3\x84\x8E" /* ㄎㄎ */ ;
    phone = UintFromPhone(u8phone);
    expect = 0;
    ok(phone == expect, "UintFromPhone `%s' shall be `%d', got `%d'", u8phone, expect, phone);

    u8phone = "\xE3\x84\xA8\xE3\x84\x8E" /* ㄨㄎ */ ;
    phone = UintFromPhone(u8phone);
    expect = 0;
    ok(phone == expect, "UintFromPhone `%s' shall be `%d', got `%d'", u8phone, expect, phone);

    PhoneFromUint(rt, sizeof(rt), 0);
    ok(strcmp(rt, "") == 0, "PhoneFromUint `%d' shall be `%s', got `%s'", 0, "", rt);

}

void test_key_and_phone()
{
    char rt[MAX_UTF8_SIZE * BOPOMOFO_SIZE + 1];

    start_testcase(NULL, fd);

    PhoneFromKey(rt, "dj", KB_DEFAULT, 1);
    ok(!strcmp(rt, "\xE3\x84\x8E\xE3\x84\xA8" /* ㄎㄨ */ ), "dj");

    PhoneFromKey(rt, "dj6", KB_DEFAULT, 1);
    ok(!strcmp(rt, "\xE3\x84\x8E\xE3\x84\xA8\xCB\x8A" /* ㄎㄨˊ */ ), "dj6");

    PhoneFromKey(rt, "dj3", KB_DEFAULT, 1);
    ok(!strcmp(rt, "\xE3\x84\x8E\xE3\x84\xA8\xCB\x87" /* ㄎㄨˇ */ ), "dj3");

    PhoneFromKey(rt, "dj4", KB_DEFAULT, 1);
    ok(!strcmp(rt, "\xE3\x84\x8E\xE3\x84\xA8\xCB\x8B" /* ㄎㄨˋ */ ), "dj4");

    PhoneFromKey(rt, "dj7", KB_DEFAULT, 1);
    ok(!strcmp(rt, "\xE3\x84\x8E\xE3\x84\xA8\xCB\x99" /* ㄎㄨ˙ */ ), "dj7");
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


    test_uint_and_phone();
    test_uint_and_phone_error();
    test_key_and_phone();

    fclose(fd);

    return exit_status();
}
