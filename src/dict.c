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

#include <stdio.h>
#include <assert.h>
#include <string.h>
#include <stdlib.h>

#include "global-private.h"
#include "private.h"
#include "plat_mmap.h"
#include "dict-private.h"

#if ! defined(USE_BINARY_DATA)
static char *fgettab( char *buf, int maxlen, FILE *fp )
{
	int i;

	for ( i = 0; i < maxlen; i++ ) {
		buf[ i ] = (char) fgetc( fp );
		if ( feof( fp ) )
			break;
		if ( buf[ i ] == '\t' )
			break;
	}
	if ( feof( fp ) )
		return 0;
	buf[ i ] = '\0';
	return buf;
}
#endif

void TerminateDict( ChewingData *pgdata )
{
#ifdef USE_BINARY_DATA
	plat_mmap_close( &pgdata->static_data.index_mmap );
	plat_mmap_close( &pgdata->static_data.dict_mmap );
#else
	if ( pgdata->static_data.dictfile ) {
		fclose( pgdata->static_data.dictfile );
		pgdata->static_data.dictfile = NULL;
	}
	free( pgdata->static_data.dict_begin );
	pgdata->static_data.dict_begin = NULL;
#endif
}

int InitDict( ChewingData *pgdata, const char *prefix )
{
#ifdef USE_BINARY_DATA
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
#else
	char filename[ PATH_MAX ];
	FILE *indexfile;
	int len;
	int i;

	pgdata->dict_begin = calloc( PHONE_PHRASE_NUM + 1, sizeof( *pgdata->dict_begin ) );
	if ( !pgdata->dict_begin )
		return -1;

	len = snprintf( filename, sizeof( filename ), "%s" PLAT_SEPARATOR "%s", prefix, DICT_FILE );
	if ( len + 1 > sizeof( filename ) )
		return -1;

	pgdata->dictfile = fopen( filename, "r" );
	if ( !pgdata->dictfile )
		return -1;

	len = snprintf( filename, sizeof( filename ), "%s" PLAT_SEPARATOR "%s", prefix, PH_INDEX_FILE );
	if ( len + 1 > sizeof( filename ) )
		return -1;

	indexfile = fopen( filename, "r" );
	if ( !indexfile )
		return -1;

	i = 0;
	/* FIXME: check if begin is big enough to store all data. */
	while ( !feof( indexfile ) )
		fscanf( indexfile, "%d", &pgdata->dict_begin[ i++ ] );
	fclose( indexfile );

	return 0;
#endif
}

static void Str2Phrase( ChewingData *pgdata, Phrase *phr_ptr )
{
#ifndef USE_BINARY_DATA
	char buf[ 1000 ];

	fgettab( buf, 1000, pgdata->static_data.dictfile );
	sscanf( buf, "%[^ ] %d", phr_ptr->phrase, &( phr_ptr->freq ) );
#else
	unsigned char size;
	size = *(unsigned char *) pgdata->static_data.dict_cur_pos;
	pgdata->static_data.dict_cur_pos = (unsigned char *)pgdata->static_data.dict_cur_pos + sizeof(unsigned char);
	memcpy( phr_ptr->phrase, pgdata->static_data.dict_cur_pos, size );
	pgdata->static_data.dict_cur_pos = (unsigned char *)pgdata->static_data.dict_cur_pos + size;
	phr_ptr->freq = *(int *) pgdata->static_data.dict_cur_pos;
	pgdata->static_data.dict_cur_pos = (unsigned char *)pgdata->static_data.dict_cur_pos + sizeof(int);
	phr_ptr->phrase[ size ] = '\0';
#endif
}

int GetPhraseFirst( ChewingData *pgdata, Phrase *phr_ptr, int phone_phr_id )
{
	assert( ( 0 <= phone_phr_id ) && ( phone_phr_id < PHONE_PHRASE_NUM ) );

#ifndef USE_BINARY_DATA
	fseek( pgdata->static_data.dictfile, pgdata->static_data.dict_begin[ phone_phr_id ], SEEK_SET );
#else
	pgdata->static_data.dict_cur_pos = (unsigned char *)pgdata->static_data.dict + pgdata->static_data.dict_begin[ phone_phr_id ];
#endif
	pgdata->static_data.dict_end_pos = pgdata->static_data.dict_begin[ phone_phr_id + 1 ];
	Str2Phrase( pgdata, phr_ptr );
	return 1;
}

int GetPhraseNext( ChewingData *pgdata, Phrase *phr_ptr )
{
#ifndef USE_BINARY_DATA
	if ( ftell( pgdata->dictfile ) >= pgdata->dict_end_pos )
		return 0;
#else
	if ( (unsigned char *)pgdata->static_data.dict_cur_pos >= (unsigned char *)pgdata->static_data.dict + pgdata->static_data.dict_end_pos )
		return 0;
#endif
	Str2Phrase( pgdata, phr_ptr );
	return 1;
}
