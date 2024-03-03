/**
 * test-fullshape.c
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

#include "chewing.h"
#include "testhelper.h"

static const TestData FULLSHAPE_DATA[] = {
    {"0", "\xEF\xBC\x90" /* ０ */ },
    {"1", "\xEF\xBC\x91" /* １ */ },
    {"2", "\xEF\xBC\x92" /* ２ */ },
    {"3", "\xEF\xBC\x93" /* ３ */ },
    {"4", "\xEF\xBC\x94" /* ４ */ },
    {"5", "\xEF\xBC\x95" /* ５ */ },
    {"6", "\xEF\xBC\x96" /* ６ */ },
    {"7", "\xEF\xBC\x97" /* ７ */ },
    {"8", "\xEF\xBC\x98" /* ８ */ },
    {"9", "\xEF\xBC\x99" /* ９ */ },
    {"a", "\xEF\xBD\x81" /* ａ */ },
    {"b", "\xEF\xBD\x82" /* ｂ */ },
    {"c", "\xEF\xBD\x83" /* ｃ */ },
    {"d", "\xEF\xBD\x84" /* ｄ */ },
    {"e", "\xEF\xBD\x85" /* ｅ */ },
    {"f", "\xEF\xBD\x86" /* ｆ */ },
    {"g", "\xEF\xBD\x87" /* ｇ */ },
    {"h", "\xEF\xBD\x88" /* ｈ */ },
    {"i", "\xEF\xBD\x89" /* ｉ */ },
    {"j", "\xEF\xBD\x8A" /* ｊ */ },
    {"k", "\xEF\xBD\x8B" /* ｋ */ },
    {"l", "\xEF\xBD\x8C" /* ｌ */ },
    {"m", "\xEF\xBD\x8D" /* ｍ */ },
    {"n", "\xEF\xBD\x8E" /* ｎ */ },
    {"o", "\xEF\xBD\x8F" /* ｏ */ },
    {"p", "\xEF\xBD\x90" /* ｐ */ },
    {"q", "\xEF\xBD\x91" /* ｑ */ },
    {"r", "\xEF\xBD\x92" /* ｒ */ },
    {"s", "\xEF\xBD\x93" /* ｓ */ },
    {"t", "\xEF\xBD\x94" /* ｔ */ },
    {"u", "\xEF\xBD\x95" /* ｕ */ },
    {"v", "\xEF\xBD\x96" /* ｖ */ },
    {"w", "\xEF\xBD\x97" /* ｗ */ },
    {"x", "\xEF\xBD\x98" /* ｘ */ },
    {"y", "\xEF\xBD\x99" /* ｙ */ },
    {"z", "\xEF\xBD\x9A" /* ｚ */ },
    {"A", "\xEF\xBC\xA1" /* Ａ */ },
    {"B", "\xEF\xBC\xA2" /* Ｂ */ },
    {"C", "\xEF\xBC\xA3" /* Ｃ */ },
    {"D", "\xEF\xBC\xA4" /* Ｄ */ },
    {"E", "\xEF\xBC\xA5" /* Ｅ */ },
    {"F", "\xEF\xBC\xA6" /* Ｆ */ },
    {"G", "\xEF\xBC\xA7" /* Ｇ */ },
    {"H", "\xEF\xBC\xA8" /* Ｈ */ },
    {"I", "\xEF\xBC\xA9" /* Ｉ */ },
    {"J", "\xEF\xBC\xAA" /* Ｊ */ },
    {"K", "\xEF\xBC\xAB" /* Ｋ */ },
    {"L", "\xEF\xBC\xAC" /* Ｌ */ },
    {"M", "\xEF\xBC\xAD" /* Ｍ */ },
    {"N", "\xEF\xBC\xAE" /* Ｎ */ },
    {"O", "\xEF\xBC\xAF" /* Ｏ */ },
    {"P", "\xEF\xBC\xB0" /* Ｐ */ },
    {"Q", "\xEF\xBC\xB1" /* Ｑ */ },
    {"R", "\xEF\xBC\xB2" /* Ｒ */ },
    {"S", "\xEF\xBC\xB3" /* Ｓ */ },
    {"T", "\xEF\xBC\xB4" /* Ｔ */ },
    {"U", "\xEF\xBC\xB5" /* Ｕ */ },
    {"V", "\xEF\xBC\xB6" /* Ｖ */ },
    {"W", "\xEF\xBC\xB7" /* Ｗ */ },
    {"X", "\xEF\xBC\xB8" /* Ｘ */ },
    {"Y", "\xEF\xBC\xB9" /* Ｙ */ },
    {"Z", "\xEF\xBC\xBA" /* Ｚ */ },
    {" ", "\xE3\x80\x80" /* 　 */ },
    {"\"", "\xE2\x80\x9D" /* ” */ },
    {"'", "\xE2\x80\x99" /* ’ */ },
    {"/", "\xEF\xBC\x8F" /* ／ */ },
    {"<<>", "\xEF\xBC\x9C" /* ＜ */ },
    {">", "\xEF\xBC\x9E" /* ＞ */ },
    {"`", "\xE2\x80\xB5" /* ‵ */ },
    {"[", "\xE3\x80\x94" /* 〔 */ },
    {"]", "\xE3\x80\x95" /* 〕 */ },
    {"{", "\xEF\xBD\x9B" /* ｛ */ },
    {"}", "\xEF\xBD\x9D" /* ｝ */ },
    {"+", "\xEF\xBC\x8B" /* ＋ */ },
    {"-", "\xEF\xBC\x8D" /* － */ },
};

FILE *fd;

void test_fullshape_input()
{
    ChewingContext *ctx;
    size_t i;

    ctx = chewing_new();
    start_testcase(ctx, fd);

    chewing_set_ChiEngMode(ctx, SYMBOL_MODE);
    chewing_set_ShapeMode(ctx, FULLSHAPE_MODE);

    for (i = 0; i < ARRAY_SIZE(FULLSHAPE_DATA); ++i) {
        type_keystroke_by_string(ctx, FULLSHAPE_DATA[i].token);
        // fullshape symbol does not present in preedit buffer.
        ok_preedit_buffer(ctx, "");
        ok_commit_buffer(ctx, FULLSHAPE_DATA[i].expected);
    }

    chewing_delete(ctx);
}

void test_set_fullshape()
{
    ChewingContext *ctx;

    ctx = chewing_new();
    start_testcase(ctx, fd);

    ok(chewing_get_ShapeMode(ctx) == HALFSHAPE_MODE, "default is HALFSHAPE_MODE");

    chewing_set_ShapeMode(ctx, FULLSHAPE_MODE);
    ok(chewing_get_ShapeMode(ctx) == FULLSHAPE_MODE, "mode shall change to FULLSHAPE_MODE");

    chewing_set_ShapeMode(ctx, -1);
    ok(chewing_get_ShapeMode(ctx) == FULLSHAPE_MODE, "mode shall not change when parameter is invalid");

    chewing_set_ShapeMode(ctx, HALFSHAPE_MODE);
    ok(chewing_get_ShapeMode(ctx) == HALFSHAPE_MODE, "mode shall change to HALFSHAPE_MODE");

    chewing_set_ShapeMode(ctx, -1);
    ok(chewing_get_ShapeMode(ctx) == HALFSHAPE_MODE, "mode shall not change when parameter is invalid");


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


    test_set_fullshape();
    test_fullshape_input();

    fclose(fd);

    return exit_status();
}
