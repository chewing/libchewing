/**
 * chewingutil.c
 *
 * Copyright (c) 1999, 2000, 2001
 *      Lu-chuan Kung and Kang-pen Chen.
 *      All rights reserved.
 *
 * Copyright (c) 2004-2006, 2008, 2010-2014
 *      libchewing Core Team. See ChangeLog for details.
 *
 * See the file "COPYING" for information on usage and redistribution
 * of this file.
 */

/* This file is encoded in UTF-8 */
#ifdef HAVE_CONFIG_H
#    include <config.h>
#endif

#include <ctype.h>
#include <string.h>
#include <assert.h>
#include <stdlib.h>
#include <stdio.h>

#include "chewing-utf8-util.h"
#include "global.h"
#include "global-private.h"
#include "chewingutil.h"
#include "bopomofo-private.h"
#include "choice-private.h"
#include "tree-private.h"
#include "userphrase-private.h"
#include "private.h"

#ifdef HAVE_ASPRINTF
/* asprintf is provided by GNU extensions and *BSD */
#    ifndef _GNU_SOURCE
#        define _GNU_SOURCE
#    endif
#    include <stdio.h>
#else
#    include "plat_path.h"
#endif

extern const char *const zhuin_tab[];
static void MakePreferInterval(ChewingData *pgdata);
static void ShiftInterval(ChewingOutput *pgo, ChewingData *pgdata);
static int ChewingKillSelectIntervalAcross(int cursor, ChewingData *pgdata);

static int FindSymbolKey(const char *symbol);

/* Note: Keep synchronize with `FindEasySymbolIndex`! */
static const char G_EASY_SYMBOL_KEY[EASY_SYMBOL_KEY_TAB_LEN] = {
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9',
    'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J',
    'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T',
    'U', 'V', 'W', 'X', 'Y', 'Z'
};
static const char NO_SYM_KEY = '\t';

/*
 * FindEasySymbolIndex(ch) = char ch's index in G_EASY_SYMBOL_KEY
 * Just return -1 if not found.
 */
static int FindEasySymbolIndex(char ch)
{
        /**
         * '0' => 0, ..., '9' => 9
         * 'A' => 10, 'B' => 11, ... 'Z' => 35
         */
    if (isdigit(ch)) {
        return ch - '0';
    } else if (isupper(ch)) {
        return ch - 'A' + 10;
    } else {
        return -1;
    }
}

void SetUpdatePhraseMsg(ChewingData *pgdata, const char *addWordSeq, int len, int state)
{
    if (state == USER_UPDATE_INSERT) {
        /* 加入： */
        snprintf(pgdata->showMsg, sizeof(pgdata->showMsg), "\xE5\x8A\xA0\xE5\x85\xA5\xEF\xBC\x9A%s", addWordSeq);
    } else {
        /* 已有： */
        snprintf(pgdata->showMsg, sizeof(pgdata->showMsg), "\xE5\xB7\xB2\xE6\x9C\x89\xEF\xBC\x9A%s", addWordSeq);
    }
    pgdata->showMsgLen = AUX_PREFIX_LEN + len;
}

int NoSymbolBetween(ChewingData *pgdata, int begin, int end)
{
    int i;

    for (i = begin; i < end; ++i) {
        if (pgdata->preeditBuf[i].category == CHEWING_SYMBOL) {
            return 0;
        }
    }

    return 1;
}

int ChewingIsEntering(ChewingData *pgdata)
{
    if (pgdata->choiceInfo.isSymbol != WORD_CHOICE)
        return 1;
    return (pgdata->chiSymbolBufLen != 0 || BopomofoIsEntering(&(pgdata->bopomofoData)));
}

int HaninSymbolInput(ChewingData *pgdata)
{
    unsigned int i;

    ChoiceInfo *pci = &(pgdata->choiceInfo);
    AvailInfo *pai = &(pgdata->availInfo);

    /* No available symbol table */
    if (!pgdata->static_data.symbol_table)
        return BOPOMOFO_ABSORB;

    pci->nTotalChoice = 0;
    for (i = 0; i < pgdata->static_data.n_symbol_entry; i++) {
        strcpy(pci->totalChoiceStr[pci->nTotalChoice], pgdata->static_data.symbol_table[i]->category);
        pci->nTotalChoice++;
    }
    pai->avail[0].len = 1;
    pai->avail[0].id = NULL;
    pai->nAvail = 1;
    pai->currentAvail = 0;
    pci->nChoicePerPage = pgdata->config.candPerPage;
    assert(pci->nTotalChoice > 0);
    pci->nPage = CEIL_DIV(pci->nTotalChoice, pci->nChoicePerPage);
    pci->pageNo = 0;
    pci->isSymbol = SYMBOL_CATEGORY_CHOICE;
    return BOPOMOFO_ABSORB;
}

static int _Inner_InternalSpecialSymbol(int key, ChewingData *pgdata, char symkey, const char *const chibuf)
{
    int kbtype;
    PreeditBuf *buf;

    if (key == symkey && NULL != chibuf) {
        assert(pgdata->chiSymbolBufLen >= pgdata->chiSymbolCursor);

        buf = &pgdata->preeditBuf[pgdata->chiSymbolCursor];

        memmove(&pgdata->preeditBuf[pgdata->chiSymbolCursor + 1],
                &pgdata->preeditBuf[pgdata->chiSymbolCursor],
                sizeof(pgdata->preeditBuf[0]) * (pgdata->chiSymbolBufLen - pgdata->chiSymbolCursor));

        strncpy(buf->char_, chibuf, ARRAY_SIZE(buf->char_) - 1);
        buf->category = CHEWING_SYMBOL;

        /* Save Symbol Key */
        memmove(&(pgdata->symbolKeyBuf[pgdata->chiSymbolCursor + 1]),
                &(pgdata->symbolKeyBuf[pgdata->chiSymbolCursor]),
                sizeof(pgdata->symbolKeyBuf[0]) * (pgdata->chiSymbolBufLen - pgdata->chiSymbolCursor));
        pgdata->symbolKeyBuf[pgdata->chiSymbolCursor] = key;
        pgdata->bUserArrCnnct[PhoneSeqCursor(pgdata)] = 0;
        pgdata->chiSymbolCursor++;
        pgdata->chiSymbolBufLen++;
        /* reset Bopomofo data */
        /* Don't forget the kbtype */
        kbtype = pgdata->bopomofoData.kbtype;
        memset(&(pgdata->bopomofoData), 0, sizeof(BopomofoData));
        pgdata->bopomofoData.kbtype = kbtype;
        return 1;
    }
    return 0;
}

static int InternalSpecialSymbol(int key, ChewingData *pgdata,
                                 int nSpecial, const char keybuf[], const char *const chibuf[])
{
    int i, rtn = BOPOMOFO_IGNORE;   /* very strange and difficult to understand */

    for (i = 0; i < nSpecial; i++) {
        if (1 == _Inner_InternalSpecialSymbol(key, pgdata, keybuf[i], chibuf[i])) {
            rtn = BOPOMOFO_ABSORB;
            break;
        }
    }
    return rtn;
}

int SpecialSymbolInput(int key, ChewingData *pgdata)
{
    static const char keybuf[] = {
        '[', ']', '{', '}', '\'', '<', ':', '\"', '>',
        '~', '!', '@', '#', '$', '%', '^', '&', '*',
        '(', ')', '_', '+', '=', '\\', '|', '?',
        ',', '.', ';'
    };

    static const char *const chibuf[] = {
        "\xE3\x80\x8C", "\xE3\x80\x8D", "\xE3\x80\x8E", "\xE3\x80\x8F",
        /* "「", "」", "『", "』" */
        "\xE3\x80\x81", "\xEF\xBC\x8C", "\xEF\xBC\x9A", "\xEF\xBC\x9B",
        /* "、", "，", "：", "；" */
        "\xE3\x80\x82", "\xEF\xBD\x9E", "\xEF\xBC\x81", "\xEF\xBC\xA0",
        /* "。", "～", "！", "＠" */
        "\xEF\xBC\x83", "\xEF\xBC\x84", "\xEF\xBC\x85", "\xEF\xB8\xBF",
        /* "＃", "＄", "％", "︿" */
        "\xEF\xBC\x86", "\xEF\xBC\x8A", "\xEF\xBC\x88", "\xEF\xBC\x89",
        /* "＆", "＊", "（", "）" */
        "\xE2\x80\x94", "\xEF\xBC\x8B", "\xEF\xBC\x9D", "\xEF\xBC\xBC",
        /* "—", "＋", "＝", "＼" */
        "\xEF\xBD\x9C", "\xEF\xBC\x9F", "\xEF\xBC\x8C", "\xE3\x80\x82",
        /* "｜", "？", "，", "。" */
        "\xEF\xBC\x9B"
            /* "；" */
    };
    STATIC_ASSERT(ARRAY_SIZE(keybuf) == ARRAY_SIZE(chibuf));

    return InternalSpecialSymbol(key, pgdata, ARRAY_SIZE(keybuf), keybuf, chibuf);
}

