/**
 * chewingio.c
 *
 * Copyright (c) 1999, 2000, 2001
 *	Lu-chuan Kung and Kang-pen Chen.
 *	All rights reserved.
 *
 * Copyright (c) 2004-2008, 2010, 2011, 2012
 *	libchewing Core Team. See ChangeLog for details.
 *
 * See the file "COPYING" for information on usage and redistribution
 * of this file.
 */

/**
 * @file chewingio.c
 * @brief Implement basic I/O routines for Chewing manipulation.
 */

#include <assert.h>
#include <string.h>
#include <ctype.h>
#include <stdlib.h>
#include <stdio.h>

#include "chewing-utf8-util.h"
#include "global.h"
#include "zuin-private.h"
#include "chewingutil.h"
#include "userphrase-private.h"
#include "choice-private.h"
#include "dict-private.h"
#include "char-private.h"
#include "hash-private.h"
#include "tree-private.h"
#include "pinyin-private.h"
#include "private.h"
#include "chewingio.h"
#include "mod_aux.h"
#include "global-private.h"
#include "plat_path.h"
#include "chewing-private.h"

const char * const kb_type_str[] = {
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

const char * const CHAR_FILES[] = {
	CHAR_FILE,
#ifdef USE_BINARY_DATA
	CHAR_INDEX_BEGIN_FILE,
	CHAR_INDEX_PHONE_FILE,
#else
	CHAR_INDEX_FILE,
#endif
	NULL,
};

const char * const DICT_FILES[] = {
	DICT_FILE,
	PH_INDEX_FILE,
	PHONE_TREE_FILE,
	NULL,
};

const char * const SYMBOL_TABLE_FILES[] = {
	SYMBOL_TABLE_FILE,
	NULL,
};

const char * const EASY_SYMBOL_FILES[] = {
	SOFTKBD_TABLE_FILE,
	NULL,
};

const char * const PINYIN_FILES[] = {
	PINYIN_TAB_NAME,
	NULL,
};

CHEWING_API int chewing_KBStr2Num( char str[] )
{
	int i;

	STATIC_ASSERT( KB_TYPE_NUM == ARRAY_SIZE( kb_type_str ), kb_type_str_needs_update);
	for ( i = 0; i < KB_TYPE_NUM; i++) {
		if ( ! strcmp( str, kb_type_str[ i ] ) )
			return i;
	}
	return KB_DEFAULT;
}

static void chooseCandidate( ChewingContext *ctx, int toSelect, int key_buf_cursor )
{
	ChewingData *pgdata = ctx->data;
	if ( toSelect ) {
		if ( ! pgdata->bSelect ) {
			ChoiceFirstAvail( pgdata );
		} else {
			if ( pgdata->config.bPhraseChoiceRearward ) {
				int avail_willbe = (pgdata->availInfo.currentAvail > 0) ?
					pgdata->availInfo.currentAvail - 1 :
					pgdata->availInfo.nAvail - 1;
				pgdata->chiSymbolCursor = pgdata->choiceInfo.oldChiSymbolCursor -
					pgdata->availInfo.avail[ avail_willbe ].len;
				if ( chewing_buffer_Len( ctx ) >
						pgdata->choiceInfo.oldChiSymbolCursor ) {
					pgdata->chiSymbolCursor++;
				}
			}
			ChoiceNextAvail( pgdata );
		}
	} else if ( pgdata->symbolKeyBuf[ key_buf_cursor ] ) {
		/* Open Symbol Choice List */
		if ( pgdata->choiceInfo.isSymbol == WORD_CHOICE )
			OpenSymbolChoice( pgdata );
	}
}

static void NullLogger( void *data UNUSED, int level UNUSED, const char *fmt UNUSED, ...)
{
}

static ChewingData * allocate_ChewingData()
{
	static const int DEFAULT_SELKEY[] = { '1', '2', '3', '4', '5', '6', '7', '8', '9', '0' };

	ChewingData *data = ALC( ChewingData, 1 );
	if ( data ) {
		data->config.candPerPage = MAX_SELKEY;
		data->config.maxChiSymbolLen = MAX_CHI_SYMBOL_LEN;
		data->logger = NullLogger;
		memcpy( data->config.selKey, DEFAULT_SELKEY, sizeof( data->config.selKey ) );
	}

	return data;
}

CHEWING_API ChewingContext *chewing_new()
{
	ChewingContext *ctx;
	int ret;
	char search_path[PATH_MAX];
	char path[PATH_MAX];

	ctx = ALC( ChewingContext, 1 );
	if ( !ctx )
		goto error;

	ctx->output = ALC ( ChewingOutput, 1 );
	if ( !ctx->output )
		goto error;

	ctx->data = allocate_ChewingData();
	if ( !ctx->data )
		goto error;

	chewing_Reset( ctx );

	ret = get_search_path( search_path, sizeof( search_path ) );
	if ( ret )
		goto error;

	ret = find_path_by_files(
		search_path, CHAR_FILES, path, sizeof( path ) );
	if ( ret )
		goto error;
	ret = InitChar( ctx->data, path );
	if ( ret )
		goto error;

	ret = find_path_by_files(
		search_path, DICT_FILES, path, sizeof( path ) );
	if ( ret )
		goto error;
	ret = InitDict( ctx->data, path );
	if ( ret )
		goto error;
	ret = InitTree( ctx->data, path );
	if ( ret )
		goto error;

	ret = InitHash( ctx->data );
	if ( !ret )
		goto error;

	ctx->cand_no = 0;

	ret = find_path_by_files(
		search_path, SYMBOL_TABLE_FILES, path, sizeof( path ) );
	if ( ret )
		goto error;
	ret = InitSymbolTable( ctx->data, path );
	if ( ret )
		goto error;

	ret = find_path_by_files(
		search_path, EASY_SYMBOL_FILES, path, sizeof( path ) );
	if ( ret )
		goto error;
	ret = InitEasySymbolInput( ctx->data, path );
	if ( ret )
		goto error;

	ret = find_path_by_files(
		search_path, PINYIN_FILES, path, sizeof( path ) );
	if ( ret )
		goto error;
	ret = InitPinyin( ctx->data, path );
	if ( !ret )
		goto error;

	return ctx;
error:
	chewing_delete( ctx );
	return NULL;
}

CHEWING_API int chewing_Init(
		const char *dataPath UNUSED,
		const char *hashPath UNUSED)
{
	return 0;
}

CHEWING_API int chewing_Reset( ChewingContext *ctx )
{
	ChewingData *pgdata = ctx->data;
	ChewingStaticData static_data;
	ChewingConfigData old_config;
	void (*logger)( void *data, int level, const char *fmt, ...);

	/* Backup old config and restore it after clearing pgdata structure. */
	old_config = pgdata->config;
	static_data = pgdata->static_data;
	logger = pgdata->logger;
	memset( pgdata, 0, sizeof( ChewingData ) );
	pgdata->config = old_config;
	pgdata->static_data = static_data;
	pgdata->logger = logger;

	/* zuinData */
	memset( &( pgdata->zuinData ), 0, sizeof( ZuinData ) );

	/* choiceInfo */
	memset( &( pgdata->choiceInfo ), 0, sizeof( ChoiceInfo ) );

	pgdata->chiSymbolCursor = 0;
	pgdata->chiSymbolBufLen = 0;
	pgdata->nPhoneSeq = 0;
	memset( pgdata->bUserArrCnnct, 0, sizeof( int ) * ( MAX_PHONE_SEQ_LEN + 1 ) );
	memset( pgdata->bUserArrBrkpt, 0, sizeof( int ) * ( MAX_PHONE_SEQ_LEN + 1 ) );
	pgdata->bChiSym = CHINESE_MODE;
	pgdata->bFullShape = HALFSHAPE_MODE;
	pgdata->bSelect = 0;
	pgdata->nSelect = 0;
	pgdata->PointStart = -1;
	pgdata->PointEnd = 0;
	pgdata->phrOut.nNumCut = 0;
	return 0;
}

CHEWING_API int chewing_set_KBType( ChewingContext *ctx, int kbtype )
{
	if ( kbtype < KB_TYPE_NUM && kbtype >= 0  ) {
		ctx->data->zuinData.kbtype = kbtype;
		return 0;
	} else {
		ctx->data->zuinData.kbtype = KB_DEFAULT;
		return -1;
	}
}

CHEWING_API int chewing_get_KBType( ChewingContext *ctx )
{
	return ctx->data->zuinData.kbtype;
}

CHEWING_API char* chewing_get_KBString( ChewingContext *ctx )
{
	return strdup( kb_type_str[ ctx->data->zuinData.kbtype ] );
}

CHEWING_API void chewing_Terminate()
{
}

CHEWING_API void chewing_delete( ChewingContext *ctx )
{
	if ( ctx ) {
		if ( ctx->data ) {
			TerminatePinyin( ctx->data );
			TerminateEasySymbolTable( ctx->data );
			TerminateSymbolTable( ctx->data );
			TerminateHash( ctx->data );
			TerminateTree( ctx->data );
			TerminateDict( ctx->data );
			TerminateChar( ctx->data );
			free( ctx->data );
		}

		if ( ctx->output )
			free( ctx->output);
		free( ctx );
	}
	return;
}

CHEWING_API void chewing_free( void *p )
{
	if ( p )
		free( p );
	return;
}

CHEWING_API int chewing_Configure( ChewingContext *ctx, ChewingConfigData *pcd )
{
	chewing_set_candPerPage( ctx, pcd->candPerPage );
	chewing_set_maxChiSymbolLen( ctx, pcd->maxChiSymbolLen );
	chewing_set_selKey( ctx, pcd->selKey, MAX_SELKEY );
	chewing_set_addPhraseDirection( ctx, pcd->bAddPhraseForward );
	chewing_set_spaceAsSelection( ctx, pcd->bSpaceAsSelection );
	chewing_set_escCleanAllBuf( ctx, pcd->bEscCleanAllBuf );
	chewing_set_autoShiftCur( ctx, pcd->bAutoShiftCur );
	chewing_set_easySymbolInput( ctx, pcd->bEasySymbolInput );
	chewing_set_phraseChoiceRearward( ctx, pcd->bPhraseChoiceRearward );

	return 0;
}

CHEWING_API void chewing_set_candPerPage( ChewingContext *ctx, int n )
{
	if ( MIN_SELKEY <= n && n <= MAX_SELKEY )
		ctx->data->config.candPerPage = n;
}

CHEWING_API int chewing_get_candPerPage( ChewingContext *ctx )
{
	return ctx->data->config.candPerPage;
}

CHEWING_API void chewing_set_maxChiSymbolLen( ChewingContext *ctx, int n )
{
	if ( MIN_CHI_SYMBOL_LEN <= n && n <= MAX_CHI_SYMBOL_LEN )
		ctx->data->config.maxChiSymbolLen = n;
}

CHEWING_API int chewing_get_maxChiSymbolLen( ChewingContext *ctx )
{
	return ctx->data->config.maxChiSymbolLen;
}

CHEWING_API void chewing_set_selKey( ChewingContext *ctx, int *selkeys,
                                     int len UNUSED)
{
	memcpy(
		ctx->data->config.selKey,
		selkeys,
		sizeof( selkeys[ 0 ] ) * MAX_SELKEY );
}

CHEWING_API int* chewing_get_selKey( ChewingContext *ctx )
{
	int *selkeys = ALC( int , MAX_SELKEY );
	if ( selkeys ) {
		memcpy( selkeys, ctx->data->config.selKey,
			sizeof( *selkeys ) * MAX_SELKEY );
	}
	return selkeys;
}

CHEWING_API void chewing_set_addPhraseDirection( ChewingContext *ctx, int direction )
{
	if ( direction == 0 || direction == 1 )
		ctx->data->config.bAddPhraseForward = direction;
}

CHEWING_API int chewing_get_addPhraseDirection( ChewingContext *ctx )
{
	return ctx->data->config.bAddPhraseForward;
}

CHEWING_API void chewing_set_spaceAsSelection( ChewingContext *ctx, int mode )
{
	if ( mode == 0 || mode == 1 )
		ctx->data->config.bSpaceAsSelection = mode;
}

CHEWING_API int chewing_get_spaceAsSelection( ChewingContext *ctx )
{
	return ctx->data->config.bSpaceAsSelection;
}

CHEWING_API void chewing_set_escCleanAllBuf( ChewingContext *ctx, int mode )
{
	if ( mode == 0 || mode == 1 )
		ctx->data->config.bEscCleanAllBuf = mode;
}

CHEWING_API int chewing_get_escCleanAllBuf( ChewingContext *ctx )
{
	return ctx->data->config.bEscCleanAllBuf;
}

CHEWING_API void chewing_set_hsuSelKeyType( ChewingContext *ctx, int mode )
{
	// XXX: This function is deprecated. No one read hsuSelKeyType.
	ctx->data->config.hsuSelKeyType = mode;
}

CHEWING_API int chewing_get_hsuSelKeyType( ChewingContext *ctx )
{
	// XXX: This function is deprecated. No one read hsuSelKeyType.
	return ctx->data->config.hsuSelKeyType;
}

CHEWING_API void chewing_set_autoShiftCur( ChewingContext *ctx, int mode )
{
	if ( mode == 0 || mode == 1 )
		ctx->data->config.bAutoShiftCur = mode;
}

CHEWING_API int chewing_get_autoShiftCur( ChewingContext *ctx )
{
	return ctx->data->config.bAutoShiftCur;
}

CHEWING_API void chewing_set_easySymbolInput( ChewingContext *ctx, int mode )
{
	if ( mode == 0 || mode == 1 )
		ctx->data->config.bEasySymbolInput = mode;
}

CHEWING_API int chewing_get_easySymbolInput( ChewingContext *ctx )
{
	return ctx->data->config.bEasySymbolInput;
}

CHEWING_API void chewing_set_phraseChoiceRearward( ChewingContext *ctx, int mode )
{
	if ( mode == 0 || mode == 1 )
		ctx->data->config.bPhraseChoiceRearward = mode;
}

CHEWING_API int chewing_get_phraseChoiceRearward( ChewingContext *ctx )
{
	return ctx->data->config.bPhraseChoiceRearward;
}

CHEWING_API void chewing_set_ChiEngMode( ChewingContext *ctx, int mode )
{
	if ( mode == CHINESE_MODE || mode == SYMBOL_MODE )
		ctx->data->bChiSym = mode;
}

CHEWING_API int chewing_get_ChiEngMode( ChewingContext *ctx )
{
	return ctx->data->bChiSym;
}

CHEWING_API void chewing_set_ShapeMode( ChewingContext *ctx, int mode )
{
	if ( mode == HALFSHAPE_MODE || mode == FULLSHAPE_MODE )
		ctx->data->bFullShape = mode;
}

CHEWING_API int chewing_get_ShapeMode( ChewingContext *ctx )
{
	return ctx->data->bFullShape;
}

static void CheckAndResetRange( ChewingData *pgdata )
{
	if ( pgdata->PointStart > -1 ) {
		pgdata->PointStart = -1;
		pgdata->PointEnd = 0;
	}
}

static int DoSelect( ChewingData *pgdata, int num )
{
	assert( pgdata->choiceInfo.pageNo >= 0 );
	if ( num >= 0 ) {
		num += pgdata->choiceInfo.pageNo * pgdata->choiceInfo.nChoicePerPage;
		/* Note: if num is larger than the total, there will be big troubles. */
		if ( num < pgdata->choiceInfo.nTotalChoice ) {
			if ( pgdata->choiceInfo.isSymbol != WORD_CHOICE ) {
				SymbolChoice( pgdata, num );
			}
			else {
				/* change the select interval & selectStr & nSelect */
				AddSelect( pgdata, num );
				/* second, call choice module */
				ChoiceSelect( pgdata, num );
				/* automatically shift the cursor to next phrase */
				if ( pgdata->config.bAutoShiftCur != 0 &&
				     /* if cursor at end of string, do not shift the cursor. */
				     pgdata->chiSymbolCursor < pgdata->chiSymbolBufLen ) {
					if ( pgdata->config.bPhraseChoiceRearward ) {
						++pgdata->chiSymbolCursor;
					} else {
						pgdata->chiSymbolCursor +=
							pgdata->availInfo.avail[ pgdata->availInfo.currentAvail ].len;
					}
				}
			}
			return 1;
		}
	}
	return 0;
}

CHEWING_API int chewing_handle_Space( ChewingContext *ctx )
{
	ChewingData *pgdata = ctx->data;

	/*
	 * Use chewing_handle_Default( ctx, ' ' ) to handle space when:
	 * - "space as selection" mode is disable
	 * - mode is not CHINESE_MODE
	 * - has incompleted bopomofo (space is needed to complete it)
	 */
	if ( !pgdata->config.bSpaceAsSelection
	     || pgdata->bChiSym != CHINESE_MODE
	     || ZuinIsEntering( &ctx->data->zuinData ) ) {
		return chewing_handle_Default( ctx, ' ' );
	}

	CheckAndResetRange( pgdata );

	if ( pgdata->bSelect ) {
		return chewing_handle_Right( ctx );
	} else {
		return chewing_handle_Down( ctx );
	}
	return 0;
}

CHEWING_API int chewing_handle_Esc( ChewingContext *ctx )
{
	ChewingData *pgdata = ctx->data;
	ChewingOutput *pgo = ctx->output;
	int keystrokeRtn = KEYSTROKE_ABSORB;

	CheckAndResetRange( pgdata );

	if ( ! ChewingIsEntering( pgdata ) ) {
		keystrokeRtn = KEYSTROKE_IGNORE;
	}
	else if ( pgdata->bSelect ) {
		ChoiceEndChoice( pgdata );
	}
	else if ( ZuinIsEntering( &( pgdata->zuinData ) ) ) {
		ZuinRemoveAll( &( pgdata->zuinData ) );
	}
	else if ( pgdata->config.bEscCleanAllBuf ) {
		CleanAllBuf( pgdata );
		pgo->nCommitStr = pgdata->chiSymbolBufLen;
	}

	MakeOutputWithRtn( pgo, pgdata, keystrokeRtn );
	return 0;
}

CHEWING_API int chewing_handle_Enter( ChewingContext *ctx )
{
	ChewingData *pgdata = ctx->data;
	ChewingOutput *pgo = ctx->output;
	int nCommitStr = pgdata->chiSymbolBufLen;
	int keystrokeRtn = KEYSTROKE_ABSORB;

	if ( ! ChewingIsEntering( pgdata ) ) {
		keystrokeRtn = KEYSTROKE_IGNORE;
	}
	else if ( pgdata->bSelect ) {
		keystrokeRtn = KEYSTROKE_ABSORB | KEYSTROKE_BELL;
	}
	else if ( pgdata->PointStart > -1 ) {
		int buf = pgdata->chiSymbolCursor;
		int key;
		if ( pgdata->PointEnd > 1 ) {
			if ( ! pgdata->config.bAddPhraseForward ) {
				pgdata->chiSymbolCursor = pgdata->PointStart;
				key = '0' + pgdata->PointEnd;
			}
			else {
				pgdata->chiSymbolCursor = pgdata->PointStart + pgdata->PointEnd;
				key = '0' + pgdata->PointEnd;
			}

			chewing_handle_CtrlNum( ctx, key );
			pgdata->chiSymbolCursor = buf;
		}
		else if ( pgdata->PointEnd < 1 ) {
			if ( pgdata->config.bAddPhraseForward )
				pgdata->chiSymbolCursor = buf - pgdata->PointEnd;
			key = '0' - pgdata->PointEnd;
			chewing_handle_CtrlNum( ctx, key );
			pgdata->chiSymbolCursor = buf;
		}
		pgdata->PointStart = -1;
		pgdata->PointEnd = 0;
	}
	else {
		keystrokeRtn = KEYSTROKE_COMMIT;
		WriteChiSymbolToBuf( pgo->commitStr, nCommitStr, pgdata );
		AutoLearnPhrase( pgdata );
		CleanAllBuf( pgdata );
		pgo->nCommitStr = nCommitStr;
	}

	MakeOutputWithRtn( pgo, pgdata, keystrokeRtn );
	return 0;
}

CHEWING_API int chewing_handle_Del( ChewingContext *ctx )
{
	ChewingData *pgdata = ctx->data;
	ChewingOutput *pgo = ctx->output;
	int keystrokeRtn = KEYSTROKE_ABSORB;

	CheckAndResetRange( pgdata );

	if ( ! ChewingIsEntering( pgdata ) ) {
		keystrokeRtn = KEYSTROKE_IGNORE;
	}

	if ( ! pgdata->bSelect ) {
		if (
			! ZuinIsEntering( &( pgdata->zuinData ) ) &&
			pgdata->chiSymbolCursor < pgdata->chiSymbolBufLen ) {
			ChewingKillChar(
				pgdata,
				pgdata->chiSymbolCursor,
				NONDECREASE_CURSOR );
		}
		CallPhrasing( pgdata );
	}
	MakeOutputWithRtn( pgo, pgdata, keystrokeRtn );
	return 0;
}

CHEWING_API int chewing_handle_Backspace( ChewingContext *ctx )
{
	ChewingData *pgdata = ctx->data;
	ChewingOutput *pgo = ctx->output;
	int keystrokeRtn = KEYSTROKE_ABSORB;

	CheckAndResetRange( pgdata );

	if ( ! ChewingIsEntering( pgdata ) ) {
		keystrokeRtn = KEYSTROKE_IGNORE;
	}

	if ( ! pgdata->bSelect ) {
		if ( ZuinIsEntering( &( pgdata->zuinData ) ) ) {
			ZuinRemoveLast( &( pgdata->zuinData ) );
		}
		else if ( pgdata->chiSymbolCursor > 0 ) {
			ChewingKillChar(
				pgdata,
				pgdata->chiSymbolCursor - 1,
				DECREASE_CURSOR );
		}
		CallPhrasing( pgdata );
	}
	MakeOutputWithRtn( pgo, pgdata, keystrokeRtn );

	return 0;
}

CHEWING_API int chewing_handle_Up( ChewingContext *ctx )
{
	ChewingData *pgdata = ctx->data;
	ChewingOutput *pgo = ctx->output;
	int keystrokeRtn = KEYSTROKE_ABSORB;
	int key_buf_cursor;

	CheckAndResetRange( pgdata );

	if ( ! ChewingIsEntering( pgdata ) ) {
		keystrokeRtn = KEYSTROKE_IGNORE;
	}

	key_buf_cursor = pgdata->chiSymbolCursor;
	// FIXME: when pgdata->chiSymbolBufLen == 0, key_buf_cursor will be -1.
	if ( pgdata->chiSymbolCursor == pgdata->chiSymbolBufLen )
		key_buf_cursor--;

	// XXX: Why close symbol choice list, but not word choice list.
	if ( ! pgdata->symbolKeyBuf[ key_buf_cursor ] ) {
		/* Close Symbol Choice List */
		chewing_handle_Esc(ctx);
	}

	MakeOutputWithRtn( pgo, pgdata, keystrokeRtn );
	return 0;
}

CHEWING_API int chewing_handle_Down( ChewingContext *ctx )
{
	ChewingData *pgdata = ctx->data;
	ChewingOutput *pgo = ctx->output;
	int toSelect = 0;
	int keystrokeRtn = KEYSTROKE_ABSORB;
	int key_buf_cursor;

	CheckAndResetRange( pgdata );

	if ( ! ChewingIsEntering( pgdata ) ) {
		keystrokeRtn = KEYSTROKE_IGNORE;
	}

	key_buf_cursor = pgdata->chiSymbolCursor;
	// FIXME: when pgdata->chiSymbolBufLen == 0, key_buf_cursor will be -1.
	if ( pgdata->chiSymbolCursor == pgdata->chiSymbolBufLen )
		key_buf_cursor--;

	/* see if to select */
	if ( ChewingIsChiAt( key_buf_cursor, pgdata ) )
			toSelect = 1;

	chooseCandidate( ctx, toSelect, key_buf_cursor );

	MakeOutputWithRtn( pgo, pgdata, keystrokeRtn );
	return 0;
}

/* Add phrase in Hanin Style */
CHEWING_API int chewing_handle_ShiftLeft( ChewingContext *ctx )
{
	ChewingData *pgdata = ctx->data;
	ChewingOutput *pgo = ctx->output;
	int keystrokeRtn = KEYSTROKE_ABSORB;

	if ( ! ChewingIsEntering( pgdata ) ) {
		keystrokeRtn = KEYSTROKE_IGNORE;
	}
	if ( ! pgdata->bSelect ) {
		/*  PointEnd locates (-9, +9) */
		if (
			! ZuinIsEntering( &( pgdata->zuinData ) ) &&
			pgdata->chiSymbolCursor > 0 &&
			pgdata->PointEnd > -9 ) {
			if ( pgdata->PointStart == -1 )
				pgdata->PointStart = pgdata->chiSymbolCursor;
			pgdata->chiSymbolCursor--;
			if (
				ChewingIsChiAt( pgdata->chiSymbolCursor, pgdata ) ) {
				pgdata->PointEnd--;
			}
			if ( pgdata->PointEnd == 0 )
				pgdata->PointStart = -1;
		}
	}

	MakeOutputWithRtn( pgo, pgdata, keystrokeRtn );
	return 0;
}

CHEWING_API int chewing_handle_Left( ChewingContext *ctx )
{
	ChewingData *pgdata = ctx->data;
	ChewingOutput *pgo = ctx->output;
	int keystrokeRtn = KEYSTROKE_ABSORB;

	if ( ! ChewingIsEntering( pgdata ) ) {
		keystrokeRtn = KEYSTROKE_IGNORE;
	}

	if ( pgdata->bSelect ) {
		assert( pgdata->choiceInfo.nPage > 0 );
		if ( pgdata->choiceInfo.pageNo > 0 )
			pgdata->choiceInfo.pageNo--;
		else
			pgdata->choiceInfo.pageNo = pgdata->choiceInfo.nPage - 1;
	}
	else {
		if (
			! ZuinIsEntering( &( pgdata->zuinData ) ) &&
			pgdata->chiSymbolCursor > 0 ) {
			CheckAndResetRange( pgdata );
			pgdata->chiSymbolCursor--;
		}
	}
	MakeOutputWithRtn( pgo, pgdata, keystrokeRtn );
	return 0;
}

/* Add phrase in Hanin Style */
CHEWING_API int chewing_handle_ShiftRight( ChewingContext *ctx )
{
	ChewingData *pgdata = ctx->data;
	ChewingOutput *pgo = ctx->output;
	int keystrokeRtn = KEYSTROKE_ABSORB;

	if ( ! ChewingIsEntering( pgdata ) ) {
		keystrokeRtn = KEYSTROKE_IGNORE;
	}

	if ( ! pgdata->bSelect ) {
		/* PointEnd locates (-9, +9) */
		if (
			! ZuinIsEntering( &( pgdata->zuinData ) ) &&
			pgdata->chiSymbolCursor < pgdata->chiSymbolBufLen &&
			pgdata->PointEnd < 9 ) {
			if ( pgdata->PointStart == -1 )
				pgdata->PointStart = pgdata->chiSymbolCursor;
			if (
				ChewingIsChiAt( pgdata->chiSymbolCursor, pgdata ) ) {
				pgdata->PointEnd++;
			}
			pgdata->chiSymbolCursor++;
			if ( pgdata->PointEnd == 0 )
				pgdata->PointStart = -1;
		}
	}

	MakeOutputWithRtn( pgo, pgdata, keystrokeRtn );
	return 0;
}

CHEWING_API int chewing_handle_Right( ChewingContext *ctx )
{
	ChewingData *pgdata = ctx->data;
	ChewingOutput *pgo = ctx->output;
	int keystrokeRtn = KEYSTROKE_ABSORB;

	if ( ! ChewingIsEntering( pgdata ) ) {
		keystrokeRtn = KEYSTROKE_IGNORE;
	}

	if ( pgdata->bSelect ) {
		if ( pgdata->choiceInfo.pageNo < pgdata->choiceInfo.nPage - 1)
			pgdata->choiceInfo.pageNo++;
		else
			pgdata->choiceInfo.pageNo = 0;
	}
	else {
		if (
			! ZuinIsEntering( &( pgdata->zuinData ) ) &&
			pgdata->chiSymbolCursor < pgdata->chiSymbolBufLen ) {
			CheckAndResetRange( pgdata );
			pgdata->chiSymbolCursor++;
		}
	}

	MakeOutputWithRtn( pgo, pgdata, keystrokeRtn );
	return 0;
}

CHEWING_API int chewing_handle_Tab( ChewingContext *ctx )
{
	int cursor;
	ChewingData *pgdata = ctx->data;
	ChewingOutput *pgo = ctx->output;
	int keystrokeRtn = KEYSTROKE_ABSORB;

	CheckAndResetRange( pgdata );

	if ( ! ChewingIsEntering( pgdata ) ) {
		keystrokeRtn = KEYSTROKE_IGNORE;
	}


	if ( ! pgdata->bSelect ) {
		if ( pgdata->chiSymbolCursor == pgdata->chiSymbolBufLen ) {
			pgdata->phrOut.nNumCut++;
		}
		else if ( ChewingIsChiAt( pgdata->chiSymbolCursor - 1, pgdata ) ) {
			cursor = PhoneSeqCursor( pgdata );
			if ( IsPreferIntervalConnted( cursor, pgdata) ) {
				pgdata->bUserArrBrkpt[ cursor ] = 1;
				pgdata->bUserArrCnnct[ cursor ] = 0;
			}
			else {
				pgdata->bUserArrBrkpt[ cursor ] = 0;
				pgdata->bUserArrCnnct[ cursor ] = 1;
			}
		}
		CallPhrasing( pgdata );
	}
	MakeOutputWithRtn( pgo, pgdata, keystrokeRtn );
	return 0;
}

CHEWING_API int chewing_handle_DblTab( ChewingContext *ctx )
{
	int cursor;
	ChewingData *pgdata = ctx->data;
	ChewingOutput *pgo = ctx->output;
	int keystrokeRtn = KEYSTROKE_ABSORB;

	CheckAndResetRange( pgdata );

	if ( ! ChewingIsEntering( pgdata ) ) {
		keystrokeRtn = KEYSTROKE_IGNORE;
	}

	if ( ! pgdata->bSelect ) {
		cursor = PhoneSeqCursor( pgdata );
		pgdata->bUserArrBrkpt[ cursor ] = 0;
		pgdata->bUserArrCnnct[ cursor ] = 0;
	}
	CallPhrasing( pgdata );

	MakeOutputWithRtn( pgo, pgdata, keystrokeRtn );
	return 0;
}


CHEWING_API int chewing_handle_Capslock( ChewingContext *ctx )
{
	ChewingData *pgdata = ctx->data;
	ChewingOutput *pgo = ctx->output;

	pgdata->bChiSym = 1 - pgdata->bChiSym;
	MakeOutputWithRtn( pgo, pgdata, KEYSTROKE_ABSORB );
	return 0;
}

CHEWING_API int chewing_handle_Home( ChewingContext *ctx )
{
	ChewingData *pgdata = ctx->data;
	ChewingOutput *pgo = ctx->output;
	int keystrokeRtn = KEYSTROKE_ABSORB;

	CheckAndResetRange( pgdata );

	if ( ! ChewingIsEntering( pgdata ) ) {
		keystrokeRtn = KEYSTROKE_IGNORE;
	}
	else if ( ! pgdata->bSelect ) {
		pgdata->chiSymbolCursor = 0;
	}
	MakeOutputWithRtn( pgo, pgdata, keystrokeRtn );
	return 0;
}

CHEWING_API int chewing_handle_End( ChewingContext *ctx )
{
	ChewingData *pgdata = ctx->data;
	ChewingOutput *pgo = ctx->output;
	int keystrokeRtn = KEYSTROKE_ABSORB;

	CheckAndResetRange( pgdata );

	if ( ! ChewingIsEntering( pgdata ) ) {
		keystrokeRtn = KEYSTROKE_IGNORE;
	}
	else if ( ! pgdata->bSelect ) {
		pgdata->chiSymbolCursor = pgdata->chiSymbolBufLen;
	}
	MakeOutputWithRtn( pgo, pgdata, keystrokeRtn );
	return 0;
}

CHEWING_API int chewing_handle_PageUp( ChewingContext *ctx )
{
	ChewingData *pgdata = ctx->data;
	ChewingOutput *pgo = ctx->output;
	int keystrokeRtn = KEYSTROKE_ABSORB;

	CheckAndResetRange( pgdata );

	if ( ! ChewingIsEntering( pgdata ) ) {
		keystrokeRtn = KEYSTROKE_IGNORE;
	}
	else if ( ! pgdata->bSelect ) {
		pgdata->chiSymbolCursor = pgdata->chiSymbolBufLen;
	}
	MakeOutputWithRtn( pgo, pgdata, keystrokeRtn );
	return 0;
}

CHEWING_API int chewing_handle_PageDown( ChewingContext *ctx )
{
	ChewingData *pgdata = ctx->data;
	ChewingOutput *pgo = ctx->output;
	int keystrokeRtn = KEYSTROKE_ABSORB;

	CheckAndResetRange( pgdata );

	if ( ! ChewingIsEntering( pgdata ) ) {
		keystrokeRtn = KEYSTROKE_IGNORE;
	}
	else if ( ! pgdata->bSelect ) {
		pgdata->chiSymbolCursor = pgdata->chiSymbolBufLen;
	}
	MakeOutputWithRtn( pgo, pgdata, keystrokeRtn );
	return 0;
}

/* Dvorak <-> Qwerty keyboard layout converter */
static int dvorak_convert( int key )
{
	const char dkey[] = {
		'\'','\"',',','<','.','>','p','P','y','Y','f','F','g','G',
		'c','C','r','R','l','L','/','?','=','+','\\','|',
		'a','A','o','O','e','E','u','U','i','I','d','D','h','H',
		't','T','n','N','s','S','-','_',
		';',':','q','Q','j','J','k','K','x','X','b','B','m','M',
		'w','W','v','V','z','Z'};
	const char qkey[] = {
		'q','Q','w','W','e','E','r','R','t','T','y','Y','u','U',
		'i','I','o','O','p','P','[','{',']','}','\\','|',
		'a','A','s','S','d','D','f','F','g','G','h','H','j','J',
		'k','K','l','L',';',':','\'','\"',
		'z','Z','x','X','c','C','v','V','b','B','n','N','m','M',
		',','<','.','>','/','?'};
	size_t i;

	STATIC_ASSERT( ARRAY_SIZE( dkey ) == ARRAY_SIZE( qkey ), update_dkey_and_qkey );

	for ( i = 0; i < ARRAY_SIZE( dkey ); i++ ) {
		if ( key == qkey[ i ] ) {
			key = dkey[ i ];
			return key;
		}
	}
	return key;
}

CHEWING_API int chewing_handle_Default( ChewingContext *ctx, int key )
{
	ChewingData *pgdata = ctx->data;
	ChewingOutput *pgo = ctx->output;
	int rtn, num;
	int keystrokeRtn = KEYSTROKE_ABSORB;
	int bQuickCommit = 0;

	/* Update lifetime */
	ctx->data->static_data.chewing_lifetime++;

	/* Skip the special key */
	if ( key & 0xFF00 ) {
		keystrokeRtn = KEYSTROKE_IGNORE;
		goto End_KeyDefault;
	}

	/* We ignore non-printable input */
	if ( ! isprint( key ) )
		goto End_KeyDefault;

	CheckAndResetRange( pgdata );

	DEBUG_CHECKPOINT();
	DEBUG_OUT( "   key=%d", key );

	/* Dvorak Hsu */
	if ( pgdata->zuinData.kbtype == KB_DVORAK_HSU ) {
		key = dvorak_convert( key );
	}

	/* selecting */
	if ( pgdata->bSelect ) {
		if ( key == ' ' )
			return chewing_handle_Right( ctx );
		/* num starts from 0 */
		num = CountSelKeyNum( key, pgdata );
		if ( num >= 0 ) {
			DoSelect( pgdata, num );
			goto End_keyproc;
		}

		/* Otherwise, use 'j' and 'k' for paging in selection mode */
		DEBUG_OUT(
			"\t\tchecking paging key, got '%c'\n",
			key );
		switch ( key ) {
			case 'j':
			case 'J':
				if ( pgdata->chiSymbolCursor > 0 ) {
					if ( ! ChewingIsEntering( pgdata ) ) {
						keystrokeRtn = KEYSTROKE_IGNORE;
					}
					CheckAndResetRange( pgdata );
					pgdata->chiSymbolCursor--;
					if ( ChewingIsChiAt( pgdata->chiSymbolCursor, pgdata ) )
						ChoiceFirstAvail( pgdata );
					else
						OpenSymbolChoice( pgdata );

				}
				goto End_Paging;
			case 'k':
			case 'K':
				if ( pgdata->chiSymbolCursor < pgdata->chiSymbolBufLen ) {
					if ( ! ChewingIsEntering( pgdata ) ) {
						keystrokeRtn = KEYSTROKE_IGNORE;
					}
					CheckAndResetRange( pgdata );
					pgdata->chiSymbolCursor++;
					if ( ChewingIsChiAt( pgdata->chiSymbolCursor, pgdata ) )
						ChoiceFirstAvail( pgdata );
					else
						OpenSymbolChoice( pgdata );
				}
				goto End_Paging;
			default:
				break;
		}
	}
	/* editing */
	else {
		if ( pgdata->bChiSym == CHINESE_MODE ) {
			if ( pgdata->config.bEasySymbolInput != 0 ) {
				EasySymbolInput( key, pgdata );
				goto End_keyproc;
			}

			/* open symbol table */
			if ( key == '`' ) {
				pgdata->bSelect = 1;
				pgdata->choiceInfo.oldChiSymbolCursor = pgdata->chiSymbolCursor;

				HaninSymbolInput( pgdata );
				goto End_KeyDefault;
			}

			rtn = ZuinPhoInput( pgdata, key );
			DEBUG_OUT(
				"\t\tChinese mode key, "
				"ZuinPhoInput return value = %d\n",
				rtn );

			if ( rtn == ZUIN_KEY_ERROR )
				rtn = SpecialSymbolInput( key, pgdata );
			switch ( rtn ) {
				case ZUIN_ABSORB:
					keystrokeRtn = KEYSTROKE_ABSORB;
					break;
				case ZUIN_COMMIT:
					AddChi( pgdata->zuinData.phone, pgdata->zuinData.phoneAlt, pgdata );
					break;
				case ZUIN_NO_WORD:
					keystrokeRtn = KEYSTROKE_BELL | KEYSTROKE_ABSORB;
					break;
				case ZUIN_KEY_ERROR:
				case ZUIN_IGNORE:
					DEBUG_OUT(
						"\t\tbefore isupper(key),key=%d\n",
						key );
					/* change upper case into lower case */
					if ( isupper( key ) )
						key = tolower( key );

					DEBUG_OUT(
						"\t\tafter isupper(key),key=%d\n",
						key );

					/* see if buffer contains nothing */
					if ( pgdata->chiSymbolBufLen == 0 ) {
						bQuickCommit = 1;
					}

					if ( pgdata->config.bEasySymbolInput == 0 ) {
						if ( pgdata->bFullShape )
							rtn = FullShapeSymbolInput( key, pgdata );
						else
							rtn = SymbolInput( key, pgdata );
					}

					if ( rtn == SYMBOL_KEY_ERROR ) {
						keystrokeRtn = KEYSTROKE_IGNORE;
						/*
						 * If the key is not a printable symbol,
						 * then it's wrong to commit it.
						 */
						bQuickCommit = 0;
					}
					else
						keystrokeRtn = KEYSTROKE_ABSORB;

					break;

			}
		}
		/* English mode */
		else {
			/* see if buffer contains nothing */
			if ( pgdata->chiSymbolBufLen == 0 ) {
				bQuickCommit = 1;
			}
			if ( pgdata->bFullShape ) {
				rtn = FullShapeSymbolInput( key, pgdata );
			}
			else {
				rtn = SymbolInput( key, pgdata );
			}

			if ( rtn == SYMBOL_KEY_ERROR ) {
				keystrokeRtn = KEYSTROKE_IGNORE;
				bQuickCommit = 0;
			}
		}
	}

End_keyproc:
	if ( ! bQuickCommit ) {
		CallPhrasing( pgdata );
		if ( ReleaseChiSymbolBuf( pgdata, pgo ) != 0 )
			keystrokeRtn = KEYSTROKE_COMMIT;
	}
	/* Quick commit */
	else {
		DEBUG_OUT(
				"\t\tQuick commit buf[0]=%c\n",
				pgdata->chiSymbolBuf[ 0 ].s[ 0 ] );
		pgo->commitStr[ 0 ] = pgdata->chiSymbolBuf[ 0 ];
		pgo->nCommitStr = 1;
		pgdata->chiSymbolBufLen = 0;
		pgdata->chiSymbolCursor = 0;
		keystrokeRtn = KEYSTROKE_COMMIT;
	}

	if ( pgdata->phrOut.nNumCut > 0 ) {
		int i;
		for ( i = 0; i < pgdata->phrOut.nDispInterval; i++ ) {
			pgdata->bUserArrBrkpt[ pgdata->phrOut.dispInterval[ i ].from ] = 1;
			pgdata->bUserArrBrkpt[ pgdata->phrOut.dispInterval[ i ].to ] = 1;
		}
		pgdata->phrOut.nNumCut = 0;
	}

End_KeyDefault:
	CallPhrasing( pgdata );
End_Paging:
	MakeOutputWithRtn( pgo, pgdata, keystrokeRtn );
	return 0;
}

CHEWING_API int chewing_handle_CtrlNum( ChewingContext *ctx, int key )
{
	ChewingData *pgdata = ctx->data;
	ChewingOutput *pgo = ctx->output;
	int keystrokeRtn = KEYSTROKE_ABSORB;
	int newPhraseLen;
	int i;
	uint16_t addPhoneSeq[ MAX_PHONE_SEQ_LEN ];
	char addWordSeq[ MAX_PHONE_SEQ_LEN * MAX_UTF8_SIZE + 1 ];
	int phraseState;
	int cursor;

	CheckAndResetRange( pgdata );

	if ( pgdata->bSelect )
		return 0;

	CallPhrasing( pgdata );
	newPhraseLen = key - '0';

	if ( key == '0' || key == '1' ) {
		pgdata->bSelect = 1;
		pgdata->choiceInfo.oldChiSymbolCursor = pgdata->chiSymbolCursor;

		HaninSymbolInput( pgdata );
		CallPhrasing( pgdata );
		MakeOutputWithRtn( pgo, pgdata, keystrokeRtn );
		return 0;
	}

	cursor = PhoneSeqCursor( pgdata );
	if ( ! pgdata->config.bAddPhraseForward ) {
		if (
			newPhraseLen >= 1 &&
			cursor + newPhraseLen - 1 <= pgdata->nPhoneSeq ) {
			if ( NoSymbolBetween(
				pgdata,
				cursor,
				cursor + newPhraseLen ) ) {
				/* Manually add phrase to the user phrase database. */
				memcpy( addPhoneSeq,
				        &pgdata->phoneSeq[ cursor ],
				        sizeof( uint16_t ) * newPhraseLen );
				addPhoneSeq[ newPhraseLen ] = 0;
				ueStrNCpy( addWordSeq,
				           ueStrSeek( (char *) &pgdata->phrOut.chiBuf,
				                      cursor ),
				           newPhraseLen, 1);


				phraseState = UserUpdatePhrase( pgdata, addPhoneSeq, addWordSeq );
				SetUpdatePhraseMsg(
					pgdata,
					addWordSeq,
					newPhraseLen,
					phraseState );

				/* Clear the breakpoint between the New Phrase */
				for ( i = 1; i < newPhraseLen; i++ )
					pgdata->bUserArrBrkpt[ cursor + i ] = 0;
			}
		}
	}
	else {
		if (
			newPhraseLen >= 1 &&
			cursor - newPhraseLen >= 0 ) {
			if ( NoSymbolBetween( pgdata,
				cursor - newPhraseLen,
				cursor ) ) {
				/* Manually add phrase to the user phrase database. */
				memcpy( addPhoneSeq,
				        &pgdata->phoneSeq[ cursor - newPhraseLen ],
				        sizeof( uint16_t ) * newPhraseLen );
				addPhoneSeq[ newPhraseLen ] = 0;
				ueStrNCpy( addWordSeq,
				           ueStrSeek( (char *) &pgdata->phrOut.chiBuf,
				           cursor - newPhraseLen ),
				           newPhraseLen, 1);

				phraseState = UserUpdatePhrase( pgdata, addPhoneSeq, addWordSeq );
				SetUpdatePhraseMsg(
					pgdata,
					addWordSeq,
					newPhraseLen,
					phraseState );

				/* Clear the breakpoint between the New Phrase */
				for ( i = 1; i < newPhraseLen; i++ )
					pgdata->bUserArrBrkpt[ cursor - newPhraseLen + i ] = 0;
			}
		}
	}
	CallPhrasing( pgdata );
	MakeOutputWithRtn( pgo, pgdata, keystrokeRtn );
	MakeOutputAddMsgAndCleanInterval( pgo, pgdata );
	return 0;
}

CHEWING_API int chewing_handle_ShiftSpace( ChewingContext *ctx )
{
	ChewingData *pgdata = ctx->data;
	ChewingOutput *pgo = ctx->output;
	int keystrokeRtn = KEYSTROKE_ABSORB;

	if ( ! pgdata->bSelect ) {
		CheckAndResetRange( pgdata );
	}
	CallPhrasing( pgdata );
	MakeOutputWithRtn( pgo, pgdata, keystrokeRtn );
	return 0;
}

CHEWING_API int chewing_handle_Numlock( ChewingContext *ctx, int key )
{
	ChewingData *pgdata = ctx->data;
	ChewingOutput *pgo = ctx->output;
	int rtn, QuickCommit = 0;
	int keystrokeRtn = KEYSTROKE_ABSORB;

	if ( ! pgdata->bSelect ) {
		/* If we're not selecting words, we should send out numeric
		 * characters at once.
		 */
		if ( pgdata->chiSymbolBufLen == 0 ) {
			QuickCommit = 1;
		}
		rtn = SymbolInput( key, pgdata );
		/* copied from chewing_handle_Default */
		if ( rtn == SYMBOL_KEY_ERROR ) {
			keystrokeRtn = KEYSTROKE_IGNORE ;
		}
		else if ( QuickCommit ) {
			pgo->commitStr[ 0 ] = pgdata->chiSymbolBuf[ 0 ];
			pgo->nCommitStr = 1;
			pgdata->chiSymbolBufLen = 0;
			pgdata->chiSymbolCursor = 0;
			keystrokeRtn = KEYSTROKE_COMMIT;
		}
		else {	/* Not quick commit */
			CallPhrasing( pgdata );
			if( ReleaseChiSymbolBuf( pgdata, pgo ) != 0 )
				keystrokeRtn = KEYSTROKE_COMMIT;
		}
	}
	else {
		/* Otherwise, if we are selecting words, we use numeric keys
		 * as selkey
		 * and submit the words.
		 */
		int num = -1;
		if ( key > '0' && key < '9' )
			num = key - '1';
		else if ( key == '0' )
			num = 9;
		DoSelect( pgdata, num );
	}
	CallPhrasing( pgdata );
	if ( ReleaseChiSymbolBuf( pgdata, pgo ) != 0 )
		keystrokeRtn = KEYSTROKE_COMMIT;
	MakeOutputWithRtn( pgo, pgdata, keystrokeRtn );
	return 0;
}

CHEWING_API unsigned short *chewing_get_phoneSeq( ChewingContext *ctx )
{
	uint16_t *seq;
	seq = ALC( uint16_t, ctx->data->nPhoneSeq );
	if ( seq )
		memcpy( seq, ctx->data->phoneSeq, sizeof(uint16_t)*ctx->data->nPhoneSeq );
	return seq;
}

CHEWING_API int chewing_get_phoneSeqLen( ChewingContext *ctx )
{
	return ctx->data->nPhoneSeq;
}

CHEWING_API void chewing_set_logger( ChewingContext *ctx,
	void (*logger)( void *data, int level, const char *fmt, ... ),
	void *data )
{
	if ( !logger ) {
		logger = NullLogger;
		data = 0;
	}
	ctx->data->logger = logger;
	ctx->data->loggerData = data;
}
