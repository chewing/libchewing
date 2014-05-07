/**
 * chewingio.c
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
 * @file chewingio.c
 * @brief Implement basic I/O routines for Chewing manipulation.
 */
#ifdef HAVE_CONFIG_H
#    include <config.h>
#endif

#include <assert.h>
#include <string.h>
#include <ctype.h>
#include <stdlib.h>
#include <stdio.h>

#include "chewing-utf8-util.h"
#include "global.h"
#include "bopomofo-private.h"
#include "chewingutil.h"
#include "userphrase-private.h"
#include "choice-private.h"
#include "dict-private.h"
#include "tree-private.h"
#include "pinyin-private.h"
#include "private.h"
#include "chewingio.h"
#include "mod_aux.h"
#include "global-private.h"
#include "plat_path.h"
#include "chewing-private.h"
#include "key2pho-private.h"

#if WITH_SQLITE3
#    include "chewing-sql.h"
#else
#    include "hash-private.h"
#endif

const char *const kb_type_str[] = {
    "KB_DEFAULT",
    "KB_HSU",
    "KB_IBM",
    "KB_GIN_YIEH",
    "KB_ET",
    "KB_ET26",
    "KB_DVORAK",
    "KB_DVORAK_HSU",
    "KB_DACHEN_CP26",
    "KB_HANYU_PINYIN",
    "KB_THL_PINYIN",
    "KB_MPS2_PINYIN"
};

const char *const DICT_FILES[] = {
    DICT_FILE,
    PHONE_TREE_FILE,
    NULL,
};

const char *const SYMBOL_TABLE_FILES[] = {
    SYMBOL_TABLE_FILE,
    NULL,
};

const char *const EASY_SYMBOL_FILES[] = {
    SOFTKBD_TABLE_FILE,
    NULL,
};

const char *const PINYIN_FILES[] = {
    PINYIN_TAB_NAME,
    NULL,
};

CHEWING_API int chewing_KBStr2Num(const char str[])
{
    int i;

    STATIC_ASSERT(KB_TYPE_NUM == ARRAY_SIZE(kb_type_str));
    for (i = 0; i < KB_TYPE_NUM; i++) {
        if (!strcmp(str, kb_type_str[i]))
            return i;
    }
    return KB_DEFAULT;
}

static void chooseCandidate(ChewingContext *ctx, int toSelect, int key_buf_cursor)
{
    ChewingData *pgdata = ctx->data;

    if (toSelect) {
        if (!pgdata->bSelect) {
            ChoiceInitAvail(pgdata);
        } else {
            if (ChoiceHasNextAvail(pgdata))
                ChoiceNextAvail(pgdata);
            else                /* rollover */
                ChoiceFirstAvail(pgdata);
        }
    } else if (pgdata->symbolKeyBuf[key_buf_cursor]) {
        /* Open Symbol Choice List */
        if (pgdata->choiceInfo.isSymbol == WORD_CHOICE) {
            OpenSymbolChoice(pgdata);
        }
        /**
         * If these's only one candidate list available, ChoiceFirstAvail
         * will re-open the list, namely turn back to the firt page.
         * However, it doesn't work for symbols, therefore we
         * set the page number to 0 directly.
         */
        else if (pgdata->bSelect) {
            pgdata->choiceInfo.pageNo = 0;
        }
    } else {
        /*
         * The cursor position is not word, nor symbol. The only
         * possible case is that user just uses ` to open symbol
         * selection. In this case, when chooseCandidate is called,
         * libchewing needs to reset pageNo to 0 to do rollover.
         */
        if (pgdata->bSelect) {
            pgdata->choiceInfo.pageNo = 0;
        }
    }
}

static void NullLogger(void *data UNUSED, int level UNUSED, const char *fmt UNUSED, ...)
{
}

static ChewingData *allocate_ChewingData(void (*logger) (void *data, int level, const char *fmt, ...), void *loggerdata)
{
    static const int DEFAULT_SELKEY[] = { '1', '2', '3', '4', '5', '6', '7', '8', '9', '0' };

    ChewingData *data = ALC(ChewingData, 1);

    if (data) {
        data->config.candPerPage = MAX_SELKEY;
        data->config.maxChiSymbolLen = MAX_CHI_SYMBOL_LEN;
        data->logger = logger;
        data->loggerData = loggerdata;
        memcpy(data->config.selKey, DEFAULT_SELKEY, sizeof(data->config.selKey));
    }

    return data;
}

CHEWING_API ChewingContext *chewing_new2(const char *syspath,
                                         const char *userpath,
                                         void (*logger) (void *data, int level, const char *fmt, ...), void *loggerdata)
{
    ChewingContext *ctx;
    ChewingData *pgdata;
    int ret;
    char search_path[PATH_MAX];
    char path[PATH_MAX];
    char *userphrase_path = NULL;

    if (!logger)
        logger = NullLogger;

    ctx = ALC(ChewingContext, 1);

    if (!ctx)
        goto error;

    ctx->output = ALC(ChewingOutput, 1);

    if (!ctx->output)
        goto error;

    pgdata = allocate_ChewingData(logger, loggerdata);
    if (!pgdata)
        goto error;
    ctx->data = pgdata;

    LOG_API("syspath = %d, userpath = %d", syspath, userpath);

    chewing_Reset(ctx);

    if (syspath) {
        strncpy(search_path, syspath, sizeof(search_path));
    } else {
        ret = get_search_path(search_path, sizeof(search_path));
        if (ret) {
            LOG_ERROR("get_search_path returns %d", ret);
            goto error;
        }
    }
    LOG_VERBOSE("search_path is %s", search_path);

    ret = find_path_by_files(search_path, DICT_FILES, path, sizeof(path));
    if (ret) {
        LOG_ERROR("find_path_by_files returns %d", ret);
        goto error;
    }

    ret = InitDict(ctx->data, path);
    if (ret) {
        LOG_ERROR("InitDict returns %d", ret);
        goto error;
    }

    ret = InitTree(ctx->data, path);
    if (ret) {
        LOG_ERROR("InitTree returns %d", ret);
        goto error;
    }

    if (userpath) {
        userphrase_path = strdup(userpath);
    } else {
        userphrase_path = GetDefaultUserPhrasePath(ctx->data);
    }

    if (!userphrase_path) {
        LOG_ERROR("GetUserPhraseStoregePath returns %#p", path);
        goto error;
    }

    ret = InitUserphrase(ctx->data, userphrase_path);
    free(userphrase_path);

    if (ret) {
        LOG_ERROR("InitSql returns %d", ret);
        goto error;
    }

    ctx->cand_no = 0;

    ret = find_path_by_files(search_path, SYMBOL_TABLE_FILES, path, sizeof(path));
    if (ret) {
        LOG_ERROR("find_path_by_files returns %d", ret);
        goto error;
    }

    ret = InitSymbolTable(ctx->data, path);
    if (ret) {
        LOG_ERROR("InitSymbolTable returns %d", ret);
        goto error;
    }

    ret = find_path_by_files(search_path, EASY_SYMBOL_FILES, path, sizeof(path));
    if (ret) {
        LOG_ERROR("find_path_by_files returns %d", ret);
        goto error;
    }

    ret = InitEasySymbolInput(ctx->data, path);
    if (ret) {
        LOG_ERROR("InitEasySymbolInput returns %d", ret);
        goto error;
    }

    ret = find_path_by_files(search_path, PINYIN_FILES, path, sizeof(path));
    if (ret) {
        LOG_ERROR("find_path_by_files returns %d", ret);
        goto error;
    }

    ret = InitPinyin(ctx->data, path);
    if (!ret) {
        LOG_ERROR("InitPinyin returns %d", ret);
        goto error;
    }

    return ctx;
  error:
    chewing_delete(ctx);
    return NULL;
}

CHEWING_API ChewingContext *chewing_new()
{
    return chewing_new2(NULL, NULL, NULL, NULL);
}

CHEWING_API int chewing_Reset(ChewingContext *ctx)
{
    ChewingData *pgdata;
    ChewingStaticData static_data;
    ChewingConfigData old_config;
    void (*logger) (void *data, int level, const char *fmt, ...);
    void *loggerData;

    if (!ctx) {
        return -1;
    }
    pgdata = ctx->data;

    LOG_API("");

    /* Backup old config and restore it after clearing pgdata structure. */
    old_config = pgdata->config;
    static_data = pgdata->static_data;
    logger = pgdata->logger;
    loggerData = pgdata->loggerData;
    memset(pgdata, 0, sizeof(ChewingData));
    pgdata->config = old_config;
    pgdata->static_data = static_data;
    pgdata->logger = logger;
    pgdata->loggerData = loggerData;

    /* bopomofoData */
    memset(&(pgdata->bopomofoData), 0, sizeof(BopomofoData));

    /* choiceInfo */
    memset(&(pgdata->choiceInfo), 0, sizeof(ChoiceInfo));

    pgdata->chiSymbolCursor = 0;
    pgdata->chiSymbolBufLen = 0;
    pgdata->nPhoneSeq = 0;
    memset(pgdata->bUserArrCnnct, 0, sizeof(int) * (MAX_PHONE_SEQ_LEN + 1));
    memset(pgdata->bUserArrBrkpt, 0, sizeof(int) * (MAX_PHONE_SEQ_LEN + 1));
    pgdata->bChiSym = CHINESE_MODE;
    pgdata->bFullShape = HALFSHAPE_MODE;
    pgdata->bSelect = 0;
    pgdata->nSelect = 0;
    pgdata->PointStart = -1;
    pgdata->PointEnd = 0;
    pgdata->phrOut.nNumCut = 0;
    return 0;
}