int FullShapeSymbolInput(int key, ChewingData *pgdata)
{
    int rtn;

    static char keybuf[] = {
        '0', '1', '2', '3', '4', '5', '6', '7', '8', '9',
        'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j',
        'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't',
        'u', 'v', 'w', 'x', 'y', 'z', 'A', 'B', 'C', 'D',
        'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N',
        'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X',
        'Y', 'Z', ' ', '\"', '\'', '/', '<', '>', '`', '[',
        ']', '{', '}', '+', '-'
    };
    static const char *chibuf[] = {
        "\xEF\xBC\x90", "\xEF\xBC\x91", "\xEF\xBC\x92", "\xEF\xBC\x93",
        /* "０","１","２","３" */
        "\xEF\xBC\x94", "\xEF\xBC\x95", "\xEF\xBC\x96", "\xEF\xBC\x97",
        /* "４","５","６","７" */
        "\xEF\xBC\x98", "\xEF\xBC\x99", "\xEF\xBD\x81", "\xEF\xBD\x82",
        /* "８","９","ａ","ｂ" */
        "\xEF\xBD\x83", "\xEF\xBD\x84", "\xEF\xBD\x85", "\xEF\xBD\x86",
        /* "ｃ","ｄ","ｅ","ｆ" */
        "\xEF\xBD\x87", "\xEF\xBD\x88", "\xEF\xBD\x89", "\xEF\xBD\x8A",
        /* "ｇ","ｈ","ｉ","ｊ" */
        "\xEF\xBD\x8B", "\xEF\xBD\x8C", "\xEF\xBD\x8D", "\xEF\xBD\x8E",
        /* "ｋ","ｌ","ｍ","ｎ" */
        "\xEF\xBD\x8F", "\xEF\xBD\x90", "\xEF\xBD\x91", "\xEF\xBD\x92",
        /* "ｏ","ｐ","ｑ","ｒ" */
        "\xEF\xBD\x93", "\xEF\xBD\x94", "\xEF\xBD\x95", "\xEF\xBD\x96",
        /* "ｓ","ｔ","ｕ","ｖ" */
        "\xEF\xBD\x97", "\xEF\xBD\x98", "\xEF\xBD\x99", "\xEF\xBD\x9A",
        /* "ｗ","ｘ","ｙ","ｚ" */
        "\xEF\xBC\xA1", "\xEF\xBC\xA2", "\xEF\xBC\xA3", "\xEF\xBC\xA4",
        /* "Ａ","Ｂ","Ｃ","Ｄ" */
        "\xEF\xBC\xA5", "\xEF\xBC\xA6", "\xEF\xBC\xA7", "\xEF\xBC\xA8",
        /* "Ｅ","Ｆ","Ｇ","Ｈ" */
        "\xEF\xBC\xA9", "\xEF\xBC\xAA", "\xEF\xBC\xAB", "\xEF\xBC\xAC",
        /* "Ｉ","Ｊ","Ｋ","Ｌ" */
        "\xEF\xBC\xAD", "\xEF\xBC\xAE", "\xEF\xBC\xAF", "\xEF\xBC\xB0",
        /* "Ｍ","Ｎ","Ｏ","Ｐ" */
        "\xEF\xBC\xB1", "\xEF\xBC\xB2", "\xEF\xBC\xB3", "\xEF\xBC\xB4",
        /* "Ｑ","Ｒ","Ｓ","Ｔ" */
        "\xEF\xBC\xB5", "\xEF\xBC\xB6", "\xEF\xBC\xB7", "\xEF\xBC\xB8",
        /* "Ｕ","Ｖ","Ｗ","Ｘ" */
        "\xEF\xBC\xB9", "\xEF\xBC\xBA", "\xE3\x80\x80", "\xE2\x80\x9D",
        /* "Ｙ","Ｚ","　","”" */
        "\xE2\x80\x99", "\xEF\xBC\x8F", "\xEF\xBC\x9C", "\xEF\xBC\x9E",
        /* "’","／","＜","＞" */
        "\xE2\x80\xB5", "\xE3\x80\x94", "\xE3\x80\x95", "\xEF\xBD\x9B",
        /* "‵","〔""〕","｛" */
        "\xEF\xBD\x9D", "\xEF\xBC\x8B", "\xEF\xBC\x8D"
            /* "｝","＋","－" */
    };
    STATIC_ASSERT(ARRAY_SIZE(keybuf) == ARRAY_SIZE(chibuf));

    rtn = InternalSpecialSymbol(key, pgdata, ARRAY_SIZE(keybuf), keybuf, chibuf);
    if (rtn == BOPOMOFO_IGNORE)
        rtn = SpecialSymbolInput(key, pgdata);
    return (rtn == BOPOMOFO_IGNORE ? SYMBOL_KEY_ERROR : SYMBOL_KEY_OK);
}

int EasySymbolInput(int key, ChewingData *pgdata)
{
    int rtn, loop, _index;
    char wordbuf[8];

    int nSpecial = EASY_SYMBOL_KEY_TAB_LEN;

    _index = FindEasySymbolIndex(key);
    if (-1 != _index) {
        for (loop = 0; loop < pgdata->static_data.g_easy_symbol_num[_index]; ++loop) {
            ueStrNCpy(wordbuf, ueStrSeek(pgdata->static_data.g_easy_symbol_value[_index], loop), 1, 1);
            (void) _Inner_InternalSpecialSymbol(key, pgdata, key, wordbuf);
        }
        return SYMBOL_KEY_OK;
    }

    rtn = InternalSpecialSymbol(key, pgdata, nSpecial,
                                G_EASY_SYMBOL_KEY, (const char **) pgdata->static_data.g_easy_symbol_value);
    if (rtn == BOPOMOFO_IGNORE)
        rtn = SpecialSymbolInput(key, pgdata);
    return (rtn == BOPOMOFO_IGNORE ? SYMBOL_KEY_ERROR : SYMBOL_KEY_OK);
}

int SymbolChoice(ChewingData *pgdata, int sel_i)
{
    int kbtype;
    int i;
    int symbol_type;
    int key;

    if (!pgdata->static_data.symbol_table && pgdata->choiceInfo.isSymbol != SYMBOL_CHOICE_UPDATE)
        return BOPOMOFO_ABSORB;

    if (pgdata->choiceInfo.isSymbol == SYMBOL_CATEGORY_CHOICE && 0 == pgdata->static_data.symbol_table[sel_i]->nSymbols)
        symbol_type = SYMBOL_CHOICE_INSERT;
    else
        symbol_type = pgdata->choiceInfo.isSymbol;

    /* level one, symbol category */
    if (symbol_type == SYMBOL_CATEGORY_CHOICE) {
        ChoiceInfo *pci = &pgdata->choiceInfo;
        AvailInfo *pai = &pgdata->availInfo;

        /* Display all symbols in this category */
        pci->nTotalChoice = 0;
        for (i = 0; i < pgdata->static_data.symbol_table[sel_i]->nSymbols; i++) {
            ueStrNCpy(pci->totalChoiceStr[pci->nTotalChoice],
                      pgdata->static_data.symbol_table[sel_i]->symbols[i], 1, 1);
            pci->nTotalChoice++;
        }
        pai->avail[0].len = 1;
        pai->avail[0].id = NULL;
        pai->nAvail = 1;
        pai->currentAvail = 0;
        pci->nChoicePerPage = pgdata->config.candPerPage;
        assert(pci->nTotalChoice > 0);
        pci->nPage = CEIL_DIV(pci->nTotalChoice, pci->nChoicePerPage);
        pci->pageNo = 0;
        pci->isSymbol = SYMBOL_CHOICE_INSERT;
    } else {                    /* level 2 symbol or OpenSymbolChoice */
        /* TODO: FIXME, this part is buggy! */
        PreeditBuf *buf = &pgdata->preeditBuf[pgdata->chiSymbolCursor];

        if (symbol_type == SYMBOL_CHOICE_INSERT) {
            assert(pgdata->chiSymbolCursor <= pgdata->chiSymbolBufLen);

            if (pgdata->chiSymbolCursor == pgdata->chiSymbolBufLen ||
                    pgdata->symbolKeyBuf[pgdata->chiSymbolCursor] != NO_SYM_KEY) {
                memmove(&pgdata->preeditBuf[pgdata->chiSymbolCursor + 1],
                        &pgdata->preeditBuf[pgdata->chiSymbolCursor],
                        sizeof(pgdata->preeditBuf[0]) * (pgdata->chiSymbolBufLen - pgdata->chiSymbolCursor));
            } else {
                symbol_type = SYMBOL_CHOICE_UPDATE;
            }
        }
        strncpy(buf->char_, pgdata->choiceInfo.totalChoiceStr[sel_i], ARRAY_SIZE(buf->char_) - 1);
        buf->category = CHEWING_SYMBOL;

        /* This is very strange */
        key = FindSymbolKey(pgdata->choiceInfo.totalChoiceStr[sel_i]);
        pgdata->symbolKeyBuf[pgdata->chiSymbolCursor] = key ? key : NO_SYM_KEY;

        pgdata->bUserArrCnnct[PhoneSeqCursor(pgdata)] = 0;
        ChoiceEndChoice(pgdata);
        /* Don't forget the kbtype */
        kbtype = pgdata->bopomofoData.kbtype;
        memset(&(pgdata->bopomofoData), 0, sizeof(BopomofoData));
        pgdata->bopomofoData.kbtype = kbtype;

        if (symbol_type == SYMBOL_CHOICE_INSERT) {
            pgdata->chiSymbolBufLen++;
            pgdata->chiSymbolCursor++;
        }

        pgdata->choiceInfo.isSymbol = WORD_CHOICE;
    }
    return BOPOMOFO_ABSORB;
}

