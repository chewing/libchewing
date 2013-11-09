/*
 * mod_aux.c
 *
 * Copyright (c) 2005, 2006, 2008
 *	libchewing Core Team. See ChangeLog for details.
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
#include "zuin-private.h"
#include "chewingio.h"
#include "chewing-utf8-util.h"

/**
 * @param ctx handle to Chewing IM context
 * @retval TRUE if it currnet input state is at the "end-of-a-char"
 */
CHEWING_API int chewing_commit_Check( ChewingContext *ctx )
{
	return !!(ctx->output->keystrokeRtn & KEYSTROKE_COMMIT);
}

/**
 * @param ctx handle to Chewing IM context
 *
 * retrun current commit string, regardless current input state.
 * Alwasy returns a char pointer, caller must free it.
 */
CHEWING_API char *chewing_commit_String( ChewingContext *ctx )
{
	return strdup( ctx->output->commitBuf );
}

/**
 * @param ctx handle to Chewing IM context
 * retrun current commit string, regardless current input state.
 * Alwasy returns a const char pointer, you have to clone them immediately,
 * if you need.
 */
CHEWING_API const char *chewing_commit_String_static( ChewingContext *ctx )
{
	return ctx->output->commitBuf;
}

CHEWING_API int chewing_buffer_Check( ChewingContext *ctx )
{
	return (ctx->output->chiSymbolBufLen != 0);
}

CHEWING_API int chewing_buffer_Len( ChewingContext *ctx )
{
	return ctx->output->chiSymbolBufLen;
}

CHEWING_API char *chewing_buffer_String( ChewingContext *ctx )
{
	return strdup( ctx->output->preeditBuf );
}

CHEWING_API const char *chewing_buffer_String_static( ChewingContext *ctx )
{
	return ctx->output->preeditBuf;
}

/**
 * @param ctx handle to Chewing IM context
 *
 * Always returns a char pointer, caller must free it.
 */
CHEWING_API char *chewing_bopomofo_String( ChewingContext *ctx)
{
	return strdup( ctx->output->bopomofoBuf );
}

/**
 * @param ctx handle to Chewing IM context
 *
 * Alwasy returns a const char pointer, you have to clone them immediately,
 * if you need.
 */
CHEWING_API const char *chewing_bopomofo_String_static( ChewingContext *ctx)
{
	return ctx->output->bopomofoBuf;
}
/**
 * @param ctx handle to Chewing IM context
 * @param zuin_count pointer to the integer of available Zuin preedit string
 *
 * Always returns a char pointer, caller must free it.
 */
CHEWING_API char *chewing_zuin_String( ChewingContext *ctx, int *zuin_count )
{
	char *s = chewing_bopomofo_String(ctx);

	if ( zuin_count )
		*zuin_count = ueStrLen(s);

	return s;
}

CHEWING_API int chewing_bopomofo_Check( ChewingContext *ctx )
{
	return ctx->output->bopomofoBuf[0] != 0;
}

CHEWING_API int chewing_zuin_Check( ChewingContext *ctx )
{
	return !chewing_bopomofo_Check(ctx);
}

CHEWING_API int chewing_cursor_Current( ChewingContext *ctx )
{
	return (ctx->output->chiSymbolCursor);
}

CHEWING_API int chewing_cand_CheckDone( ChewingContext *ctx )
{
	return (! ctx->output->pci);
}

CHEWING_API int chewing_cand_TotalPage( ChewingContext *ctx )
{
	return (ctx->output->pci ? ctx->output->pci->nPage : 0);
}

CHEWING_API int chewing_cand_ChoicePerPage( ChewingContext *ctx )
{
	return (ctx->output->pci ? ctx->output->pci->nChoicePerPage : 0);
}

CHEWING_API int chewing_cand_TotalChoice( ChewingContext *ctx )
{
	return (ctx->output->pci ? ctx->output->pci->nTotalChoice : 0);
}

CHEWING_API int chewing_cand_CurrentPage( ChewingContext *ctx )
{
	return (ctx->output->pci ? ctx->output->pci->pageNo : -1);
}

CHEWING_API void chewing_cand_Enumerate( ChewingContext *ctx )
{
	ctx->cand_no = ctx->output->pci->pageNo * ctx->output->pci->nChoicePerPage;
}

CHEWING_API int chewing_cand_hasNext( ChewingContext *ctx )
{
	return (ctx->cand_no < ctx->output->pci->nTotalChoice);
}

CHEWING_API const char *chewing_cand_String_static( ChewingContext *ctx )
{
	char *s;
	if ( chewing_cand_hasNext( ctx ) ) {
		s = ctx->output->pci->totalChoiceStr[ ctx->cand_no ];
		ctx->cand_no++;
	} else {
		s = "";
	}
	return s;
}

CHEWING_API char *chewing_cand_String( ChewingContext *ctx )
{
	return strdup(chewing_cand_String_static(ctx));
}

CHEWING_API void chewing_interval_Enumerate( ChewingContext *ctx )
{
	ctx->it_no = 0;
}

CHEWING_API int chewing_interval_hasNext( ChewingContext *ctx )
{
	return (ctx->it_no < ctx->output->nDispInterval);
}

CHEWING_API void chewing_interval_Get( ChewingContext *ctx, IntervalType *it )
{
	if ( chewing_interval_hasNext( ctx ) ) {
		if ( it ) {
			it->from = ctx->output->dispInterval[ ctx->it_no ].from;
			it->to = ctx->output->dispInterval[ ctx->it_no ].to;
		}
		ctx->it_no++;
	}
}

CHEWING_API int chewing_aux_Check( ChewingContext *ctx )
{
	return (ctx->data->bShowMsg);
}

CHEWING_API int chewing_aux_Length( ChewingContext *ctx )
{
	return (ctx->data->bShowMsg ? ctx->data->showMsgLen : 0);
}

CHEWING_API char *chewing_aux_String( ChewingContext *ctx )
{
	return strdup( ctx->data->showMsg );
}

CHEWING_API int chewing_keystroke_CheckIgnore( ChewingContext *ctx )
{
	return !!(ctx->output->keystrokeRtn & KEYSTROKE_IGNORE);
}

CHEWING_API int chewing_keystroke_CheckAbsorb( ChewingContext *ctx )
{
	return !!(ctx->output->keystrokeRtn & KEYSTROKE_ABSORB);
}

CHEWING_API int chewing_kbtype_Total( ChewingContext *ctx UNUSED )
{
	return KB_TYPE_NUM;
}

CHEWING_API void chewing_kbtype_Enumerate( ChewingContext *ctx )
{
	ctx->kb_no = 0;
}

CHEWING_API int chewing_kbtype_hasNext( ChewingContext *ctx )
{
	return ctx->kb_no < KB_TYPE_NUM;
}

extern const char * const kb_type_str[];

CHEWING_API const char *chewing_kbtype_String_static( ChewingContext *ctx )
{
	char *s;
	if ( chewing_kbtype_hasNext( ctx ) ) {
		s = (char *)kb_type_str[ ctx->kb_no ];
		ctx->kb_no++;
	}
	else {
		s =  "";
	}
	return s;
}

CHEWING_API char *chewing_kbtype_String( ChewingContext *ctx )
{
	return strdup(chewing_kbtype_String_static(ctx));
}

