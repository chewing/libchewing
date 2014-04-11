/**
 * test-easy-symbol.c
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

FILE *fd;

static const TestData EASY_SYMBOL[] = {
    {"Q", "\xE3\x80\x94" /* 〔 */ },
    {"W", "\xE3\x80\x95" /* 〕 */ },
    {"A", "\xE3\x80\x90" /* 【 */ },
    {"S", "\xE3\x80\x91" /* 】 */ },
    {"Z", "\xE3\x80\x8A" /* 《 */ },
    {"X", "\xE3\x80\x8B" /* 》 */ },
    {"E", "\xEF\xBD\x9B" /* ｛ */ },
    {"R", "\xEF\xBD\x9D" /* ｝ */ },
    {"D", "\xE3\x80\x8C" /* 「 */ },
    {"F", "\xE3\x80\x8D" /* 」 */ },
    {"C", "\xE3\x80\x8E" /* 『 */ },
    {"V", "\xE3\x80\x8F" /* 』 */ },
    {"T", "\xE2\x80\x98" /* ‘ */ },
    {"Y", "\xE2\x80\x99" /* ’ */ },
    {"G", "\xE2\x80\x9C" /* “ */ },
    {"H", "\xE2\x80\x9D" /* ” */ },
    {"B", "\xE3\x80\x9D" /* 〝 */ },
    {"N", "\xE3\x80\x9E" /* 〞 */ },
    {"U", "\xEF\xBC\x8B" /* ＋ */ },
    {"I", "\xEF\xBC\x8D" /* － */ },
    {"O", "\xC3\x97" /* × */ },
    {"P", "\xC3\xB7" /* ÷ */ },
    {"J", "\xE2\x89\xA0" /* ≠ */ },
    {"K", "\xE2\x89\x92" /* ≒ */ },
    {"L", "Orz"},
    {"M", "\xE2\x80\xA6" /* … */ },
};

static const TestData CHINESE = { "hk4g4<E>", "\xE6\xB8\xAC\xE8\xA9\xA6" /* 測試 */  };

void test_type_easy_symbol()
{
    ChewingContext *ctx;
    size_t i;

    ctx = chewing_new();
    start_testcase(ctx, fd);

    chewing_set_maxChiSymbolLen(ctx, 16);
    chewing_set_easySymbolInput(ctx, 1);

    for (i = 0; i < ARRAY_SIZE(EASY_SYMBOL); ++i) {
        type_keystroke_by_string(ctx, EASY_SYMBOL[i].token);
        ok_preedit_buffer(ctx, EASY_SYMBOL[i].expected);
        type_keystroke_by_string(ctx, "<E>");
        ok_commit_buffer(ctx, EASY_SYMBOL[i].expected);
    }

    chewing_delete(ctx);
}

void test_mode_change()
{
    ChewingContext *ctx;

    ctx = chewing_new();
    start_testcase(ctx, fd);

    chewing_set_maxChiSymbolLen(ctx, 16);

    type_keystroke_by_string(ctx, CHINESE.token);
    ok_commit_buffer(ctx, CHINESE.expected);

    chewing_set_easySymbolInput(ctx, 1);
    type_keystroke_by_string(ctx, EASY_SYMBOL[0].token);
    type_keystroke_by_string(ctx, "<E>");
    ok_commit_buffer(ctx, EASY_SYMBOL[0].expected);

    chewing_set_easySymbolInput(ctx, 0);
    type_keystroke_by_string(ctx, CHINESE.token);
    ok_commit_buffer(ctx, CHINESE.expected);

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


    test_type_easy_symbol();
    test_mode_change();

    fclose(fd);

    return exit_status();
}