int SymbolInput(int key, ChewingData *pgdata)
{
    if (isprint((char) key) &&  /* other character was ignored */
        (pgdata->chiSymbolBufLen < MAX_PHONE_SEQ_LEN)) {        /* protect the buffer */
        PreeditBuf *buf = &pgdata->preeditBuf[pgdata->chiSymbolCursor];

        assert(pgdata->chiSymbolCursor <= pgdata->chiSymbolBufLen);

        memmove(&pgdata->preeditBuf[pgdata->chiSymbolCursor + 1],
                &pgdata->preeditBuf[pgdata->chiSymbolCursor],
                sizeof(pgdata->preeditBuf[0]) * (pgdata->chiSymbolBufLen - pgdata->chiSymbolCursor));

        buf->char_[0] = (char) key;
        buf->char_[1] = 0;
        buf->category = CHEWING_SYMBOL;

        /* Save Symbol Key */
        memmove(&(pgdata->symbolKeyBuf[pgdata->chiSymbolCursor + 1]),
                &(pgdata->symbolKeyBuf[pgdata->chiSymbolCursor]),
                sizeof(pgdata->symbolKeyBuf[0]) * (pgdata->chiSymbolBufLen - pgdata->chiSymbolCursor));
        pgdata->symbolKeyBuf[pgdata->chiSymbolCursor] = toupper(key);

        pgdata->bUserArrCnnct[PhoneSeqCursor(pgdata)] = 0;
        pgdata->chiSymbolCursor++;
        pgdata->chiSymbolBufLen++;
        return SYMBOL_KEY_OK;
    }
    return SYMBOL_KEY_ERROR;
}

static int CompInterval(const IntervalType * a, const IntervalType * b)
{
    int cmp = a->from - b->from;

    if (cmp)
        return cmp;
    return (a->to - b->to);
}

static int FindIntervalFrom(int from, IntervalType inte[], int nInte)
{
    int i;

    for (i = 0; i < nInte; i++)
        if (inte[i].from == from)
            return i;
    return -1;
}

void WriteChiSymbolToCommitBuf(ChewingData *pgdata, ChewingOutput *pgo, int len)
{
    int i;
    char *pos;

    assert(pgdata);
    assert(pgo);

    pgo->commitBufLen = len;

    pos = pgo->commitBuf;
    for (i = 0; i < pgo->commitBufLen; ++i) {
        assert(pos + MAX_UTF8_SIZE + 1 < pgo->commitBuf + sizeof(pgo->commitBuf));
        strcpy(pos, pgdata->preeditBuf[i].char_);
        pos += strlen(pgdata->preeditBuf[i].char_);
    }
    *pos = 0;
}

static int CountReleaseNum(ChewingData *pgdata)
{
    int remain, i;

    remain = pgdata->config.maxChiSymbolLen - pgdata->chiSymbolBufLen;
    if (remain >= 0)
        return 0;

    qsort(pgdata->preferInterval, pgdata->nPrefer, sizeof(IntervalType), (CompFuncType) CompInterval);

    if (!ChewingIsChiAt(0, pgdata)) {
        for (i = 0; i < pgdata->chiSymbolCursor; ++i) {
            if (ChewingIsChiAt(i, pgdata)) {
                break;
            }
        }
        return i;
    }

    i = FindIntervalFrom(0, pgdata->preferInterval, pgdata->nPrefer);
    if (i >= 0) {
        return (pgdata->preferInterval[i].to - pgdata->preferInterval[i].from);
    }

    return 1;
}

static void KillFromLeft(ChewingData *pgdata, int nKill)
{
    int i;

    for (i = 0; i < nKill; i++)
        ChewingKillChar(pgdata, 0, DECREASE_CURSOR);
}

void CleanAllBuf(ChewingData *pgdata)
{
    /* 1 */
    pgdata->nPhoneSeq = 0;
    memset(pgdata->phoneSeq, 0, sizeof(pgdata->phoneSeq));
    /* 2 */
    pgdata->chiSymbolBufLen = 0;
    memset(pgdata->preeditBuf, 0, sizeof(pgdata->preeditBuf));
    /* 3 */
    memset(pgdata->bUserArrBrkpt, 0, sizeof(pgdata->bUserArrBrkpt));
    /* 4 */
    pgdata->nSelect = 0;
    /* 5 */
    pgdata->chiSymbolCursor = 0;
    /* 6 */
    memset(pgdata->bUserArrCnnct, 0, sizeof(pgdata->bUserArrCnnct));

    pgdata->phrOut.nNumCut = 0;

    memset(pgdata->symbolKeyBuf, 0, sizeof(pgdata->symbolKeyBuf));

    pgdata->nPrefer = 0;
}

int ReleaseChiSymbolBuf(ChewingData *pgdata, ChewingOutput *pgo)
{
    int throwEnd;

    throwEnd = CountReleaseNum(pgdata);

    /*
     * When current buffer size exceeds maxChiSymbolLen,
     * we need to throw some of the characters at the head of the buffer and
     * commit them.
     */
    if (throwEnd) {
        /*
         * count how many chinese words in "chiSymbolBuf[ 0 .. (throwEnd - 1)]"
         * And release from "chiSymbolBuf" && "phoneSeq"
         */
        WriteChiSymbolToCommitBuf(pgdata, pgo, throwEnd);
        KillFromLeft(pgdata, throwEnd);
    }
    return throwEnd;
}

static int ChewingIsBreakPoint(int cursor, ChewingData *pgdata)
{
    static const char *const BREAK_WORD[] = {
        "\xE6\x98\xAF", "\xE7\x9A\x84", "\xE4\xBA\x86", "\xE4\xB8\x8D",
        /* 是              的              了              不 */
        "\xE4\xB9\x9F", "\xE8\x80\x8C", "\xE4\xBD\xA0", "\xE6\x88\x91",
        /* 也              而              你              我 */
        "\xE4\xBB\x96", "\xE8\x88\x87", "\xE5\xAE\x83", "\xE5\xA5\xB9",
        /* 他              與              它              她 */
        "\xE5\x85\xB6", "\xE5\xB0\xB1", "\xE5\x92\x8C", "\xE6\x88\x96",
        /* 其              就              和              或 */
        "\xE5\x80\x91", "\xE6\x80\xA7", "\xE5\x93\xA1", "\xE5\xAD\x90",
        /* 們              性              員              子 */
        "\xE4\xB8\x8A", "\xE4\xB8\x8B", "\xE4\xB8\xAD", "\xE5\x85\xA7",
        /* 上              下              中              內 */
        "\xE5\xA4\x96", "\xE5\x8C\x96", "\xE8\x80\x85", "\xE5\xAE\xB6",
        /* 外              化              者              家 */
        "\xE5\x85\x92", "\xE5\xB9\xB4", "\xE6\x9C\x88", "\xE6\x97\xA5",
        /* 兒              年              月              日 */
        "\xE6\x99\x82", "\xE5\x88\x86", "\xE7\xA7\x92", "\xE8\xA1\x97",
        /* 時              分              秒              街 */
        "\xE8\xB7\xAF", "\xE6\x9D\x91",
        /* 路              村 */
        "\xE5\x9C\xA8",
        /* 在 */
    };
    size_t i;

    if (!ChewingIsChiAt(cursor, pgdata))
        return 1;

    for (i = 0; i < ARRAY_SIZE(BREAK_WORD); ++i)
        if (!strcmp(pgdata->preeditBuf[cursor].char_, BREAK_WORD[i]))
            return 1;

    return 0;
}

