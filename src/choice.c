/**
 * choice.c
 *
 * Copyright (c) 1999, 2000, 2001
 *      Lu-chuan Kung and Kang-pen Chen.
 *      All rights reserved.
 *
 * Copyright (c) 2004-2008, 2010-2014
 *      libchewing Core Team. See ChangeLog for details.
 *
 * See the file "COPYING" for information on usage and redistribution
 * of this file.
 */

/**
 * @file choice.c
 * @brief Choice module
 */

#include <string.h>
#include <assert.h>

#include "chewing-utf8-util.h"
#include "global.h"
#include "dict-private.h"
#include "chewingutil.h"
#include "tree-private.h"
#include "userphrase-private.h"
#include "choice-private.h"
#include "bopomofo-private.h"
#include "private.h"

static void ChangeSelectIntervalAndBreakpoint(ChewingData *pgdata, int from, int to, const char *str)
{
    int i;
    int user_alloc;

    IntervalType inte;

    inte.from = from;
    inte.to = to;
    for (i = 0; i < pgdata->nSelect; i++) {
        if (IsIntersect(inte, pgdata->selectInterval[i])) {
            RemoveSelectElement(i, pgdata);
            i--;
        }
    }

    pgdata->selectInterval[pgdata->nSelect].from = from;
    pgdata->selectInterval[pgdata->nSelect].to = to;

    /* No available selection */
    if ((user_alloc = (to - from)) == 0)
        return;

    ueStrNCpy(pgdata->selectStr[pgdata->nSelect], str, user_alloc, 1);
    pgdata->nSelect++;

    if (user_alloc > 1) {
        memset(&pgdata->bUserArrBrkpt[from + 1], 0, sizeof(int) * (user_alloc - 1));
        memset(&pgdata->bUserArrCnnct[from + 1], 0, sizeof(int) * (user_alloc - 1));
    }
}

/** @brief Loading all possible phrases after the cursor from long to short into AvailInfo structure.*/
static void SetAvailInfo(ChewingData *pgdata, int begin, int end)
{
    AvailInfo *pai = &(pgdata->availInfo);
    const uint16_t *phoneSeq = pgdata->phoneSeq;
    int nPhoneSeq = pgdata->nPhoneSeq;
    const int *bSymbolArrBrkpt = pgdata->bSymbolArrBrkpt;
    int symbolArrBrkpt[ARRAY_SIZE(pgdata->bSymbolArrBrkpt)] = { 0 };

    const TreeType *tree_pos;
    int diff;
    uint16_t userPhoneSeq[MAX_PHONE_SEQ_LEN];

    int i, head, head_tmp;
    int tail, tail_tmp;
    int pos;

    head = tail = 0;

    pai->nAvail = 0;

    /*
     * XXX: The phoneSeq, nPhoneSeq skip any symbol in preedit buffer,
     * while bSymbolArrBrkpt, does not skip any symbol in preedit
     * buffer. So we need to do some translate here.
     */
    for (i = 0; i < pgdata->chiSymbolBufLen; ++i) {
        if (bSymbolArrBrkpt[i]) {
            /*
             * XXX: If preedit buffer starts with symbol, the pos
             * will become negative. In this case, we just ignore
             * this symbol because it does not create any break
             * point.
             */
            pos = i - CountSymbols(pgdata, i + 1);
            if (pos >= 0)
                symbolArrBrkpt[pos] = 1;
        }
    }

    if (pgdata->config.bPhraseChoiceRearward) {
        for (i = end; i >= begin; i--) {
            if (symbolArrBrkpt[i])
                break;
            head = i;
        }
        head_tmp = end;
    } else {
        head_tmp = head = begin;
    }

    if (pgdata->config.bPhraseChoiceRearward) {
        tail_tmp = tail = end;
    } else {
        for (i = begin; i < nPhoneSeq; i++) {
            tail = i;
            if (symbolArrBrkpt[i])
                break;
        }
        tail_tmp = begin;
    }

    while (head <= head_tmp && tail_tmp <= tail) {
        diff = tail_tmp - head_tmp;
        tree_pos = TreeFindPhrase(pgdata, head_tmp, tail_tmp, phoneSeq);

        if (tree_pos) {
            /* save it! */
            pai->avail[pai->nAvail].len = diff + 1;
            pai->avail[pai->nAvail].id = tree_pos;
            pai->nAvail++;
        } else {
            memcpy(userPhoneSeq, &phoneSeq[head_tmp], sizeof(uint16_t) * (diff + 1));
            userPhoneSeq[diff + 1] = 0;
            if (UserGetPhraseFirst(pgdata, userPhoneSeq)) {
                /* save it! */
                pai->avail[pai->nAvail].len = diff + 1;
                pai->avail[pai->nAvail].id = NULL;
                pai->nAvail++;
            } else {
                pai->avail[pai->nAvail].len = 0;
                pai->avail[pai->nAvail].id = NULL;
            }
            UserGetPhraseEnd(pgdata, userPhoneSeq);
        }

        if (pgdata->config.bPhraseChoiceRearward) {
            head_tmp--;
        } else {
            tail_tmp++;
        }
    }
}

