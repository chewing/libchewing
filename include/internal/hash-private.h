/**
 * hash-private.h
 *
 * Copyright (c) 2008
 *	libchewing Core Team. See ChangeLog for details.
 *
 * See the file "COPYING" for information on usage and redistribution
 * of this file.
 */

#ifndef _CHEWING_HASH_PRIVATE_H
#define _CHEWING_HASH_PRIVATE_H

#include "global.h"
#include "userphrase-private.h"

#ifdef __MacOSX__
#define CHEWING_HASH_PATH "/Library/ChewingOSX"
#else
#define CHEWING_HASH_PATH "/.chewing"
#endif

#define FIELD_SIZE (125)
#define BIN_HASH_SIG "CBiH"
#define HASH_FILE  "uhash.dat"

HASH_ITEM *HashFindPhone( const uint16 phoneSeq[] );
HASH_ITEM *HashFindEntry( ChewingData *pgdata, const uint16 phoneSeq[], const char wordSeq[] );
HASH_ITEM *HashInsert( ChewingData *pgdata, UserPhraseData *pData );
HASH_ITEM *HashFindPhonePhrase( ChewingData *pgdata, const uint16 phoneSeq[], HASH_ITEM *pHashLast );
void HashModify( ChewingData *pgdata, HASH_ITEM *pItem );
int AlcUserPhraseSeq( UserPhraseData *pData, int phonelen, int wordlen );
int InitHash( ChewingData *ctx );
void FreeHashTable( void );

#endif
