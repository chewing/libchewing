/**
 * test-keyboard.c
 *
 * Copyright (c) 2012
 *      libchewing Core Team.
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

#include "chewing.h"
#include "testhelper.h"

static const char *const KEYBOARD_STRING[] = {
    "KB_DEFAULT",
    "KB_HSU",
    "KB_IBM",
    "KB_GIN_YIEH",
    "KB_ET",
    "KB_ET26",
    "KB_DVORAK",
    "KB_DVORAK_HSU",
    "KB_DACHEN_CP26",
    "KB_HANYU_PINYIN",
    "KB_THL_PINYIN",
    "KB_MPS2_PINYIN",
    "KB_CARPALX",
    "KB_COLEMAK_DH_ANSI",
    "KB_COLEMAK_DH_ORTH",
    "KB_WORKMAN",
    "KB_COLEMAK"
};

static const int KEYBOARD_DEFAULT_TYPE = 0;

FILE *fd;

void test_set_keyboard_type()
{
    ChewingContext *ctx;
    size_t i;
    char *keyboard_string;
    int keyboard_type;

    ctx = chewing_new();
    start_testcase(ctx, fd);

    keyboard_string = chewing_get_KBString(ctx);
    ok(strcmp(keyboard_string, KEYBOARD_STRING[KEYBOARD_DEFAULT_TYPE]) == 0,
       "`%s' shall be `%s'", keyboard_string, KEYBOARD_STRING[KEYBOARD_DEFAULT_TYPE]);
    chewing_free(keyboard_string);
    keyboard_type = chewing_get_KBType(ctx);
    ok(keyboard_type == KEYBOARD_DEFAULT_TYPE, "`%d' shall be `%d'", keyboard_type, KEYBOARD_DEFAULT_TYPE);

    for (i = 0; i < ARRAY_SIZE(KEYBOARD_STRING); ++i) {
        ok(chewing_set_KBType(ctx, i) == 0, "return shall be 0");

        keyboard_string = chewing_get_KBString(ctx);
        ok(strcmp(keyboard_string, KEYBOARD_STRING[i]) == 0, "`%s' shall be `%s'", keyboard_string, KEYBOARD_STRING[i]);
        chewing_free(keyboard_string);
        keyboard_type = chewing_get_KBType(ctx);
        ok(keyboard_type == (int) i, "`%d' shall be `%d'", keyboard_type, (int) i);
    }

    // The invalid KBType will reset KBType to default value.
    ok(chewing_set_KBType(ctx, -1) == -1, "return shall be -1");
    keyboard_type = chewing_get_KBType(ctx);
    ok(keyboard_type == KEYBOARD_DEFAULT_TYPE, "`%d' shall be `%d'", keyboard_type, KEYBOARD_DEFAULT_TYPE);

    ok(chewing_set_KBType(ctx, ARRAY_SIZE(KEYBOARD_STRING) + 1), "return shall be -1");
    keyboard_type = chewing_get_KBType(ctx);
    ok(keyboard_type == KEYBOARD_DEFAULT_TYPE, "`%d' shall be `%d'", keyboard_type, KEYBOARD_DEFAULT_TYPE);

    chewing_delete(ctx);
}

void test_set_keyboard_type_options()
{
    ChewingContext *ctx;
    size_t i;
    char *keyboard_string;
    int keyboard_type;

    ctx = chewing_new();
    start_testcase(ctx, fd);

    for (i = 0; i < ARRAY_SIZE(KEYBOARD_STRING); ++i) {
        ok(chewing_config_set_str(ctx, "chewing.keyboard_type", KEYBOARD_STRING[i]) == 0,
            "chewing_config_set_str should return OK");

        ok(chewing_config_get_str(ctx, "chewing.keyboard_type", &keyboard_string) == 0,
            "chewing_config_get_str should return OK");
        ok(strcmp(keyboard_string, KEYBOARD_STRING[i]) == 0, "`%s' shall be `%s'", keyboard_string, KEYBOARD_STRING[i]);
        chewing_free(keyboard_string);
        keyboard_type = chewing_get_KBType(ctx);
        ok(keyboard_type == (int) i, "`%d' shall be `%d'", keyboard_type, (int) i);
    }

    ok(chewing_config_set_str(ctx, "chewing.keyboard_type", "KB_DEFAULT") == 0,
        "chewing_config_set_str should return OK");
    // The invalid KBType should be no-op
    ok(chewing_config_set_str(ctx, "chewing.keyboard_type", "KB_UNKNOWN") == -1,
        "chewing_config_set_str should return ERROR");
    keyboard_type = chewing_get_KBType(ctx);
    ok(keyboard_type == KEYBOARD_DEFAULT_TYPE, "`%d' shall be `%d'", keyboard_type, KEYBOARD_DEFAULT_TYPE);

    chewing_delete(ctx);
}

void test_KBStr2Num()
{
    int i;
    int ret;

    start_testcase(NULL, fd);

    for (i = 0; i < (int) ARRAY_SIZE(KEYBOARD_STRING); ++i) {
        // XXX: chewing_KBStr2Num shall accept const char *.
        ret = chewing_KBStr2Num(KEYBOARD_STRING[i]);
        ok(ret == i, "%d shall be %d", ret, i);
    }
}

void test_enumerate_keyboard_type()
{
    ChewingContext *ctx;
    size_t i;
    char *keyboard_string;

    ctx = chewing_new();
    start_testcase(ctx, fd);

    ok(chewing_kbtype_Total(ctx) == ARRAY_SIZE(KEYBOARD_STRING),
       "total keyboard_string type shall be %d", ARRAY_SIZE(KEYBOARD_STRING));

    chewing_kbtype_Enumerate(ctx);
    for (i = 0; i < ARRAY_SIZE(KEYBOARD_STRING); ++i) {
        ok(chewing_kbtype_hasNext(ctx) == 1, "shall have next keyboard_string type");
        keyboard_string = chewing_kbtype_String(ctx);
        ok(strcmp(keyboard_string, KEYBOARD_STRING[i]) == 0, "`%s' shall be `%s'", keyboard_string, KEYBOARD_STRING[i]);
        chewing_free(keyboard_string);
    }
    ok(chewing_kbtype_hasNext(ctx) == 0, "shall not have next keyboard_string type");
    keyboard_string = chewing_kbtype_String(ctx);
    ok(strcmp(keyboard_string, "") == 0, "`%s' shall be `%s'", keyboard_string, "");
    chewing_free(keyboard_string);

    chewing_delete(ctx);
}

void test_hsu_po_to_bo()
{
    // https://github.com/chewing/libchewing/issues/170
    ChewingContext *ctx;

    ctx = chewing_new();
    start_testcase(ctx, fd);

    chewing_set_KBType(ctx, chewing_KBStr2Num("KB_HSU"));

    type_keystroke_by_string(ctx, "p");
    ok_bopomofo_buffer(ctx, "\xE3\x84\x86" /* ㄆ */ );

    type_keystroke_by_string(ctx, "b");
    ok_bopomofo_buffer(ctx, "\xE3\x84\x85" /* ㄅ */ );

    chewing_delete(ctx);
}

void test_hsu()
{
    test_hsu_po_to_bo();
}

void test_et26_po_to_bo()
{
    // https://github.com/chewing/libchewing/issues/170
    ChewingContext *ctx;

    ctx = chewing_new();
    start_testcase(ctx, fd);

    chewing_set_KBType(ctx, chewing_KBStr2Num("KB_ET26"));

    type_keystroke_by_string(ctx, "p");
    ok_bopomofo_buffer(ctx, "\xE3\x84\x86" /* ㄆ */ );

    type_keystroke_by_string(ctx, "b");
    ok_bopomofo_buffer(ctx, "\xE3\x84\x85" /* ㄅ */ );

    chewing_delete(ctx);
}

void test_et26()
{
    test_et26_po_to_bo();
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

    test_set_keyboard_type();
    test_set_keyboard_type_options();
    test_KBStr2Num();
    test_enumerate_keyboard_type();

    test_hsu();
    test_et26();

    fclose(fd);

    return exit_status();
}
