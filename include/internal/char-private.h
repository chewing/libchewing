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
#include "chewing-private.h"

#ifndef SEEK_SET
#define SEEK_SET 0
#endif

typedef struct {
	char word[ MAX_UTF8_SIZE+1 ];
} Word;

int GetCharFirst( ChewingData *, Word *, uint16_t );
int GetCharNext ( ChewingData *, Word * );
int InitChar( ChewingData *pgdata, const char * prefix );
void TerminateChar( ChewingData *pgdata );

#endif
