/**
 * hash-private.h
 *
 * Copyright (c) 2008
 *      libchewing Core Team. See ChangeLog for details.
 *
 * See the file "COPYING" for information on usage and redistribution
 * of this file.
 */

/* *INDENT-OFF* */
#ifndef _CHEWING_HASH_PRIVATE_H
#define _CHEWING_HASH_PRIVATE_H
/* *INDENT-ON* */

#include "global.h"
#include "userphrase-private.h"

#ifdef __MacOSX__
#    define CHEWING_HASH_PATH "/Library/ChewingOSX"
#else
#    define CHEWING_HASH_PATH "/.chewing"
#endif

#define FIELD_SIZE (125)
#define BIN_HASH_SIG "CBiH"
#define HASH_FILE  "uhash.dat"

typedef struct HASH_ITEM {
    int item_index;
    UserPhraseData data;
    struct HASH_ITEM *next;
} HASH_ITEM;

HASH_ITEM *HashFindPhone(const uint16_t phoneSeq[]);
HASH_ITEM **HashFindHead(struct ChewingData *pgdata, const uint16_t phoneSeq[]);
HASH_ITEM *HashFindEntry(struct ChewingData *pgdata, const uint16_t phoneSeq[], const char wordSeq[]);
HASH_ITEM *HashInsert(struct ChewingData *pgdata, UserPhraseData *pData);
HASH_ITEM *HashFindPhonePhrase(struct ChewingData *pgdata, const uint16_t phoneSeq[], HASH_ITEM *pHashLast);
HASH_ITEM *FindNextHash(const struct ChewingData *pgdata, HASH_ITEM *curr);
void HashModify(struct ChewingData *pgdata, HASH_ITEM *pItem);
void FreeHashItem(HASH_ITEM *pItem);
int AlcUserPhraseSeq(UserPhraseData *pData, int phonelen, int wordlen);
int InitUserphrase(struct ChewingData *pgdata, const char *path);
void TerminateUserphrase(struct ChewingData *pgdata);
void FreeHashTable(void);

/* *INDENT-OFF* */
#endif
/* *INDENT-ON* */
