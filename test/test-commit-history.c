/**
 * test-userphrase.c
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
#include <string.h>
#include <stdlib.h>
#include <stdio.h>

#include "chewing.h"
#include "plat_types.h"
#include "testhelper.h"

FILE *fd;

void test_commit_history_check_empty()
{
    int ret;
    ChewingContext *ctx;

    clean_userphrase();

    ctx = chewing_new();
    start_testcase(ctx, fd);
    chewing_set_maxChiSymbolLen(ctx, 16);

    ret = chewing_commit_history_enumerate(ctx);
    ok(ret == 0, "chewing_commit_history_enumerate() returns %d", ret);
    ret = chewing_commit_history_has_next(ctx);
    ok(ret == 0, "chewing_commit_history_has_next returns %d", ret);

    chewing_delete(ctx);
}

void test_commit_history_get()
{
    int ret;
    int length;
    int word_len;
    char *words;
    unsigned short *phones;
    static const char phrase[] = "\xE6\xB8\xAC\xE8\xA9\xA6" /* 測試 */ ;
    static const char phrase2[] = "\xE6\x96\xB0\xE9\x85\xB7\xE9\x9F\xB3" /* 新酷音 */ ;
    ChewingContext *ctx;

    clean_userphrase();

    ctx = chewing_new();
    start_testcase(ctx, fd);
    chewing_set_maxChiSymbolLen(ctx, 16);

    ret = chewing_commit_history_enumerate(ctx);
    ok(ret == 0, "chewing_commit_history_enumerate() returns %d", ret);
    ret = chewing_commit_history_has_next(ctx);
    ok(ret == 0, "chewing_commit_history_has_next() returns %d", ret);

    type_keystroke_by_string(ctx, "hk4g4<E>");
    type_keystroke_by_string(ctx, "vup dj4up <E>");

    /* 測試 */
    ret = chewing_commit_history_has_next(ctx);
    ok(ret == 1, "chewing_commit_history_has_next() returns %d", ret);
    ret = chewing_commit_history_get(ctx, &length, &words, &phones);
    ok(ret == 0, "chewing_commit_history_get() returns %d", ret);

    word_len = ueStrLen(phrase);
    ok(length == word_len, "length shall be %d, returns %d", word_len, length);
    ok(strcmp(words, phrase) == 0, "shall get `%s', get `%s' ", phrase, words);

    free(words);
    free(phones);

    /* 新酷音 */
    ret = chewing_commit_history_has_next(ctx);
    ok(ret == 1, "chewing_commit_history_has_next() returns %d", ret);
    ret = chewing_commit_history_get(ctx, &length, &words, &phones);
    ok(ret == 0, "chewing_commit_history_get() returns %d", ret);

    word_len = ueStrLen(phrase2);
    ok(length == word_len, "length shall be %d, returns %d", word_len, length);
    ok(strcmp(words, phrase2) == 0, "shall get `%s', get `%s' ", phrase2, words);

    free(words);
    free(phones);

    chewing_delete(ctx);
}

void test_commit_history_remove()
{
    int ret;
    ChewingContext *ctx;
    static const char phrase[] = "\xE6\xB8\xAC\xE8\xA9\xA6" /* 測試 */ ;

    clean_userphrase();

    ctx = chewing_new();
    start_testcase(ctx, fd);
    chewing_set_maxChiSymbolLen(ctx, 16);

    type_keystroke_by_string(ctx, "hk4g4<E>");
    type_keystroke_by_string(ctx, "hk4g4<E>");

    ret = chewing_commit_history_remove(ctx, phrase);
    ok(ret == 2, "chewing_commit_history_remove() returns %d, shall be %d", ret, 2);
    ret = chewing_commit_history_enumerate(ctx);
    ok(ret == 0, "chewing_commit_history_enumerate() returns %d, shall be %d", ret, 0);
    ret = chewing_commit_history_has_next(ctx);
    ok(ret == 0, "chewing_commit_history_has_next returns %d, shall be %d", ret, 0);

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

    test_commit_history_check_empty();
    test_commit_history_get();
    test_commit_history_remove();

    fclose(fd);

    return exit_status();
}
