/*
 * mod_aux.c
 *
 * Copyright (c) 2005, 2006, 2008, 2009, 2012-2014
 *      libchewing Core Team. See ChangeLog for details.
 *
 * See the file "COPYING" for information on usage and redistribution
 * of this file.
 */

/**
 * @file mod_aux.c
 * @brief Auxiliary module
 */

#include <string.h>
#include <stdlib.h>

#include "global.h"
#include "chewing-private.h"
#include "bopomofo-private.h"
#include "chewingio.h"
#include "chewing-utf8-util.h"
#include "private.h"

/**
 * @param ctx handle to Chewing IM context
 * @retval TRUE if it currnet input state is at the "end-of-a-char"
 */
CHEWING_API int chewing_commit_Check(const ChewingContext *ctx)
{
    const ChewingData *pgdata;

    if (!ctx) {
        return -1;
    }
    pgdata = ctx->data;

    LOG_API("");

    return ! !(ctx->output->keystrokeRtn & KEYSTROKE_COMMIT);
}

/**
 * @param ctx handle to Chewing IM context
 *
 * retrun current commit string, regardless current input state.
 * Always returns a char pointer, caller must free it.
 */
CHEWING_API char *chewing_commit_String(const ChewingContext *ctx)
{
    const ChewingData *pgdata;

    if (!ctx) {
        return strdup("");
    }
    pgdata = ctx->data;

    LOG_API("");

    return strdup(ctx->output->commitBuf);
}

/**
 * @param ctx handle to Chewing IM context
 * retrun current commit string, regardless current input state.
 * Always returns a const char pointer, you have to clone them immediately,
 * if you need.
 */
CHEWING_API const char *chewing_commit_String_static(const ChewingContext *ctx)
{
    const ChewingData *pgdata;

    if (!ctx) {
        return "";
    }
    pgdata = ctx->data;

    LOG_API("");

    return ctx->output->commitBuf;
}

CHEWING_API int chewing_buffer_Check(const ChewingContext *ctx)
{
    const ChewingData *pgdata;

    if (!ctx) {
        return -1;
    }
    pgdata = ctx->data;

    LOG_API("");

    return (ctx->output->chiSymbolBufLen != 0);
}

CHEWING_API int chewing_buffer_Len(const ChewingContext *ctx)
{
    const ChewingData *pgdata;

    if (!ctx) {
        return -1;
    }
    pgdata = ctx->data;

    LOG_API("");

    return ctx->output->chiSymbolBufLen;
}

CHEWING_API char *chewing_buffer_String(const ChewingContext *ctx)
{
    const ChewingData *pgdata;

    if (!ctx) {
        return strdup("");
    }
    pgdata = ctx->data;

    LOG_API("");

    return strdup(ctx->output->preeditBuf);
}

CHEWING_API const char *chewing_buffer_String_static(const ChewingContext *ctx)
{
    const ChewingData *pgdata;

    if (!ctx) {
        return "";
    }
    pgdata = ctx->data;

    LOG_API("");

    return ctx->output->preeditBuf;
}

/**
 * @param ctx handle to Chewing IM context
 *
 * Always returns a const char pointer, you have to clone them immediately,
 * if you need.
 */
CHEWING_API const char *chewing_bopomofo_String_static(const ChewingContext *ctx)
{
    const ChewingData *pgdata;

    if (!ctx) {
        return "";
    }
    pgdata = ctx->data;

    LOG_API("");

    return ctx->output->bopomofoBuf;
}

CHEWING_API int chewing_bopomofo_Check(const ChewingContext *ctx)
{
    const ChewingData *pgdata;

    if (!ctx) {
        return -1;
    }
    pgdata = ctx->data;

    LOG_API("");

    return ctx->output->bopomofoBuf[0] != 0;
}

CHEWING_API int chewing_cursor_Current(const ChewingContext *ctx)
{
    const ChewingData *pgdata;

    if (!ctx) {
        return -1;
    }
    pgdata = ctx->data;

    LOG_API("");

    return (ctx->output->chiSymbolCursor);
}

CHEWING_API int chewing_cand_CheckDone(const ChewingContext *ctx)
{
    const ChewingData *pgdata;

    if (!ctx) {
        return -1;
    }
    pgdata = ctx->data;

    LOG_API("");

    return (!ctx->output->pci);
}

CHEWING_API int chewing_cand_TotalPage(const ChewingContext *ctx)
{
    const ChewingData *pgdata;

    if (!ctx) {
        return -1;
    }
    pgdata = ctx->data;

    LOG_API("");

    return (ctx->output->pci ? ctx->output->pci->nPage : 0);
}

CHEWING_API int chewing_cand_ChoicePerPage(const ChewingContext *ctx)
{
    const ChewingData *pgdata;

    if (!ctx) {
        return -1;
    }
    pgdata = ctx->data;

    LOG_API("");

    return (ctx->output->pci ? ctx->output->pci->nChoicePerPage : 0);
}

CHEWING_API int chewing_cand_TotalChoice(const ChewingContext *ctx)
{
    const ChewingData *pgdata;

    if (!ctx) {
        return -1;
    }
    pgdata = ctx->data;

    LOG_API("");

    return (ctx->output->pci ? ctx->output->pci->nTotalChoice : 0);
}

CHEWING_API int chewing_cand_CurrentPage(const ChewingContext *ctx)
{
    const ChewingData *pgdata;

    if (!ctx) {
        return -1;
    }
    pgdata = ctx->data;

    LOG_API("");

    return (ctx->output->pci ? ctx->output->pci->pageNo : -1);
}