/* FIXME: Improper use of len parameter */
static int ChoiceTheSame(ChoiceInfo *pci, const char *str, int len)
{
    int i;

    for (i = 0; i < pci->nTotalChoice; i++)
        if (!strncmp(pci->totalChoiceStr[i], str, len))
            return 1;
    return 0;
}

static void ChoiceInfoAppendChi(ChewingData *pgdata, ChoiceInfo *pci, uint16_t phone)
{
    Phrase tempWord;
    int len;

    if (GetCharFirst(pgdata, &tempWord, phone)) {
        do {
            len = ueBytesFromChar(tempWord.phrase[0]);
            if (ChoiceTheSame(pci, tempWord.phrase, len))
                continue;
            assert(pci->nTotalChoice < MAX_CHOICE);
            memcpy(pci->totalChoiceStr[pci->nTotalChoice], tempWord.phrase, len);
            pci->totalChoiceStr[pci->nTotalChoice]
                [len] = '\0';
            pci->nTotalChoice++;
        } while (GetVocabNext(pgdata, &tempWord));
    }
}

/** @brief Loading all possible phrases of certain length.
 *
 * Loading all possible phrases of certain length into ChoiceInfo structure
 * from static and dynamic dictionaries, including number of total pages and
 * the number of current page.
 */
