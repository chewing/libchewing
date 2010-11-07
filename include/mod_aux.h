/*
 * mod_aux.c
 *
 * Copyright (c) 2005, 2008
 *	libchewing Core Team. See ChangeLog for details.
 *
 * See the file "COPYING" for information on usage and redistribution
 * of this file.
 */

#ifndef CHEWING_MOD_AUX_H
#define CHEWING_MOD_AUX_H

/*! \file mod_aux.h
 *  \brief Auxiliary module
 *  \author libchewing Core Team
 */

#include "global.h"

/**
 * @brief Chewing the state for input context during commit process
 * @param ctx handle to Chewing IM context
 * @retval TRUE if it currnet input state is at the "end-of-a-char"
 */
CHEWING_API int chewing_commit_Check( ChewingContext *ctx );

/**
 * @brief Get current commit string regardless of current input state
 * @param ctx handle to Chewing IM context
 *
 * Alwasy returns a char pointer, caller must free it.
 */
CHEWING_API char *chewing_commit_String( ChewingContext *ctx );


/*! \name Preedit string buffer
 */

/*@{*/
CHEWING_API char *chewing_buffer_String( ChewingContext *ctx );
CHEWING_API int chewing_buffer_Check( ChewingContext *ctx );
CHEWING_API int chewing_buffer_Len( ChewingContext *ctx );
/*@}*/


/*@{*/
/**
 * @param ctx handle to Chewing IM context
 * @param[out] zuin_count pointer to the integer of available Zuin preedit string
 *
 * Always returns a C-style string (char pointer), caller must free it.
 */
CHEWING_API char *chewing_zuin_String( ChewingContext *ctx, int *zuin_count );

CHEWING_API int chewing_zuin_Check( ChewingContext *ctx );
/*@}*/

CHEWING_API int chewing_cursor_Current( ChewingContext *ctx );

/*@{*/
CHEWING_API int chewing_cand_CheckDone( ChewingContext *ctx );
CHEWING_API int chewing_cand_TotalPage( ChewingContext *ctx );
CHEWING_API int chewing_cand_ChoicePerPage( ChewingContext *ctx );
CHEWING_API int chewing_cand_TotalChoice( ChewingContext *ctx );
CHEWING_API int chewing_cand_CurrentPage( ChewingContext *ctx );
CHEWING_API void chewing_cand_Enumerate( ChewingContext *ctx );
CHEWING_API int chewing_cand_hasNext( ChewingContext *ctx );
CHEWING_API char *chewing_cand_String( ChewingContext *ctx );
/*@}*/


/*@{*/
CHEWING_API void chewing_interval_Enumerate( ChewingContext *ctx );
CHEWING_API int chewing_interval_hasNext( ChewingContext *ctx );
CHEWING_API void chewing_interval_Get( ChewingContext *ctx, IntervalType *it );
/*@}*/

/*@{*/
CHEWING_API int chewing_aux_Check( ChewingContext *ctx );
CHEWING_API int chewing_aux_Length( ChewingContext *ctx );
CHEWING_API char *chewing_aux_String( ChewingContext *ctx );
/*@}*/


/*@{*/
CHEWING_API int chewing_keystroke_CheckIgnore( ChewingContext *ctx );
CHEWING_API int chewing_keystroke_CheckAbsorb( ChewingContext *ctx );
/*@}*/


/*@{*/
CHEWING_API int chewing_kbtype_Total( ChewingContext *ctx );
CHEWING_API void chewing_kbtype_Enumerate( ChewingContext *ctx );
CHEWING_API int chewing_kbtype_hasNext( ChewingContext *ctx );
CHEWING_API char *chewing_kbtype_String( ChewingContext *ctx );
/*@}*/

#endif /* CHEWING_MOD_AUX_H */
