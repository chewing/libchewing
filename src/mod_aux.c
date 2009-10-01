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
#include "private.h"

/**
 * @param ctx handle to Chewing IM context
 * @retval TRUE if it currnet input state is at the "end-of-a-char"
 */
CHEWING_API int chewing_commit_Check( ChewingContext *ctx )
{
	return (ctx->output->keystrokeRtn & KEYSTROKE_COMMIT);
}

/**
 * @param ctx handle to Chewing IM context
 *
 * retrun current commit string, regardless current input state.
 * Alwasy returns a char pointer, caller must free it.
 */
CHEWING_API char *chewing_commit_String( ChewingContext *ctx )
{
	int i;
	char *s = (char *) calloc(
		1 + ctx->output->nCommitStr,
		sizeof(char) * MAX_UTF8_SIZE );
	for ( i = 0; i < ctx->output->nCommitStr; i++ ) {
		strcat( s, (char *) (ctx->output->commitStr[ i ].s) );
	}
	return s;
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
	int i;
	char *s = (char *) calloc(
		1 + ctx->output->chiSymbolBufLen,
		sizeof(char) * MAX_UTF8_SIZE );
	for ( i = 0; i < ctx->output->chiSymbolBufLen; i++ ) {
		strcat( s, (char *) (ctx->output->chiSymbolBuf[ i ].s) );
	}
	return s;
}

/**
 * @param ctx handle to Chewing IM context
 * @param zuin_count pointer to the integer of available Zuin preedit string
 *
 * Always returns a char pointer, caller must free it.
 */
CHEWING_API char *chewing_zuin_String( ChewingContext *ctx, int *zuin_count )
{
	char *s;
	int i;
	if ( zuin_count )
		*zuin_count = 0;
	s = (char*) calloc(
		1 + ZUIN_SIZE,
		sizeof(char) * WCH_SIZE );
	for ( i = 0; i < ZUIN_SIZE; i++ ) {
		if ( ctx->output->zuinBuf[ i ].s[ 0 ] != '\0' ) {
			strcat( s, (char *) (ctx->output->zuinBuf[ i ].s) );
			if ( zuin_count )
				(*zuin_count)++;
		}
	}
	return s;
}

CHEWING_API int chewing_zuin_Check( ChewingContext *ctx )
{
	int ret = 0;
	if ( ctx->output->zuinBuf[ 0 ].s[ 0 ] == '\0' ) {
		ret = 1;
	}
	return ret;
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

CHEWING_API char *chewing_cand_String( ChewingContext *ctx )
{
	char *s;
	if ( chewing_cand_hasNext( ctx ) ||
	     (ctx->cand_no < ctx->output->pci->nTotalChoice) ) {
		s = strdup( ctx->output->pci->totalChoiceStr[ ctx->cand_no ] );
		ctx->cand_no++;
	} else {
		s = strdup( "" );
	}
	return s;
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
	return (ctx->output->bShowMsg);
}

CHEWING_API int chewing_aux_Length( ChewingContext *ctx )
{
	return (ctx->output->bShowMsg ? ctx->output->showMsgLen : 0);
}

CHEWING_API char *chewing_aux_String( ChewingContext *ctx )
{
	int i;
	char *msg = (char *) calloc(
		1 + ctx->output->showMsgLen,
		sizeof(char) * MAX_UTF8_SIZE );
	for ( i = 0; i < ctx->output->showMsgLen; ++i )
		strcat( msg, (char *)(ctx->output->showMsg[ i ].s) );
	return msg;

}

CHEWING_API int chewing_keystroke_CheckIgnore( ChewingContext *ctx )
{ 
	return (ctx->output->keystrokeRtn & KEYSTROKE_IGNORE);
} 

CHEWING_API int chewing_keystroke_CheckAbsorb( ChewingContext *ctx )
{ 
	return (ctx->output->keystrokeRtn & KEYSTROKE_ABSORB);
}

CHEWING_API int chewing_kbtype_Total( ChewingContext *ctx )
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

extern char *kb_type_str[];

CHEWING_API char *chewing_kbtype_String( ChewingContext *ctx )
{
	char *s;
	if ( chewing_kbtype_hasNext( ctx ) ) {
		s = strdup( kb_type_str[ ctx->kb_no ] );
		ctx->kb_no++;
	}
	else {
		s = strdup( "" );
	}
	return s;
}
