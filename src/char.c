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
#ifdef HAVE_CONFIG_H
  #include <config.h>
#endif

#include <stdio.h>
#include <stdlib.h>
#include <assert.h>
#include <string.h>

#include "global-private.h"
#include "chewing-definition.h"
#include "char-private.h"
#include "private.h"
#include "plat_mmap.h"

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

static int CompUint16( const uint16 *pa, const uint16 *pb )
{
	return ( (*pa) - (*pb) );
}

void TerminateChar( ChewingData *pgdata )
{
#ifdef USE_BINARY_DATA
	pgdata->arrPhone = NULL;
	plat_mmap_close( &pgdata->char_phone_mmap );

	pgdata->char_begin = NULL;
	plat_mmap_close( &pgdata->char_begin_mmap );

	pgdata->char_ = NULL;
	plat_mmap_close( &pgdata->char_mmap );

	pgdata->phone_num = 0;
#else
	if ( pgdata->charfile )
		fclose( pgdata->charfile );
	free( pgdata->char_begin );
	free( pgdata->arrPhone );
	pgdata->phone_num = 0;
#endif
}

int InitChar( ChewingData *pgdata , const char * prefix )
{
#ifdef USE_BINARY_DATA
	char filename[ PATH_MAX ];
	int len;
	size_t offset;
	size_t file_size;
	size_t csize;

	len = snprintf( filename, sizeof( filename ), "%s" PLAT_SEPARATOR "%s", prefix, CHAR_FILE );
	if ( len + 1 > sizeof( filename ) )
		return -1;

	plat_mmap_set_invalid( &pgdata->char_mmap );
	file_size = plat_mmap_create( &pgdata->char_mmap, filename, FLAG_ATTRIBUTE_READ );
	if ( file_size <= 0 )
		return -1;

	csize = file_size;
	offset = 0;
	pgdata->char_ = plat_mmap_set_view( &pgdata->char_mmap, &offset, &csize );
	if ( !pgdata->char_ )
		return -1;

	len = snprintf( filename, sizeof( filename ), "%s" PLAT_SEPARATOR "%s", prefix, CHAR_INDEX_BEGIN_FILE );
	if ( len + 1 > sizeof( filename ) )
		return -1;

	plat_mmap_set_invalid( &pgdata->char_begin_mmap );
	file_size = plat_mmap_create( &pgdata->char_begin_mmap, filename, FLAG_ATTRIBUTE_READ );
	if ( file_size <= 0 )
		return -1;

	pgdata->phone_num = file_size / sizeof( int );

	offset = 0;
	csize = file_size;
	pgdata->char_begin = plat_mmap_set_view( &pgdata->char_begin_mmap, &offset, &csize );
	if ( !pgdata->char_begin )
		return -1;

	len = snprintf( filename, sizeof( filename ), "%s" PLAT_SEPARATOR "%s", prefix, CHAR_INDEX_PHONE_FILE );
	if ( len + 1 > sizeof( filename ) )
		return -1;

	plat_mmap_set_invalid( &pgdata->char_phone_mmap );
	file_size = plat_mmap_create( &pgdata->char_phone_mmap, filename, FLAG_ATTRIBUTE_READ );
	if ( file_size <= 0 )
		return -1;

	if ( pgdata->phone_num != file_size / sizeof( uint16 ))
		return -1;

	offset = 0;
	csize = file_size;
	pgdata->arrPhone = plat_mmap_set_view( &pgdata->char_phone_mmap, &offset, &csize );
	if ( !pgdata->arrPhone )
		return -1;

	return 0;
#else
	char filename[ PATH_MAX ];
	int len;
	int i;
	FILE *indexfile = NULL;

	pgdata->phone_num = PHONE_NUM;

	pgdata->arrPhone = calloc( pgdata->phone_num, sizeof( *pgdata->arrPhone ) );
	if ( !pgdata->arrPhone )
	    return -1;

	pgdata->char_begin = calloc( pgdata->phone_num, sizeof( *pgdata->char_begin ) );
	if ( !pgdata->char_begin )
	    return -1;

	len = snprintf( filename, sizeof( filename ), "%s" PLAT_SEPARATOR "%s", prefix, CHAR_FILE );
	if ( len + 1 > sizeof( filename ) )
		return -1;

	pgdata->charfile = fopen( filename, "r" );
	if ( !pgdata->charfile )
		return -1;

	len = snprintf( filename, sizeof( filename ), "%s" PLAT_SEPARATOR "%s", prefix, CHAR_INDEX_FILE );
	if ( len + 1 > sizeof( filename ) )
		return -1;

	indexfile = fopen( filename, "r" );
	if ( !indexfile )
		return -1;

	for ( i = 0; i < pgdata->phone_num; ++i )
		fscanf( indexfile, "%hu %d", &pgdata->arrPhone[i], &pgdata->char_begin[i] );

	fclose( indexfile );
	return 0;
#endif
}

static void Str2Word( ChewingData *pgdata, Word *wrd_ptr )
{
#ifndef USE_BINARY_DATA
	char buf[ 1000 ];
	uint16 sh;

	fgettab( buf, 1000, pgdata->charfile );
	/* only read 6 bytes to wrd_ptr->word avoid buffer overflow */
	sscanf( buf, "%hu %6[^ ]", &sh, wrd_ptr->word );
	assert( wrd_ptr->word != '\0' );
#else
	unsigned char size;
	size = *(unsigned char *) pgdata->char_cur_pos;
	pgdata->char_cur_pos = (unsigned char*) pgdata->char_cur_pos + sizeof(unsigned char);
	memcpy( wrd_ptr->word, pgdata->char_cur_pos, size );
	pgdata->char_cur_pos = (unsigned char*) pgdata->char_cur_pos + size;
	wrd_ptr->word[ size ] = '\0';
#endif
}

int GetCharFirst( ChewingData *pgdata, Word *wrd_ptr, uint16 phoneid )
{
	uint16 *pinx;

	pinx = (uint16 *) bsearch(
		&phoneid, pgdata->arrPhone, pgdata->phone_num,
		sizeof( uint16 ), (CompFuncType) CompUint16 );
	if ( ! pinx )
		return 0;

#ifndef USE_BINARY_DATA
	fseek( pgdata->charfile, pgdata->char_begin[ pinx - pgdata->arrPhone ], SEEK_SET );
#else
	pgdata->char_cur_pos = (unsigned char*)pgdata->char_ + pgdata->char_begin[ pinx - pgdata->arrPhone ];
#endif
	pgdata->char_end_pos = pgdata->char_begin[ pinx - pgdata->arrPhone + 1 ];
	Str2Word( pgdata, wrd_ptr );
	return 1;
}

int GetCharNext( ChewingData *pgdata, Word *wrd_ptr )
{
#ifndef USE_BINARY_DATA
	if ( ftell( pgdata->charfile ) >= pgdata->char_end_pos )
		return 0;
#else
	if ( (unsigned char*)pgdata->char_cur_pos >= (unsigned char*)pgdata->char_ + pgdata->char_end_pos )
		return 0;
#endif
	Str2Word( pgdata, wrd_ptr );
	return 1;
}
