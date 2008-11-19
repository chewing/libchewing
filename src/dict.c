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

#include "global-private.h"
#include "private.h"
#include "plat_mmap.h"
#include "dict-private.h"

#ifdef USE_BINARY_DATA
static int *begin = NULL;
static plat_mmap index_mmap;
static void *dict = NULL;
static void *cur_pos = NULL;
static plat_mmap dict_mmap;
#else
static int begin[ PHONE_PHRASE_NUM + 1 ];
static FILE *dictfile;
#endif
static int end_pos;

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

static void TerminateDict()
{
#ifdef USE_BINARY_DATA
	plat_mmap_close( &index_mmap );
	plat_mmap_close( &dict_mmap );
#else
	if ( dictfile )
		fclose( dictfile );
#endif
}

int InitDict( const char *prefix )
{
	char filename[ PATH_MAX ];

#ifdef USE_BINARY_DATA
	long file_size;
	size_t offset = 0;
	size_t csize;
#else
	FILE *indexfile;
	int i;
#endif

	sprintf( filename, "%s" PLAT_SEPARATOR "%s", prefix, DICT_FILE );
#ifdef USE_BINARY_DATA
	plat_mmap_set_invalid( &dict_mmap );
	file_size = plat_mmap_create( &dict_mmap, filename, FLAG_ATTRIBUTE_READ );
	assert( plat_mmap_is_valid( &dict_mmap ) );
	if ( file_size < 0 )
		return 0;
	csize = file_size;
	dict = (void *) plat_mmap_set_view( &dict_mmap, &offset, &csize );
	assert( dict );
#else
	dictfile = fopen( filename, "r" );
#endif

	sprintf( filename, "%s" PLAT_SEPARATOR "%s", prefix, PH_INDEX_FILE );

#ifdef USE_BINARY_DATA
	plat_mmap_set_invalid( &index_mmap );
	file_size = plat_mmap_create( &index_mmap, filename, FLAG_ATTRIBUTE_READ );
	assert( plat_mmap_is_valid( &index_mmap ) );
	if ( file_size < 0 )
		return 0;

	csize = file_size;
	begin = (int *) plat_mmap_set_view( &index_mmap, &offset, &csize );
	assert( begin );
#else
	indexfile = fopen( filename, "r" );
	assert( dictfile && indexfile );
	i = 0;
	while ( !feof( indexfile ) )
		fscanf( indexfile, "%d", &begin[ i++ ] );
	fclose( indexfile );
#endif
	addTerminateService( TerminateDict );
	return 1;
}

static void Str2Phrase( Phrase *phr_ptr )
{
#ifndef USE_BINARY_DATA
	char buf[ 1000 ];

	fgettab( buf, 1000, dictfile );
	sscanf( buf, "%[^ ] %d", phr_ptr->phrase, &( phr_ptr->freq ) );
#else
	unsigned char size;
	size = *(unsigned char *) cur_pos;
	cur_pos = (unsigned char *)cur_pos + sizeof(unsigned char);
	memcpy( phr_ptr->phrase, cur_pos, size );
	cur_pos = (unsigned char *)cur_pos + size;
	phr_ptr->freq = *(int *) cur_pos;
	cur_pos = (unsigned char *)cur_pos + sizeof(int);
	phr_ptr->phrase[ size ] = '\0';
#endif
}

int GetPhraseFirst( Phrase *phr_ptr, int phone_phr_id )
{
	assert( ( 0 <= phone_phr_id ) && ( phone_phr_id < PHONE_PHRASE_NUM ) );

#ifndef USE_BINARY_DATA
	fseek( dictfile, begin[ phone_phr_id ], SEEK_SET );
#else
	cur_pos = (unsigned char *)dict + begin[ phone_phr_id ];
#endif
	end_pos = begin[ phone_phr_id + 1 ];
	Str2Phrase( phr_ptr );
	return 1;
}

int GetPhraseNext( Phrase *phr_ptr )
{
#ifndef USE_BINARY_DATA
	if ( ftell( dictfile ) >= end_pos )
		return 0;
#else
	if ( (unsigned char *)cur_pos >= (unsigned char *)dict + end_pos )
		return 0;
#endif
	Str2Phrase( phr_ptr );
	return 1;
}
