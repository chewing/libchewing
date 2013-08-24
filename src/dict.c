/**
 * dict.c
 *
 * Copyright (c) 1999, 2000, 2001
 *	Lu-chuan Kung and Kang-pen Chen.
 *	All rights reserved.
 *
 * Copyright (c) 2004, 2005, 2008
 *	libchewing Core Team. See ChangeLog for details.
 *
 * See the file "COPYING" for information on usage and redistribution
 * of this file.
 */
#ifdef HAVE_CONFIG_H
  #include <config.h>
#endif

#include <stdio.h>
#include <assert.h>
#include <string.h>

#include "global-private.h"
#include "plat_mmap.h"
#include "dict-private.h"
#include "memory-private.h"

void TerminateDict( ChewingData *pgdata )
{
	plat_mmap_close( &pgdata->static_data.index_mmap );
	plat_mmap_close( &pgdata->static_data.dict_mmap );
}

int InitDict( ChewingData *pgdata, const char *prefix )
{
	char filename[ PATH_MAX ];
	size_t len;
	size_t offset;
	size_t file_size;
	size_t csize;

	len = snprintf( filename, sizeof( filename ), "%s" PLAT_SEPARATOR "%s", prefix, DICT_FILE );
	if ( len + 1 > sizeof( filename ) )
		return -1;

	plat_mmap_set_invalid( &pgdata->static_data.dict_mmap );
	file_size = plat_mmap_create( &pgdata->static_data.dict_mmap, filename, FLAG_ATTRIBUTE_READ );
	if ( file_size <= 0 )
		return -1;

	offset = 0;
	csize = file_size;
	pgdata->static_data.dict = plat_mmap_set_view( &pgdata->static_data.dict_mmap, &offset, &csize );
	if ( !pgdata->static_data.dict )
		return -1;

	len = snprintf( filename, sizeof( filename ), "%s" PLAT_SEPARATOR "%s", prefix, PH_INDEX_FILE );
	if ( len + 1 > sizeof( filename ) )
		return -1;

	plat_mmap_set_invalid( &pgdata->static_data.index_mmap );
	file_size = plat_mmap_create( &pgdata->static_data.index_mmap, filename, FLAG_ATTRIBUTE_READ );
	if ( file_size <= 0 )
		return -1;

	offset = 0;
	csize = file_size;
	pgdata->static_data.dict_begin = plat_mmap_set_view( &pgdata->static_data.index_mmap, &offset, &csize );
	if ( !pgdata->static_data.dict_begin )
		return -1;

	return 0;
}

static void Str2Phrase( ChewingData *pgdata, Phrase *phr_ptr )
{
	unsigned char size;
	size = *(unsigned char *) pgdata->static_data.dict_cur_pos;
	pgdata->static_data.dict_cur_pos = (unsigned char *)pgdata->static_data.dict_cur_pos + sizeof(unsigned char);
	memcpy( phr_ptr->phrase, pgdata->static_data.dict_cur_pos, size );
	pgdata->static_data.dict_cur_pos = (unsigned char *)pgdata->static_data.dict_cur_pos + size;
	phr_ptr->freq = GetInt32(pgdata->static_data.dict_cur_pos);
	pgdata->static_data.dict_cur_pos = (unsigned char *)pgdata->static_data.dict_cur_pos + sizeof(int);
	phr_ptr->phrase[ size ] = '\0';
}

int GetPhraseFirst( ChewingData *pgdata, Phrase *phr_ptr, int phone_phr_id )
{
	assert( ( 0 <= phone_phr_id ) && ( phone_phr_id < PHONE_PHRASE_NUM ) );

	pgdata->static_data.dict_cur_pos = (unsigned char *)pgdata->static_data.dict + pgdata->static_data.dict_begin[ phone_phr_id ];
	pgdata->static_data.dict_end_pos = pgdata->static_data.dict_begin[ phone_phr_id + 1 ];
	Str2Phrase( pgdata, phr_ptr );
	return 1;
}

int GetPhraseNext( ChewingData *pgdata, Phrase *phr_ptr )
{
	if ( (unsigned char *)pgdata->static_data.dict_cur_pos >= (unsigned char *)pgdata->static_data.dict + pgdata->static_data.dict_end_pos )
		return 0;
	Str2Phrase( pgdata, phr_ptr );
	return 1;
}
