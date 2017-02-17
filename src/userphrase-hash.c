/**
 * userphrase-hash.c
 *
 * Copyright (c) 1999, 2000, 2001
 *      Lu-chuan Kung and Kang-pen Chen.
 *      All rights reserved.
 *
 * Copyright (c) 2004-2006, 2008, 2012-2014
 *      libchewing Core Team. See ChangeLog for details.
 *
 * See the file "COPYING" for information on usage and redistribution
 * of this file.
 */

#include <assert.h>
#include <stdlib.h>
#include <string.h>
#include <stdio.h>

#include "chewing-utf8-util.h"
#include "hash-private.h"
#include "dict-private.h"
#include "tree-private.h"
#include "userphrase-private.h"
#include "private.h"

/* load the original frequency from the static dict */
static int LoadOriginalFreq(ChewingData *pgdata, const uint16_t phoneSeq[], const char wordSeq[], int len)
{
    const TreeType *tree_pos;
    int retval;
    Phrase *phrase = ALC(Phrase, 1);

    tree_pos = TreeFindPhrase(pgdata, 0, len - 1, phoneSeq);
    if (tree_pos) {
        GetPhraseFirst(pgdata, phrase, tree_pos);
        do {
            /* find the same phrase */
            if (!strcmp(phrase->phrase, wordSeq)) {
                retval = phrase->freq;
                free(phrase);
                return retval;
            }
        } while (GetVocabNext(pgdata, phrase));
    }

    free(phrase);
    return FREQ_INIT_VALUE;
}

/* find the maximum frequency of the same phrase */
static int LoadMaxFreq(ChewingData *pgdata, const uint16_t phoneSeq[], int len)
{
    const TreeType *tree_pos;
    Phrase *phrase = ALC(Phrase, 1);
    int maxFreq = FREQ_INIT_VALUE;
    UserPhraseData *uphrase;

    tree_pos = TreeFindPhrase(pgdata, 0, len - 1, phoneSeq);
    if (tree_pos) {
        GetPhraseFirst(pgdata, phrase, tree_pos);
        do {
            if (phrase->freq > maxFreq)
                maxFreq = phrase->freq;
        } while (GetVocabNext(pgdata, phrase));
    }
    free(phrase);

    uphrase = UserGetPhraseFirst(pgdata, phoneSeq);
    while (uphrase) {
        if (uphrase->userfreq > maxFreq)
            maxFreq = uphrase->userfreq;
        uphrase = UserGetPhraseNext(pgdata, phoneSeq);
    }

    return maxFreq;
}

/* compute the new updated freqency */
static int UpdateFreq(int freq, int maxfreq, int origfreq, int deltatime)
{
    int delta;

    /* Short interval */
    if (deltatime < 4000) {
        delta = (freq >= maxfreq) ?
            min((maxfreq - origfreq) / 5 + 1,
                SHORT_INCREASE_FREQ) : max((maxfreq - origfreq) / 5 + 1, SHORT_INCREASE_FREQ);
        return min(freq + delta, MAX_ALLOW_FREQ);
    }
    /* Medium interval */
    else if (deltatime < 50000) {
        delta = (freq >= maxfreq) ?
            min((maxfreq - origfreq) / 10 + 1,
                MEDIUM_INCREASE_FREQ) : max((maxfreq - origfreq) / 10 + 1, MEDIUM_INCREASE_FREQ);
        return min(freq + delta, MAX_ALLOW_FREQ);
    }
    /* long interval */
    else {
        delta = max((freq - origfreq) / 5, LONG_DECREASE_FREQ);
        return max(freq - delta, origfreq);
    }
}

static void LogUserPhrase(ChewingData *pgdata,
                          const uint16_t phoneSeq[],
                          const char wordSeq[], int orig_freq, int max_freq, int user_freq, int recent_time)
{
    /* Size of each phone is len("0x1234 ") = 7 */
    char buf[7 * MAX_PHRASE_LEN + 1] = { 0 };
    int i;

    for (i = 0; i < MAX_PHRASE_LEN; ++i) {
        if (phoneSeq[i] == 0)
            break;
        snprintf(buf + 7 * i, 7 + 1, "%#06x ", phoneSeq[i]);
    }

    LOG_INFO("userphrase %s, phone = %s, orig_freq = %d, max_freq = %d, user_freq = %d, recent_time = %d",
             wordSeq, buf, orig_freq, max_freq, user_freq, recent_time);
}

