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
#define N_HASH_BIT (14)
#define BIN_HASH_SIG "CBiH"
#define HASH_FILE  "uhash.dat"
#define HASH_TABLE_SIZE (1<<N_HASH_BIT)

typedef struct tag_HASH_ITEM {
	int item_index;
	UserPhraseData data;
	struct tag_HASH_ITEM *next;
} HASH_ITEM;

HASH_ITEM *HashFindPhone( const uint16 phoneSeq[] );
HASH_ITEM *HashFindEntry( const uint16 phoneSeq[], const char wordSeq[] );
HASH_ITEM *HashInsert( UserPhraseData *pData );
HASH_ITEM *HashFindPhonePhrase( const uint16 phoneSeq[], HASH_ITEM *pHashLast );
void HashModify( HASH_ITEM *pItem );
int AlcUserPhraseSeq( UserPhraseData *pData, int phonelen, int wordlen );
int InitHash( const char *path );
void FreeHashTable( void );

#endif