CHEWING_API void chewing_cand_Enumerate(ChewingContext *ctx)
{
    ChewingData *pgdata;

    if (!ctx) {
        return;
    }
    pgdata = ctx->data;

    LOG_API("");

    ctx->cand_no = ctx->output->pci->pageNo * ctx->output->pci->nChoicePerPage;
}

CHEWING_API int chewing_cand_hasNext(ChewingContext *ctx)
{
    ChewingData *pgdata;

    if (!ctx) {
        return -1;
    }
    pgdata = ctx->data;

    LOG_API("");

    return (ctx->cand_no < ctx->output->pci->nTotalChoice);
}

CHEWING_API const char *chewing_cand_String_static(ChewingContext *ctx)
{
    ChewingData *pgdata;
    const char *s = "";

    if (!ctx) {
        return s;
    }
    pgdata = ctx->data;

    LOG_API("");

    if (chewing_cand_hasNext(ctx)) {
        s = ctx->output->pci->totalChoiceStr[ctx->cand_no];
        ctx->cand_no++;
    }

    return s;
}

CHEWING_API char *chewing_cand_String(ChewingContext *ctx)
{
    return strdup(chewing_cand_String_static(ctx));
}

CHEWING_API void chewing_interval_Enumerate(ChewingContext *ctx)
{
    ChewingData *pgdata;

    if (!ctx) {
        return;
    }
    pgdata = ctx->data;

    LOG_API("");

    ctx->it_no = 0;
}

CHEWING_API int chewing_interval_hasNext(ChewingContext *ctx)
{
    ChewingData *pgdata;

    if (!ctx) {
        return -1;
    }
    pgdata = ctx->data;

    LOG_API("");

    return (ctx->it_no < ctx->output->nDispInterval);
}

CHEWING_API void chewing_interval_Get(ChewingContext *ctx, IntervalType * it)
{
    ChewingData *pgdata;

    if (!ctx) {
        return;
    }
    pgdata = ctx->data;

    LOG_API("");

    if (chewing_interval_hasNext(ctx)) {
        if (it) {
            it->from = ctx->output->dispInterval[ctx->it_no].from;
            it->to = ctx->output->dispInterval[ctx->it_no].to;
        }
        ctx->it_no++;
    }
}

CHEWING_API int chewing_aux_Check(const ChewingContext *ctx)
{
    const ChewingData *pgdata;

    if (!ctx) {
        return -1;
    }
    pgdata = ctx->data;

    LOG_API("");

    return (ctx->data->bShowMsg);
}

CHEWING_API int chewing_aux_Length(const ChewingContext *ctx)
{
    const ChewingData *pgdata;

    if (!ctx) {
        return -1;
    }
    pgdata = ctx->data;

    LOG_API("");

    return (ctx->data->bShowMsg ? ctx->data->showMsgLen : 0);
}

CHEWING_API const char *chewing_aux_String_static(const ChewingContext *ctx)
{
    const ChewingData *pgdata;

    if (!ctx) {
        return "";
    }
    pgdata = ctx->data;

    LOG_API("");

    return ctx->data->showMsg;
}

CHEWING_API char *chewing_aux_String(const ChewingContext *ctx)
{
    const ChewingData *pgdata;

    if (!ctx) {
        return strdup("");
    }
    pgdata = ctx->data;

    LOG_API("");

    return strdup(chewing_aux_String_static(ctx));
}

CHEWING_API int chewing_keystroke_CheckIgnore(const ChewingContext *ctx)
{
    const ChewingData *pgdata;

    if (!ctx) {
        return -1;
    }
    pgdata = ctx->data;

    LOG_API("");

    return ! !(ctx->output->keystrokeRtn & KEYSTROKE_IGNORE);
}

CHEWING_API int chewing_keystroke_CheckAbsorb(const ChewingContext *ctx)
{
    const ChewingData *pgdata;

    if (!ctx) {
        return -1;
    }
    pgdata = ctx->data;

    LOG_API("");

    return ! !(ctx->output->keystrokeRtn & KEYSTROKE_ABSORB);
}

CHEWING_API int chewing_kbtype_Total(const ChewingContext *ctx UNUSED)
{
    return KB_TYPE_NUM;
}

CHEWING_API void chewing_kbtype_Enumerate(ChewingContext *ctx)
{
    ChewingData *pgdata;

    if (!ctx) {
        return;
    }
    pgdata = ctx->data;

    LOG_API("");

    ctx->kb_no = 0;
}

CHEWING_API int chewing_kbtype_hasNext(ChewingContext *ctx)
{
    ChewingData *pgdata;

    if (!ctx) {
        return -1;
    }
    pgdata = ctx->data;

    LOG_API("");

    return ctx->kb_no < KB_TYPE_NUM;
}

extern const char *const kb_type_str[];

CHEWING_API const char *chewing_kbtype_String_static(ChewingContext *ctx)
{
    ChewingData *pgdata;
    const char *s = "";

    if (!ctx) {
        return s;
    }
    pgdata = ctx->data;

    LOG_API("");

    if (chewing_kbtype_hasNext(ctx)) {
        s = kb_type_str[ctx->kb_no];
        ctx->kb_no++;
    }

    return s;
}

CHEWING_API char *chewing_kbtype_String(ChewingContext *ctx)
{
    ChewingData *pgdata;

    if (!ctx) {
        return strdup("");
    }
    pgdata = ctx->data;

    LOG_API("");

    return strdup(chewing_kbtype_String_static(ctx));
}