void UserUpdatePhraseBegin(ChewingData *pgdata)
{
    /* compatibile with sqlite userphrase */
}

int UserUpdatePhrase(ChewingData *pgdata, const uint16_t phoneSeq[], const char wordSeq[])
{
    HASH_ITEM *pItem;
    UserPhraseData data;
    int len;

    len = ueStrLen(wordSeq);
    if (len > MAX_PHRASE_LEN)
        return USER_UPDATE_FAIL;

    pItem = HashFindEntry(pgdata, phoneSeq, wordSeq);
    if (!pItem) {
        if (!AlcUserPhraseSeq(&data, len, strlen(wordSeq))) {
            return USER_UPDATE_FAIL;
        }

        memcpy(data.phoneSeq, phoneSeq, len * sizeof(phoneSeq[0]));
        data.phoneSeq[len] = 0;
        strcpy(data.wordSeq, wordSeq);

        /* load initial freq */
        data.origfreq = LoadOriginalFreq(pgdata, phoneSeq, wordSeq, len);
        data.maxfreq = LoadMaxFreq(pgdata, phoneSeq, len);

        data.userfreq = data.origfreq;
        data.recentTime = pgdata->static_data.chewing_lifetime;
        pItem = HashInsert(pgdata, &data);
        DestroyUserPhraseData(&data);
        LogUserPhrase(pgdata, phoneSeq, wordSeq, pItem->data.origfreq, pItem->data.maxfreq, pItem->data.userfreq,
                      pItem->data.recentTime);
        HashModify(pgdata, pItem);
        return USER_UPDATE_INSERT;
    } else {
        pItem->data.maxfreq = LoadMaxFreq(pgdata, phoneSeq, len);
        pItem->data.userfreq = UpdateFreq(pItem->data.userfreq,
                                          pItem->data.maxfreq,
                                          pItem->data.origfreq,
                                          pgdata->static_data.chewing_lifetime - pItem->data.recentTime);
        pItem->data.recentTime = pgdata->static_data.chewing_lifetime;
        LogUserPhrase(pgdata, phoneSeq, wordSeq, pItem->data.origfreq, pItem->data.maxfreq, pItem->data.userfreq,
                      pItem->data.recentTime);
        HashModify(pgdata, pItem);
        return USER_UPDATE_MODIFY;
    }
}

void UserUpdatePhraseEnd(ChewingData *pgdata)
{
    /* compatibile with sqlite userphrase */
}

int UserRemovePhrase(ChewingData *pgdata, const uint16_t phoneSeq[], const char wordSeq[])
{
    HASH_ITEM **prev = NULL;
    HASH_ITEM *item = NULL;

    assert(pgdata);
    assert(phoneSeq);
    assert(wordSeq);

    prev = HashFindHead(pgdata, phoneSeq);
    item = *prev;

    while (item) {
        if (strcmp(item->data.wordSeq, wordSeq) == 0) {
            /* Remove this phrase by removing */
            item->data.phoneSeq[0] = 0;
            HashModify(pgdata, item);

            *prev = item->next;
            item->next = NULL;
            FreeHashItem(item);

            return 1;
        }
        prev = &item->next;
        item = item->next;
    }

    return 0;
}

UserPhraseData *UserGetPhraseFirst(ChewingData *pgdata, const uint16_t phoneSeq[])
{
    pgdata->prev_userphrase = HashFindPhonePhrase(pgdata, phoneSeq, NULL);
    if (!pgdata->prev_userphrase)
        return NULL;
    return &(pgdata->prev_userphrase->data);
}

UserPhraseData *UserGetPhraseNext(ChewingData *pgdata, const uint16_t phoneSeq[])
{
    pgdata->prev_userphrase = HashFindPhonePhrase(pgdata, phoneSeq, pgdata->prev_userphrase);
    if (!pgdata->prev_userphrase)
        return NULL;
    return &(pgdata->prev_userphrase->data);
}

void UserGetPhraseEnd(ChewingData *pgdata, const uint16_t phoneSeq[])
{
    /* FIXME: Remove this */
}

void IncreaseLifeTime(ChewingData *pgdata)
{
    assert(pgdata);

    ++pgdata->static_data.chewing_lifetime;
}
