/**
 * char.c
 *
 * Copyright (c) 1999, 2000, 2001
 *	Lu-chuan Kung and Kang-pen Chen.
 *	All rights reserved.
 *
 * Copyright (c) 2004, 2005, 2006, 2008
 *	libchewing Core Team. See ChangeLog for details.
 *
 * See the file "COPYING" for information on usage and redistribution
 * of this file.
 */

/**
 * @file char.c
 * @brief word data file
 */
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "global-private.h"
#include "char-private.h"
#include "private.h"
#include "plat_mmap.h"

static int CompTreeType( const void *pa, const void *pb )
{
	return ( ((TreeType*)pa)->key - ((TreeType*)pb)->key );
}

void TerminateChar( ChewingData *pgdata )
{
}

int InitChar( ChewingData *pgdata , const char * prefix )
{
	return 0;
}

/*
 * The function gets string of a Chinese character from dictionary, and stores
 * it into buffer given by wrd_ptr.
 */
static void Str2Word( ChewingData *pgdata, Word *wrd_ptr )
{
	const TreeType *pLeaf = &pgdata->static_data.tree[ pgdata->static_data.char_cur_pos ];

	strcpy(wrd_ptr->word, pgdata->static_data.dict + pLeaf->phrase.pos);
	pgdata->static_data.char_cur_pos++;
}

int GetCharFirst( ChewingData *pgdata, Word *wrd_ptr, uint16_t key )
{
	const TreeType *pinx;
	TreeType keyNode = {0};

	keyNode.key = key;
	pinx = (const TreeType*) bsearch(
		&keyNode, pgdata->static_data.tree + pgdata->static_data.tree[0].child.begin,
		pgdata->static_data.tree[0].child.end - pgdata->static_data.tree[0].child.begin,
		sizeof( TreeType ), CompTreeType );
	if ( ! pinx )
		return 0;

	pgdata->static_data.char_cur_pos = pinx->child.begin;
	pgdata->static_data.char_end_pos = pinx->child.end;
	Str2Word( pgdata, wrd_ptr );
	return 1;
}

int GetCharNext( ChewingData *pgdata, Word *wrd_ptr )
{
	if ( pgdata->static_data.char_cur_pos >= pgdata->static_data.char_end_pos
		|| pgdata->static_data.tree[ pgdata->static_data.char_cur_pos ].key != 0)
		return 0;
	Str2Word( pgdata, wrd_ptr );
	return 1;
}