CHEWING_API int chewing_set_KBType(ChewingContext *ctx, int kbtype)
{
    ChewingData *pgdata;

    if (!ctx) {
        return -1;
    }
    pgdata = ctx->data;

    LOG_API("kbtype = %d", kbtype);

    if (kbtype < KB_TYPE_NUM && kbtype >= 0) {
        ctx->data->bopomofoData.kbtype = kbtype;
        return 0;
    } else {
        ctx->data->bopomofoData.kbtype = KB_DEFAULT;
        return -1;
    }
}

CHEWING_API int chewing_get_KBType(const ChewingContext *ctx)
{
    const ChewingData *pgdata;

    if (!ctx) {
        return -1;
    }
    pgdata = ctx->data;

    LOG_API("kbtype = %d", ctx->data->bopomofoData.kbtype);

    return ctx->data->bopomofoData.kbtype;
}

CHEWING_API char *chewing_get_KBString(const ChewingContext *ctx)
{
    const ChewingData *pgdata;

    if (!ctx) {
        return strdup("");
    }
    pgdata = ctx->data;

    LOG_API("KBString = %s", kb_type_str[ctx->data->bopomofoData.kbtype]);

    return strdup(kb_type_str[ctx->data->bopomofoData.kbtype]);
}

CHEWING_API void chewing_delete(ChewingContext *ctx)
{
    if (ctx) {
        if (ctx->data) {
            TerminatePinyin(ctx->data);
            TerminateEasySymbolTable(ctx->data);
            TerminateSymbolTable(ctx->data);
            TerminateUserphrase(ctx->data);
            TerminateTree(ctx->data);
            TerminateDict(ctx->data);
            free(ctx->data);
        }

        if (ctx->output)
            free(ctx->output);
        free(ctx);
    }
    return;
}

CHEWING_API void chewing_free(void *p)
{
    free(p);
}

CHEWING_API void chewing_set_candPerPage(ChewingContext *ctx, int n)
{
    ChewingData *pgdata;

    if (!ctx) {
        return;
    }
    pgdata = ctx->data;

    LOG_API("n = %d", n);

    if (MIN_SELKEY <= n && n <= MAX_SELKEY)
        ctx->data->config.candPerPage = n;
}

CHEWING_API int chewing_get_candPerPage(const ChewingContext *ctx)
{
    const ChewingData *pgdata;

    if (!ctx) {
        return -1;
    }
    pgdata = ctx->data;

    LOG_API("candPerPage = %d", ctx->data->config.candPerPage);

    return ctx->data->config.candPerPage;
}

CHEWING_API void chewing_set_maxChiSymbolLen(ChewingContext *ctx, int n)
{
    ChewingData *pgdata;

    if (!ctx) {
        return;
    }
    pgdata = ctx->data;

    LOG_API("n = %d", n);

    if (MIN_CHI_SYMBOL_LEN <= n && n <= MAX_CHI_SYMBOL_LEN)
        ctx->data->config.maxChiSymbolLen = n;
}

CHEWING_API int chewing_get_maxChiSymbolLen(const ChewingContext *ctx)
{
    const ChewingData *pgdata;

    if (!ctx) {
        return -1;
    }
    pgdata = ctx->data;

    LOG_API("maxChiSymbolLen = %d", ctx->data->config.maxChiSymbolLen);

    return ctx->data->config.maxChiSymbolLen;
}

CHEWING_API void chewing_set_selKey(ChewingContext *ctx, const int *selkeys, int len)
{
    ChewingData *pgdata;

    if (!ctx) {
        return;
    }
    pgdata = ctx->data;

    LOG_API("");

    if (!selkeys) {
        return;
    }

    if (MIN_SELKEY <= len && len <= MAX_SELKEY) {
        memset(ctx->data->config.selKey, 0, sizeof(ctx->data->config.selKey));
        memcpy(ctx->data->config.selKey, selkeys, sizeof(*selkeys) * len);
    }
}

CHEWING_API int *chewing_get_selKey(const ChewingContext *ctx)
{
    const ChewingData *pgdata;
    int *selkeys;

    if (!ctx) {
        return NULL;
    }
    pgdata = ctx->data;

    LOG_API("");

    selkeys = ALC(int, MAX_SELKEY);
    if (selkeys) {
        memcpy(selkeys, ctx->data->config.selKey, sizeof(*selkeys) * MAX_SELKEY);
    }
    return selkeys;
}

CHEWING_API void chewing_set_addPhraseDirection(ChewingContext *ctx, int direction)
{
    ChewingData *pgdata;

    if (!ctx) {
        return;
    }
    pgdata = ctx->data;

    LOG_API("direction = %d", direction);

    if (direction == 0 || direction == 1)
        ctx->data->config.bAddPhraseForward = direction;
}

CHEWING_API int chewing_get_addPhraseDirection(const ChewingContext *ctx)
{
    const ChewingData *pgdata;

    if (!ctx) {
        return -1;
    }
    pgdata = ctx->data;

    LOG_API("bAddPhraseForward = %d", ctx->data->config.bAddPhraseForward);

    return ctx->data->config.bAddPhraseForward;
}

CHEWING_API void chewing_set_spaceAsSelection(ChewingContext *ctx, int mode)
{
    ChewingData *pgdata;

    if (!ctx) {
        return;
    }
    pgdata = ctx->data;

    LOG_API("mode = %d", mode);

    if (mode == 0 || mode == 1)
        ctx->data->config.bSpaceAsSelection = mode;
}

CHEWING_API int chewing_get_spaceAsSelection(const ChewingContext *ctx)
{
    const ChewingData *pgdata;

    if (!ctx) {
        return -1;
    }
    pgdata = ctx->data;

    LOG_API("bSpaceAsSelection = %d", ctx->data->config.bSpaceAsSelection);

    return ctx->data->config.bSpaceAsSelection;
}

CHEWING_API void chewing_set_escCleanAllBuf(ChewingContext *ctx, int mode)
{
    ChewingData *pgdata;

    if (!ctx) {
        return;
    }
    pgdata = ctx->data;

    LOG_API("mode = %d", mode);

    if (mode == 0 || mode == 1)
        ctx->data->config.bEscCleanAllBuf = mode;
}

CHEWING_API int chewing_get_escCleanAllBuf(const ChewingContext *ctx)
{
    const ChewingData *pgdata;

    if (!ctx) {
        return -1;
    }
    pgdata = ctx->data;

    LOG_API("bEscCleanAllBuf = %d", ctx->data->config.bEscCleanAllBuf);

    return ctx->data->config.bEscCleanAllBuf;
}

CHEWING_API void chewing_set_autoShiftCur(ChewingContext *ctx, int mode)
{
    ChewingData *pgdata;

    if (!ctx) {
        return;
    }
    pgdata = ctx->data;

    LOG_API("mode = %d", mode);

    if (mode == 0 || mode == 1)
        ctx->data->config.bAutoShiftCur = mode;
}

CHEWING_API int chewing_get_autoShiftCur(const ChewingContext *ctx)
{
    const ChewingData *pgdata;

    if (!ctx) {
        return -1;
    }
    pgdata = ctx->data;

    LOG_API("bAutoShiftCur = %d", ctx->data->config.bAutoShiftCur);

    return ctx->data->config.bAutoShiftCur;
}

CHEWING_API void chewing_set_easySymbolInput(ChewingContext *ctx, int mode)
{
    ChewingData *pgdata;

    if (!ctx) {
        return;
    }
    pgdata = ctx->data;

    LOG_API("mode = %d", mode);

    if (mode == 0 || mode == 1)
        ctx->data->config.bEasySymbolInput = mode;
}

CHEWING_API int chewing_get_easySymbolInput(const ChewingContext *ctx)
{
    const ChewingData *pgdata;

    if (!ctx) {
        return -1;
    }
    pgdata = ctx->data;

    LOG_API("bEasySymbolInput = %d", ctx->data->config.bEasySymbolInput);

    return ctx->data->config.bEasySymbolInput;
}

CHEWING_API void chewing_set_phraseChoiceRearward(ChewingContext *ctx, int mode)
{
    ChewingData *pgdata;

    if (!ctx) {
        return;
    }
    pgdata = ctx->data;

    LOG_API("mode = %d", mode);

    if (mode == 0 || mode == 1)
        ctx->data->config.bPhraseChoiceRearward = mode;
}