void AutoLearnPhrase(ChewingData *pgdata)
{
    uint16_t bufPhoneSeq[MAX_PHONE_SEQ_LEN + 1];
    char bufWordSeq[MAX_PHONE_SEQ_LEN * MAX_UTF8_SIZE + 1] = { 0 };
    char *pos;
    int i;
    int from;
    int fromPreeditBuf;
    int len;
    int prev_pos = 0;
    int pending_pos = 0;

    /*
     * FIXME: pgdata->preferInterval does not consider symbol, so we need to
     * do translate when using APIs that considering symbol.
     */

    UserUpdatePhraseBegin(pgdata);

    for (i = 0; i < pgdata->nPrefer; i++) {
        from = pgdata->preferInterval[i].from;
        len = pgdata->preferInterval[i].to - from;
        fromPreeditBuf = toPreeditBufIndex(pgdata, from);

        LOG_VERBOSE("interval from = %d, fromPreeditBuf = %d, len = %d, pending_pos = %d", from, fromPreeditBuf, len,
                    pending_pos);

        if (pending_pos != 0 && pending_pos < fromPreeditBuf) {
            /*
             * There is a pending phrase in buffer and it is not
             * connected to current phrase. We store it as
             * userphrase here.
             */
            UserUpdatePhrase(pgdata, bufPhoneSeq, bufWordSeq);
            prev_pos = 0;
            pending_pos = 0;
        }

        if (len == 1 && !ChewingIsBreakPoint(fromPreeditBuf, pgdata)) {
            /*
             * There is a length one phrase and it is not a break
             * point. We store it and try to connect to other length
             * one phrase if possible.
             */
            memcpy(bufPhoneSeq + prev_pos, &pgdata->phoneSeq[from], sizeof(uint16_t) * len);
            bufPhoneSeq[prev_pos + len] = (uint16_t) 0;

            pos = ueStrSeek(bufWordSeq, prev_pos);
            copyStringFromPreeditBuf(pgdata, fromPreeditBuf, len, pos, bufWordSeq + sizeof(bufWordSeq) - pos);
            prev_pos += len;
            pending_pos = fromPreeditBuf + len;

        } else {
            if (pending_pos) {
                /*
                 * Clean pending phrase because we cannot join
                 * it with current phrase.
                 */
                UserUpdatePhrase(pgdata, bufPhoneSeq, bufWordSeq);
                prev_pos = 0;
                pending_pos = 0;
            }
            memcpy(bufPhoneSeq, &pgdata->phoneSeq[from], sizeof(uint16_t) * len);
            bufPhoneSeq[len] = (uint16_t) 0;
            copyStringFromPreeditBuf(pgdata, fromPreeditBuf, len, bufWordSeq, sizeof(bufWordSeq));
            UserUpdatePhrase(pgdata, bufPhoneSeq, bufWordSeq);
        }
    }

    if (pending_pos) {
        UserUpdatePhrase(pgdata, bufPhoneSeq, bufWordSeq);
    }

    UserUpdatePhraseEnd(pgdata);
}

int AddChi(uint16_t phone, uint16_t phoneAlt, ChewingData *pgdata)
{
    int i;
    int cursor = PhoneSeqCursor(pgdata);

    /* shift the selectInterval */
    for (i = 0; i < pgdata->nSelect; i++) {
        if (pgdata->selectInterval[i].from >= cursor) {
            pgdata->selectInterval[i].from++;
            pgdata->selectInterval[i].to++;
        }
    }

    /* shift the Brkpt */
    assert(pgdata->nPhoneSeq >= cursor);
    memmove(&(pgdata->bUserArrBrkpt[cursor + 2]),
            &(pgdata->bUserArrBrkpt[cursor + 1]), sizeof(int) * (pgdata->nPhoneSeq - cursor));
    memmove(&(pgdata->bUserArrCnnct[cursor + 2]),
            &(pgdata->bUserArrCnnct[cursor + 1]), sizeof(int) * (pgdata->nPhoneSeq - cursor));

    /* add to phoneSeq */
    memmove(&(pgdata->phoneSeq[cursor + 1]),
            &(pgdata->phoneSeq[cursor]), sizeof(uint16_t) * (pgdata->nPhoneSeq - cursor));
    pgdata->phoneSeq[cursor] = phone;
    memmove(&(pgdata->phoneSeqAlt[cursor + 1]),
            &(pgdata->phoneSeqAlt[cursor]), sizeof(uint16_t) * (pgdata->nPhoneSeq - cursor));
    pgdata->phoneSeqAlt[cursor] = phoneAlt;
    pgdata->nPhoneSeq++;

    /* add to chiSymbolBuf */
    assert(pgdata->chiSymbolBufLen >= pgdata->chiSymbolCursor);
    memmove(&(pgdata->preeditBuf[pgdata->chiSymbolCursor + 1]),
            &(pgdata->preeditBuf[pgdata->chiSymbolCursor]),
            sizeof(pgdata->preeditBuf[0]) * (pgdata->chiSymbolBufLen - pgdata->chiSymbolCursor));
    /* "0" means Chinese word */
    pgdata->preeditBuf[pgdata->chiSymbolCursor].category = CHEWING_CHINESE;
    pgdata->chiSymbolBufLen++;
    pgdata->chiSymbolCursor++;

    return 0;
}

static void ShowChewingData(ChewingData *pgdata)
{
    int i;

    DEBUG_OUT("nPhoneSeq : %d\n" "phoneSeq  : ", pgdata->nPhoneSeq);
    for (i = 0; i < pgdata->nPhoneSeq; i++)
        DEBUG_OUT("%hu ", pgdata->phoneSeq[i]);
    DEBUG_OUT("[cursor : %d]\n"
              "nSelect : %d\n" "selectStr       selectInterval\n", PhoneSeqCursor(pgdata), pgdata->nSelect);
    for (i = 0; i < pgdata->nSelect; i++) {
        DEBUG_OUT("  %14s%4d%4d\n", pgdata->selectStr[i], pgdata->selectInterval[i].from, pgdata->selectInterval[i].to);
    }

    DEBUG_OUT("bUserArrCnnct : ");
    for (i = 0; i <= pgdata->nPhoneSeq; i++)
        DEBUG_OUT("%d ", pgdata->bUserArrCnnct[i]);
    DEBUG_OUT("\n");

    DEBUG_OUT("bUserArrBrkpt : ");
    for (i = 0; i <= pgdata->nPhoneSeq; i++)
        DEBUG_OUT("%d ", pgdata->bUserArrBrkpt[i]);
    DEBUG_OUT("\n");

    DEBUG_OUT("bArrBrkpt     : ");
    for (i = 0; i <= pgdata->nPhoneSeq; i++)
        DEBUG_OUT("%d ", pgdata->bArrBrkpt[i]);
    DEBUG_OUT("\n");

    DEBUG_OUT("bChiSym : %d , bSelect : %d\n", pgdata->bChiSym, pgdata->bSelect);
}

int CallPhrasing(ChewingData *pgdata, int all_phrasing)
{
    /* set "bSymbolArrBrkpt" && "bArrBrkpt" */
    int i, ch_count = 0;

    memcpy(pgdata->bArrBrkpt, pgdata->bUserArrBrkpt, (MAX_PHONE_SEQ_LEN + 1) * sizeof(int));
    memset(pgdata->bSymbolArrBrkpt, 0, (MAX_PHONE_SEQ_LEN + 1) * sizeof(int));

    for (i = 0; i < pgdata->chiSymbolBufLen; i++) {
        if (ChewingIsChiAt(i, pgdata))
            ch_count++;
        else {
            pgdata->bArrBrkpt[ch_count] = 1;
            pgdata->bSymbolArrBrkpt[i] = 1;
        }
    }

    /* kill select interval */
    for (i = 0; i < pgdata->nPhoneSeq; i++) {
        if (pgdata->bArrBrkpt[i]) {
            ChewingKillSelectIntervalAcross(i, pgdata);
        }
    }

    ShowChewingData(pgdata);

    /* then phrasing */
    Phrasing(pgdata, all_phrasing);

    /* and then make prefer interval */
    MakePreferInterval(pgdata);

    return 0;
}


