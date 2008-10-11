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

#include "private.h"
#include "plat_mmap.h"
#include "dict-private.h"

#ifdef USE_BINARY_DATA
static int *begin = NULL;
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
	if ( begin )
		free( begin );
#endif
}

int InitDict( const char *prefix )
{
	char filename[ 100 ];

#ifdef USE_BINARY_DATA
	int indexfile;
	struct stat file_stat;
#else
	FILE *indexfile;
	int i;
#endif

	sprintf( filename, "%s" PLAT_SEPARATOR "%s", prefix, DICT_FILE );
	dictfile = fopen( filename, "r" );

	sprintf( filename, "%s" PLAT_SEPARATOR "%s", prefix, PH_INDEX_FILE );

#ifdef USE_BINARY_DATA
	indexfile = open( filename, O_RDONLY);

	if ( indexfile == -1 )
		return 0;
	fstat( indexfile, &file_stat );

	if ( (begin = (int *) malloc( (size_t) file_stat.st_size + sizeof(int)) ) )
		read( indexfile, begin, (size_t) file_stat.st_size );

	close( indexfile );
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
	char buf[ 1000 ];

	fgettab( buf, 1000, dictfile );
	sscanf( buf, "%[^ ] %d", phr_ptr->phrase, &( phr_ptr->freq ) );
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