CHEWING_API int chewing_get_phraseChoiceRearward(const ChewingContext *ctx)
{
    const ChewingData *pgdata;

    if (!ctx) {
        return -1;
    }
    pgdata = ctx->data;

    LOG_API("bPhraseChoiceRearward = %d", ctx->data->config.bPhraseChoiceRearward);

    return ctx->data->config.bPhraseChoiceRearward;
}

CHEWING_API void chewing_set_ChiEngMode(ChewingContext *ctx, int mode)
{
    ChewingData *pgdata;

    if (!ctx) {
        return;
    }
    pgdata = ctx->data;

    LOG_API("mode = %d", mode);

    if (mode == CHINESE_MODE || mode == SYMBOL_MODE) {
        // remove all data inside buffer as switching mode.
        BopomofoRemoveAll(&(ctx->data->bopomofoData));
        MakeOutputWithRtn(ctx->output, ctx->data, KEYSTROKE_ABSORB);
        ctx->data->bChiSym = mode;
    }
}

CHEWING_API int chewing_get_ChiEngMode(const ChewingContext *ctx)
{
    const ChewingData *pgdata;

    if (!ctx) {
        return -1;
    }
    pgdata = ctx->data;

    LOG_API("bChiSym = %d", ctx->data->bChiSym);

    return ctx->data->bChiSym;
}

CHEWING_API void chewing_set_ShapeMode(ChewingContext *ctx, int mode)
{
    ChewingData *pgdata;

    if (!ctx) {
        return;
    }
    pgdata = ctx->data;

    LOG_API("mode = %d", mode);

    if (mode == HALFSHAPE_MODE || mode == FULLSHAPE_MODE)
        ctx->data->bFullShape = mode;
}

CHEWING_API int chewing_get_ShapeMode(const ChewingContext *ctx)
{
    const ChewingData *pgdata;

    if (!ctx) {
        return -1;
    }
    pgdata = ctx->data;

    LOG_API("ctx->data->bFullShape = %d", ctx->data->bFullShape);

    return ctx->data->bFullShape;
}

static void CheckAndResetRange(ChewingData *pgdata)
{
    if (pgdata->PointStart > -1) {
        pgdata->PointStart = -1;
        pgdata->PointEnd = 0;
    }
}

static int SelectCandidate(ChewingData *pgdata, int num)
{
    assert(pgdata);
    assert(pgdata->choiceInfo.pageNo >= 0);

    if (0 <= num && num < pgdata->choiceInfo.nTotalChoice) {
        if (pgdata->choiceInfo.isSymbol != WORD_CHOICE) {
            SymbolChoice(pgdata, num);
        } else {
            /* change the select interval & selectStr & nSelect */
            AddSelect(pgdata, num);
            /* second, call choice module */
            ChoiceSelect(pgdata, num);
            /* automatically shift the cursor to next phrase */
            if (pgdata->config.bAutoShiftCur != 0 &&
                /* if cursor at end of string, do not shift the cursor. */
                pgdata->chiSymbolCursor < pgdata->chiSymbolBufLen) {
                if (pgdata->config.bPhraseChoiceRearward) {
                    ++pgdata->chiSymbolCursor;
                } else {
                    pgdata->chiSymbolCursor += pgdata->availInfo.avail[pgdata->availInfo.currentAvail].len;
                }
            }
        }
        return 0;
    }

    return -1;
}

static void DoSelect(ChewingData *pgdata, int num)
{
    assert(pgdata->choiceInfo.pageNo >= 0);
    if (num >= 0) {
        num += pgdata->choiceInfo.pageNo * pgdata->choiceInfo.nChoicePerPage;
        SelectCandidate(pgdata, num);
    }
}

CHEWING_API int chewing_handle_Space(ChewingContext *ctx)
{
    ChewingData *pgdata;

    if (!ctx) {
        return -1;
    }
    pgdata = ctx->data;

    LOG_API("");

    /*
     * Use chewing_handle_Default( ctx, ' ' ) to handle space when:
     * - "space as selection" mode is disable
     * - mode is not CHINESE_MODE
     * - has incompleted bopomofo (space is needed to complete it)
     */
    if (!pgdata->config.bSpaceAsSelection || pgdata->bChiSym != CHINESE_MODE || BopomofoIsEntering(&ctx->data->bopomofoData)) {
        return chewing_handle_Default(ctx, ' ');
    }

    CheckAndResetRange(pgdata);

    /*
     * space = right when the follogin conditions are true
     * 1. In select mode
     * 2. The candidate page is not last page
     *
     * Otherwise, space = down
     */
    if (pgdata->bSelect && ctx->output->pci->pageNo < ctx->output->pci->nPage - 1) {
        return chewing_handle_Right(ctx);
    } else {
        return chewing_handle_Down(ctx);
    }
    return 0;
}

CHEWING_API int chewing_handle_Esc(ChewingContext *ctx)
{
    ChewingData *pgdata;
    ChewingOutput *pgo;
    int keystrokeRtn = KEYSTROKE_ABSORB;

    if (!ctx) {
        return -1;
    }
    pgdata = ctx->data;
    pgo = ctx->output;

    LOG_API("");

    CheckAndResetRange(pgdata);

    if (!ChewingIsEntering(pgdata)) {
        keystrokeRtn = KEYSTROKE_IGNORE;
    } else if (pgdata->bSelect) {
        ChoiceEndChoice(pgdata);
    } else if (BopomofoIsEntering(&(pgdata->bopomofoData))) {
        BopomofoRemoveAll(&(pgdata->bopomofoData));
    } else if (pgdata->config.bEscCleanAllBuf) {
        CleanAllBuf(pgdata);
        pgo->commitBufLen = pgdata->chiSymbolBufLen;
    }

    MakeOutputWithRtn(pgo, pgdata, keystrokeRtn);
    return 0;
}

CHEWING_API int chewing_handle_Enter(ChewingContext *ctx)
{
    ChewingData *pgdata;
    ChewingOutput *pgo;
    int nCommitStr;
    int keystrokeRtn = KEYSTROKE_ABSORB;

    if (!ctx) {
        return -1;
    }
    pgdata = ctx->data;
    pgo = ctx->output;

    LOG_API("");

    nCommitStr = pgdata->chiSymbolBufLen;

    if (!ChewingIsEntering(pgdata)) {
        keystrokeRtn = KEYSTROKE_IGNORE;
    } else if (pgdata->bSelect) {
        keystrokeRtn = KEYSTROKE_ABSORB | KEYSTROKE_BELL;
    } else if (pgdata->PointStart > -1) {
        int buf = pgdata->chiSymbolCursor;
        int key;

        if (pgdata->PointEnd > 1) {
            if (!pgdata->config.bAddPhraseForward) {
                pgdata->chiSymbolCursor = pgdata->PointStart;
                key = '0' + pgdata->PointEnd;
            } else {
                pgdata->chiSymbolCursor = pgdata->PointStart + pgdata->PointEnd;
                key = '0' + pgdata->PointEnd;
            }

            chewing_handle_CtrlNum(ctx, key);
            pgdata->chiSymbolCursor = buf;
        } else if (pgdata->PointEnd < 1) {
            if (pgdata->config.bAddPhraseForward)
                pgdata->chiSymbolCursor = buf - pgdata->PointEnd;
            key = '0' - pgdata->PointEnd;
            chewing_handle_CtrlNum(ctx, key);
            pgdata->chiSymbolCursor = buf;
        }
        pgdata->PointStart = -1;
        pgdata->PointEnd = 0;
    } else {
        keystrokeRtn = KEYSTROKE_COMMIT;
        WriteChiSymbolToCommitBuf(pgdata, pgo, nCommitStr);
        AutoLearnPhrase(pgdata);
        CleanAllBuf(pgdata);
        pgo->commitBufLen = nCommitStr;
    }

    MakeOutputWithRtn(pgo, pgdata, keystrokeRtn);
    return 0;
}

CHEWING_API int chewing_handle_Del(ChewingContext *ctx)
{
    ChewingData *pgdata;
    ChewingOutput *pgo;
    int keystrokeRtn = KEYSTROKE_ABSORB;

    if (!ctx) {
        return -1;
    }
    pgdata = ctx->data;
    pgo = ctx->output;

    LOG_API("");

    CheckAndResetRange(pgdata);

    if (!ChewingIsEntering(pgdata)) {
        keystrokeRtn = KEYSTROKE_IGNORE;
    }

    if (!pgdata->bSelect) {
        if (!BopomofoIsEntering(&(pgdata->bopomofoData)) && pgdata->chiSymbolCursor < pgdata->chiSymbolBufLen) {
            ChewingKillChar(pgdata, pgdata->chiSymbolCursor, NONDECREASE_CURSOR);
        }
        CallPhrasing(pgdata, 0);
    }
    MakeOutputWithRtn(pgo, pgdata, keystrokeRtn);
    return 0;
}