static void Union(int set1, int set2, int parent[])
{
    if (set1 != set2)
        parent[max(set1, set2)] = min(set1, set2);
}

static int SameSet(int set1, int set2, int parent[])
{
    while (parent[set1] != 0) {
        set1 = parent[set1];
    }
    while (parent[set2] != 0) {
        set2 = parent[set2];
    }
    return (set1 == set2);
}

/* make prefer interval from phrOut->dispInterval */
static void MakePreferInterval(ChewingData *pgdata)
{
    int i, j, set_no;
    int belong_set[MAX_PHONE_SEQ_LEN + 1];
    int parent[MAX_PHONE_SEQ_LEN + 1];

    memset(belong_set, 0, sizeof(int) * (MAX_PHONE_SEQ_LEN + 1));
    memset(parent, 0, sizeof(int) * (MAX_PHONE_SEQ_LEN + 1));

    /* for each interval */
    for (i = 0; i < pgdata->phrOut.nDispInterval; i++) {
        for (j = pgdata->phrOut.dispInterval[i].from; j < pgdata->phrOut.dispInterval[i].to; j++) {
            belong_set[j] = i + 1;
        }
    }
    set_no = i + 1;
    for (i = 0; i < pgdata->nPhoneSeq; i++)
        if (belong_set[i] == 0)
            belong_set[i] = set_no++;

    /* for each connect point */
    for (i = 1; i < pgdata->nPhoneSeq; i++) {
        if (pgdata->bUserArrCnnct[i]) {
            Union(belong_set[i - 1], belong_set[i], parent);
        }
    }

    /* generate new intervals */
    pgdata->nPrefer = 0;
    i = 0;
    while (i < pgdata->nPhoneSeq) {
        for (j = i + 1; j < pgdata->nPhoneSeq; j++)
            if (!SameSet(belong_set[i], belong_set[j], parent))
                break;

        pgdata->preferInterval[pgdata->nPrefer].from = i;
        pgdata->preferInterval[pgdata->nPrefer].to = j;
        pgdata->nPrefer++;
        i = j;
    }
}

/* for MakeOutput */
static void ShiftInterval(ChewingOutput *pgo, ChewingData *pgdata)
{
    int i, arrPos[MAX_PHONE_SEQ_LEN], k = 0, from, len;

    for (i = 0; i < pgdata->chiSymbolBufLen; i++) {
        if (ChewingIsChiAt(i, pgdata)) {
            arrPos[k++] = i;
        }
    }
    arrPos[k] = i;

    pgo->nDispInterval = pgdata->nPrefer;
    for (i = 0; i < pgdata->nPrefer; i++) {
        from = pgdata->preferInterval[i].from;
        len = pgdata->preferInterval[i].to - from;
        pgo->dispInterval[i].from = arrPos[from];
        pgo->dispInterval[i].to = arrPos[from] + len;
    }
}

int MakeOutput(ChewingOutput *pgo, ChewingData *pgdata)
{
    int i;
    int inx;
    char *pos;

    /* fill zero to chiSymbolBuf first */
    pgo->preeditBuf[0] = 0;
    pgo->bopomofoBuf[0] = 0;

    pos = pgo->preeditBuf;
    for (i = 0; i < pgdata->chiSymbolBufLen && pos < pgo->preeditBuf + sizeof(pgo->preeditBuf) + MAX_UTF8_SIZE + 1; ++i) {
        strncpy(pos, pgdata->preeditBuf[i].char_, MAX_UTF8_SIZE + 1);
        pos += strlen(pgdata->preeditBuf[i].char_);
    }

    /* fill point */
    pgo->PointStart = pgdata->PointStart;
    pgo->PointEnd = pgdata->PointEnd;

    /* fill other fields */
    pgo->chiSymbolBufLen = pgdata->chiSymbolBufLen;
    pgo->chiSymbolCursor = pgdata->chiSymbolCursor;

    /* fill bopomofoBuf */
    if (pgdata->bopomofoData.kbtype >= KB_HANYU_PINYIN) {
        strcpy(pgo->bopomofoBuf, pgdata->bopomofoData.pinYinData.keySeq);
    } else {
        for (i = 0; i < BOPOMOFO_SIZE; i++) {
            inx = pgdata->bopomofoData.pho_inx[i];
            if (inx != 0) {
                ueStrNCpy(pgo->bopomofoBuf + strlen(pgo->bopomofoBuf),
                          ueConstStrSeek(zhuin_tab[i], inx - 1),
                          1, STRNCPY_CLOSE);
            }
        }
    }

    ShiftInterval(pgo, pgdata);
    memcpy(pgo->dispBrkpt, pgdata->bUserArrBrkpt, sizeof(pgo->dispBrkpt[0]) * (MAX_PHONE_SEQ_LEN + 1));
    pgo->pci = &(pgdata->choiceInfo);
    pgo->bChiSym = pgdata->bChiSym;
    memcpy(pgo->selKey, pgdata->config.selKey, sizeof(pgdata->config.selKey));
    pgdata->bShowMsg = 0;
    return 0;
}

int MakeOutputWithRtn(ChewingOutput *pgo, ChewingData *pgdata, int keystrokeRtn)
{
    pgo->keystrokeRtn = keystrokeRtn;
    return MakeOutput(pgo, pgdata);
}

void MakeOutputAddMsgAndCleanInterval(ChewingOutput *pgo, ChewingData *pgdata)
{
    pgdata->bShowMsg = 1;
    pgo->nDispInterval = 0;
}

int AddSelect(ChewingData *pgdata, int sel_i)
{
    int length, nSelect, cursor;

    /* save the typing time */
    length = pgdata->availInfo.avail[pgdata->availInfo.currentAvail].len;
    nSelect = pgdata->nSelect;

    /* change "selectStr" , "selectInterval" , and "nSelect" of ChewingData */
    ueStrNCpy(pgdata->selectStr[nSelect], pgdata->choiceInfo.totalChoiceStr[sel_i], length, 1);
    cursor = PhoneSeqCursor(pgdata);
    pgdata->selectInterval[nSelect].from = cursor;
    pgdata->selectInterval[nSelect].to = cursor + length;
    pgdata->nSelect++;
    return 0;
}

int CountSelKeyNum(int key, const ChewingData *pgdata)
        /* return value starts from 0.  If less than zero : error key */
{
    int i;

    for (i = 0; i < MAX_SELKEY; i++)
        if (pgdata->config.selKey[i] == key)
            return i;
    return -1;
}

int CountSymbols(ChewingData *pgdata, int to)
{
    int chi;
    int i;

    for (chi = i = 0; i < to; i++) {
        if (ChewingIsChiAt(i, pgdata))
            chi++;
    }
    return to - chi;
}

int PhoneSeqCursor(ChewingData *pgdata)
{
    int cursor = pgdata->chiSymbolCursor - CountSymbols(pgdata, pgdata->chiSymbolCursor);

    return cursor > 0 ? cursor : 0;
}

int ChewingIsChiAt(int chiSymbolCursor, ChewingData *pgdata)
{
    assert(0 <= chiSymbolCursor);
    assert(chiSymbolCursor < ARRAY_SIZE(pgdata->preeditBuf));
    return pgdata->preeditBuf[chiSymbolCursor].category == CHEWING_CHINESE;
}

void RemoveSelectElement(int i, ChewingData *pgdata)
{
    if (--pgdata->nSelect == i)
        return;
    pgdata->selectInterval[i] = pgdata->selectInterval[pgdata->nSelect];
    strcpy(pgdata->selectStr[i], pgdata->selectStr[pgdata->nSelect]);
}

static int ChewingKillSelectIntervalAcross(int cursor, ChewingData *pgdata)
{
    int i;

    for (i = 0; i < pgdata->nSelect; i++) {
        if (pgdata->selectInterval[i].from < cursor && pgdata->selectInterval[i].to > cursor) {
            RemoveSelectElement(i, pgdata);
            i--;
        }
    }
    return 0;
}

