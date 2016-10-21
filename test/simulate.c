/**
 * simulate.c
 *
 * Copyright (c) 2008, 2010
 *      libchewing Core Team. See ChangeLog for details.
 *
 * See the file "COPYING" for information on usage and redistribution
 * of this file.
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#ifdef HAVE_UNISTD_H
#    include <unistd.h>
#else
#    include "plat_types.h"
#endif

#include "chewing.h"
#include "testhelper.h"
#include "internal/chewing-utf8-util.h"

#define FN_MATERIALS "materials.txt"

static FILE *fp = NULL;

static int selKey_define[11] = { '1', '2', '3', '4', '5', '6', '7', '8', '9', '0', 0 }; /* Default */

int init_sim()
{
    if (0 == access(FN_MATERIALS "-random", R_OK))
        fp = fopen(FN_MATERIALS "-random", "r");
    else
        fp = fopen(FN_MATERIALS, "r");
    return (fp != NULL);
}

int fini_sim()
{
    if (fp)
        fclose(fp);
    fflush(stdout);
    return 0;
}

#define MAXLEN 149
static char linebuf[MAXLEN];
static char commit_string_buf[MAXLEN];
static char expect_string_buf[MAXLEN];

int tested_word_count = 0;
int failed_word_count = 0;

static int get_test_case()
{
    while (fgets(linebuf, sizeof(linebuf), fp)) {
        char *pos;
        if (linebuf[0] == '#' || linebuf[0] == ' ')
            continue;

        /* input string */
        pos = strstr(linebuf, "<E>");
        if (!pos) {
            fprintf(stderr, "error: no <E> in input string\n");
            return 0;
        }
        *(pos + 3) = '\0';

        /* expected string */
        pos += 4;
        while (*pos == '\t' || *pos == ' ')
            pos++;
        strcpy(expect_string_buf, pos);
        return 1;
    }
    return 0;
}

static void commit_string(ChewingContext *ctx)
{
    char *s;

    if (chewing_commit_Check(ctx)) {
        s = chewing_commit_String(ctx);
        strcat(commit_string_buf, s);
        free(s);
    }
}

void compare_per_run()
{
    int i, len;
    char utf8buf_expect[16];
    char utf8buf_commit[16];

    printf("Expected:  %s", expect_string_buf);
    printf("Committed: ");

    tested_word_count += (len = ueStrLen(expect_string_buf) - 1);
    /* omit the suffix character */
    for (i = 0; i < len; i++) {
        ueStrNCpy(utf8buf_expect, ueStrSeek(expect_string_buf, i), 1, STRNCPY_CLOSE);
        ueStrNCpy(utf8buf_commit, ueStrSeek(commit_string_buf, i), 1, STRNCPY_CLOSE);
        if (!strcmp(utf8buf_expect, utf8buf_commit))
            printf("%s", utf8buf_commit);
        else {
            printf("\033[44;37m%s\033[m", utf8buf_commit);
            failed_word_count++;
        }
    }
    memset(commit_string_buf, 0, MAXLEN);
    printf("\n\n");
}

int main()
{
    if (!init_sim())
        return 1;

    /* Initialize libchewing */
    putenv("CHEWING_PATH=" CHEWING_DATA_PREFIX);
    /* for the sake of testing, we should not change existing hash data */
    putenv("CHEWING_USER_PATH=" TEST_HASH_DIR);

    while (get_test_case()) {
        /* Request handle to ChewingContext */
        ChewingContext *ctx = chewing_new();

        chewing_set_KBType(ctx, chewing_KBStr2Num("KB_DEFAULT"));
        chewing_set_candPerPage(ctx, 9);
        chewing_set_maxChiSymbolLen(ctx, 16);
        chewing_set_addPhraseDirection(ctx, 1);
        chewing_set_selKey(ctx, selKey_define, 10);
        chewing_set_spaceAsSelection(ctx, 1);

        int ch;
        char *keystroke = linebuf;
        while ((ch = get_keystroke(get_char_by_string, &keystroke)) != END) {
            type_single_keystroke(ctx, ch);
            commit_string(ctx);
        }
        compare_per_run();

        chewing_delete(ctx);
    }

    printf("_________________________________________________________________________\n" "[ Report ]\n");
    printf("Checks: %d words,  Failures: %d words\n", tested_word_count, failed_word_count);
    printf("Ratio: %.2f%%\n", (float) (tested_word_count - failed_word_count) / tested_word_count * 100);

    fini_sim();
    return 0;
}
