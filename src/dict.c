/**
 * dict.h
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
static plat_mmap dict_mmap;
#else
static int begin[ PHONE_PHRASE_NUM + 1 ];
#endif
static FILE *dictfile;
static int end_pos;

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

static void TerminateDict()
{
	if ( dictfile )
		fclose( dictfile );
#ifdef USE_BINARY_DATA
	plat_mmap_close( &dict_mmap );
#endif
}

int InitDict( const char *prefix )
{
	char filename[ 100 ];

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
	dictfile = fopen( filename, "rb" );
#else
	dictfile = fopen( filename, "r" );
#endif

	sprintf( filename, "%s" PLAT_SEPARATOR "%s", prefix, PH_INDEX_FILE );

#ifdef USE_BINARY_DATA
	file_size = plat_mmap_create( &dict_mmap, filename, FLAG_ATTRIBUTE_READ );
	assert( file_size );
	if ( file_size < 0 )
		return 0;

	csize = file_size + sizeof(int);
	begin = (int *) plat_mmap_set_view( &dict_mmap, &offset, &csize );
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
	fread( &size, sizeof( unsigned char ), 1, dictfile );
	fread( phr_ptr->phrase, size, 1, dictfile );
	fread( &( phr_ptr->freq ), sizeof( int ), 1, dictfile );
	phr_ptr->phrase[size] = '\0';
#endif
}

int GetPhraseFirst( Phrase *phr_ptr, int phone_phr_id )
{
	assert( ( 0 <= phone_phr_id ) && ( phone_phr_id < PHONE_PHRASE_NUM ) );

	fseek( dictfile, begin[ phone_phr_id ], SEEK_SET );
	end_pos = begin[ phone_phr_id + 1 ];
	Str2Phrase( phr_ptr );
	return 1;
}

int GetPhraseNext( Phrase *phr_ptr )
{
	if ( ftell( dictfile ) >= end_pos )
		return 0;
	Str2Phrase( phr_ptr );
	return 1;
}