CHEWING_API int chewing_handle_Backspace(ChewingContext *ctx)
{
    ChewingData *pgdata;
    ChewingOutput *pgo;
    int keystrokeRtn = KEYSTROKE_ABSORB;

    if (!ctx) {
        return -1;
    }
    pgdata = ctx->data;
    pgo = ctx->output;

    LOG_API("");

    CheckAndResetRange(pgdata);

    if (!ChewingIsEntering(pgdata)) {
        keystrokeRtn = KEYSTROKE_IGNORE;
    }

    if (!pgdata->bSelect) {
        if (BopomofoIsEntering(&(pgdata->bopomofoData))) {
            BopomofoRemoveLast(&(pgdata->bopomofoData));
        } else if (pgdata->chiSymbolCursor > 0) {
            ChewingKillChar(pgdata, pgdata->chiSymbolCursor - 1, DECREASE_CURSOR);
        }
        CallPhrasing(pgdata, 0);
    } else if (pgdata->bSelect) {
        chewing_cand_close(ctx);
    }

    MakeOutputWithRtn(pgo, pgdata, keystrokeRtn);
    return 0;
}

CHEWING_API int chewing_handle_Up(ChewingContext *ctx)
{
    ChewingData *pgdata;
    ChewingOutput *pgo;
    int keystrokeRtn = KEYSTROKE_ABSORB;
    int key_buf_cursor;

    if (!ctx) {
        return -1;
    }
    pgdata = ctx->data;
    pgo = ctx->output;

    LOG_API("");

    CheckAndResetRange(pgdata);

    if (!ChewingIsEntering(pgdata)) {
        keystrokeRtn = KEYSTROKE_IGNORE;
    }

    key_buf_cursor = pgdata->chiSymbolCursor;
    // FIXME: when pgdata->chiSymbolBufLen == 0, key_buf_cursor will be -1.
    if (pgdata->chiSymbolCursor == pgdata->chiSymbolBufLen)
        key_buf_cursor--;

    /* close candidate list, compared to Down key to open candidate list. */
    if (pgdata->bSelect) {
        ChoiceEndChoice(pgdata);
    }

    MakeOutputWithRtn(pgo, pgdata, keystrokeRtn);
    return 0;
}

CHEWING_API int chewing_handle_Down(ChewingContext *ctx)
{
    ChewingData *pgdata;
    ChewingOutput *pgo;
    int toSelect = 0;
    int keystrokeRtn = KEYSTROKE_ABSORB;
    int key_buf_cursor;

    if (!ctx) {
        return -1;
    }
    pgdata = ctx->data;
    pgo = ctx->output;

    LOG_API("");

    CheckAndResetRange(pgdata);

    if (!ChewingIsEntering(pgdata)) {
        keystrokeRtn = KEYSTROKE_IGNORE;
    }

    key_buf_cursor = pgdata->chiSymbolCursor;
    if (pgdata->chiSymbolCursor == pgdata->chiSymbolBufLen && key_buf_cursor > 0)
        key_buf_cursor--;

    /* see if to select */
    if (ChewingIsChiAt(key_buf_cursor, pgdata))
        toSelect = 1;

    chooseCandidate(ctx, toSelect, key_buf_cursor);

    MakeOutputWithRtn(pgo, pgdata, keystrokeRtn);
    return 0;
}

/* Add phrase in Hanin Style */
CHEWING_API int chewing_handle_ShiftLeft(ChewingContext *ctx)
{
    ChewingData *pgdata;
    ChewingOutput *pgo;
    int keystrokeRtn = KEYSTROKE_ABSORB;

    if (!ctx) {
        return -1;
    }
    pgdata = ctx->data;
    pgo = ctx->output;

    LOG_API("");

    if (!ChewingIsEntering(pgdata)) {
        keystrokeRtn = KEYSTROKE_IGNORE;
    }
    if (!pgdata->bSelect) {
        /*  PointEnd locates (-9, +9) */
        if (!BopomofoIsEntering(&(pgdata->bopomofoData)) && pgdata->chiSymbolCursor > 0 && pgdata->PointEnd > -9) {
            if (pgdata->PointStart == -1)
                pgdata->PointStart = pgdata->chiSymbolCursor;
            pgdata->chiSymbolCursor--;
            if (ChewingIsChiAt(pgdata->chiSymbolCursor, pgdata)) {
                pgdata->PointEnd--;
            }
            if (pgdata->PointEnd == 0)
                pgdata->PointStart = -1;
        }
    }

    MakeOutputWithRtn(pgo, pgdata, keystrokeRtn);
    return 0;
}

CHEWING_API int chewing_handle_Left(ChewingContext *ctx)
{
    ChewingData *pgdata;
    ChewingOutput *pgo;
    int keystrokeRtn = KEYSTROKE_ABSORB;

    if (!ctx) {
        return -1;
    }
    pgdata = ctx->data;
    pgo = ctx->output;

    LOG_API("");

    if (!ChewingIsEntering(pgdata)) {
        keystrokeRtn = KEYSTROKE_IGNORE;
    }

    if (pgdata->bSelect) {
        assert(pgdata->choiceInfo.nPage > 0);
        if (pgdata->choiceInfo.pageNo > 0)
            pgdata->choiceInfo.pageNo--;
        else
            pgdata->choiceInfo.pageNo = pgdata->choiceInfo.nPage - 1;
    } else {
        if (!BopomofoIsEntering(&(pgdata->bopomofoData)) && pgdata->chiSymbolCursor > 0) {
            CheckAndResetRange(pgdata);
            pgdata->chiSymbolCursor--;
        }
    }
    MakeOutputWithRtn(pgo, pgdata, keystrokeRtn);
    return 0;
}

/* Add phrase in Hanin Style */
CHEWING_API int chewing_handle_ShiftRight(ChewingContext *ctx)
{
    ChewingData *pgdata;
    ChewingOutput *pgo;
    int keystrokeRtn = KEYSTROKE_ABSORB;

    if (!ctx) {
        return -1;
    }
    pgdata = ctx->data;
    pgo = ctx->output;

    LOG_API("");

    if (!ChewingIsEntering(pgdata)) {
        keystrokeRtn = KEYSTROKE_IGNORE;
    }

    if (!pgdata->bSelect) {
        /* PointEnd locates (-9, +9) */
        if (!BopomofoIsEntering(&(pgdata->bopomofoData)) &&
            pgdata->chiSymbolCursor < pgdata->chiSymbolBufLen && pgdata->PointEnd < 9) {
            if (pgdata->PointStart == -1)
                pgdata->PointStart = pgdata->chiSymbolCursor;
            if (ChewingIsChiAt(pgdata->chiSymbolCursor, pgdata)) {
                pgdata->PointEnd++;
            }
            pgdata->chiSymbolCursor++;
            if (pgdata->PointEnd == 0)
                pgdata->PointStart = -1;
        }
    }

    MakeOutputWithRtn(pgo, pgdata, keystrokeRtn);
    return 0;
}

CHEWING_API int chewing_handle_Right(ChewingContext *ctx)
{
    ChewingData *pgdata;
    ChewingOutput *pgo;
    int keystrokeRtn = KEYSTROKE_ABSORB;

    if (!ctx) {
        return -1;
    }
    pgdata = ctx->data;
    pgo = ctx->output;

    LOG_API("");

    if (!ChewingIsEntering(pgdata)) {
        keystrokeRtn = KEYSTROKE_IGNORE;
    }

    if (pgdata->bSelect) {
        if (pgdata->choiceInfo.pageNo < pgdata->choiceInfo.nPage - 1)
            pgdata->choiceInfo.pageNo++;
        else
            pgdata->choiceInfo.pageNo = 0;
    } else {
        if (!BopomofoIsEntering(&(pgdata->bopomofoData)) && pgdata->chiSymbolCursor < pgdata->chiSymbolBufLen) {
            CheckAndResetRange(pgdata);
            pgdata->chiSymbolCursor++;
        }
    }

    MakeOutputWithRtn(pgo, pgdata, keystrokeRtn);
    return 0;
}

CHEWING_API int chewing_handle_Tab(ChewingContext *ctx)
{
    ChewingData *pgdata;
    ChewingOutput *pgo;
    int keystrokeRtn = KEYSTROKE_ABSORB;
    int all_phrasing = 0;
    int cursor;

    if (!ctx) {
        return -1;
    }
    pgdata = ctx->data;
    pgo = ctx->output;

    LOG_API("");

    CheckAndResetRange(pgdata);

    if (!ChewingIsEntering(pgdata)) {
        keystrokeRtn = KEYSTROKE_IGNORE;
    }


    if (!pgdata->bSelect) {
        if (pgdata->chiSymbolCursor == pgdata->chiSymbolBufLen) {
            pgdata->phrOut.nNumCut++;
            all_phrasing = 1;
        } else if (ChewingIsChiAt(pgdata->chiSymbolCursor - 1, pgdata)) {
            cursor = PhoneSeqCursor(pgdata);
            if (IsPreferIntervalConnted(cursor, pgdata)) {
                pgdata->bUserArrBrkpt[cursor] = 1;
                pgdata->bUserArrCnnct[cursor] = 0;
            } else {
                pgdata->bUserArrBrkpt[cursor] = 0;
                pgdata->bUserArrCnnct[cursor] = 1;
            }
        }
        CallPhrasing(pgdata, all_phrasing);
    }
    MakeOutputWithRtn(pgo, pgdata, keystrokeRtn);
    return 0;
}

