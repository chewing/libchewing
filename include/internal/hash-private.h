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

#define CHEWING_DB_TABLE_NAME	"userphrase_v1"
#define CHEWING_DB_CREATE_TABLE	"CREATE TABLE IF NOT EXISTS " CHEWING_DB_TABLE_NAME " (" \
	"time INTEGER," \
	"user_freq INTEGER," \
	"max_freq INTEGER," \
	"orig_freq INTEGER," \
	"phone BLOB," \
	"phrase TEXT," \
	"PRIMARY KEY (phone, phrase)" \
	");"

#define CHEWING_DB_COLUMN "time, user_freq, max_freq, orig_freq, phone, phrase"

/*
 * The SELECT index starts from 0, but the INSERT/REPLACE index starts from 1,
 * so we cannot use the same index for both SELECT & INSERT/REPLACE.
 */
#define CHEWING_DB_SEL_INDEX_TIME	(0)
#define CHEWING_DB_SEL_INDEX_USER_FREQ	(1)
#define CHEWING_DB_SEL_INDEX_MAX_FREQ	(2)
#define CHEWING_DB_SEL_INDEX_ORIG_FREQ	(3)
#define CHEWING_DB_SEL_INDEX_PHONE	(4)
#define CHEWING_DB_SEL_INDEX_PHRASE	(5)

#define CHEWING_DB_INS_INDEX_TIME	(CHEWING_DB_SEL_INDEX_TIME + 1)
#define CHEWING_DB_INS_INDEX_USER_FREQ	(CHEWING_DB_SEL_INDEX_USER_FREQ + 1)
#define CHEWING_DB_INS_INDEX_MAX_FREQ	(CHEWING_DB_SEL_INDEX_MAX_FREQ + 1)
#define CHEWING_DB_INS_INDEX_ORIG_FREQ	(CHEWING_DB_SEL_INDEX_ORIG_FREQ + 1)
#define CHEWING_DB_INS_INDEX_PHONE	(CHEWING_DB_SEL_INDEX_PHONE + 1)
#define CHEWING_DB_INS_INDEX_PHRASE	(CHEWING_DB_SEL_INDEX_PHRASE + 1)

#define CHEWING_DB_SELECT_BY_PHONE "SELECT " CHEWING_DB_COLUMN " FROM " \
	CHEWING_DB_TABLE_NAME " WHERE phone = ?4"

#define CHEWING_DB_SELECT_BY_PHONE_PHRASE "SELECT " CHEWING_DB_COLUMN " FROM " \
	CHEWING_DB_TABLE_NAME " WHERE phone = ?4 AND phrase = ?5"

#define CHEWING_DB_UPSERT "INSERT OR REPLACE INTO " CHEWING_DB_TABLE_NAME \
	"(" CHEWING_DB_COLUMN ") VALUES (?1,?2,?3,?4,?5,?6)"

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