static int KillCharInSelectIntervalAndBrkpt(ChewingData *pgdata, int cursorToKill)
{
    int i;

    for (i = 0; i < pgdata->nSelect; i++) {
        if (pgdata->selectInterval[i].from <= cursorToKill && pgdata->selectInterval[i].to > cursorToKill) {
            RemoveSelectElement(i, pgdata);
            i--;                /* the last one was swap to i, we need to recheck i */
        } else if (pgdata->selectInterval[i].from > cursorToKill) {
            pgdata->selectInterval[i].from--;
            pgdata->selectInterval[i].to--;
        }
    }
    assert(pgdata->nPhoneSeq >= cursorToKill);
    memmove(&(pgdata->bUserArrBrkpt[cursorToKill]),
            &(pgdata->bUserArrBrkpt[cursorToKill + 1]), sizeof(int) * (pgdata->nPhoneSeq - cursorToKill));
    memmove(&(pgdata->bUserArrCnnct[cursorToKill]),
            &(pgdata->bUserArrCnnct[cursorToKill + 1]), sizeof(int) * (pgdata->nPhoneSeq - cursorToKill));

    return 0;
}

int ChewingKillChar(ChewingData *pgdata, int chiSymbolCursorToKill, int minus)
{
    int tmp, cursorToKill;

    tmp = pgdata->chiSymbolCursor;
    pgdata->chiSymbolCursor = chiSymbolCursorToKill;
    cursorToKill = PhoneSeqCursor(pgdata);
    pgdata->chiSymbolCursor = tmp;
    if (ChewingIsChiAt(chiSymbolCursorToKill, pgdata)) {
        KillCharInSelectIntervalAndBrkpt(pgdata, cursorToKill);
        assert(pgdata->nPhoneSeq - cursorToKill - 1 >= 0);
        memmove(&(pgdata->phoneSeq[cursorToKill]),
                &(pgdata->phoneSeq[cursorToKill + 1]), (pgdata->nPhoneSeq - cursorToKill - 1) * sizeof(uint16_t));
        pgdata->nPhoneSeq--;
    }
    pgdata->symbolKeyBuf[chiSymbolCursorToKill] = 0;
    assert(pgdata->chiSymbolBufLen - chiSymbolCursorToKill);
    memmove(&pgdata->symbolKeyBuf[chiSymbolCursorToKill],
            &pgdata->symbolKeyBuf[chiSymbolCursorToKill + 1],
            sizeof(pgdata->symbolKeyBuf[0]) * (pgdata->chiSymbolBufLen - chiSymbolCursorToKill));
    memmove(&pgdata->preeditBuf[chiSymbolCursorToKill],
            &pgdata->preeditBuf[chiSymbolCursorToKill + 1],
            sizeof(pgdata->preeditBuf[0]) * (pgdata->chiSymbolBufLen - chiSymbolCursorToKill));
    pgdata->chiSymbolBufLen--;
    pgdata->chiSymbolCursor -= minus;
    if (pgdata->chiSymbolCursor < 0)
        pgdata->chiSymbolCursor = 0;
    return 0;
}

int IsPreferIntervalConnted(int cursor, ChewingData *pgdata)
{
    int i;

    for (i = 0; i < pgdata->nPrefer; i++) {
        if (pgdata->preferInterval[i].from < cursor && pgdata->preferInterval[i].to > cursor)
            return 1;
    }
    return 0;
}