static void SetChoiceInfo(ChewingData *pgdata)
{
    Phrase tempPhrase;
    int len;
    UserPhraseData *pUserPhraseData;
    uint16_t userPhoneSeq[MAX_PHONE_SEQ_LEN];

    ChoiceInfo *pci = &(pgdata->choiceInfo);
    AvailInfo *pai = &(pgdata->availInfo);
    uint16_t *phoneSeq = pgdata->phoneSeq;
    uint16_t *phoneSeqAlt = pgdata->phoneSeqAlt;
    int cursor = PhoneSeqCursor(pgdata);
    int candPerPage = pgdata->config.candPerPage;

    /* Clears previous candidates. */
    memset(pci->totalChoiceStr, '\0', MAX_CHOICE * MAX_PHRASE_LEN * MAX_UTF8_SIZE + 1);

    pci->nTotalChoice = 0;
    len = pai->avail[pai->currentAvail].len;
    assert(len);

    /* secondly, read tree phrase */
    if (len == 1) {             /* single character */
        ChoiceInfoAppendChi(pgdata, pci, phoneSeq[cursor]);

        if (phoneSeq[cursor] != phoneSeqAlt[cursor]) {
            ChoiceInfoAppendChi(pgdata, pci, phoneSeqAlt[cursor]);
        }

        if (pgdata->bopomofoData.kbtype == KB_HSU || pgdata->bopomofoData.kbtype == KB_DVORAK_HSU) {
            switch (phoneSeq[cursor]) {
            case 0x2800:       /* 'ㄘ' */
                ChoiceInfoAppendChi(pgdata, pci, 0x30);     /* 'ㄟ' */
                break;
            case 0x80:         /* 'ㄧ' */
                ChoiceInfoAppendChi(pgdata, pci, 0x20);     /* 'ㄝ' */
                break;
            case 0x2A00:       /* 'ㄙ' */
                ChoiceInfoAppendChi(pgdata, pci, 0x1);      /* '˙' */
                break;
            case 0xA00:        /* 'ㄉ' */
                ChoiceInfoAppendChi(pgdata, pci, 0x2);      /* 'ˊ' */
                break;
            case 0x800:        /* 'ㄈ' */
                ChoiceInfoAppendChi(pgdata, pci, 0x3);      /* 'ˇ' */
                break;
            case 0x18:         /* 'ㄜ' */
                ChoiceInfoAppendChi(pgdata, pci, 0x1200);   /* 'ㄍ' */
                break;
            case 0x10:         /* 'ㄛ' */
                ChoiceInfoAppendChi(pgdata, pci, 0x1600);   /* 'ㄏ' */
                break;
            case 0x1E00:       /* 'ㄓ' */
                ChoiceInfoAppendChi(pgdata, pci, 0x1800);   /* 'ㄐ' */
                ChoiceInfoAppendChi(pgdata, pci, 0x4);      /* 'ˋ' */
                break;
            case 0x58:         /* 'ㄤ' */
                ChoiceInfoAppendChi(pgdata, pci, 0x1400);   /* 'ㄎ' */
                break;
            case 0x68:         /* 'ㄦ' */
                ChoiceInfoAppendChi(pgdata, pci, 0x1000);   /* 'ㄌ' */
                ChoiceInfoAppendChi(pgdata, pci, 0x60);     /* 'ㄥ' */
                break;
            case 0x2200:       /* 'ㄕ' */
                ChoiceInfoAppendChi(pgdata, pci, 0x1C00);   /* 'ㄒ' */
                break;
            case 0x2000:       /* 'ㄔ' */
                ChoiceInfoAppendChi(pgdata, pci, 0x1A00);   /* 'ㄑ' */
                break;
            case 0x50:         /* 'ㄣ' */
                ChoiceInfoAppendChi(pgdata, pci, 0xE00);    /* 'ㄋ' */
                break;
            case 0x48:         /* 'ㄢ' */
                ChoiceInfoAppendChi(pgdata, pci, 0x600);    /* 'ㄇ' */
                break;
            default:
                break;
            }
        }

        if (pgdata->bopomofoData.kbtype == KB_ET26) {
            switch (phoneSeq[cursor]) {
            case 0x40:      /* 'ㄡ' */
                ChoiceInfoAppendChi(pgdata, pci, 0x400);    /* 'ㄆ' */
                break;
            case 0x58:      /* 'ㄤ' */
                ChoiceInfoAppendChi(pgdata, pci, 0xC00);    /* 'ㄊ' */
                break;
            case 0x2800:    /* 'ㄘ' */
                ChoiceInfoAppendChi(pgdata, pci, 0x20);     /* 'ㄝ' */
                break;
            case 0x2600:    /* 'ㄗ' */
                ChoiceInfoAppendChi(pgdata, pci, 0x30);     /* 'ㄟ' */
                break;
            case 0x1E00:    /* 'ㄓ' */
                ChoiceInfoAppendChi(pgdata, pci, 0x1800);   /* 'ㄐ' */
                break;
            case 0x68:      /* 'ㄦ' */
                ChoiceInfoAppendChi(pgdata, pci, 0x1600);   /* 'ㄏ' */
                break;
            case 0x60:      /* 'ㄥ' */
                ChoiceInfoAppendChi(pgdata, pci, 0x1000);   /* 'ㄌ' */
                break;
            case 0x2200:    /* 'ㄕ' */
                ChoiceInfoAppendChi(pgdata, pci, 0x1C00);   /* 'ㄒ' */
                break;
            case 0x1200:    /* 'ㄍ' */
                ChoiceInfoAppendChi(pgdata, pci, 0x1A00);   /* 'ㄑ' */
                break;
            case 0x50:      /* 'ㄣ' */
                ChoiceInfoAppendChi(pgdata, pci, 0xE00);    /* 'ㄋ' */
                break;
            case 0x48:      /* 'ㄢ' */
                ChoiceInfoAppendChi(pgdata, pci, 0x600);    /* 'ㄇ' */
                break;
            case 0xA00:     /* 'ㄉ' */
                ChoiceInfoAppendChi(pgdata, pci, 0x1);      /* '˙' */
                break;
            case 0x800:     /* 'ㄈ' */
                ChoiceInfoAppendChi(pgdata, pci, 0x2);      /* 'ˊ' */
                break;
            case 0x2400:    /* 'ㄖ' */
                ChoiceInfoAppendChi(pgdata, pci, 0x3);      /* 'ˇ' */
                break;
            case 0x1400:    /* 'ㄎ' */
                ChoiceInfoAppendChi(pgdata, pci, 0x4);      /* 'ˋ' */
                break;
            default:
                break;
            }
        }
    }
    /* phrase */
    else {
        if (pai->avail[pai->currentAvail].id) {
            GetPhraseFirst(pgdata, &tempPhrase, pai->avail[pai->currentAvail].id);
            do {
                if (ChoiceTheSame(pci, tempPhrase.phrase, len * ueBytesFromChar(tempPhrase.phrase[0]))) {
                    continue;
                }
                ueStrNCpy(pci->totalChoiceStr[pci->nTotalChoice], tempPhrase.phrase, len, 1);
                pci->nTotalChoice++;
            } while (GetVocabNext(pgdata, &tempPhrase));
        }

        memcpy(userPhoneSeq, &phoneSeq[cursor], sizeof(uint16_t) * len);
        userPhoneSeq[len] = 0;
        pUserPhraseData = UserGetPhraseFirst(pgdata, userPhoneSeq);
        if (pUserPhraseData) {
            do {
                /* check if the phrase is already in the choice list */
                if (ChoiceTheSame(pci, pUserPhraseData->wordSeq, len * ueBytesFromChar(pUserPhraseData->wordSeq[0])))
                    continue;
                /* otherwise store it */
                ueStrNCpy(pci->totalChoiceStr[pci->nTotalChoice], pUserPhraseData->wordSeq, len, 1);
                pci->nTotalChoice++;
            } while ((pUserPhraseData = UserGetPhraseNext(pgdata, userPhoneSeq)) != NULL);
        }
        UserGetPhraseEnd(pgdata, userPhoneSeq);

    }

    /* magic number */
    pci->nChoicePerPage = candPerPage;
    assert(pci->nTotalChoice > 0);
    pci->nPage = CEIL_DIV(pci->nTotalChoice, pci->nChoicePerPage);
    pci->pageNo = 0;
    pci->isSymbol = WORD_CHOICE;
}

