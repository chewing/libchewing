/**
 * dict-private.h
 *
 * Copyright (c) 2008
 *	libchewing Core Team. See ChangeLog for details.
 *
 * See the file "COPYING" for information on usage and redistribution
 * of this file.
 */

#ifndef _CHEWING_DICT_PRIVATE_H
#define _CHEWING_DICT_PRIVATE_H

#include "chewing-private.h"

#ifndef SEEK_SET
#define SEEK_SET 0
#endif

#define PHONE_PHRASE_NUM (162244)

int GetPhraseFirst( Phrase *phr_ptr, int phone_phr_id );
int GetPhraseNext ( Phrase *phr_ptr );
int InitDict( const char * );

#endif
