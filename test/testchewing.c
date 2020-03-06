/**
 * testchewing.c
 *
 * Copyright (c) 2004, 2005, 2008, 2011
 *      libchewing Core Team. See ChangeLog for details.
 *
 * See the file "COPYING" for information on usage and redistribution
 * of this file.
 */

#include "chewing.h"
#include "testhelper.h"

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

static int selKey_define[11] = { '1', '2', '3', '4', '5', '6', '7', '8', '9', '0', 0 }; /* Default */

void commit_string(ChewingContext *ctx)
{
    char *s;

    if (chewing_commit_Check(ctx)) {
        s = chewing_commit_String(ctx);
        printf("%s", s);
        fflush(stdout);
        free(s);
    }
}

int main(int argc, char *argv[])
{
    ChewingContext *ctx;
    int i;
    FILE *fp = stdin;

    /* Initialize libchewing */
    putenv("CHEWING_PATH=" CHEWING_DATA_PREFIX);
    /* for the sake of testing, we should not change existing hash data */
    putenv("CHEWING_USER_PATH=" TEST_HASH_DIR);

    if (argc == 2) {
        fp = fopen(argv[1], "r");
        if (!fp) {
            fprintf(stderr, "failed to open '%s'\n", argv[1]);
            return 1;
        }
    }

    /* Request handle to ChewingContext */
    ctx = chewing_new();

    /* Set keyboard type */
    chewing_set_KBType(ctx, chewing_KBStr2Num("KB_DEFAULT"));

    chewing_set_candPerPage(ctx, 9);
    chewing_set_maxChiSymbolLen(ctx, 16);
    chewing_set_addPhraseDirection(ctx, 1);
    chewing_set_selKey(ctx, selKey_define, 10);
    chewing_set_spaceAsSelection(ctx, 1);

    while (1) {
        i = get_keystroke(get_char_from_fp, fp);
        if (i == END)
            goto end;
        type_single_keystroke(ctx, i);
        commit_string(ctx);
    }
  end:
    /* Free Chewing IM handle */
    chewing_delete(ctx);
    if (fp)
        fclose(fp);

    return 0;
}