CHEWING_API int chewing_handle_DblTab(ChewingContext *ctx)
{
    ChewingData *pgdata;
    ChewingOutput *pgo;
    int keystrokeRtn = KEYSTROKE_ABSORB;
    int cursor;

    if (!ctx) {
        return -1;
    }
    pgdata = ctx->data;
    pgo = ctx->output;

    LOG_API("");

    CheckAndResetRange(pgdata);

    if (!ChewingIsEntering(pgdata)) {
        keystrokeRtn = KEYSTROKE_IGNORE;
    }

    if (!pgdata->bSelect) {
        cursor = PhoneSeqCursor(pgdata);
        pgdata->bUserArrBrkpt[cursor] = 0;
        pgdata->bUserArrCnnct[cursor] = 0;
    }
    CallPhrasing(pgdata, 0);

    MakeOutputWithRtn(pgo, pgdata, keystrokeRtn);
    return 0;
}


CHEWING_API int chewing_handle_Capslock(ChewingContext *ctx)
{
    ChewingData *pgdata;
    ChewingOutput *pgo;

    if (!ctx) {
        return -1;
    }
    pgdata = ctx->data;
    pgo = ctx->output;

    LOG_API("");

    chewing_set_ChiEngMode(ctx, 1 - chewing_get_ChiEngMode(ctx));
    MakeOutputWithRtn(pgo, pgdata, KEYSTROKE_ABSORB);
    return 0;
}

CHEWING_API int chewing_handle_Home(ChewingContext *ctx)
{
    ChewingData *pgdata;
    ChewingOutput *pgo;
    int keystrokeRtn = KEYSTROKE_ABSORB;

    if (!ctx) {
        return -1;
    }
    pgdata = ctx->data;
    pgo = ctx->output;

    LOG_API("");

    CheckAndResetRange(pgdata);

    if (!ChewingIsEntering(pgdata)) {
        keystrokeRtn = KEYSTROKE_IGNORE;
    } else if (!pgdata->bSelect) {
        pgdata->chiSymbolCursor = 0;
    }
    MakeOutputWithRtn(pgo, pgdata, keystrokeRtn);
    return 0;
}

CHEWING_API int chewing_handle_End(ChewingContext *ctx)
{
    ChewingData *pgdata;
    ChewingOutput *pgo;
    int keystrokeRtn = KEYSTROKE_ABSORB;

    if (!ctx) {
        return -1;
    }
    pgdata = ctx->data;
    pgo = ctx->output;

    LOG_API("");

    CheckAndResetRange(pgdata);

    if (!ChewingIsEntering(pgdata)) {
        keystrokeRtn = KEYSTROKE_IGNORE;
    } else if (!pgdata->bSelect) {
        pgdata->chiSymbolCursor = pgdata->chiSymbolBufLen;
    }
    MakeOutputWithRtn(pgo, pgdata, keystrokeRtn);
    return 0;
}

CHEWING_API int chewing_handle_PageUp(ChewingContext *ctx)
{
    ChewingData *pgdata;
    ChewingOutput *pgo;
    int keystrokeRtn = KEYSTROKE_ABSORB;

    if (!ctx) {
        return -1;
    }
    pgdata = ctx->data;
    pgo = ctx->output;

    LOG_API("");

    CheckAndResetRange(pgdata);

    if (!ChewingIsEntering(pgdata)) {
        keystrokeRtn = KEYSTROKE_IGNORE;
    } else if (!pgdata->bSelect) {
        pgdata->chiSymbolCursor = pgdata->chiSymbolBufLen;
    } else if (pgdata->bSelect) {
        assert(pgdata->choiceInfo.nPage > 0);
        if (pgdata->choiceInfo.pageNo > 0)
            pgdata->choiceInfo.pageNo--;
        else
            pgdata->choiceInfo.pageNo = pgdata->choiceInfo.nPage - 1;
    }
    MakeOutputWithRtn(pgo, pgdata, keystrokeRtn);
    return 0;
}

CHEWING_API int chewing_handle_PageDown(ChewingContext *ctx)
{
    ChewingData *pgdata;
    ChewingOutput *pgo;
    int keystrokeRtn = KEYSTROKE_ABSORB;

    if (!ctx) {
        return -1;
    }
    pgdata = ctx->data;
    pgo = ctx->output;

    LOG_API("");

    CheckAndResetRange(pgdata);

    if (!ChewingIsEntering(pgdata)) {
        keystrokeRtn = KEYSTROKE_IGNORE;
    } else if (!pgdata->bSelect) {
        pgdata->chiSymbolCursor = pgdata->chiSymbolBufLen;
    } else if (pgdata->bSelect) {
        if (pgdata->choiceInfo.pageNo < pgdata->choiceInfo.nPage - 1)
            pgdata->choiceInfo.pageNo++;
        else
            pgdata->choiceInfo.pageNo = 0;
    }
    MakeOutputWithRtn(pgo, pgdata, keystrokeRtn);
    return 0;
}

/* Dvorak <-> Qwerty keyboard layout converter */
static int dvorak_convert(int key)
{
    const char dkey[] = {
        '\'', '\"', ',', '<', '.', '>', 'p', 'P', 'y', 'Y', 'f', 'F', 'g', 'G',
        'c', 'C', 'r', 'R', 'l', 'L', '/', '?', '=', '+', '\\', '|',
        'a', 'A', 'o', 'O', 'e', 'E', 'u', 'U', 'i', 'I', 'd', 'D', 'h', 'H',
        't', 'T', 'n', 'N', 's', 'S', '-', '_',
        ';', ':', 'q', 'Q', 'j', 'J', 'k', 'K', 'x', 'X', 'b', 'B', 'm', 'M',
        'w', 'W', 'v', 'V', 'z', 'Z'
    };
    const char qkey[] = {
        'q', 'Q', 'w', 'W', 'e', 'E', 'r', 'R', 't', 'T', 'y', 'Y', 'u', 'U',
        'i', 'I', 'o', 'O', 'p', 'P', '[', '{', ']', '}', '\\', '|',
        'a', 'A', 's', 'S', 'd', 'D', 'f', 'F', 'g', 'G', 'h', 'H', 'j', 'J',
        'k', 'K', 'l', 'L', ';', ':', '\'', '\"',
        'z', 'Z', 'x', 'X', 'c', 'C', 'v', 'V', 'b', 'B', 'n', 'N', 'm', 'M',
        ',', '<', '.', '>', '/', '?'
    };
    size_t i;

    STATIC_ASSERT(ARRAY_SIZE(dkey) == ARRAY_SIZE(qkey));

    for (i = 0; i < ARRAY_SIZE(dkey); i++) {
        if (key == qkey[i]) {
            key = dkey[i];
            return key;
        }
    }
    return key;
}

