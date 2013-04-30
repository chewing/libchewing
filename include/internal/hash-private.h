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

typedef struct tag_HASH_ITEM {
	int item_index;
	UserPhraseData data;
	struct tag_HASH_ITEM *next;
} HASH_ITEM;

HASH_ITEM *HashFindPhone( const uint16_t phoneSeq[] );
HASH_ITEM *HashFindEntry( struct tag_ChewingData *pgdata, const uint16_t phoneSeq[], const char wordSeq[] );
HASH_ITEM *HashInsert( struct tag_ChewingData *pgdata, UserPhraseData *pData );
HASH_ITEM *HashFindPhonePhrase( struct tag_ChewingData *pgdata, const uint16_t phoneSeq[], HASH_ITEM *pHashLast );
void HashModify( struct tag_ChewingData *pgdata, HASH_ITEM *pItem );
int AlcUserPhraseSeq( UserPhraseData *pData, int phonelen, int wordlen );
int InitHash( struct tag_ChewingData *ctx );
void TerminateHash( struct tag_ChewingData *pgdata );
void FreeHashTable( void );

#endif
