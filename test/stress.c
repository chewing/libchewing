/**
 * stress.c
 *
 * Copyright (c) 2015
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
#include <signal.h>
#include <assert.h>

#ifndef _MSC_VER
#include <unistd.h>
#else
#include <io.h>
#endif

static int selKey_define[11] = { '1', '2', '3', '4', '5', '6', '7', '8', '9', '0', 0 }; /* Default */

static int all_key[] = {
    KEY_LEFT,
    KEY_SLEFT,
    KEY_RIGHT,
    KEY_SRIGHT,
    KEY_UP,
    KEY_DOWN,
    KEY_SPACE,
    KEY_ENTER,
    KEY_BACKSPACE,
    KEY_ESC,
    KEY_DELETE,
    KEY_HOME,
    KEY_END,
    KEY_TAB,
    KEY_CAPSLOCK,
    KEY_NPAGE,
    KEY_PPAGE,
    KEY_SSPACE,
    KEY_DBLTAB,
    KEY_CTRL_BASE + '0',
    KEY_CTRL_BASE + '1',
    KEY_CTRL_BASE + '2',
    KEY_CTRL_BASE + '3',
    KEY_CTRL_BASE + '4',
    KEY_CTRL_BASE + '5',
    KEY_CTRL_BASE + '6',
    KEY_CTRL_BASE + '7',
    KEY_CTRL_BASE + '8',
    KEY_CTRL_BASE + '9',
    KEY_NUMPAD_BASE + '0',
    KEY_NUMPAD_BASE + '1',
    KEY_NUMPAD_BASE + '2',
    KEY_NUMPAD_BASE + '3',
    KEY_NUMPAD_BASE + '4',
    KEY_NUMPAD_BASE + '5',
    KEY_NUMPAD_BASE + '6',
    KEY_NUMPAD_BASE + '7',
    KEY_NUMPAD_BASE + '8',
    KEY_NUMPAD_BASE + '9',
    KEY_NUMPAD_BASE + '+',
    KEY_NUMPAD_BASE + '-',
    KEY_NUMPAD_BASE + '*',
    KEY_NUMPAD_BASE + '/',
    KEY_NUMPAD_BASE + '.',
    // all isprint()
    '~', '!', '@', '#', '$', '%', '^', '&', '*', '(', ')', '_', '+',
    '`', '1', '2', '3', '4', '5', '6', '7', '8', '9', '0', '-', '=',
    'Q', 'W', 'E', 'R', 'T', 'Y', 'U', 'I', 'O', 'P', '{', '}', '|',
    'q', 'w', 'e', 'r', 't', 'y', 'u', 'i', 'o', 'p', '[', ']', '\\',
    'A', 'S', 'D', 'F', 'G', 'H', 'J', 'K', 'L', ':', '"',
    'a', 's', 'd', 'f', 'g', 'h', 'j', 'k', 'l', ';', '\'',
    'Z', 'X', 'C', 'V', 'B', 'N', 'M', '<', '>', '?',
    'z', 'x', 'c', 'v', 'b', 'n', 'm', ',', '.', '/',
};

static int random256()
{
    return rand() % 256;
}

void commit_string(ChewingContext *ctx)
{
    char *s;

    if (chewing_commit_Check(ctx)) {
        s = chewing_commit_String(ctx);
        free(s);
    }
}

int get_stdin()
{
    unsigned char c;
    int len = read(0, &c, 1);
    if (len <= 0)
	return EOF;
    return c;
}

