/**
 * char-private.h
 *
 * Copyright (c) 2008
 *	libchewing Core Team. See ChangeLog for details.
 *
 * See the file "COPYING" for information on usage and redistribution
 * of this file.
 */

#ifndef _CHEWING_CHAR_PRIVATE_H
#define _CHEWING_CHAR_PRIVATE_H

#include "global.h"

#ifndef SEEK_SET
#define SEEK_SET 0
#endif

typedef struct {
	char word[ 7 ];
} Word;

int GetCharFirst( Word *, uint16 );
int GetCharNext ( Word * );
int InitChar( const char * );

#endif
