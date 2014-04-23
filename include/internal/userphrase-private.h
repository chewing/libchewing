/**
 * userphrase-private.h
 *
 * Copyright (c) 2008
 *      libchewing Core Team. See ChangeLog for details.
 *
 * See the file "COPYING" for information on usage and redistribution
 * of this file.
 */

/* *INDENT-OFF* */
#ifndef _CHEWING_USERPHRASE_PRIVATE_H
#define _CHEWING_USERPHRASE_PRIVATE_H
/* *INDENT-ON* */

#ifdef HAVE_CONFIG_H
#    include <config.h>
#endif

#ifdef HAVE_INTTYPES_H
#    include <inttypes.h>
#elif defined HAVE_STDINT_H
#    include <stdint.h>
#endif

#if WITH_SQLITE3
#    define DB_NAME	"chewing.sqlite3"
#else
#    define DB_NAME	"uhash.dat"
#endif

#define FREQ_INIT_VALUE (1)
#define SHORT_INCREASE_FREQ (10)
#define MEDIUM_INCREASE_FREQ (5)
#define LONG_DECREASE_FREQ (10)
#define MAX_ALLOW_FREQ (99999999)

#define USER_UPDATE_FAIL (4)
#define USER_UPDATE_INSERT (1)
#define USER_UPDATE_MODIFY (2)
#define USER_UPDATE_IGNORE (8)

/* Forward declaration */
struct ChewingData;

typedef struct UserPhraseData {
    uint16_t *phoneSeq;
    char *wordSeq;
    int userfreq;
    int recentTime;
    int origfreq;               /* the initial frequency of this phrase */
    int maxfreq;                /* the maximum frequency of the phrase of the same pid */
} UserPhraseData;

void UserUpdatePhraseBegin(struct ChewingData *pgdata);

/**
 * @brief Update or add a new UserPhrase.
 *
 * @param phoneSeq[] Phone sequence
 * @param wordSeq[] Phrase against the phone sequence
 *
 * @return
 * @retval USER_UPDATE_FAIL Update fail.
 * @retval USER_UPDATE_INSERT Sequence is new, add new entry.
 * @retval USER_UPDATE_MODIFY Sequence is existing, update it's data.
 */
int UserUpdatePhrase(struct ChewingData *pgdata, const uint16_t phoneSeq[], const char wordSeq[]);

void UserUpdatePhraseEnd(struct ChewingData *pgdata);
int UserRemovePhrase(struct ChewingData *pgdata, const uint16_t phoneSeq[], const char wordSeq[]);

/**
 * @brief Read the first phrase of the phone in user phrase database.
 *
 * @param phoneSeq[] Phone sequence
 *
 * @return UserPhraseData, if it's not existing then return NULL.
 */
UserPhraseData *UserGetPhraseFirst(struct ChewingData *pgdata, const uint16_t phoneSeq[]);

/**
 * @brief Read the next phrase of the phone in user phrase database.
 *
 * @param phoneSeq[] Phone sequence
 *
 * @return UserPhraseData, if it's not existing then return NULL.
 */
UserPhraseData *UserGetPhraseNext(struct ChewingData *pgdata, const uint16_t phoneSeq[]);

void UserGetPhraseEnd(struct ChewingData *pgdata, const uint16_t phoneSeq[]);

void IncreaseLifeTime(struct ChewingData *pgdata);

char *GetDefaultUserPhrasePath(struct ChewingData *pgdata);

/* *INDENT-OFF* */
#endif
/* *INDENT-ON* */