static const char *const symbol_buf[][50] = {
    {"0", "\xC3\xB8", 0},
    /* "ø" */
    {"[", "\xE3\x80\x8C", "\xE3\x80\x8E", "\xE3\x80\x8A", "\xE3\x80\x88",
     "\xE3\x80\x90", "\xE3\x80\x94", 0},
    /* "「", "『", "《", "〈", "【", "〔" */
    {"]", "\xE3\x80\x8D", "\xE3\x80\x8F", "\xE3\x80\x8B", "\xE3\x80\x89",
     "\xE3\x80\x91", "\xE3\x80\x95", 0},
    /* "」", "』", "》", "〉", "】", "〕" */
    {"{", "\xEF\xBD\x9B", 0},
    /* "｛" */
    {"}", "\xEF\xBD\x9D", 0},
    /* "｝" */
    {"<", "\xEF\xBC\x8C", "\xE2\x86\x90", 0},
    /* "，", "←" */
    {">", "\xE3\x80\x82", "\xE2\x86\x92", "\xEF\xBC\x8E", 0},
    /* "。", "→", "．" */
    {"?", "\xEF\xBC\x9F", "\xC2\xBF", 0},
    /* "？", "¿" */
    {"!", "\xEF\xBC\x81", "\xE2\x85\xA0", "\xC2\xA1", 0},
    /* "！", "Ⅰ","¡" */
    {"@", "\xEF\xBC\xA0", "\xE2\x85\xA1", "\xE2\x8A\x95", "\xE2\x8A\x99",
     "\xE3\x8A\xA3", "\xEF\xB9\xAB", 0},
    /* "＠", "Ⅱ", "⊕", "⊙", "㊣", "﹫" */
    {"#", "\xEF\xBC\x83", "\xE2\x85\xA2", "\xEF\xB9\x9F", 0},
    /* "＃", "Ⅲ", "﹟" */
    {"$", "\xEF\xBC\x84", "\xE2\x85\xA3", "\xE2\x82\xAC", "\xEF\xB9\xA9",
     "\xEF\xBF\xA0", "\xE2\x88\xAE", "\xEF\xBF\xA1", "\xEF\xBF\xA5", 0},
    /* "＄", "Ⅳ", "€", "﹩", "￠", "∮","￡", "￥" */
    {"%", "\xEF\xBC\x85", "\xE2\x85\xA4", 0},
    /* "％", "Ⅴ" */
    {"^", "\xEF\xB8\xBF", "\xE2\x85\xA5", "\xEF\xB9\x80", "\xEF\xB8\xBD",
     "\xEF\xB8\xBE", 0},
    /* "︿", "Ⅵ", "﹀", "︽", "︾" */
    {"&", "\xEF\xBC\x86", "\xE2\x85\xA6", "\xEF\xB9\xA0", 0},
    /* "＆", "Ⅶ", "﹠" */
    {"*", "\xEF\xBC\x8A", "\xE2\x85\xA7", "\xC3\x97", "\xE2\x80\xBB",
     "\xE2\x95\xB3", "\xEF\xB9\xA1", "\xE2\x98\xAF", "\xE2\x98\x86",
     "\xE2\x98\x85", 0},
    /* "＊", "Ⅷ", "×", "※", "╳", "﹡", "☯", "☆", "★" */
    {"(", "\xEF\xBC\x88", "\xE2\x85\xA8", 0},
    /* "（", "Ⅸ" */
    {")", "\xEF\xBC\x89", "\xE2\x85\xA9", 0},
    /* "）", "Ⅹ" */
    {"_", "\xE2\x80\x94", "\xEF\xBC\x8D", "\xE2\x80\x95", "\xE2\x80\x93",
     "\xE2\x86\x90", "\xE2\x86\x92", "\xEF\xBC\xBF", "\xEF\xBF\xA3",
     "\xEF\xB9\x8D", "\xEF\xB9\x89", "\xEF\xB9\x8E", "\xEF\xB9\x8A",
     "\xEF\xB9\x8F", "\xef\xb9\x8b", "\xE2\x80\xA6", "\xE2\x80\xA5",
     "\xC2\xAF", 0},
    /* "—", "－", "―", "–"
     * "←", "→", "＿", "￣"
     * "﹍", "﹉", "﹎", "﹊"
     * "﹏", "﹋", "…", "‥"
     * "¯" */
    {"+", "\xEF\xBC\x8B", "\xC2\xB1", "\xEF\xB9\xA2", 0},
    /* "＋", "±", "﹢" */
    {"=", "\xEF\xBC\x9D", "\xE2\x89\x92", "\xE2\x89\xA0", "\xE2\x89\xA1",
     "\xE2\x89\xA6", "\xE2\x89\xA7", "\xEF\xB9\xA6", 0},
    /* "＝", "≒", "≠", "≡", "≦", "≧", "﹦" */
    {"`", "\xE3\x80\x8F", "\xE3\x80\x8E", "\xE2\x80\xB2", "\xE2\x80\xB5", 0},
    /* "』", "『", "′", "‵" */
    {"~", "\xEF\xBD\x9E", 0},
    /* "～" */
    {":", "\xEF\xBC\x9A", "\xEF\xBC\x9B", "\xEF\xB8\xB0", "\xEF\xB9\x95", 0},
    /* "：", "；", "︰", "﹕" */
    {"\"", "\xEF\xBC\x9B", 0},
    /* "；" */
    {"\'", "\xE3\x80\x81", "\xE2\x80\xA6", "\xE2\x80\xA5", 0},
    /* "、", "…", "‥" */
    {"\\", "\xEF\xBC\xBC", "\xE2\x86\x96", "\xE2\x86\x98", "\xEF\xB9\xA8", 0},
    /* "＼", "↖", "↘", "﹨" */
    {"-", "\xE2\x80\x94", "\xEF\xBC\x8D", "\xE2\x80\x95", "\xE2\x80\x93",
     "\xE2\x86\x90", "\xE2\x86\x92", "\xEF\xBC\xBF", "\xEF\xBF\xA3",
     "\xEF\xB9\x8D", "\xEF\xB9\x89", "\xEF\xB9\x8E", "\xEF\xB9\x8A",
     "\xEF\xB9\x8F", "\xef\xb9\x8b", "\xE2\x80\xA6", "\xE2\x80\xA5",
     "\xC2\xAF", 0},
    /* "—", "－", "―", "–"
     * "←", "→", "＿", "￣"
     * "﹍", "﹉", "﹎", "﹊"
     * "﹏", "﹋", "…", "‥"
     * "¯" */
    {"/", "\xEF\xBC\x8F", "\xC3\xB7", "\xE2\x86\x97", "\xE2\x86\x99",
     "\xE2\x88\x95", 0},
    /* "／","÷","↗","↙","∕" */
    {"|", "\xE2\x86\x91", "\xE2\x86\x93", "\xE2\x88\xA3", "\xE2\x88\xA5",
     "\xEF\xB8\xB1", "\xEF\xB8\xB3", "\xEF\xB8\xB4", 0},
    /* "↑", "↓", "∣", "∥", "︱", "︳", "︴" */
    {"A", "\xC3\x85", "\xCE\x91", "\xCE\xB1", "\xE2\x94\x9C", "\xE2\x95\xA0",
     "\xE2\x95\x9F", "\xE2\x95\x9E", 0},
    /* "Å","Α", "α", "├", "╠", "╟", "╞" */
    {"B", "\xCE\x92", "\xCE\xB2", "\xE2\x88\xB5", 0},
    /* "Β", "β","∵" */
    {"C", "\xCE\xA7", "\xCF\x87", "\xE2\x94\x98", "\xE2\x95\xAF",
     "\xE2\x95\x9D", "\xE2\x95\x9C", "\xE2\x95\x9B", "\xE3\x8F\x84",
     "\xE2\x84\x83", "\xE3\x8E\x9D", "\xE2\x99\xA3", "\xC2\xA9", 0},
    /* "Χ", "χ", "┘", "╯", "╝", "╜", "╛"
     * "㏄", "℃", "㎝", "♣", "©" */
    {"D", "\xCE\x94", "\xCE\xB4", "\xE2\x97\x87", "\xE2\x97\x86",
     "\xE2\x94\xA4", "\xE2\x95\xA3", "\xE2\x95\xA2", "\xE2\x95\xA1",
     "\xE2\x99\xA6", 0},
    /* "Δ", "δ", "◇", "◆", "┤", "╣", "╢", "╡","♦" */
    {"E", "\xCE\x95", "\xCE\xB5", "\xE2\x94\x90", "\xE2\x95\xAE",
     "\xE2\x95\x97", "\xE2\x95\x93", "\xE2\x95\x95", 0},
    /* "Ε", "ε", "┐", "╮", "╗", "╓", "╕" */
    {"F", "\xCE\xA6", "\xCF\x88", "\xE2\x94\x82", "\xE2\x95\x91",
     "\xE2\x99\x80", 0},
    /* "Φ", "ψ", "│", "║", "♀" */
    {"G", "\xCE\x93", "\xCE\xB3", 0},
    /* "Γ", "γ" */
    {"H", "\xCE\x97", "\xCE\xB7", "\xE2\x99\xA5", 0},
    /* "Η", "η","♥" */
    {"I", "\xCE\x99", "\xCE\xB9", 0},
    /* "Ι", "ι" */
    {"J", "\xCF\x86", 0},
    /* "φ" */
    {"K", "\xCE\x9A", "\xCE\xBA", "\xE3\x8E\x9E", "\xE3\x8F\x8E", 0},
    /* "Κ", "κ","㎞", "㏎" */
    {"L", "\xCE\x9B", "\xCE\xBB", "\xE3\x8F\x92", "\xE3\x8F\x91", 0},
    /* "Λ", "λ","㏒", "㏑" */
    {"M", "\xCE\x9C", "\xCE\xBC", "\xE2\x99\x82", "\xE2\x84\x93",
     "\xE3\x8E\x8E", "\xE3\x8F\x95", "\xE3\x8E\x9C", "\xE3\x8E\xA1", 0},
    /* "Μ", "μ", "♂", "ℓ", "㎎", "㏕", "㎜","㎡" */
    {"N", "\xCE\x9D", "\xCE\xBD", "\xE2\x84\x96", 0},
    /* "Ν", "ν","№" */
    {"O", "\xCE\x9F", "\xCE\xBF", 0},
    /* "Ο", "ο" */
    {"P", "\xCE\xA0", "\xCF\x80", 0},
    /* "Π", "π" */
    {"Q", "\xCE\x98", "\xCE\xB8", "\xD0\x94", "\xE2\x94\x8C", "\xE2\x95\xAD",
     "\xE2\x95\x94", "\xE2\x95\x93", "\xE2\x95\x92", 0},
    /* "Θ", "θ","Д","┌", "╭", "╔", "╓", "╒" */
    {"R", "\xCE\xA1", "\xCF\x81", "\xE2\x94\x80", "\xE2\x95\x90", "\xC2\xAE", 0},
    /* "Ρ", "ρ", "─", "═" ,"®" */
    {"S", "\xCE\xA3", "\xCF\x83", "\xE2\x88\xB4", "\xE2\x96\xA1",
     "\xE2\x96\xA0", "\xE2\x94\xBC", "\xE2\x95\xAC", "\xE2\x95\xAA",
     "\xE2\x95\xAB", "\xE2\x88\xAB", "\xC2\xA7", "\xE2\x99\xA0", 0},
    /* "Σ", "σ", "∴", "□", "■", "┼", "╬", "╪", "╫"
     * "∫", "§", "♠" */
    {"T", "\xCE\xA4", "\xCF\x84", "\xCE\xB8", "\xE2\x96\xB3", "\xE2\x96\xB2",
     "\xE2\x96\xBD", "\xE2\x96\xBC", "\xE2\x84\xA2", "\xE2\x8A\xBF",
     "\xE2\x84\xA2", 0},
    /* "Τ", "τ","θ","△","▲","▽","▼","™","⊿", "™" */
    {"U", "\xCE\xA5", "\xCF\x85", "\xCE\xBC", "\xE2\x88\xAA", "\xE2\x88\xA9", 0},
    /* "Υ", "υ","μ","∪", "∩" */
    {"V", "\xCE\xBD", 0},
    {"W", "\xE2\x84\xA6", "\xCF\x89", "\xE2\x94\xAC", "\xE2\x95\xA6",
     "\xE2\x95\xA4", "\xE2\x95\xA5", 0},
    /* "Ω", "ω", "┬", "╦", "╤", "╥" */
    {"X", "\xCE\x9E", "\xCE\xBE", "\xE2\x94\xB4", "\xE2\x95\xA9",
     "\xE2\x95\xA7", "\xE2\x95\xA8", 0},
    /* "Ξ", "ξ", "┴", "╩", "╧", "╨" */
    {"Y", "\xCE\xA8", 0},
    /* "Ψ" */
    {"Z", "\xCE\x96", "\xCE\xB6", "\xE2\x94\x94", "\xE2\x95\xB0",
     "\xE2\x95\x9A", "\xE2\x95\x99", "\xE2\x95\x98", 0},
    /* "Ζ", "ζ", "└", "╰", "╚", "╙", "╘" */
};

static int FindSymbolKey(const char *symbol)
{
    unsigned int i;
    const char *const *buf;

    for (i = 0; i < ARRAY_SIZE(symbol_buf); ++i) {
        for (buf = symbol_buf[i]; *buf; ++buf) {
            if (0 == strcmp(*buf, symbol))
                return *symbol_buf[i][0];
        }
    }
    return 0;
}