/*
 * Seek the start of the phrase (English characters are skipped.)
 */
static int SeekPhraseHead(ChewingData *pgdata)
{
    int i;
    int phoneSeq = PhoneSeqCursor(pgdata);

    for (i = pgdata->nPrefer - 1; i >= 0; i--) {
        if (pgdata->preferInterval[i].from > phoneSeq || pgdata->preferInterval[i].to < phoneSeq)
            continue;
        return pgdata->preferInterval[i].from;
    }
    return 0;
}

/** @brief Enter choice mode and relating initialisations. */
int ChoiceInitAvail(ChewingData *pgdata)
{
    int end, begin;

    /* save old cursor position */
    pgdata->choiceInfo.oldChiSymbolCursor = pgdata->chiSymbolCursor;

    /* see if there is some word in the cursor position */
    if (pgdata->chiSymbolBufLen == pgdata->chiSymbolCursor) {
        pgdata->chiSymbolCursor--;
    }

    end = PhoneSeqCursor(pgdata);

    if (pgdata->config.bPhraseChoiceRearward) {
        pgdata->chiSymbolCursor = SeekPhraseHead(pgdata) + CountSymbols(pgdata, pgdata->chiSymbolCursor);
    }
    begin = PhoneSeqCursor(pgdata);

    pgdata->bSelect = 1;

    SetAvailInfo(pgdata, begin, end);

    if (!pgdata->availInfo.nAvail)
        return ChoiceEndChoice(pgdata);

    pgdata->availInfo.currentAvail = pgdata->availInfo.nAvail - 1;
    SetChoiceInfo(pgdata);
    return 0;
}