CHEWING_API int chewing_handle_Default(ChewingContext *ctx, int key)
{
    ChewingData *pgdata;
    ChewingOutput *pgo;
    int keystrokeRtn = KEYSTROKE_ABSORB;
    int rtn;
    int num;
    int bQuickCommit = 0;

    if (!ctx) {
        return -1;
    }
    pgdata = ctx->data;
    pgo = ctx->output;

    LOG_API("key = %d", key);

    /* Update lifetime */
    IncreaseLifeTime(ctx->data);

    /* Skip the special key */
    if (key & 0xFF00) {
        keystrokeRtn = KEYSTROKE_IGNORE;
        goto End_KeyDefault;
    }

    /* We ignore non-printable input */
    if (!isprint(key))
        goto End_KeyDefault;

    CheckAndResetRange(pgdata);

    DEBUG_CHECKPOINT();
    DEBUG_OUT("   key=%d", key);

    /* Dvorak Hsu */
    if (pgdata->bopomofoData.kbtype == KB_DVORAK_HSU) {
        key = dvorak_convert(key);
    }

    /* selecting */
    if (pgdata->bSelect) {
        if (key == ' ')
            return chewing_handle_Right(ctx);
        /* num starts from 0 */
        num = CountSelKeyNum(key, pgdata);
        if (num >= 0) {
            DoSelect(pgdata, num);
            goto End_keyproc;
        }

        /* Otherwise, use 'j' and 'k' for paging in selection mode */
        DEBUG_OUT("\t\tchecking paging key, got '%c'\n", key);
        switch (key) {
        case 'j':
        case 'J':
            if (pgdata->chiSymbolCursor > 0) {
                if (!ChewingIsEntering(pgdata)) {
                    keystrokeRtn = KEYSTROKE_IGNORE;
                }
                CheckAndResetRange(pgdata);
                pgdata->chiSymbolCursor--;
                if (ChewingIsChiAt(pgdata->chiSymbolCursor, pgdata))
                    ChoiceInitAvail(pgdata);
                else
                    OpenSymbolChoice(pgdata);

            }
            goto End_Paging;
        case 'k':
        case 'K':
            if (pgdata->chiSymbolCursor < pgdata->chiSymbolBufLen) {
                if (!ChewingIsEntering(pgdata)) {
                    keystrokeRtn = KEYSTROKE_IGNORE;
                }
                CheckAndResetRange(pgdata);
                pgdata->chiSymbolCursor++;
                if (ChewingIsChiAt(pgdata->chiSymbolCursor, pgdata))
                    ChoiceInitAvail(pgdata);
                else
                    OpenSymbolChoice(pgdata);
            }
            goto End_Paging;
        default:
            break;
        }
    }
    /* editing */
    else {
        if (pgdata->bChiSym == CHINESE_MODE) {
            if (pgdata->config.bEasySymbolInput != 0) {
                EasySymbolInput(key, pgdata);
                goto End_keyproc;
            }

            rtn = BopomofoPhoInput(pgdata, key);
            DEBUG_OUT("\t\tChinese mode key, " "BopomofoPhoInput return value = %d\n", rtn);

            if (rtn == BOPOMOFO_KEY_ERROR)
                rtn = SpecialSymbolInput(key, pgdata);
            switch (rtn) {
            case BOPOMOFO_ABSORB:
                keystrokeRtn = KEYSTROKE_ABSORB;
                break;
            case BOPOMOFO_COMMIT:
                AddChi(pgdata->bopomofoData.phone, pgdata->bopomofoData.phoneAlt, pgdata);
                break;
            case BOPOMOFO_NO_WORD:
                keystrokeRtn = KEYSTROKE_BELL | KEYSTROKE_ABSORB;
                break;
            case BOPOMOFO_KEY_ERROR:
            case BOPOMOFO_IGNORE:
                DEBUG_OUT("\t\tbefore isupper(key),key=%d\n", key);
                /* change upper case into lower case */
                if (isupper(key))
                    key = tolower(key);

                DEBUG_OUT("\t\tafter isupper(key),key=%d\n", key);

                /* see if buffer contains nothing */
                if (pgdata->chiSymbolBufLen == 0) {
                    bQuickCommit = 1;
                }

                if (pgdata->config.bEasySymbolInput == 0) {
                    if (pgdata->bFullShape)
                        rtn = FullShapeSymbolInput(key, pgdata);
                    else
                        rtn = SymbolInput(key, pgdata);
                }

                if (rtn == SYMBOL_KEY_ERROR) {
                    keystrokeRtn = KEYSTROKE_IGNORE;
                    /*
                     * If the key is not a printable symbol,
                     * then it's wrong to commit it.
                     */
                    bQuickCommit = 0;
                } else
                    keystrokeRtn = KEYSTROKE_ABSORB;

                break;
            default:
                goto End_KeyDefault;
            }
        }
        /* English mode */
        else {
            /* see if buffer contains nothing */
            if (pgdata->chiSymbolBufLen == 0) {
                bQuickCommit = 1;
            }
            if (pgdata->bFullShape) {
                rtn = FullShapeSymbolInput(key, pgdata);
            } else {
                rtn = SymbolInput(key, pgdata);
            }

            if (rtn == SYMBOL_KEY_ERROR) {
                keystrokeRtn = KEYSTROKE_IGNORE;
                bQuickCommit = 0;
            }
        }
    }

  End_keyproc:
    if (!bQuickCommit) {
        CallPhrasing(pgdata, 0);
        if (ReleaseChiSymbolBuf(pgdata, pgo) != 0)
            keystrokeRtn = KEYSTROKE_COMMIT;
    }
    /* Quick commit */
    else {
        DEBUG_OUT("\t\tQuick commit buf[0]=%c\n", pgdata->preeditBuf[0].char_);
        WriteChiSymbolToCommitBuf(pgdata, pgo, 1);
        pgdata->chiSymbolBufLen = 0;
        pgdata->chiSymbolCursor = 0;
        keystrokeRtn = KEYSTROKE_COMMIT;
    }

    if (pgdata->phrOut.nNumCut > 0) {
        int i;

        for (i = 0; i < pgdata->phrOut.nDispInterval; i++) {
            pgdata->bUserArrBrkpt[pgdata->phrOut.dispInterval[i].from] = 1;
            pgdata->bUserArrBrkpt[pgdata->phrOut.dispInterval[i].to] = 1;
        }
        pgdata->phrOut.nNumCut = 0;
    }

  End_KeyDefault:
    CallPhrasing(pgdata, 0);
  End_Paging:
    MakeOutputWithRtn(pgo, pgdata, keystrokeRtn);
    return 0;
}

CHEWING_API int chewing_handle_CtrlNum(ChewingContext *ctx, int key)
{
    ChewingData *pgdata;
    ChewingOutput *pgo;
    int keystrokeRtn = KEYSTROKE_ABSORB;
    int newPhraseLen;
    int i;
    uint16_t addPhoneSeq[MAX_PHONE_SEQ_LEN];
    char addWordSeq[MAX_PHONE_SEQ_LEN * MAX_UTF8_SIZE + 1];
    int phraseState;
    int cursor;

    if (!ctx) {
        return -1;
    }
    pgdata = ctx->data;
    pgo = ctx->output;

    LOG_API("");

    CheckAndResetRange(pgdata);

    if (pgdata->bSelect)
        return 0;

    CallPhrasing(pgdata, 0);
    newPhraseLen = key - '0';

    if (key == '0' || key == '1') {
        pgdata->bSelect = 1;
        pgdata->choiceInfo.oldChiSymbolCursor = pgdata->chiSymbolCursor;

        HaninSymbolInput(pgdata);
        CallPhrasing(pgdata, 0);
        MakeOutputWithRtn(pgo, pgdata, keystrokeRtn);
        return 0;
    }

    cursor = PhoneSeqCursor(pgdata);
    if (!pgdata->config.bAddPhraseForward) {
        if (newPhraseLen >= 1 && cursor + newPhraseLen - 1 <= pgdata->nPhoneSeq) {
            if (NoSymbolBetween(pgdata, cursor, cursor + newPhraseLen)) {
                /* Manually add phrase to the user phrase database. */
                memcpy(addPhoneSeq, &pgdata->phoneSeq[cursor], sizeof(uint16_t) * newPhraseLen);
                addPhoneSeq[newPhraseLen] = 0;

                copyStringFromPreeditBuf(pgdata, cursor, newPhraseLen, addWordSeq, sizeof(addWordSeq));

                phraseState = UserUpdatePhrase(pgdata, addPhoneSeq, addWordSeq);
                SetUpdatePhraseMsg(pgdata, addWordSeq, newPhraseLen, phraseState);

                /* Clear the breakpoint between the New Phrase */
                for (i = 1; i < newPhraseLen; i++)
                    pgdata->bUserArrBrkpt[cursor + i] = 0;
            }
        }
    } else {
        if (newPhraseLen >= 1 && cursor - newPhraseLen >= 0) {
            if (NoSymbolBetween(pgdata, cursor - newPhraseLen, cursor)) {
                /* Manually add phrase to the user phrase database. */
                memcpy(addPhoneSeq, &pgdata->phoneSeq[cursor - newPhraseLen], sizeof(uint16_t) * newPhraseLen);
                addPhoneSeq[newPhraseLen] = 0;

                copyStringFromPreeditBuf(pgdata, cursor - newPhraseLen, newPhraseLen, addWordSeq, sizeof(addWordSeq));

                phraseState = UserUpdatePhrase(pgdata, addPhoneSeq, addWordSeq);
                SetUpdatePhraseMsg(pgdata, addWordSeq, newPhraseLen, phraseState);

                /* Clear the breakpoint between the New Phrase */
                for (i = 1; i < newPhraseLen; i++)
                    pgdata->bUserArrBrkpt[cursor - newPhraseLen + i] = 0;
            }
        }
    }
    CallPhrasing(pgdata, 0);
    MakeOutputWithRtn(pgo, pgdata, keystrokeRtn);
    MakeOutputAddMsgAndCleanInterval(pgo, pgdata);
    return 0;
}

CHEWING_API int chewing_handle_ShiftSpace(ChewingContext *ctx)
{
    ChewingData *pgdata;
    ChewingOutput *pgo;
    int keystrokeRtn = KEYSTROKE_ABSORB;

    if (!ctx) {
        return -1;
    }
    pgdata = ctx->data;
    pgo = ctx->output;

    LOG_API("");

    if (!pgdata->bSelect) {
        CheckAndResetRange(pgdata);
    }

    chewing_set_ShapeMode(ctx, 1 - chewing_get_ShapeMode(ctx));

    CallPhrasing(pgdata, 0);
    MakeOutputWithRtn(pgo, pgdata, keystrokeRtn);
    return 0;
}

