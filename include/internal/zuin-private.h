/**
 * zuin-private.h
 *
 * Copyright (c) 2008, 2012
 *	libchewing Core Team. See ChangeLog for details.
 *
 * See the file "COPYING" for information on usage and redistribution
 * of this file.
 */

#ifndef _CHEWING_ZUIN_PRIVATE_H
#define _CHEWING_ZUIN_PRIVATE_H

#include "chewing-private.h"

/** Chewing Phonetic Definitions */
enum{
	ZUIN_IGNORE,
	ZUIN_ABSORB,
	ZUIN_COMMIT,
	ZUIN_KEY_ERROR,
	ZUIN_ERROR,
	ZUIN_NO_WORD,
	ZUIN_OPEN_SYMBOL_TABLE
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
	KB_TYPE_NUM
};

int ZuinPhoInput( ChewingData *, int key );  /* assume `key' is "ascii" code. */
int ZuinRemoveLast( ZuinData * );
int ZuinRemoveAll( ZuinData * );
int ZuinIsEntering( ZuinData * );

#endif