static void ChangeCurrentAvailInfo(ChewingData *pgdata, int current)
{
    assert(pgdata);

    if (pgdata->config.bPhraseChoiceRearward) {
        pgdata->chiSymbolCursor = pgdata->choiceInfo.oldChiSymbolCursor - pgdata->availInfo.avail[current].len + 1;

        /*
         * When oldChiSymbolCursor == chiSymbolBufLen, the old cursor is at the
         * end of buffer. This mean the end of phrase is in (oldChiSymbolCursor
         * - 1) instead of oldChiSymbolCursor. In this case, we need to adjust
         * chiSymbolCursor.
         */
        assert(pgdata->choiceInfo.oldChiSymbolCursor <= pgdata->chiSymbolBufLen);
        if (pgdata->choiceInfo.oldChiSymbolCursor == pgdata->chiSymbolBufLen) {
            --pgdata->chiSymbolCursor;
        }
    }

    pgdata->availInfo.currentAvail = current;
}

int ChoiceFirstAvail(ChewingData *pgdata)
{
    assert(pgdata);
    assert(pgdata->bSelect);

    if (pgdata->choiceInfo.isSymbol != WORD_CHOICE)
        return 0;

    ChangeCurrentAvailInfo(pgdata, pgdata->availInfo.nAvail - 1);

    SetChoiceInfo(pgdata);

    return 0;
}

int ChoiceLastAvail(ChewingData *pgdata)
{
    assert(pgdata);
    assert(pgdata->bSelect);

    if (pgdata->choiceInfo.isSymbol != WORD_CHOICE)
        return 0;

    ChangeCurrentAvailInfo(pgdata, 0);

    SetChoiceInfo(pgdata);

    return 0;
}

int ChoiceHasNextAvail(ChewingData *pgdata)
{
    assert(pgdata);
    assert(pgdata->bSelect);

    if (pgdata->choiceInfo.isSymbol != WORD_CHOICE)
        return 0;

    return pgdata->availInfo.currentAvail > 0;
}


int ChoiceHasPrevAvail(ChewingData *pgdata)
{
    assert(pgdata);
    assert(pgdata->bSelect);

    if (pgdata->choiceInfo.isSymbol != WORD_CHOICE)
        return 0;

    return pgdata->availInfo.currentAvail < pgdata->availInfo.nAvail - 1;
}

int ChoicePrevAvail(ChewingData *pgdata)
{
    assert(pgdata);
    assert(pgdata->bSelect);

    if (!ChoiceHasPrevAvail(pgdata)) {
        return -1;
    }

    ChangeCurrentAvailInfo(pgdata, pgdata->availInfo.currentAvail + 1);

    SetChoiceInfo(pgdata);

    return 0;
}

int ChoiceNextAvail(ChewingData *pgdata)
{
    assert(pgdata);
    assert(pgdata->bSelect);

    if (!ChoiceHasNextAvail(pgdata)) {
        return -1;
    }

    ChangeCurrentAvailInfo(pgdata, pgdata->availInfo.currentAvail - 1);

    SetChoiceInfo(pgdata);

    return 0;
}

int ChoiceEndChoice(ChewingData *pgdata)
{
    pgdata->bSelect = 0;
    pgdata->choiceInfo.nTotalChoice = 0;
    pgdata->choiceInfo.nPage = 0;

    pgdata->chiSymbolCursor = pgdata->choiceInfo.oldChiSymbolCursor;
    assert(pgdata->chiSymbolCursor <= pgdata->chiSymbolBufLen);

    pgdata->choiceInfo.isSymbol = WORD_CHOICE;
    return 0;
}

/** @brief commit the selected phrase. */
int ChoiceSelect(ChewingData *pgdata, int selectNo)
{
    ChoiceInfo *pci = &(pgdata->choiceInfo);
    AvailInfo *pai = &(pgdata->availInfo);

    ChangeSelectIntervalAndBreakpoint(pgdata,
                                      PhoneSeqCursor(pgdata),
                                      PhoneSeqCursor(pgdata) + pai->avail[pai->currentAvail].len,
                                      pci->totalChoiceStr[selectNo]);
    ChoiceEndChoice(pgdata);
    return 0;
}