CHEWING_API int chewing_handle_Numlock(ChewingContext *ctx, int key)
{
    ChewingData *pgdata;
    ChewingOutput *pgo;
    int keystrokeRtn = KEYSTROKE_ABSORB;
    int rtn;
    int QuickCommit = 0;

    if (!ctx) {
        return -1;
    }
    pgdata = ctx->data;
    pgo = ctx->output;

    LOG_API("");

    if (!pgdata->bSelect) {
        /* If we're not selecting words, we should send out numeric
         * characters at once.
         */
        if (pgdata->chiSymbolBufLen == 0) {
            QuickCommit = 1;
        }
        rtn = SymbolInput(key, pgdata);
        /* copied from chewing_handle_Default */
        if (rtn == SYMBOL_KEY_ERROR) {
            keystrokeRtn = KEYSTROKE_IGNORE;
        } else if (QuickCommit) {
            WriteChiSymbolToCommitBuf(pgdata, pgo, 1);
            pgdata->chiSymbolBufLen = 0;
            pgdata->chiSymbolCursor = 0;
            keystrokeRtn = KEYSTROKE_COMMIT;
        } else {                /* Not quick commit */
            CallPhrasing(pgdata, 0);
            if (ReleaseChiSymbolBuf(pgdata, pgo) != 0)
                keystrokeRtn = KEYSTROKE_COMMIT;
        }
    } else {
        /* Otherwise, if we are selecting words, we use numeric keys
         * as selkey
         * and submit the words.
         */
        int num = -1;

        if (key > '0' && key <= '9')
            num = key - '1';
        else if (key == '0')
            num = 9;
        DoSelect(pgdata, num);
    }
    CallPhrasing(pgdata, 0);
    if (ReleaseChiSymbolBuf(pgdata, pgo) != 0)
        keystrokeRtn = KEYSTROKE_COMMIT;
    MakeOutputWithRtn(pgo, pgdata, keystrokeRtn);
    return 0;
}

CHEWING_API unsigned short *chewing_get_phoneSeq(const ChewingContext *ctx)
{
    const ChewingData *pgdata;
    uint16_t *seq;

    if (!ctx) {
        return NULL;
    }
    pgdata = ctx->data;

    LOG_API("");

    seq = ALC(uint16_t, ctx->data->nPhoneSeq);
    if (seq)
        memcpy(seq, ctx->data->phoneSeq, sizeof(uint16_t) * ctx->data->nPhoneSeq);
    return seq;
}

CHEWING_API int chewing_get_phoneSeqLen(const ChewingContext *ctx)
{
    const ChewingData *pgdata;

    if (!ctx) {
        return -1;
    }
    pgdata = ctx->data;

    LOG_API("nPhoneSeq = %d", ctx->data->nPhoneSeq);

    return ctx->data->nPhoneSeq;
}

CHEWING_API void chewing_set_logger(ChewingContext *ctx,
                                    void (*logger) (void *data, int level, const char *fmt, ...), void *data)
{
    ChewingData *pgdata;

    if (!ctx) {
        return;
    }
    pgdata = ctx->data;

    LOG_API("");

    if (!logger) {
        logger = NullLogger;
        data = 0;
    }
    ctx->data->logger = logger;
    ctx->data->loggerData = data;
}

CHEWING_API int chewing_userphrase_enumerate(ChewingContext *ctx)
{
    ChewingData *pgdata;

#if WITH_SQLITE3
    int ret;
#endif

    if (!ctx) {
        return -1;
    }

    pgdata = ctx->data;

    LOG_API("");

#if WITH_SQLITE3
    assert(pgdata->static_data.stmt_userphrase[STMT_USERPHRASE_SELECT]);
    ret = sqlite3_reset(pgdata->static_data.stmt_userphrase[STMT_USERPHRASE_SELECT]);
    if (ret != SQLITE_OK) {
        LOG_ERROR("sqlite3_reset returns %d", ret);
        return -1;
    }
#else
    pgdata->static_data.userphrase_enum = FindNextHash(pgdata, NULL);
#endif
    return 0;
}

CHEWING_API int chewing_userphrase_has_next(ChewingContext *ctx, unsigned int *phrase_len, unsigned int *bopomofo_len)
{
    ChewingData *pgdata;

#if WITH_SQLITE3
    int ret;
#endif

    if (!ctx || !phrase_len || !bopomofo_len) {
        return 0;
    }
    pgdata = ctx->data;

    LOG_API("");

#if WITH_SQLITE3
    ret = sqlite3_step(pgdata->static_data.stmt_userphrase[STMT_USERPHRASE_SELECT]);
    if (ret != SQLITE_ROW) {
        if (ret != SQLITE_DONE) {
            LOG_ERROR("sqlite3_step returns %d", ret);
        }
        return 0;
    }

    *phrase_len = strlen((const char *) sqlite3_column_text(pgdata->static_data.stmt_userphrase[STMT_USERPHRASE_SELECT],
                                                            SQL_STMT_USERPHRASE[STMT_USERPHRASE_SELECT].column
                                                            [COLUMN_USERPHRASE_PHRASE])) + 1;

    *bopomofo_len = GetBopomofoBufLen(sqlite3_column_int(pgdata->static_data.stmt_userphrase[STMT_USERPHRASE_SELECT],
                                                         SQL_STMT_USERPHRASE[STMT_USERPHRASE_SELECT].column
                                                         [COLUMN_USERPHRASE_LENGTH]));

    return 1;
#else
    if (pgdata->static_data.userphrase_enum) {
        *phrase_len = strlen(pgdata->static_data.userphrase_enum->data.wordSeq) + 1;
        *bopomofo_len = BopomofoFromUintArray(NULL, 0, pgdata->static_data.userphrase_enum->data.phoneSeq);
        return 1;

    }
    return 0;
#endif
}

CHEWING_API int chewing_userphrase_get(ChewingContext *ctx,
                                       char *phrase_buf, unsigned int phrase_len,
                                       char *bopomofo_buf, unsigned int bopomofo_len)
{
    ChewingData *pgdata;

#if WITH_SQLITE3
    const char *phrase;
    int length;
    int i;
    uint16_t phone_array[MAX_PHRASE_LEN + 1] = { 0 };
#endif

    if (!ctx || !phrase_buf || !phrase_len || !bopomofo_buf || !bopomofo_len) {
        return -1;
    }
    pgdata = ctx->data;

    LOG_API("");

#if WITH_SQLITE3
    phrase = (const char *) sqlite3_column_text(pgdata->static_data.stmt_userphrase[STMT_USERPHRASE_SELECT],
                                                SQL_STMT_USERPHRASE[STMT_USERPHRASE_SELECT].column
                                                [COLUMN_USERPHRASE_PHRASE]);
    length =
        sqlite3_column_int(pgdata->static_data.stmt_userphrase[STMT_USERPHRASE_SELECT],
                           SQL_STMT_USERPHRASE[STMT_USERPHRASE_SELECT].column[COLUMN_USERPHRASE_LENGTH]);

    if (phrase_len < strlen(phrase) + 1) {
        LOG_ERROR("phrase_len %d is smaller than %d", phrase_len, strlen(phrase) + 1);
        return -1;
    }

    if (bopomofo_len < GetBopomofoBufLen(length)) {
        LOG_ERROR("bopomofo_len %d is smaller than %d", bopomofo_len, GetBopomofoBufLen(length));
        return -1;
    }

    for (i = 0; i < length && i < ARRAY_SIZE(phone_array); ++i) {
        phone_array[i] = sqlite3_column_int(pgdata->static_data.stmt_userphrase[STMT_USERPHRASE_SELECT],
                                            SQL_STMT_USERPHRASE[STMT_USERPHRASE_SELECT].column[COLUMN_USERPHRASE_PHONE_0
                                                                                               + i]);
    }

    strncpy(phrase_buf, phrase, phrase_len);
    BopomofoFromUintArray(bopomofo_buf, bopomofo_len, phone_array);

    return 0;
#else
    if (pgdata->static_data.userphrase_enum) {
        strncpy(phrase_buf, pgdata->static_data.userphrase_enum->data.wordSeq, phrase_len);
        phrase_buf[phrase_len - 1] = 0;

        BopomofoFromUintArray(bopomofo_buf, bopomofo_len, pgdata->static_data.userphrase_enum->data.phoneSeq);
        bopomofo_buf[bopomofo_len - 1] = 0;

        pgdata->static_data.userphrase_enum = FindNextHash(pgdata, pgdata->static_data.userphrase_enum);

        return 0;
    }

    return -1;
#endif
}

