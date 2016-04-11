/**
 * bopomofo-private.h
 *
 * Copyright (c) 2008, 2012
 *      libchewing Core Team. See ChangeLog for details.
 *
 * See the file "COPYING" for information on usage and redistribution
 * of this file.
 */

/* *INDENT-OFF* */
#ifndef _CHEWING_BOPOMOFO_PRIVATE_H
#define _CHEWING_BOPOMOFO_PRIVATE_H
/* *INDENT-ON* */

#include "chewing-private.h"

/** Chewing Phonetic Definitions */
enum {
    BOPOMOFO_IGNORE,
    BOPOMOFO_ABSORB,
    BOPOMOFO_COMMIT,
    BOPOMOFO_KEY_ERROR,
    BOPOMOFO_ERROR,
    BOPOMOFO_NO_WORD,
    BOPOMOFO_OPEN_SYMBOL_TABLE
};

/** keyboard layout */
enum {
    KB_DEFAULT,
    KB_HSU,
    KB_IBM,
    KB_GIN_YIEH,
    KB_ET,
    KB_ET26,
    KB_DVORAK,
    KB_DVORAK_HSU,
    KB_DACHEN_CP26,
    KB_HANYU_PINYIN,
    KB_THL_PINYIN,
    KB_MPS2_PINYIN,
    KB_CARPALX,
    KB_TYPE_NUM
};

int BopomofoPhoInput(ChewingData *, int key);       /* assume `key' is "ascii" code. */
int BopomofoRemoveLast(BopomofoData *);
int BopomofoRemoveAll(BopomofoData *);
int BopomofoIsEntering(BopomofoData *);

/* *INDENT-OFF* */
#endif
/* *INDENT-ON* */
