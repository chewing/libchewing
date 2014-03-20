/**
 * test-special-symbol.c
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

#include "chewing.h"
#include "testhelper.h"

static const TestData SPECIAL_SYMBOL_TABLE[] = {
    {"[", "\xE3\x80\x8C" /* 「 */ },
    {"]", "\xE3\x80\x8D" /* 」 */ },
    {"{", "\xE3\x80\x8E" /* 『 */ },
    {"}", "\xE3\x80\x8F" /* 』 */ },
    {"'", "\xE3\x80\x81" /* 、 */ },
    {"<<>", "\xEF\xBC\x8C" /* ， */ },
    {":", "\xEF\xBC\x9A" /* ： */ },
    {"\"", "\xEF\xBC\x9B" /* ； */ },
    {">", "\xE3\x80\x82" /* 。 */ },
    {"~", "\xEF\xBD\x9E" /* ～ */ },
    {"!", "\xEF\xBC\x81" /* ！ */ },
    {"@", "\xEF\xBC\xA0" /* ＠ */ },
    {"#", "\xEF\xBC\x83" /* ＃ */ },
    {"$", "\xEF\xBC\x84" /* ＄ */ },
    {"%", "\xEF\xBC\x85" /* ％ */ },
    {"^", "\xEF\xB8\xBF" /* ︿ */ },
    {"&", "\xEF\xBC\x86" /* ＆ */ },
    {"*", "\xEF\xBC\x8A" /* ＊ */ },
    {"(", "\xEF\xBC\x88" /* （ */ },
    {")", "\xEF\xBC\x89" /* ） */ },
    {"_", "\xE2\x80\x94" /* — */ },
    {"+", "\xEF\xBC\x8B" /* ＋ */ },
    {"=", "\xEF\xBC\x9D" /* ＝ */ },
    {"\\", "\xEF\xBC\xBC" /* ＼ */ },
    {"|", "\xEF\xBD\x9C" /* ｜ */ },
    {"?", "\xEF\xBC\x9F" /* ？ */ },
    {",", "\xEF\xBC\x8C" /* ， */ },
    {".", "\xE3\x80\x82" /* 。 */ },
    {";", "\xEF\xBC\x9B" /* ； */ },
};

FILE *fd;

int is_bopomofo_collision_key(const char *key)
{
    static const char *COLLISION_KEY[] = {
        "<<>",
        ">",
        ";",
        ",",
        ".",
    };
    size_t i;

    for (i = 0; i < ARRAY_SIZE(COLLISION_KEY); ++i) {
        if (strcmp(key, COLLISION_KEY[i]) == 0) {
            return 1;
        }
    }
    return 0;
}

void test_in_chinese_mode()
{
    ChewingContext *ctx;
    size_t i;

    ctx = chewing_new();
    start_testcase(ctx, fd);

    chewing_set_maxChiSymbolLen(ctx, 16);

    for (i = 0; i < ARRAY_SIZE(SPECIAL_SYMBOL_TABLE); ++i) {
        // If bopomofo symbol is collided with special symbol, use
        // bopomofo symbol
        if (is_bopomofo_collision_key(SPECIAL_SYMBOL_TABLE[i].token))
            continue;

        type_keystroke_by_string(ctx, SPECIAL_SYMBOL_TABLE[i].token);
        ok_preedit_buffer(ctx, SPECIAL_SYMBOL_TABLE[i].expected);
        type_keystroke_by_string(ctx, "<E>");
        ok_commit_buffer(ctx, SPECIAL_SYMBOL_TABLE[i].expected);
    }

    chewing_delete(ctx);
}

void test_in_easy_symbol_mode()
{
    ChewingContext *ctx;
    size_t i;

    ctx = chewing_new();
    start_testcase(ctx, fd);

    chewing_set_maxChiSymbolLen(ctx, 16);
    chewing_set_easySymbolInput(ctx, 1);

    for (i = 0; i < ARRAY_SIZE(SPECIAL_SYMBOL_TABLE); ++i) {
        type_keystroke_by_string(ctx, SPECIAL_SYMBOL_TABLE[i].token);
        ok_preedit_buffer(ctx, SPECIAL_SYMBOL_TABLE[i].expected);
        type_keystroke_by_string(ctx, "<E>");
        ok_commit_buffer(ctx, SPECIAL_SYMBOL_TABLE[i].expected);
    }

    chewing_delete(ctx);
}

int is_fullshape_collision_key(const char *key)
{
    static const char *COLLISION_KEY[] = {
        "\"",
        "'",
        "/",
        "<<>",
        ">",
        "`",
        "[",
        "]",
        "{",
        "}",
        "+",
        "-",
    };
    size_t i;

    for (i = 0; i < ARRAY_SIZE(COLLISION_KEY); ++i) {
        if (strcmp(key, COLLISION_KEY[i]) == 0) {
            return 1;
        }
    }
    return 0;
}

void test_in_fullshape_mode()
{
    ChewingContext *ctx;
    size_t i;

    ctx = chewing_new();
    start_testcase(ctx, fd);

    chewing_set_maxChiSymbolLen(ctx, 16);
    chewing_set_ChiEngMode(ctx, SYMBOL_MODE);
    chewing_set_ShapeMode(ctx, FULLSHAPE_MODE);

    for (i = 0; i < ARRAY_SIZE(SPECIAL_SYMBOL_TABLE); ++i) {
        // If fullshape symbol is collided with special symbol, use
        // fullshape symbol
        if (is_fullshape_collision_key(SPECIAL_SYMBOL_TABLE[i].token))
            continue;

        type_keystroke_by_string(ctx, SPECIAL_SYMBOL_TABLE[i].token);
        ok_preedit_buffer(ctx, "");
        ok_commit_buffer(ctx, SPECIAL_SYMBOL_TABLE[i].expected);
    }

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


    test_in_chinese_mode();
    test_in_easy_symbol_mode();
    test_in_fullshape_mode();

    fclose(fd);

    return exit_status();
}