CHEWING_API int chewing_userphrase_add(ChewingContext *ctx, const char *phrase_buf, const char *bopomofo_buf)
{
    ChewingData *pgdata;
    ssize_t phrase_len;
    ssize_t phone_len;
    uint16_t *phone_buf = 0;
    int ret;

    if (!ctx || !phrase_buf || !bopomofo_buf) {
        return -1;
    }
    pgdata = ctx->data;

    LOG_API("");

    phrase_len = ueStrLen(phrase_buf);
    phone_len = UintArrayFromBopomofo(NULL, 0, bopomofo_buf);

    if (phrase_len != phone_len) {
        return 0;
    }

    phone_buf = ALC(uint16_t, phone_len + 1);
    if (!phone_buf)
        return -1;
    ret = UintArrayFromBopomofo(phone_buf, phone_len + 1, bopomofo_buf);
    if (ret == -1) {
        free(phone_buf);
        return 0;
    }

    ret = UserUpdatePhrase(pgdata, phone_buf, phrase_buf);
    free(phone_buf);

    if (ret == USER_UPDATE_FAIL) {
        return 0;
    }

    return 1;
}

CHEWING_API int chewing_userphrase_remove(ChewingContext *ctx, const char *phrase_buf, const char *bopomofo_buf)
{
    ChewingData *pgdata;
    ssize_t phone_len;
    uint16_t *phone_buf = 0;
    int ret;

    if (!ctx || !phrase_buf || !bopomofo_buf) {
        return -1;
    }
    pgdata = ctx->data;

    LOG_API("");

    phone_len = UintArrayFromBopomofo(NULL, 0, bopomofo_buf);
    phone_buf = ALC(uint16_t, phone_len + 1);
    if (!phone_buf)
        return 0;
    ret = UintArrayFromBopomofo(phone_buf, phone_len + 1, bopomofo_buf);
    if (ret == -1) {
        free(phone_buf);
        return 0;
    }
    ret = UserRemovePhrase(pgdata, phone_buf, phrase_buf);
    free(phone_buf);

    return ret;
}

CHEWING_API int chewing_userphrase_lookup(ChewingContext *ctx, const char *phrase_buf, const char *bopomofo_buf)
{
    ChewingData *pgdata;
    ssize_t phone_len;
    uint16_t *phone_buf = 0;
    int ret;
    UserPhraseData *user_phrase_data;

    if (!ctx || !phrase_buf || !bopomofo_buf) {
        return 0;
    }
    pgdata = ctx->data;

    LOG_API("");

    phone_len = UintArrayFromBopomofo(NULL, 0, bopomofo_buf);
    phone_buf = ALC(uint16_t, phone_len + 1);
    if (!phone_buf)
        return 0;
    ret = UintArrayFromBopomofo(phone_buf, phone_len + 1, bopomofo_buf);
    if (ret == -1) {
        free(phone_buf);
        return 0;
    }

    user_phrase_data = UserGetPhraseFirst(pgdata, phone_buf);
    while (user_phrase_data) {
        if (strcmp(phrase_buf, user_phrase_data->wordSeq) == 0)
            break;
        user_phrase_data = UserGetPhraseNext(pgdata, phone_buf);
    }
    UserGetPhraseEnd(pgdata, phone_buf);
    free(phone_buf);
    return user_phrase_data == NULL ? 0 : 1;
}

CHEWING_API const char *chewing_cand_string_by_index_static(ChewingContext *ctx, int index)
{
    ChewingData *pgdata;
    char *s;

    if (!ctx) {
        return NULL;
    }
    pgdata = ctx->data;

    LOG_API("index = %d", index);

    if (0 <= index && index < ctx->output->pci->nTotalChoice) {
        s = ctx->output->pci->totalChoiceStr[index];
    } else {
        s = "";
    }
    return s;
}

CHEWING_API int chewing_cand_choose_by_index(ChewingContext *ctx, int index)
{
    ChewingData *pgdata;
    ChewingOutput *pgo;

    int ret;

    if (!ctx) {
        return -1;
    }
    pgdata = ctx->data;
    pgo = ctx->output;

    LOG_API("index = %d", index);

    if (pgdata->choiceInfo.nTotalChoice == 0)
        return -1;

    ret = SelectCandidate(pgdata, index);
    if (ret == 0) {
        CallPhrasing(pgdata, 0);
        MakeOutputWithRtn(pgo, pgdata, KEYSTROKE_ABSORB);
    }
    return ret;
}

CHEWING_API int chewing_cand_open(ChewingContext *ctx)
{
    ChewingData *pgdata;
    int pos;

    if (!ctx) {
        return -1;
    }
    pgdata = ctx->data;

    LOG_API("");

    if (pgdata->bSelect)
        return 0;
    if (pgdata->chiSymbolBufLen == 0)
        return -1;

    pos = pgdata->chiSymbolCursor;
    if (pgdata->chiSymbolCursor == pgdata->chiSymbolBufLen)
        --pos;

    chooseCandidate(ctx, ChewingIsChiAt(pos, pgdata), pos);

    return 0;
}

CHEWING_API int chewing_cand_close(ChewingContext *ctx)
{
    ChewingData *pgdata;

    if (!ctx) {
        return -1;
    }
    pgdata = ctx->data;

    LOG_API("");

    if (ctx->data->bSelect) {
        ChoiceEndChoice(ctx->data);
    }

    return 0;

}

CHEWING_API int chewing_cand_list_first(ChewingContext *ctx)
{
    ChewingData *pgdata;

    if (!ctx) {
        return -1;
    }
    pgdata = ctx->data;

    LOG_API("");

    if (!pgdata->bSelect)
        return -1;

    return ChoiceFirstAvail(pgdata);

    return 0;
}

CHEWING_API int chewing_cand_list_last(ChewingContext *ctx)
{
    ChewingData *pgdata;

    if (!ctx) {
        return -1;
    }
    pgdata = ctx->data;

    LOG_API("");

    if (!pgdata->bSelect)
        return -1;

    return ChoiceLastAvail(pgdata);
}

CHEWING_API int chewing_cand_list_has_next(ChewingContext *ctx)
{
    ChewingData *pgdata;

    if (!ctx) {
        return 0;
    }
    pgdata = ctx->data;

    LOG_API("");

    if (!pgdata->bSelect)
        return 0;

    return ChoiceHasNextAvail(pgdata);
}

CHEWING_API int chewing_cand_list_has_prev(ChewingContext *ctx)
{
    ChewingData *pgdata;

    if (!ctx) {
        return 0;
    }
    pgdata = ctx->data;

    LOG_API("");

    if (!pgdata->bSelect)
        return 0;

    return ChoiceHasPrevAvail(pgdata);
}

CHEWING_API int chewing_cand_list_next(ChewingContext *ctx)
{
    ChewingData *pgdata;

    if (!ctx) {
        return -1;
    }
    pgdata = ctx->data;

    LOG_API("");

    if (!pgdata->bSelect)
        return -1;

    return ChoiceNextAvail(pgdata);
}

CHEWING_API int chewing_cand_list_prev(ChewingContext *ctx)
{
    ChewingData *pgdata;

    if (!ctx) {
        return -1;
    }
    pgdata = ctx->data;

    LOG_API("");

    if (!pgdata->bSelect)
        return -1;

    return ChoicePrevAvail(pgdata);
}

CHEWING_API int chewing_commit_preedit_buf(ChewingContext *ctx)
{
    ChewingData *pgdata;
    ChewingOutput *pgo;
    int len;

    if (!ctx) {
        return -1;
    }
    pgdata = ctx->data;
    pgo = ctx->output;

    LOG_API("");

    if (pgdata->bSelect)
        return -1;

    len = pgdata->chiSymbolBufLen;

    if (!len)
        return -1;

    WriteChiSymbolToCommitBuf(pgdata, pgo, len);
    AutoLearnPhrase(pgdata);
    CleanAllBuf(pgdata);

    MakeOutputWithRtn(pgo, pgdata, KEYSTROKE_COMMIT);

    return 0;
}

CHEWING_API int chewing_clean_preedit_buf(ChewingContext *ctx)
{
    ChewingData *pgdata;
    ChewingOutput *pgo;

    if (!ctx) {
        return -1;
    }
    pgdata = ctx->data;
    pgo = ctx->output;

    LOG_API("");

    if (pgdata->bSelect)
        return -1;

    CleanAllBuf(pgdata);

    MakeOutput(pgo, pgdata);
    return 0;
}

CHEWING_API int chewing_clean_bopomofo_buf(ChewingContext *ctx)
{
    ChewingData *pgdata;
    ChewingOutput *pgo;

    if (!ctx) {
        return -1;
    }
    pgdata = ctx->data;
    pgo = ctx->output;

    LOG_API("");

    if (BopomofoIsEntering(&pgdata->bopomofoData)) {
        BopomofoRemoveAll(&pgdata->bopomofoData);
    }

    MakeOutput(pgo, pgdata);
    return 0;
}