int OpenSymbolChoice(ChewingData *pgdata)
{
    int i, symbol_buf_len = ARRAY_SIZE(symbol_buf);
    const char *const *pBuf;
    ChoiceInfo *pci = &(pgdata->choiceInfo);

    pci->oldChiSymbolCursor = pgdata->chiSymbolCursor;

    /* see if there is some word in the cursor position */
    if (pgdata->chiSymbolCursor == pgdata->chiSymbolBufLen && pgdata->chiSymbolCursor > 0)
        pgdata->chiSymbolCursor--;
    if (pgdata->symbolKeyBuf[pgdata->chiSymbolCursor] == NO_SYM_KEY) {
        pgdata->bSelect = 1;
        HaninSymbolInput(pgdata);
        return 0;
    }
    for (i = 0; i < symbol_buf_len; i++) {
        if (symbol_buf[i][0][0] == pgdata->symbolKeyBuf[pgdata->chiSymbolCursor]) {
            pBuf = symbol_buf[i];
            break;
        }
    }
    if (i == symbol_buf_len) {
        ChoiceEndChoice(pgdata);
        return 0;
    }
    pci->nTotalChoice = 0;
    for (i = 1; pBuf[i]; i++) {
        ueStrNCpy(pci->totalChoiceStr[pci->nTotalChoice], pBuf[i], ueStrLen(pBuf[i]), 1);
        pci->nTotalChoice++;
    }

    pci->nChoicePerPage = pgdata->config.candPerPage;
    assert(pci->nTotalChoice > 0);
    pci->nPage = CEIL_DIV(pci->nTotalChoice, pci->nChoicePerPage);
    pci->pageNo = 0;
    pci->isSymbol = SYMBOL_CHOICE_UPDATE;

    pgdata->bSelect = 1;
    pgdata->availInfo.nAvail = 1;
    pgdata->availInfo.currentAvail = 0;
    pgdata->availInfo.avail[0].id = NULL;
    pgdata->availInfo.avail[0].len = 1;
    return 0;
}

int InitSymbolTable(ChewingData *pgdata, const char *prefix)
{
    static const unsigned int MAX_SYMBOL_ENTRY = 100;
    static const size_t LINE_LEN = 512; // shall be long enough?

    char *filename = NULL;
    FILE *file = NULL;
    char *line = NULL;
    SymbolEntry **entry = NULL;
    char *category_end;
    const char *symbols;
    char *symbols_end;
    const char *symbol;
    size_t i;
    size_t len;
    size_t size;
    int ret = -1;

    pgdata->static_data.n_symbol_entry = 0;
    pgdata->static_data.symbol_table = NULL;

    ret = asprintf(&filename, "%s" PLAT_SEPARATOR "%s", prefix, SYMBOL_TABLE_FILE);
    if (ret == -1)
        goto error;

    file = fopen(filename, "r");
    if (!file)
        goto error;

    line = ALC(char, LINE_LEN);

    if (!line)
        goto error;

    entry = ALC(SymbolEntry *, MAX_SYMBOL_ENTRY);

    if (!entry)
        goto error;

    while (fgets(line, LINE_LEN, file) && pgdata->static_data.n_symbol_entry < MAX_SYMBOL_ENTRY) {

        category_end = strpbrk(line, "=\r\n");
        if (!category_end)
            goto error;

        symbols = category_end + 1;
        symbols_end = strpbrk(symbols, "\r\n");
        if (symbols_end) {
            *symbols_end = 0;
            len = ueStrLen(symbols);

            entry[pgdata->static_data.n_symbol_entry] =
                (SymbolEntry *) malloc(sizeof(entry[0][0]) + sizeof(entry[0][0].symbols[0]) * len);
            if (!entry[pgdata->static_data.n_symbol_entry])
                goto error;
            entry[pgdata->static_data.n_symbol_entry]
                ->nSymbols = len;

            symbol = symbols;

            for (i = 0; i < len; ++i) {
                ueStrNCpy(entry[pgdata->static_data.n_symbol_entry]->symbols[i], symbol, 1, 1);
                // FIXME: What if symbol is combining sequences.
                symbol += ueBytesFromChar(symbol[0]);
            }


        } else {
            entry[pgdata->static_data.n_symbol_entry] = (SymbolEntry *) malloc(sizeof(entry[0][0]));
            if (!entry[pgdata->static_data.n_symbol_entry])
                goto error;

            entry[pgdata->static_data.n_symbol_entry]
                ->nSymbols = 0;
        }

        *category_end = 0;
        ueStrNCpy(entry[pgdata->static_data.n_symbol_entry]->category, line, MAX_PHRASE_LEN, 1);

        ++pgdata->static_data.n_symbol_entry;
    }

    size = sizeof(*pgdata->static_data.symbol_table) * pgdata->static_data.n_symbol_entry;
    if (!size)
        goto end;
    pgdata->static_data.symbol_table = (SymbolEntry **) malloc(size);
    if (!pgdata->static_data.symbol_table)
        goto error;
    memcpy(pgdata->static_data.symbol_table, entry, size);

    ret = 0;
  end:
    free(entry);
    free(line);
    fclose(file);
    free(filename);
    return ret;

  error:
    for (i = 0; i < pgdata->static_data.n_symbol_entry; ++i) {
        free(entry[i]);
    }
    goto end;
}

void TerminateSymbolTable(ChewingData *pgdata)
{
    unsigned int i;

    if (pgdata->static_data.symbol_table) {
        for (i = 0; i < pgdata->static_data.n_symbol_entry; ++i)
            free(pgdata->static_data.symbol_table[i]);
        free(pgdata->static_data.symbol_table);
        pgdata->static_data.n_symbol_entry = 0;
        pgdata->static_data.symbol_table = NULL;
    }
}

int InitEasySymbolInput(ChewingData *pgdata, const char *prefix)
{
    static const size_t LINE_LEN = 512; // shall be long enough?

    FILE *file = NULL;
    char *filename = NULL;
    char *line = NULL;
    int len;
    int _index;
    char *symbol;
    int ret = -1;

    ret = asprintf(&filename, "%s" PLAT_SEPARATOR "%s", prefix, SOFTKBD_TABLE_FILE);
    if (ret == -1)
        goto filenamefail;

    file = fopen(filename, "r");
    if (!file)
        goto fileopenfail;

    line = ALC(char, LINE_LEN);
    if (!line)
        goto linefail;

    while (fgets(line, LINE_LEN, file)) {
        if (' ' != line[1])
            continue;

        // Remove tailing \n
        len = strcspn(line, "\r\n");

        line[len] = '\0';

        _index = FindEasySymbolIndex(line[0]);
        if (-1 == _index)
            continue;

        len = ueStrLen(&line[2]);
        if (0 == len || len > MAX_PHRASE_LEN)
            continue;

        symbol = ALC(char, strlen(&line[2]) + 1);

        if (!symbol)
            goto end;

        ueStrNCpy(symbol, &line[2], len, 1);

        free(pgdata->static_data.g_easy_symbol_value[_index]);
        pgdata->static_data.g_easy_symbol_value[_index] = symbol;
        pgdata->static_data.g_easy_symbol_num[_index] = len;
    }
    ret = 0;

end:
    free(line);

linefail:
    fclose(file);

fileopenfail:
    free(filename);

filenamefail:
    return ret;
}

void TerminateEasySymbolTable(ChewingData *pgdata)
{
    unsigned int i;

    for (i = 0; i < EASY_SYMBOL_KEY_TAB_LEN; ++i) {
        if (NULL != pgdata->static_data.g_easy_symbol_value[i]) {
            free(pgdata->static_data.g_easy_symbol_value[i]);
            pgdata->static_data.g_easy_symbol_value[i] = NULL;
        }
        pgdata->static_data.g_easy_symbol_num[i] = 0;
    }
}

void copyStringFromPreeditBuf(ChewingData *pgdata, int pos, int len, char *output, int output_len)
{
    int i;
    int x;

    assert(pgdata);
    assert(0 <= pos && (size_t) (pos + len) < ARRAY_SIZE(pgdata->preeditBuf));
    assert(output);
    assert(output_len);

    LOG_VERBOSE("Copy pos %d, len %d from preeditBuf", pos, len);

    for (i = pos; i < pos + len; ++i) {
        x = strlen(pgdata->preeditBuf[i].char_);
        if (x >= output_len)    // overflow
            return;
        memcpy(output, pgdata->preeditBuf[i].char_, x);
        output += x;
        output_len -= x;
    }
    output[0] = 0;
}

/*
 * This function converts phoneSeq index (which does not count symbol) to
 * preeditBuf index (which does count symbol).
 */
int toPreeditBufIndex(ChewingData *pgdata, int pos)
{
    int word_count;
    int i;

    assert(pgdata);
    assert(0 <= pos && pos <= MAX_CHI_SYMBOL_LEN);

    for (i = 0, word_count = 0; i < MAX_CHI_SYMBOL_LEN; ++i) {
        if (ChewingIsChiAt(i, pgdata))
            ++word_count;

        /*
         * pos = 0 means finding the first word, so we need to add one
         * here.
         */
        if (word_count == pos + 1)
            break;
    }

    LOG_VERBOSE("translate phoneSeq index %d to preeditBuf index %d", pos, i);

    return i;
}
