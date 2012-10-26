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

static int CompUint16( const uint16_t *pa, const uint16_t *pb )
{
	return ( (*pa) - (*pb) );
}

void TerminateChar( ChewingData *pgdata )
{
#ifdef USE_BINARY_DATA
	pgdata->static_data.arrPhone = NULL;
	plat_mmap_close( &pgdata->static_data.char_phone_mmap );

	pgdata->static_data.char_begin = NULL;
	plat_mmap_close( &pgdata->static_data.char_begin_mmap );

	pgdata->static_data.char_ = NULL;
	plat_mmap_close( &pgdata->static_data.char_mmap );

	pgdata->static_data.phone_num = 0;
#else
	if ( pgdata->static_data.charfile )
		fclose( pgdata->static_data.charfile );
	free( pgdata->static_data.char_begin );
	free( pgdata->static_data.arrPhone );
	pgdata->static_data.phone_num = 0;
#endif
}

int InitChar( ChewingData *pgdata , const char * prefix )
{
#ifdef USE_BINARY_DATA
	char filename[ PATH_MAX ];
	size_t len;
	size_t offset;
	size_t file_size;
	size_t csize;

	len = snprintf( filename, sizeof( filename ), "%s" PLAT_SEPARATOR "%s", prefix, CHAR_FILE );
	if ( len + 1 > sizeof( filename ) )
		return -1;

	plat_mmap_set_invalid( &pgdata->static_data.char_mmap );
	file_size = plat_mmap_create( &pgdata->static_data.char_mmap, filename, FLAG_ATTRIBUTE_READ );
	if ( file_size <= 0 )
		return -1;

	csize = file_size;
	offset = 0;
	pgdata->static_data.char_ = plat_mmap_set_view( &pgdata->static_data.char_mmap, &offset, &csize );
	if ( !pgdata->static_data.char_ )
		return -1;

	len = snprintf( filename, sizeof( filename ), "%s" PLAT_SEPARATOR "%s", prefix, CHAR_INDEX_BEGIN_FILE );
	if ( len + 1 > sizeof( filename ) )
		return -1;

	plat_mmap_set_invalid( &pgdata->static_data.char_begin_mmap );
	file_size = plat_mmap_create( &pgdata->static_data.char_begin_mmap, filename, FLAG_ATTRIBUTE_READ );
	if ( file_size <= 0 )
		return -1;

	pgdata->static_data.phone_num = file_size / sizeof( int );

	offset = 0;
	csize = file_size;
	pgdata->static_data.char_begin = plat_mmap_set_view( &pgdata->static_data.char_begin_mmap, &offset, &csize );
	if ( !pgdata->static_data.char_begin )
		return -1;

	len = snprintf( filename, sizeof( filename ), "%s" PLAT_SEPARATOR "%s", prefix, CHAR_INDEX_PHONE_FILE );
	if ( len + 1 > sizeof( filename ) )
		return -1;

	plat_mmap_set_invalid( &pgdata->static_data.char_phone_mmap );
	file_size = plat_mmap_create( &pgdata->static_data.char_phone_mmap, filename, FLAG_ATTRIBUTE_READ );
	if ( file_size <= 0 )
		return -1;

	if ( pgdata->static_data.phone_num != file_size / sizeof( uint16_t ))
		return -1;

	offset = 0;
	csize = file_size;
	pgdata->static_data.arrPhone = plat_mmap_set_view( &pgdata->static_data.char_phone_mmap, &offset, &csize );
	if ( !pgdata->static_data.arrPhone )
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
	uint16_t sh;

	fgettab( buf, 1000, pgdata->charfile );
	/* only read 6 bytes to wrd_ptr->word avoid buffer overflow */
	sscanf( buf, "%hu %6[^ ]", &sh, wrd_ptr->word );
	assert( wrd_ptr->word != '\0' );
#else
	unsigned char size;
	size = *(unsigned char *) pgdata->static_data.char_cur_pos;
	pgdata->static_data.char_cur_pos = (unsigned char*) pgdata->static_data.char_cur_pos + sizeof(unsigned char);
	memcpy( wrd_ptr->word, pgdata->static_data.char_cur_pos, size );
	pgdata->static_data.char_cur_pos = (unsigned char*) pgdata->static_data.char_cur_pos + size;
	wrd_ptr->word[ size ] = '\0';
#endif
}

int GetCharFirst( ChewingData *pgdata, Word *wrd_ptr, uint16_t phoneid )
{
	uint16_t *pinx;

	pinx = (uint16_t *) bsearch(
		&phoneid, pgdata->static_data.arrPhone, pgdata->static_data.phone_num,
		sizeof( uint16_t ), (CompFuncType) CompUint16 );
	if ( ! pinx )
		return 0;

#ifndef USE_BINARY_DATA
	fseek( pgdata->charfile, pgdata->char_begin[ pinx - pgdata->arrPhone ], SEEK_SET );
#else
	pgdata->static_data.char_cur_pos = (unsigned char*)pgdata->static_data.char_ + pgdata->static_data.char_begin[ pinx - pgdata->static_data.arrPhone ];
#endif
	pgdata->static_data.char_end_pos = pgdata->static_data.char_begin[ pinx - pgdata->static_data.arrPhone + 1 ];
	Str2Word( pgdata, wrd_ptr );
	return 1;
}

int GetCharNext( ChewingData *pgdata, Word *wrd_ptr )
{
#ifndef USE_BINARY_DATA
	if ( ftell( pgdata->charfile ) >= pgdata->char_end_pos )
		return 0;
#else
	if ( (unsigned char*)pgdata->static_data.char_cur_pos >= (unsigned char*)pgdata->static_data.char_ + pgdata->static_data.char_end_pos )
		return 0;
#endif
	Str2Word( pgdata, wrd_ptr );
	return 1;
}
