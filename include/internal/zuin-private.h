/**
 * zuin-private.h
 *
 * Copyright (c) 2008
 *	libchewing Core Team. See ChangeLog for details.
 *
 * See the file "COPYING" for information on usage and redistribution
 * of this file.
 */

#ifndef _CHEWING_ZUIN_PRIVATE_H
#define _CHEWING_ZUIN_PRIVATE_H

#include "chewing-private.h"

/** Chewing Phonetic Definitions */
#define ZUIN_IGNORE 0
#define ZUIN_ABSORB 1
#define ZUIN_COMMIT 2
#define ZUIN_KEY_ERROR 4
#define ZUIN_ERROR 8
#define ZUIN_NO_WORD 16

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
	KB_TYPE_NUM
};

int ZuinPhoInput( ZuinData *,int key );  /* assume `key' is "ascii" code. */
int ZuinRemoveLast( ZuinData * );
int ZuinRemoveAll( ZuinData * );
int ZuinIsEntering( ZuinData * );
         
#endif