int main(int argc, char *argv[])
{
    int i;
    int flag_random_init = 0;
    int flag_random_extra = 0;
    int flag_loop = -1;
    int (*get_input)() = &random256;

    for (i = 1; i < argc; i++) {
	if (strcmp(argv[i], "-init") == 0)
	    flag_random_init = 1;
	else if (strcmp(argv[i], "-extra") == 0 && argv[i + 1])
	    flag_random_extra = 1;
	else if (strcmp(argv[i], "-loop") == 0 && argv[i + 1])
	    flag_loop = atoi(argv[++i]);
	else if (strcmp(argv[i], "-stdin") == 0)
	    get_input = &get_stdin;
	else {
	    printf("Usage: %s [-init] [-extra] [-loop N] [-stdin]\n", argv[0]);
	    printf("\t-init           Random initial configuration\n");
	    printf("\t-extra          Random change all configurations during input.\n");
	    printf("\t                This is usually unexpected.\n");
	    printf("\t-stdin          Get random input from stdin\n");
	    printf("\t-loop N         How many iterations to test (default infinite=-1)\n");
	    exit(1);
	}
    }

    /* Initialize libchewing */
    putenv("CHEWING_PATH=" CHEWING_DATA_PREFIX);
    /* for the sake of testing, we should not change existing hash data */
    putenv("CHEWING_USER_PATH=" TEST_HASH_DIR);

    for (i = 0; i != flag_loop; i++) {
	ChewingContext *ctx = chewing_new();

	/* typical configuration */
	chewing_set_KBType(ctx, chewing_KBStr2Num("KB_DEFAULT"));
	chewing_set_candPerPage(ctx, 9);
	chewing_set_maxChiSymbolLen(ctx, 16);
	chewing_set_addPhraseDirection(ctx, 1);
	chewing_set_selKey(ctx, selKey_define, 10);
	chewing_set_spaceAsSelection(ctx, 1);

	if (flag_random_init) {
	    chewing_set_KBType(ctx, get_input());
	    chewing_set_candPerPage(ctx, get_input());
	    chewing_set_maxChiSymbolLen(ctx, get_input());
	    chewing_set_addPhraseDirection(ctx, get_input());
	    chewing_set_selKey(ctx, selKey_define, get_input() % 11);
	    chewing_set_spaceAsSelection(ctx, get_input());
	    chewing_set_escCleanAllBuf(ctx, get_input());
	    chewing_set_autoShiftCur(ctx, get_input());
	    chewing_set_easySymbolInput(ctx, get_input());
	    chewing_set_phraseChoiceRearward(ctx, get_input());
	}

	while (1) {
	    /* Random value: [0, max_key) for keys, [max_key, 0xff] for
	     * configurations. Use a fixed range here because I don't want the
	     * meaning of input changed a lot frequently if we add more keys in
	     * the future. */
	    const int max_key = 192;  /* arbitrary number */
	    int v = get_input();
	    if (v == EOF)
		break;
	    assert(max_key * sizeof(all_key[0]) >= sizeof(all_key));
	    if (v >= max_key) {
		const int typical = 2;
		int handled = 1;
		v = v - max_key;
		if (flag_random_extra || v < typical) {
		    switch (v) {
			/* typical configurations may be changed during input */
			case 0:
			    chewing_set_ChiEngMode(ctx, get_input());
			    break;
			case 1:
			    chewing_set_ShapeMode(ctx, get_input());
			    break;

			/* usually not changed during input */
			case 2:
			    chewing_set_KBType(ctx, get_input());
			    break;
			case 3:
			    chewing_set_candPerPage(ctx, get_input());
			    break;
			case 4:
			    chewing_set_maxChiSymbolLen(ctx, get_input());
			    break;
			case 5:
			    chewing_set_addPhraseDirection(ctx, get_input());
			    break;
			case 6:
			    chewing_set_selKey(ctx, selKey_define, get_input() % 11);
			    break;
			case 7:
			    chewing_set_spaceAsSelection(ctx, get_input());
			    break;
			case 8:
			    chewing_set_escCleanAllBuf(ctx, get_input());
			    break;
			case 9:
			    chewing_set_autoShiftCur(ctx, get_input());
			    break;
			case 10:
			    chewing_set_easySymbolInput(ctx, get_input());
			    break;
			case 11:
			    chewing_set_phraseChoiceRearward(ctx, get_input());
			    break;
			default:
			    handled = 0;
			    break;
		    }
		} else {
		    handled = 0;
		}
		if (!handled)
		    break;
	    } else {
		if (0 <= v && v < max_key) {
		    int key = all_key[v];
		    type_single_keystroke(ctx, key);
		} else {
		    break;
		}
	    }
	    commit_string(ctx);
	}
	chewing_delete(ctx);

	if (i % 10000 == 0)
	    printf("%d\n", i);

#if !defined(_WIN32) && !defined(_WIN64) && !defined(_WIN32_WCE)
	if (getenv("AFL_PERSISTENT"))
	    raise(SIGSTOP);
#endif
    }

    return 0;
}
